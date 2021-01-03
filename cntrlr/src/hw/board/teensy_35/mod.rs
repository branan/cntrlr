// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Board-specific functionality for the Teensy 3.5

use super::teensy_common::SetClockError;
use crate::hw::mcu::kinetis::mk64fx512::{
    Clock, Mcg, Osc, OscRange, PeripheralClockSource, Sim, SysTick, UsbClockSource, Watchdog,
};
use core::{
    ptr::write_volatile,
    sync::atomic::{AtomicUsize, Ordering},
};

pub mod digital;
pub mod io;
pub mod time;

static CPU_FREQ: AtomicUsize = AtomicUsize::new(0);
static BUS_FREQ: AtomicUsize = AtomicUsize::new(0);

/// Set the clock for this board, in Hz.
///
/// Valid values are 120, 96, 72, 48, 32, 24, 16, 12, 8, 6, 4, or 3 MHz
pub fn set_clock(clock: usize) -> Result<(), SetClockError> {
    let (core, bus, flex, flash, usb_num, usb_den, pll_num, pll_den) = match clock {
        120_000_000 => (1, 2, 3, 5, 2, 5, 30, 4),
        96_000_000 => (1, 2, 3, 4, 1, 2, 24, 4),
        72_000_000 => (1, 2, 2, 3, 2, 3, 27, 6),
        48_000_000 => (1, 1, 2, 2, 1, 1, 24, 8),
        32_000_000 => (3, 3, 3, 3, 1, 2, 24, 4),
        24_000_000 => (2, 2, 2, 2, 1, 1, 24, 8),
        16_000_000 => (3, 3, 3, 3, 1, 1, 24, 8),
        12_000_000 => (4, 4, 4, 4, 1, 1, 24, 8),
        8_000_000 => (6, 6, 6, 6, 1, 1, 24, 8),
        6_000_000 => (8, 8, 8, 8, 1, 1, 24, 8),
        4_000_000 => (12, 12, 12, 12, 1, 1, 24, 8),
        3_000_000 => (16, 16, 16, 16, 1, 1, 24, 8),
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
                .map_err(|e| SetClockError::Osc(e))?;
            fei.use_external(512, OscRange::VeryHigh, Some(osc_token))
                .map_err(|e| SetClockError::Mcg(e))?
        }
        Clock::Fbe(fbe) => fbe,
        Clock::Pbe(pbe) => pbe.disable_pll(),
        Clock::Pee(pee) => pee.bypass_pll().disable_pll(),
    };

    // Next, update the main dividers for our new clock rate
    sim.set_dividers(core, bus, flex, flash);
    sim.set_usb_dividers(usb_num, usb_den);

    // Finally, re-enable the PLL at our preferred speed
    fbe.enable_pll(pll_num, pll_den)
        .map_err(|e| SetClockError::Mcg(e))?
        .use_pll();

    // Switch peripherals over to the PLL
    sim.set_usb_source(UsbClockSource::PllFll);
    sim.set_peripheral_source(PeripheralClockSource::Pll);

    // Reset SysTick for new clock rate.
    if let Some(mut systick) = SysTick::get() {
        systick.enable(false);
        let reload = (clock / 1000) - 1;
        systick.set_reload_value(reload as u32);
        systick.set_current_value(0);
        systick.enable(true);
    }

    CPU_FREQ.store(clock, Ordering::Relaxed);
    BUS_FREQ.store(clock * core as usize / bus as usize, Ordering::Relaxed);
    Ok(())
}

/// Early startup for the Teensy 3.5 board
///
/// Disables the watchdog.
///
/// This will be included automatically if you are using the standard
/// Cntrlr runtime. It should be invoked directly as part of startup
/// only if you are overriding Cntrlr runtime behavior.
#[cfg_attr(board = "teensy_35", export_name = "__cntrlr_board_start")]
pub unsafe extern "C" fn start() {
    Watchdog::get().disable();
}

/// Late startup for the Teensy 3.5 board.
///
/// Sets the processor clock and enables interrupts and
/// exceptions.
///
/// This will be included automatically if you are using the standard
/// Cntrlr runtime. It should be invoked directly as part of startup
/// only if you are overriding Cntrlr runtime behavior.
#[cfg_attr(board = "teensy_35", export_name = "__cntrlr_board_init")]
pub unsafe extern "C" fn init() {
    // Switch systick to core clock and enable the systick
    // interrupt. The rest of the systick configuration happens as
    // part of setting the clock.
    if let Some(mut systick) = SysTick::get() {
        systick.use_core_clock(true);
        systick.enable_interrupt(true);
    }

    set_clock(120_000_000).expect("Could not set core clock at init");

    // TODO: Create an NVIC peripheral
    const NVIC_ISER: *mut u32 = 0xE000_E100 as *mut _;
    for intr in &[31, 33, 35, 37, 66, 68] {
        let reg = intr / 32;
        let bit = intr % 32;

        write_volatile(NVIC_ISER.add(reg), 1 << bit);
    }
}

use crate::runtime::unused_interrupt;

/// The Teensy 3.5 interrupt table
///
/// This will automatically be included as the standard interrupt
/// table when this board is selected.
#[cfg_attr(board = "teensy_35", link_section = ".__CNTRLR_INTERRUPTS")]
#[cfg_attr(board = "teensy_35", export_name = "__cntrlr_interrupts")]
pub static INTERRUPTS: [unsafe extern "C" fn(); 86] = [
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
    unused_interrupt,  // 012
    unused_interrupt,  // 013
    unused_interrupt,  // 014
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
    io::serial_1_intr, // 031
    unused_interrupt,  // 032
    io::serial_2_intr, // 033
    unused_interrupt,  // 034
    io::serial_3_intr, // 035
    unused_interrupt,  // 036
    io::serial_4_intr, // 037
    unused_interrupt,  // 038
    unused_interrupt,  // 039
    unused_interrupt,  // 040
    unused_interrupt,  // 041
    unused_interrupt,  // 042
    unused_interrupt,  // 043
    unused_interrupt,  // 044
    unused_interrupt,  // 045
    unused_interrupt,  // 046
    unused_interrupt,  // 047
    unused_interrupt,  // 048
    unused_interrupt,  // 049
    unused_interrupt,  // 050
    unused_interrupt,  // 051
    unused_interrupt,  // 052
    unused_interrupt,  // 053
    unused_interrupt,  // 054
    unused_interrupt,  // 055
    unused_interrupt,  // 056
    unused_interrupt,  // 057
    unused_interrupt,  // 058
    unused_interrupt,  // 059
    unused_interrupt,  // 060
    unused_interrupt,  // 061
    unused_interrupt,  // 062
    unused_interrupt,  // 063
    unused_interrupt,  // 064
    unused_interrupt,  // 065
    io::serial_5_intr, // 066
    unused_interrupt,  // 067
    io::serial_6_intr, // 068
    unused_interrupt,  // 069
    unused_interrupt,  // 070
    unused_interrupt,  // 071
    unused_interrupt,  // 072
    unused_interrupt,  // 073
    unused_interrupt,  // 074
    unused_interrupt,  // 075
    unused_interrupt,  // 076
    unused_interrupt,  // 077
    unused_interrupt,  // 078
    unused_interrupt,  // 079
    unused_interrupt,  // 080
    unused_interrupt,  // 081
    unused_interrupt,  // 082
    unused_interrupt,  // 083
    unused_interrupt,  // 084
    unused_interrupt,  // 085
];

/// The Teensy 3.5 exception table
///
/// This will automatically be included as the standard interrupt
/// table when this board is selected.
#[cfg_attr(board = "teensy_35", link_section = ".__CNTRLR_EXCEPTIONS")]
#[cfg_attr(board = "teensy_35", export_name = "__cntrlr_exceptions")]
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
