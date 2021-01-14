// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Board specific functionality for the Teensy 3.6

use super::teensy_common::SetClockError;
use crate::hw::mcu::kinetis::mk66fx1m0::{
    Clock, Mcg, Osc, OscRange, PeripheralClockSource, Sim, Smc, SysTick, UsbClockSource, Watchdog,
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
/// Valid values are 256, 240, 216, 192, 180, 168, 144, 120, 96, 72,
/// 48, 32, 24, 16, or 12 MHz.
///
/// Values above 120MHz will cause the board to function incorrectly
/// due to missing HSRUN support.
pub fn set_clock(clock: usize) -> Result<(), SetClockError> {
    // MK66 has a secret divide by two on the PLL output, so our
    // dividers look a little weird
    let (core, bus, flex, flash, usb_num, usb_den, pll_num, pll_den) = match clock {
        256_000_000 => (1, 4, 4, 8, 1, None, 32, 1),
        240_000_000 => (1, 4, 4, 8, 1, Some(5), 30, 1),
        216_000_000 => (1, 4, 4, 8, 1, None, 27, 1),
        192_000_000 => (1, 4, 4, 8, 1, Some(4), 24, 1),
        180_000_000 => (1, 3, 3, 8, 1, None, 45, 2),
        168_000_000 => (1, 3, 3, 7, 1, None, 21, 1),
        144_000_000 => (1, 3, 3, 6, 1, None, 18, 1),
        120_000_000 => (1, 2, 3, 5, 2, Some(5), 30, 2),
        96_000_000 => (1, 2, 2, 4, 1, Some(2), 24, 2),
        72_000_000 => (2, 2, 2, 3, 2, Some(3), 18, 2),
        48_000_000 => (2, 2, 2, 4, 1, Some(2), 24, 2),
        32_000_000 => (3, 3, 3, 6, 1, Some(2), 24, 2),
        24_000_000 => (4, 4, 4, 4, 1, Some(2), 24, 2),
        16_000_000 => (6, 6, 6, 6, 1, Some(2), 24, 2),
        12_000_000 => (8, 8, 8, 8, 1, Some(2), 24, 2),
        _ => return Err(SetClockError::InvalidClockRate),
    };

    let mut mcg = Mcg::get().ok_or(SetClockError::McgInUse)?;
    let mut sim = Sim::get().ok_or(SetClockError::SimInUse)?;
    let mut smc = Smc::get().ok_or(SetClockError::SmcInUse)?;

    if clock > 120_000_000 {
        smc.enter_hsrun();
    }

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
    sim.set_dividers(core, bus, flex, flash);
    sim.set_usb_source(UsbClockSource::PllFll);
    if let Some(usb_den) = usb_den {
        sim.set_peripheral_source(PeripheralClockSource::Pll);
        sim.set_usb_dividers(usb_num, usb_den);
    } else {
        sim.set_peripheral_source(PeripheralClockSource::Irc48);
        sim.set_usb_dividers(1, 1);
    }

    // Finally, re-enable the PLL at our preferred speed
    fbe.enable_pll(pll_num, pll_den)
        .map_err(SetClockError::Mcg)?
        .use_pll();

    if clock <= 120_000_000 {
        smc.exit_hsrun();
    }

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
///
/// # Safety
/// This function unsafely accesses the watchdog peripheral.
#[cfg_attr(board = "teensy_36", export_name = "__cntrlr_board_start")]
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
///
/// # Safety
/// This function unsafely accesses the NVIC peripheral.
#[cfg_attr(board = "teensy_36", export_name = "__cntrlr_board_init")]
pub unsafe extern "C" fn init() {
    // Switch systick to core clock and enable the systick
    // interrupt. The rest of the systick configuration happens as
    // part of setting the clock.
    if let Some(mut systick) = SysTick::get() {
        systick.use_core_clock(true);
        systick.enable_interrupt(true);
    }

    if let Some(mut smc) = Smc::get() {
        smc.allow_all_modes();
    }

    set_clock(120_000_000).expect("Could not set core clock at init");

    // TODO: Create an NVIC peripheral
    const NVIC_ISER: *mut u32 = 0xE000_E100 as *mut _;
    for intr in &[26, 27, 31, 33, 35, 37, 65, 66, 68] {
        let reg = intr / 32;
        let bit = intr % 32;

        write_volatile(NVIC_ISER.add(reg), 1 << bit);
    }
}

use crate::runtime::unused_interrupt;

/// The Teensy 3.6 interrupt table
///
/// This will automatically be included as the standard interrupt
/// table when this board is selected.
#[cfg_attr(board = "teensy_36", link_section = ".__CNTRLR_INTERRUPTS")]
#[cfg_attr(board = "teensy_36", export_name = "__cntrlr_interrupts")]
pub static INTERRUPTS: [unsafe extern "C" fn(); 100] = [
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
    io::spi_1_intr,    // 026
    io::spi_2_intr,    // 027
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
    io::spi_3_intr,    // 065
    io::serial_5_intr, // 066
    unused_interrupt,  // 067
    unused_interrupt,  // 068
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
    unused_interrupt,  // 086
    unused_interrupt,  // 087
    unused_interrupt,  // 088
    unused_interrupt,  // 089
    unused_interrupt,  // 090
    unused_interrupt,  // 091
    unused_interrupt,  // 092
    unused_interrupt,  // 093
    unused_interrupt,  // 094
    unused_interrupt,  // 095
    unused_interrupt,  // 096
    unused_interrupt,  // 097
    unused_interrupt,  // 009
    unused_interrupt,  // 099
];

/// The Teensy 3.6 exception table
///
/// This will automatically be included as the standard interrupt
/// table when this board is selected.
#[cfg_attr(board = "teensy_36", link_section = ".__CNTRLR_EXCEPTIONS")]
#[cfg_attr(board = "teensy_36", export_name = "__cntrlr_exceptions")]
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
