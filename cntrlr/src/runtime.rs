use crate::sync::enable_interrupts;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

/// Default interrupt handler
pub extern "C" fn unused_interrupt() {}

/// The default reset vector
///
/// This is the entrypoint used when a custom reset has not been
/// implemented
#[no_mangle]
pub unsafe extern "C" fn __cntrlr_default_reset() -> ! {
    extern "C" {
        static mut __cntrlr_data_start: u8;
        static mut __cntrlr_data_end: u8;
        static __cntrlr_data_flash_start: u8;
        static mut __cntrlr_bss_start: u8;
        static mut __cntrlr_bss_end: u8;
        static mut __cntrlr_heap_start: u8;
        fn __cntrlr_main() -> !;
        fn __cntrlr_board_init();
        fn __cntrlr_board_start();
    }
    __cntrlr_board_start();
    init_data(
        &mut __cntrlr_data_start,
        &mut __cntrlr_data_end,
        &__cntrlr_data_flash_start,
    );
    init_bss(&mut __cntrlr_bss_start, &mut __cntrlr_bss_end);
    init_heap(&mut __cntrlr_heap_start);
    __cntrlr_board_init();
    enable_interrupts();
    __cntrlr_main();
}

unsafe fn init_data(data: *mut u8, data_end: *mut u8, data_flash: *const u8) {
    let data_len = data_end as usize - data as usize;
    for i in 0..data_len {
        *data.add(i) = *data_flash.add(i)
    }
}

unsafe fn init_bss(bss: *mut u8, bss_end: *mut u8) {
    let bss_len = bss_end as usize - bss as usize;
    for i in 0..bss_len {
        *bss.add(i) = 0;
    }
}

unsafe fn init_heap(brk: *mut u8) {
    crate::allocator::init(brk);
}

#[cfg_attr(target_arch = "arm", link_section = ".__CNTRLR_EXCEPTIONS")]
#[cfg_attr(target_arch = "arm", export_name = "__cntrlr_exceptions")]
#[allow(dead_code)]
pub static ARM_EXCEPTIONS: [unsafe extern "C" fn(); 14] = [
    unused_interrupt,
    unused_interrupt,
    unused_interrupt,
    unused_interrupt,
    unused_interrupt,
    unused_interrupt,
    unused_interrupt,
    unused_interrupt,
    unused_interrupt,
    unused_interrupt,
    unused_interrupt,
    unused_interrupt,
    unused_interrupt,
    unused_interrupt,
];
