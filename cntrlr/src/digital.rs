// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Digital pin functionality for Cntrlr boards

use cntrlr_macros::board_fn;

/// Mode of a digital pin
#[non_exhaustive]
pub enum PinMode {
    /// The pin is an input, without any pull-up or pull-down resistors
    Input,

    /// The pin is an input, with a pull-up or pull-down resistor
    PulledInput(Pull),

    /// The pin is an output, with a push-pull drive
    Output,

    /// The pin is an output, with an open drain (or open collector) drive
    OpenDrainOutput,
}

/// Pull-up or -down configuration
#[non_exhaustive]
pub enum Pull {
    /// The pin is pulled up when no signal is applied
    Up,

    /// The pin is pulled down when no signal is applied
    Down,
}

/// Set a digital pin high or low
///
/// If `pin` is not a valid pin, does nothing.
///
/// Interactions with this method may be unpredictable if the pin
/// is not in an output mode or is in use by another module. In
/// particular, whether the write takes effect when the pin
/// becomes a digital output is MCU-specific and should not be
/// relied upon.
#[board_fn(digital, red_v, teensy_30, teensy_32, teensy_35, teensy_36, teensy_lc)]
pub fn digital_write(pin: usize, value: bool) {}

/// Read the state of a digital pin.
///
/// If `pin` is not a valid pin, returns `false`
///
/// The return value is implementation-specific and should not be
/// relied upon in the following cases:
/// * The pin is not set as a digital input
/// * The pin is in use by a different module
/// * The pin number is outside the range of pins on the board.
#[board_fn(digital, red_v, teensy_30, teensy_32, teensy_35, teensy_36, teensy_lc)]
pub fn digital_read(pin: usize) -> bool {}

/// Set a pin as a digital input or output
///
/// If `pin` is not a valid pin, does nothing.
///
/// Interactions with this method may be unpredictable if the pin
/// is in use by another module. In particular, whether or not
/// changes made by this method will take effect when the other
/// module releases the pin is implementation specific and should
/// not be relied upon.
#[board_fn(digital, red_v, teensy_30, teensy_32, teensy_35, teensy_36, teensy_lc)]
pub fn pin_mode(pin: usize, mode: PinMode) {}
