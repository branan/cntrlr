// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Oscillator

use crate::{register::Register, sync::Flag};
use bit_field::BitField;
use core::sync::atomic::Ordering;

/// Error for [`Osc::enable`]
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The provided capacitance value is invalid
    InvalidCapacitance,

    /// The oscillator was already enabled
    AlreadyEnabled,
}

#[repr(C)]
struct OscRegs {
    cr: Register<u8>,
}

/// The handle to the OSC
pub struct Osc {
    regs: &'static mut OscRegs,
}

/// An opaque token indicating that the Oscillator has been
/// configured and is not in use.
pub struct OscToken {
    _private: (),
}

static LOCK: Flag = Flag::new(false);

impl Osc {
    /// Get the handle to the OSC
    ///
    /// Returns `None` if the Osc is already in use.
    pub fn get() -> Option<Self> {
        unsafe {
            if LOCK.swap(true, Ordering::Acquire) {
                None
            } else {
                Some(Self {
                    regs: &mut *(0x4006_5000 as *mut _),
                })
            }
        }
    }

    /// Enable the internal oscillator with the specified capacitance
    pub fn enable(&mut self, capacitance: u8) -> Result<OscToken, Error> {
        if capacitance % 2 == 1 || capacitance > 30 {
            return Err(Error::InvalidCapacitance);
        }

        let mut cr = self.regs.cr.read();
        if cr.get_bit(7) {
            Err(Error::AlreadyEnabled)
        } else {
            cr.set_bit(7, true);
            cr.set_bit(3, capacitance.get_bit(1));
            cr.set_bit(2, capacitance.get_bit(2));
            cr.set_bit(1, capacitance.get_bit(3));
            cr.set_bit(0, capacitance.get_bit(4));

            self.regs.cr.write(cr);

            Ok(OscToken { _private: () })
        }
    }
}

impl Drop for Osc {
    fn drop(&mut self) {
        LOCK.store(false, Ordering::Release);
    }
}
