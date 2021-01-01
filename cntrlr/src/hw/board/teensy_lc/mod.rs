// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Board-specific functionality for the Teensy LC

use core::sync::atomic::{AtomicUsize, Ordering};

pub mod digital;
pub mod io;

static CPU_FREQ: AtomicUsize = AtomicUsize::new(0);
static BUS_FREQ: AtomicUsize = AtomicUsize::new(0);

/// Set the clock for this board, in Hz.
///
/// Valid values are 48, 32, 24, 16, 12, 8, 6, 4, or 3 MHz
pub fn set_clock(clock: usize) {
    use crate::hw::mcu::kinetis::mkl26z64::{Clock, Mcg, Osc, OscRange, Sim};
    let (core, bus, flash, usb_num, usb_den, pll_num, pll_den) = match clock {
        48_000_000 => (1, 1, 2, 1, 1, 24, 8),
        32_000_000 => (3, 3, 3, 1, 2, 24, 4),
        24_000_000 => (2, 2, 2, 1, 1, 24, 8),
        16_000_000 => (3, 3, 3, 1, 1, 24, 8),
        12_000_000 => (4, 4, 4, 1, 1, 24, 8),
        8_000_000 => (6, 6, 6, 1, 1, 24, 8),
        6_000_000 => (8, 8, 8, 1, 1, 24, 8),
        4_000_000 => (12, 12, 12, 1, 1, 24, 8),
        3_000_000 => (16, 16, 16, 1, 1, 24, 8),
        _ => panic!("Invalid clock rate for Teensy LC: {}", clock),
    };

    CPU_FREQ.store(clock, Ordering::Relaxed);
    BUS_FREQ.store(clock * core as usize / bus as usize, Ordering::Relaxed);

    // First, switch back to a slow clock so it's safe to update dividers
    let mut mcg = Mcg::get();
    let fbe = match mcg.clock() {
        Clock::Fei(fei) => {
            let osc_token = Osc::get().enable(10);
            fei.use_external(512, OscRange::VeryHigh, osc_token)
        }
        Clock::Fbe(fbe) => fbe,
        Clock::Pbe(pbe) => pbe.disable_pll(),
        Clock::Pee(pee) => pee.bypass_pll().disable_pll(),
    };

    // Next, update the main dividers for our new clock rate
    let mut sim = Sim::get();
    sim.set_dividers(core, bus, flash);
    sim.set_usb_dividers(usb_num, usb_den);

    // Finally, re-enable the PLL at our preferred speed
    fbe.enable_pll(pll_num, pll_den).use_pll();

    // TODO: Setup systick timer
}

/// Early startup for the Teensy LC board
///
/// Disables the watchdog.
///
/// This will be included automatically if you are using the standard
/// Cntrlr runtime. It should be invoked directly as part of startup
/// only if you are overriding Cntrlr runtime behavior.
#[cfg_attr(board = "teensy_lc", export_name = "__cntrlr_board_start")]
pub unsafe extern "C" fn start() {
    use crate::hw::mcu::kinetis::mkl26z64::Watchdog;
    Watchdog::get().disable();
}

/// Late startup for the Teensy LC board.
///
/// Sets the processor clock and enables interrupts and
/// exceptions.
///
/// This will be included automatically if you are using the standard
/// Cntrlr runtime. It should be invoked directly as part of startup
/// only if you are overriding Cntrlr runtime behavior.
#[cfg_attr(board = "teensy_lc", export_name = "__cntrlr_board_init")]
pub unsafe extern "C" fn init() {
    // Switch systick to core clock and enable the systick
    // interrupt. The rest of the systick configuration happens as
    // part of setting the clock.
    // SysTick::get().use_core_clock(true);
    // SysTick::get().enable_interrupt(true);

    set_clock(48_000_000);

    const NVIC_ISER: *mut u32 = 0xE000_E100 as *mut _;

    for intr in &[12, 13, 14] {
        let reg = intr / 32;
        let bit = intr % 32;

        core::ptr::write_volatile(NVIC_ISER.add(reg), 1 << bit);
    }

    // Enable bus + usage faults
    const SHCSR: *mut u32 = 0xE000_ED24 as *mut _;
    let mut shcsr = core::ptr::read_volatile(SHCSR);
    shcsr |= 0x60000;
    core::ptr::write_volatile(SHCSR, shcsr);
}

use crate::runtime::unused_interrupt;

/// The Teensy LC interrupt table
///
/// This will automatically be included as the standard interrupt
/// table when this board is selected.
#[cfg_attr(board = "teensy_lc", link_section = ".__CNTRLR_INTERRUPTS")]
#[cfg_attr(board = "teensy_lc", export_name = "__cntrlr_interrupts")]
pub static INTERRUPTS: [unsafe extern "C" fn(); 32] = [
    unused_interrupt,  // 000
    unused_interrupt,  // 001
    unused_interrupt,  // 002
    unused_interrupt,  // 003
    unused_interrupt,  // 004
    unused_interrupt,  // 005
    unused_interrupt,  // 006
    unused_interrupt,  // 007
    unused_interrupt,  // 008
    unused_interrupt,  // 009
    unused_interrupt,  // 010
    unused_interrupt,  // 011
    io::serial_1_intr, // 012
    io::serial_2_intr, // 013
    io::serial_3_intr, // 014
    unused_interrupt,  // 015
    unused_interrupt,  // 016
    unused_interrupt,  // 017
    unused_interrupt,  // 018
    unused_interrupt,  // 019
    unused_interrupt,  // 020
    unused_interrupt,  // 021
    unused_interrupt,  // 022
    unused_interrupt,  // 023
    unused_interrupt,  // 024
    unused_interrupt,  // 025
    unused_interrupt,  // 026
    unused_interrupt,  // 027
    unused_interrupt,  // 028
    unused_interrupt,  // 029
    unused_interrupt,  // 030
    unused_interrupt,  // 031
];
