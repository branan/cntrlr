// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Digital pin support specific to the Teensy LC

use crate::{
    digital::PinMode,
    hw::{
        board::teensy_common::digital::{ModeOp, PinOp, ReadOp, WriteOp},
        mcu::kinetis::mkl26z64::{Port, Sim},
    },
    sync::Once,
};

/// Invoke an operation on a pin.
///
/// This abstracts the mapping of Teensy pins to MCU pins, allowing
/// operations to be defined generically and then invoked for any
/// given pin.
#[inline]
pub fn pin_op<Op: PinOp>(pin: usize, arg: Op::Arg) -> Option<Op::Result> {
    match pin {
        0 => Op::do_op(port_b().pin::<16>(), arg),
        1 => Op::do_op(port_b().pin::<17>(), arg),
        2 => Op::do_op(port_d().pin::<0>(), arg),
        3 => Op::do_op(port_a().pin::<1>(), arg),
        4 => Op::do_op(port_a().pin::<2>(), arg),
        5 => Op::do_op(port_d().pin::<7>(), arg),
        6 => Op::do_op(port_d().pin::<4>(), arg),
        7 => Op::do_op(port_d().pin::<2>(), arg),
        8 => Op::do_op(port_d().pin::<3>(), arg),
        9 => Op::do_op(port_c().pin::<3>(), arg),
        10 => Op::do_op(port_c().pin::<4>(), arg),
        11 => Op::do_op(port_c().pin::<6>(), arg),
        12 => Op::do_op(port_c().pin::<7>(), arg),
        13 => Op::do_op(port_c().pin::<5>(), arg),
        14 => Op::do_op(port_d().pin::<1>(), arg),
        15 => Op::do_op(port_c().pin::<0>(), arg),
        16 => Op::do_op(port_b().pin::<0>(), arg),
        17 => Op::do_op(port_b().pin::<1>(), arg),
        18 => Op::do_op(port_b().pin::<3>(), arg),
        19 => Op::do_op(port_b().pin::<2>(), arg),
        20 => Op::do_op(port_d().pin::<5>(), arg),
        21 => Op::do_op(port_d().pin::<6>(), arg),
        22 => Op::do_op(port_c().pin::<1>(), arg),
        23 => Op::do_op(port_c().pin::<2>(), arg),
        24 => Op::do_op(port_e().pin::<20>(), arg),
        25 => Op::do_op(port_e().pin::<21>(), arg),
        26 => Op::do_op(port_e().pin::<30>(), arg),
        _ => None,
    }
}

/// Set a digital pin high or low.
///
/// The digital pins on the Teensy LC are 5V tolerant, but use
/// 3.3V thresholds.
///
/// If `pin` is not a valid pin, does nothing.
///
/// Interactions with this method may be unpredictable if the pin
/// is not in an output mode or is in use by another module. In
/// particular, whether the write takes effect when the pin
/// becomes a digital output is MCU-specific and should not be
/// relied upon.
#[inline]
pub fn digital_write(pin: usize, value: bool) {
    pin_op::<WriteOp>(pin, value);
}

/// Read the state of a digital pin.
///
/// The digital pins on the Teensy LC output 3.3V.
///
/// If `pin` is not a valid pin, returns `false`
///
/// The return value is implementation-specific and should not be
/// relied upon in the following cases:
/// * The pin is not set as a digital input
/// * The pin is in use by a different module
/// * The pin number is outside the range of pins on the board.
#[inline]
pub fn digital_read(pin: usize) -> bool {
    pin_op::<ReadOp>(pin, ()).unwrap_or(false)
}

/// Set a pin as a digital input or output
///
/// If `pin` is not a valid pin, does nothing.
///
/// Interactions with this method may be unpredictable if the pin
/// is in use by another module. In particular, whether or not
/// changes made by this method will take effect when the other
/// module releases the pin is implementation specific and should
/// not be relied upon.
#[inline]
pub fn pin_mode(pin: usize, mode: PinMode) {
    pin_op::<ModeOp>(pin, mode);
}

/// Port A
///
/// The global instance of PORT A, used to share port ownership among
/// different board modules.
pub fn port_a() -> &'static Port<0> {
    static PORT: Once<Port<0>> = Once::new();
    PORT.get_or_init(|| Sim::get().enable_peripheral())
}

/// Port B
///
/// The global instance of PORT B, used to share port ownership among
/// different board modules.
pub fn port_b() -> &'static Port<1> {
    static PORT: Once<Port<1>> = Once::new();
    PORT.get_or_init(|| Sim::get().enable_peripheral())
}

/// Port C
///
/// The global instance of PORT C, used to share port ownership among
/// different board modules.
pub fn port_c() -> &'static Port<2> {
    static PORT: Once<Port<2>> = Once::new();
    PORT.get_or_init(|| Sim::get().enable_peripheral())
}

/// Port D
///
/// The global instance of PORT D, used to share port ownership among
/// different board modules.
pub fn port_d() -> &'static Port<3> {
    static PORT: Once<Port<3>> = Once::new();
    PORT.get_or_init(|| Sim::get().enable_peripheral())
}

/// Port E
///
/// The global instance of PORT E, used to share port ownership among
/// different board modules.
pub fn port_e() -> &'static Port<4> {
    static PORT: Once<Port<4>> = Once::new();
    PORT.get_or_init(|| Sim::get().enable_peripheral())
}
