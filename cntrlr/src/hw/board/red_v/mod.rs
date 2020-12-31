use crate::sync::enable_interrupts;
use core::sync::atomic::{AtomicUsize, Ordering};

pub mod digital;
pub mod io;

static CPU_FREQ: AtomicUsize = AtomicUsize::new(0);

pub fn set_clock(clock: usize) {
    use crate::hw::mcu::sifive::fe310g002::{Prci, Spi};

    let (r, f, q, div, spi_div) = match clock {
        384_000_000 => (2, 96, 2, 1, 8),
        256_000_000 => (2, 64, 2, 1, 6),
        _ => panic!("Invalid clock rate for Red-V: {}", clock),
    };
    CPU_FREQ.store(clock, Ordering::Relaxed);

    let mut spi = Spi::<(), (), 0>::get();
    spi.set_divider(spi_div);

    let mut prci = Prci::get();
    prci.use_pll(r, f, q, div);
}

#[cfg_attr(board = "red_v", export_name = "__cntrlr_board_start")]
pub unsafe extern "C" fn start() {}

#[cfg_attr(board = "red_v", export_name = "__cntrlr_board_init")]
pub unsafe extern "C" fn init() {
    use crate::hw::mcu::sifive::fe310g002::Plic;
    set_clock(256_000_000);

    let mut plic = Plic::get();
    plic.mask_all();
    plic.set_threshold(0);
    for intr in &[3, 4] {
        plic.enable(*intr);
        plic.set_priority(*intr, 1);
    }
    // Enable all interrupt sources and set up the runtime trap vec.
    #[cfg(board = "red_v")]
    asm!("
        la {0}, {1}
        csrw mtvec, {0}
        li {0}, 0x0888
        csrw mie, {0}", out(reg) _, sym trap_vec);
    enable_interrupts();
}

#[cfg_attr(board = "red_v", link_section = ".__CNTRLR_START")]
#[cfg_attr(board = "red_v", export_name = "__cntrlr_redv_reset")]
#[cfg_attr(board = "red_v", naked)]
pub unsafe extern "C" fn reset() {
    extern "C" {
        fn __cntrlr_reset();
        static __cntrlr_stack_top: u8;
    }
    #[cfg(board = "red_v")]
    asm!("
        la t0, {}
        csrw mtvec, t0
        la sp, {}
        jal ra, {}
",
         sym early_trap, sym __cntrlr_stack_top, sym __cntrlr_reset, options(noreturn)
    );
}

#[cfg_attr(board = "red_v", link_section = ".__CNTRLR_EARLY_TRAP")]
#[cfg_attr(board = "red_v", naked)]
pub unsafe extern "C" fn early_trap() {
    #[cfg(board = "red_v")]
    asm!(
        "
        csrr t0, mcause
        csrr t1, mepc
        csrr t2, mtval
    hang:
        j hang",
        options(noreturn)
    );
}

#[cfg_attr(board = "red_v", link_section = ".__CNTRLR_TRAP")]
#[cfg_attr(board = "red_v", naked)]
#[allow(dead_code)]
pub unsafe extern "C" fn trap_vec() {
    #[cfg(board = "red_v")]
    asm!("
        csrw mscratch, sp
        andi sp, sp, -8
        addi sp, sp, -64
        sw ra, 0(sp)
        sw t0, 4(sp)
        sw t1, 8(sp)
        sw t2, 12(sp)
        sw t3, 16(sp)
        sw t4, 20(sp)
        sw t5, 24(sp)
        sw t6, 28(sp)
        sw a0, 32(sp)
        sw a1, 36(sp)
        sw a2, 40(sp)
        sw a3, 44(sp)
        sw a4, 48(sp)
        sw a5, 52(sp)
        sw a6, 56(sp)
        sw a7, 60(sp)
        csrr a0, mcause
        csrr a1, mepc
        csrr a2, mtval
        jal ra, {}
        lw ra, 0(sp)
        lw t0, 4(sp)
        lw t1, 8(sp)
        lw t2, 12(sp)
        lw t3, 16(sp)
        lw t4, 20(sp)
        lw t5, 24(sp)
        lw t6, 28(sp)
        lw a0, 32(sp)
        lw a1, 36(sp)
        lw a2, 40(sp)
        lw a3, 44(sp)
        lw a4, 48(sp)
        lw a5, 52(sp)
        lw a6, 56(sp)
        lw a7, 60(sp)
        csrr sp, mscratch
        mret", sym handle_trap, options(noreturn));
}

#[allow(dead_code)]
unsafe extern "C" fn handle_trap(mcause: u32, mepc: u32, mtval: u32) {
    use crate::hw::mcu::sifive::fe310g002::Plic;

    match mcause {
        0 => panic!("Misaligned Instruction at 0x{:8X}", mepc),
        1 => panic!("Instruction access fault at 0x{:8X}", mepc),
        2 => panic!("Illegal instuction at 0x{:8X}", mepc),
        3 => panic!("Breakpoint at 0x{:8X}", mepc),
        4 => panic!(
            "Misaligned load of 0x{:8X} by instruction at 0x{:8X}",
            mtval, mepc
        ),
        5 => panic!(
            "Load fault of 0x{:8X} by instuction at 0x{:8X}",
            mtval, mepc
        ),
        6 => panic!(
            "Misaligned store or atomic operation of 0x{:8X} by instruction at 0x{:8X}",
            mtval, mepc
        ),
        7 => panic!(
            "Store or atmoic fault of 0x{:8X} by instruction at 0x{:8X}",
            mtval, mepc
        ),
        0x8000_0007 => {
            // TODO: Update millis
            panic!("Timer interrupt");
        }
        0x8000_000B => loop {
            let mut plic = Plic::get();
            let intr = plic.claim();
            match intr {
                0 => break,
                3 => io::serial_1_intr(),
                4 => io::serial_2_intr(),
                _ => {}
            }
            plic.complete(intr);
        },
        _ => panic!("Unknown trap"),
    }
}