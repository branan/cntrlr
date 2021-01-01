// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Digital pin functionality shared between the various Teensy 3.x boards

use crate::{digital::PinMode, hw::mcu::kinetis::peripheral::port};

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
    fn op<M, const N: usize, const P: usize>(
        pin: port::Pin<'_, M, N, P>,
        arg: Self::Arg,
    ) -> Self::Result;

    /// The operation, optionally performed on an optional pin.
    #[inline(always)]
    fn do_op<M, const N: usize, const P: usize>(
        pin: Option<port::Pin<'_, M, N, P>>,
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

    #[inline(always)]
    fn op<M, const N: usize, const P: usize>(pin: port::Pin<'_, M, N, P>, value: bool) {
        pin.into_gpio().write(value);
    }
}

/// An operation to read a pin as high or low
pub struct ReadOp;
impl PinOp for ReadOp {
    type Arg = ();
    type Result = bool;

    #[inline(always)]
    fn op<M, const N: usize, const P: usize>(pin: port::Pin<'_, M, N, P>, _: ()) -> bool {
        pin.into_gpio().read()
    }
}

/// An operation to set a pin's [mode](`PinMode`)
pub struct ModeOp;
impl PinOp for ModeOp {
    type Arg = PinMode;
    type Result = ();

    #[inline(always)]
    fn op<M, const N: usize, const P: usize>(pin: port::Pin<'_, M, N, P>, mode: PinMode) {
        let mut pin = pin.into_gpio();
        match mode {
            PinMode::Input => {
                pin.set_pull(None);
                pin.set_output(false);
            }
            PinMode::PulledInput(pull) => {
                pin.set_pull(Some(pull));
                pin.set_output(false);
            }
            PinMode::Output => {
                pin.set_open_drain(false);
                pin.set_output(true);
                pin.set_pull(None);
            }
            PinMode::OpenDrainOutput => {
                pin.set_open_drain(true);
                pin.set_output(true);
                pin.set_pull(None);
            }
        }
    }
}
