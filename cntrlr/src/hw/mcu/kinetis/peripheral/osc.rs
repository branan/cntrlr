// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Oscillator

use crate::{register::Register, sync::Flag};
use bit_field::BitField;
use core::{marker::PhantomData, sync::atomic::Ordering};

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
pub struct Osc<M> {
    regs: &'static mut OscRegs,
    _mcu: PhantomData<M>,
}

/// An opaque token indicating that the Oscillator has been
/// configured and is not in use.
pub struct OscToken {
    _private: (),
}

static LOCK: Flag = Flag::new(false);

macro_rules! get {
    ($m:ident, $s:literal) => {
        #[cfg(any(doc, mcu = $s))]
        #[cfg_attr(feature = "doc-cfg", doc(cfg(mcu = $s)))]
        impl super::Peripheral for Osc<super::super::$m> {
            fn get() -> Option<Self> {
                unsafe {
                    if LOCK.swap(true, Ordering::Acquire) {
                        None
                    } else {
                        Some(Self {
                            regs: &mut *(0x4006_5000 as *mut _),
                            _mcu: PhantomData,
                        })
                    }
                }
            }
        }
    };
}

get!(Mk20Dx128, "mk20dx128");
get!(Mk20Dx256, "mk20dx256");
get!(Mk64Fx512, "mk64fx512");
get!(Mk66Fx1M0, "mk66fx1m0");
get!(Mkl26Z64, "mkl26z64");

impl<M> Osc<M>
where
    Osc<M>: super::Peripheral,
{
    /// Get the handdle to the OSC
    ///
    /// Returns `None` if the OSC is already in use.
    pub fn get() -> Option<Self> {
        super::Peripheral::get()
    }
}

impl<M> Osc<M> {
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

impl<M> Drop for Osc<M> {
    fn drop(&mut self) {
        LOCK.store(false, Ordering::Release);
    }
}
