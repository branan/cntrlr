// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Common board functionality for the Teensy 3.x series

pub mod digital;
pub mod io;
pub mod time;

/// Error type for Teensy 3.x clock setting functions.
#[derive(Debug)]
#[non_exhaustive]
pub enum SetClockError {
    /// The core clock cannot be changed because the clock generator is in use.
    McgInUse,

    /// The core clock cannot be changed because the integration module is in use.
    SimInUse,

    /// The core clock cannot be changed because the oscillator is in use.
    OscInUse,

    /// The core clock cannot be set because the requested speed is invalid.
    InvalidClockRate,

    /// There was an error initializing the oscillator.
    Osc(crate::hw::mcu::kinetis::peripheral::osc::Error),

    /// There was an error initializing the mcg
    Mcg(crate::hw::mcu::kinetis::peripheral::mcg::Error),
}

const FLASH_SECURITY: u8 = 0xDE;
const FLASH_OPTIONS: u8 = 0xF9;

/// The flash configuration
///
/// This will automatically be included as the standard flash
/// configuration when a board using this MCU is selected.
#[cfg_attr(
    any(
        board = "teensy_30",
        board = "teensy_32",
        board = "teensy_35",
        board = "teensy_36",
        board = "teensy_lc"
    ),
    link_section = ".__CNTRLR_FLASH_CONFIG"
)]
#[cfg_attr(
    any(
        board = "teensy_30",
        board = "teensy_32",
        board = "teensy_35",
        board = "teensy_36",
        board = "teensy_lc"
    ),
    export_name = "__cntrlr_flash_configuration"
)]
pub static FLASH_CONFIGURATION: [u8; 16] = [
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    0xFF,
    FLASH_SECURITY,
    FLASH_OPTIONS,
    0xFF,
    0xFF,
];
