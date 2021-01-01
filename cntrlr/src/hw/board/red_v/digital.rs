// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Digital pin support specific to the Sparkfun Red V

use crate::{
    digital::{PinMode, Pull},
    hw::mcu::sifive::fe310g002::{Gpio, Pin},
    sync::Once,
};

/// An operation on a pin.
///
/// This abstracts the mapping of Teensy pins to MCU pins, allowing
/// operations to be defined generically and then invoked for any
/// given pin.
pub trait PinOp {
    /// The type of argument the operation expects
    type Arg;

    /// The result of the operation
    type Result;

    /// The operation, performed on a single pin
    fn op<const N: usize, const P: usize>(pin: Pin<'_, N, P>, arg: Self::Arg) -> Self::Result;

    /// The operation, optionally performed on an optional pin.
    fn do_op<const N: usize, const P: usize>(
        pin: Option<Pin<'_, N, P>>,
        arg: Self::Arg,
    ) -> Option<Self::Result> {
        if let Some(pin) = pin {
            Some(Self::op(pin, arg))
        } else {
            None
        }
    }
}

/// An operation to write a pin as high or low
pub struct WriteOp;
impl PinOp for WriteOp {
    type Arg = bool;
    type Result = ();

    fn op<const N: usize, const P: usize>(pin: Pin<'_, N, P>, value: bool) {
        pin.into_gpio().write(value);
    }
}

/// An operation to read a pin as high or low
pub struct ReadOp;
impl PinOp for ReadOp {
    type Arg = ();
    type Result = bool;

    #[inline]
    fn op<const N: usize, const P: usize>(pin: Pin<'_, N, P>, _: ()) -> bool {
        pin.into_gpio().read()
    }
}

/// An operation to set a pin's [mode](`PinMode`)
pub struct ModeOp;
impl PinOp for ModeOp {
    type Arg = PinMode;
    type Result = ();

    #[inline]
    fn op<const N: usize, const P: usize>(pin: Pin<'_, N, P>, mode: PinMode) {
        let mut pin = pin.into_gpio();
        match mode {
            PinMode::Input => {
                pin.enable_pullup(false);
                pin.set_output(false);
            }
            PinMode::PulledInput(pull) => {
                match pull {
                    Pull::Up => pin.enable_pullup(true),
                    _ => {}
                }
                pin.set_output(false);
            }
            PinMode::Output => {
                pin.set_output(true);
                pin.enable_pullup(false);
            }
            PinMode::OpenDrainOutput => {
                pin.set_output(true);
                pin.enable_pullup(false);
            }
        }
    }
}

/// Invoke an operation on a pin.
///
/// This abstracts the mapping of Teensy pins to MCU pins, allowing
/// operations to be defined generically and then invoked for any
/// given pin.
pub fn pin_op<Op: PinOp>(pin: usize, arg: Op::Arg) -> Option<Op::Result> {
    match pin {
        0 => Op::do_op(gpio().pin::<16>(), arg),
        1 => Op::do_op(gpio().pin::<17>(), arg),
        2 => Op::do_op(gpio().pin::<18>(), arg),
        3 => Op::do_op(gpio().pin::<19>(), arg),
        4 => Op::do_op(gpio().pin::<20>(), arg),
        5 => Op::do_op(gpio().pin::<21>(), arg),
        6 => Op::do_op(gpio().pin::<22>(), arg),
        7 => Op::do_op(gpio().pin::<23>(), arg),
        8 => Op::do_op(gpio().pin::<0>(), arg),
        9 => Op::do_op(gpio().pin::<1>(), arg),
        10 => Op::do_op(gpio().pin::<2>(), arg),
        11 => Op::do_op(gpio().pin::<3>(), arg),
        12 => Op::do_op(gpio().pin::<4>(), arg),
        13 => Op::do_op(gpio().pin::<5>(), arg),
        // Pin 14 is absent on this board
        15 => Op::do_op(gpio().pin::<9>(), arg),
        16 => Op::do_op(gpio().pin::<10>(), arg),
        17 => Op::do_op(gpio().pin::<11>(), arg),
        18 => Op::do_op(gpio().pin::<12>(), arg),
        19 => Op::do_op(gpio().pin::<13>(), arg),
        _ => None,
    }
}

/// Set a digital pin high or low.
///
/// The digital pins on the Red-V are 5V tolerant, but use
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
/// The digital pins on the Red-V output 3.3V.
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
/// The Red-V does not support pulldown or open-drain
/// configurations. Attempting to use a pull-down will create an
/// unpulled input. Attempting to use open-drain will result in a
/// push-pull output.
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

/// The GPIO
///
/// The global instance of the GPIO, used to share ownership among
/// different board modules.
pub fn gpio() -> &'static Gpio<0> {
    static PORT: Once<Gpio<0>> = Once::new();
    PORT.get_or_init(|| Gpio::get())
}
