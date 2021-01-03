// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Board-specific functionality for the Teensy LC

use super::teensy_common::SetClockError;
use crate::hw::mcu::kinetis::mkl26z64::{
    Clock, Mcg, Osc, OscRange, PeripheralClockSource, Sim, SysTick, UartClockSource, UsbClockSource,
};
use core::{
    ptr::write_volatile,
    sync::atomic::{AtomicUsize, Ordering},
};

pub mod digital;
pub mod io;
pub mod time;

static PLL_FREQ: AtomicUsize = AtomicUsize::new(0);
static CPU_FREQ: AtomicUsize = AtomicUsize::new(0);
static BUS_FREQ: AtomicUsize = AtomicUsize::new(0);

/// Set the clock for this board, in Hz.
///
/// Valid values are 48, 32, 24, 16, 12, 8, 6, 4, or 3 MHz
pub fn set_clock(clock: usize) -> Result<(), SetClockError> {
    // Unlike the other Kinetis MCUs, on this one the bus clock is
    // derived from the CPU clock, so its dividers are calculated
    // slightly differently.
    let (core, bus, pll_num, pll_den) = match clock {
        48_000_000 => (2, 2, 24, 4),
        32_000_000 => (3, 2, 24, 4),
        24_000_000 => (4, 1, 24, 4),
        16_000_000 => (6, 1, 24, 4),
        12_000_000 => (8, 1, 24, 4),
        8_000_000 => (12, 1, 24, 4),
        6_000_000 => (16, 1, 24, 4),
        _ => return Err(SetClockError::InvalidClockRate),
    };

    let mut mcg = Mcg::get().ok_or(SetClockError::McgInUse)?;
    let mut sim = Sim::get().ok_or(SetClockError::SimInUse)?;

    // First, switch back to a slow clock so it's safe to update dividers
    let fbe = match mcg.clock() {
        Clock::Fei(fei) => {
            let osc_token = Osc::get()
                .ok_or(SetClockError::OscInUse)?
                .enable(10)
                .map_err(SetClockError::Osc)?;
            fei.use_external(512, OscRange::VeryHigh, Some(osc_token))
                .map_err(SetClockError::Mcg)?
        }
        Clock::Fbe(fbe) => fbe,
        Clock::Pbe(pbe) => pbe.disable_pll(),
        Clock::Pee(pee) => pee.bypass_pll().disable_pll(),
    };

    // Next, update the main dividers for our new clock rate
    sim.set_dividers(core, bus);

    // Finally, re-enable the PLL at our preferred speed
    fbe.enable_pll(pll_num, pll_den)
        .map_err(SetClockError::Mcg)?
        .use_pll();

    // Switch peripherals over to the PLL
    sim.set_usb_source(UsbClockSource::PllFll);
    sim.set_uart0_source(Some(UartClockSource::PllFll));
    sim.set_peripheral_source(PeripheralClockSource::Pll);

    // Reset SysTick for new clock rate.
    if let Some(mut systick) = SysTick::get() {
        systick.enable(false);
        let reload = (clock / 1000) - 1;
        systick.set_reload_value(reload as u32);
        systick.set_current_value(0);
        systick.enable(true);
    }

    PLL_FREQ.store(clock * core as usize, Ordering::Relaxed);
    CPU_FREQ.store(clock, Ordering::Relaxed);
    BUS_FREQ.store(clock * core as usize / bus as usize, Ordering::Relaxed);
    Ok(())
}

/// Early startup for the Teensy LC board
///
/// Disables the watchdog.
///
/// This will be included automatically if you are using the standard
/// Cntrlr runtime. It should be invoked directly as part of startup
/// only if you are overriding Cntrlr runtime behavior.
///
/// # Safety
/// This function unsafely accesses the watchdog peripheral.
#[cfg_attr(board = "teensy_lc", export_name = "__cntrlr_board_start")]
pub unsafe extern "C" fn start() {
    Sim::get()
        .expect("Could not acquire SIM to disable watchdog")
        .disable_cop();
}

/// Late startup for the Teensy LC board.
///
/// Sets the processor clock and enables interrupts and
/// exceptions.
///
/// This will be included automatically if you are using the standard
/// Cntrlr runtime. It should be invoked directly as part of startup
/// only if you are overriding Cntrlr runtime behavior.
///
/// # Safety
/// This function unsafely accesses the NVIC peripheral.
#[cfg_attr(board = "teensy_lc", export_name = "__cntrlr_board_init")]
pub unsafe extern "C" fn init() {
    // Switch systick to core clock and enable the systick
    // interrupt. The rest of the systick configuration happens as
    // part of setting the clock.
    if let Some(mut systick) = SysTick::get() {
        systick.use_core_clock(true);
        systick.enable_interrupt(true);
    }

    set_clock(48_000_000).expect("Could not set core clock at init");

    // TODO: Create a peripheral for the NVIC
    const NVIC_ISER: *mut u32 = 0xE000_E100 as *mut _;
    for intr in &[12, 13, 14] {
        let reg = intr / 32;
        let bit = intr % 32;

        write_volatile(NVIC_ISER.add(reg), 1 << bit);
    }
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

/// The Teensy LC exception table
///
/// This will automatically be included as the standard interrupt
/// table when this board is selected.
#[cfg_attr(board = "teensy_lc", link_section = ".__CNTRLR_EXCEPTIONS")]
#[cfg_attr(board = "teensy_lc", export_name = "__cntrlr_exceptions")]
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
    time::systick_intr,
];
