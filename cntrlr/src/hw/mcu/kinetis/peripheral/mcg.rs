// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Multipurpose Clock Generator

use super::{
    super::{Mk20Dx128, Mk20Dx256, Mk64Fx512, Mk66Fx1M0, Mkl26Z64},
    osc::OscToken,
};
use crate::{
    register::{Register, Reserved},
    sync::Flag,
};
use bit_field::BitField;
use core::{marker::PhantomData, sync::atomic::Ordering};

#[repr(C)]
struct McgRegs {
    c1: Register<u8>,
    c2: Register<u8>,
    c3: Register<u8>,
    c4: Register<u8>,
    c5: Register<u8>,
    c6: Register<u8>,
    s: Register<u8>,
    _reserved_0: Reserved<u8>,
    sc: Register<u8>,
    _reserved_1: Reserved<u8>,
    atcvh: Register<u8>,
    atcvl: Register<u8>,
    c7: Register<u8>,
    c8: Register<u8>,
}

/// The handle to the MCG
pub struct Mcg<M> {
    regs: &'static mut McgRegs,
    _mcu: PhantomData<M>,
}

/// FLL Enabled, Internal reference
///
/// In FEI mode, the system clock is running off of the FLL, which is
/// in turn referenced to the internal low-accuracy oscillator.
///
/// This is the default mode at system reset.
pub struct Fei<'a, M>(&'a mut Mcg<M>);

/// FLL Bypassed, External reference
///
/// In FBE mode, the system clock will be running off of the
/// external reference with a divider as set by the argument to
/// this method. The FLL is enabled, but not in use.
pub struct Fbe<'a, M>(&'a mut Mcg<M>);

/// PLL Bypassed, External reference
///
/// In FBE mode, the system clock will be running off of the
/// external reference with a divider as set by the argument to
/// this method. The PLL is enabled, but not in use.
pub struct Pbe<'a, M>(&'a mut Mcg<M>);

/// PLL Enabled, External reference
///
/// In PEE mode, the system clock is running off of the PLL, which is
/// in turn referenced to the external oscillator.
pub struct Pee<'a, M>(&'a mut Mcg<M>);

/// The current mode of the system clock.
#[non_exhaustive]
pub enum Clock<'a, M> {
    /// FEI mode
    Fei(Fei<'a, M>),

    /// FBE mode
    Fbe(Fbe<'a, M>),

    /// PBE mode
    Pbe(Pbe<'a, M>),

    /// PEE mode
    Pee(Pee<'a, M>),
}

/// Error type for MCG operations
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The specified divider is invalid
    InvalidDivider,
}

static LOCK: Flag = Flag::new(false);

impl<M> Mcg<M> {
    /// Get the handdle to the MCG
    ///
    /// Returns `None` if the MCG is already in use.
    pub fn get() -> Option<Self> {
        unsafe {
            if LOCK.swap(true, Ordering::Acquire) {
                None
            } else {
                Some(Self {
                    regs: &mut *(0x4006_4000 as *mut _),
                    _mcu: PhantomData,
                })
            }
        }
    }

    /// Get the current clock mode
    ///
    /// # Panics
    /// This method will panic if the current clock mode cannot be
    /// represented as a value of [`Clock`]
    pub fn clock(&mut self) -> Clock<M> {
        let source: OscSource = self.regs.c1.read().get_bits(6..8).into();
        let fll_internal = self.regs.c1.read().get_bit(2);
        let pll_enabled = self.regs.c6.read().get_bit(6);

        match (fll_internal, pll_enabled, source) {
            (true, false, OscSource::LockedLoop) => Clock::Fei(Fei(self)),
            (false, false, OscSource::LockedLoop) => panic!("FEE mode not yet supported"),
            (true, false, OscSource::Internal) => panic!("FBI mod not yet supported"),
            (false, false, OscSource::External) => Clock::Fbe(Fbe(self)),
            (_, true, OscSource::External) => Clock::Pbe(Pbe(self)),
            (_, true, OscSource::LockedLoop) => Clock::Pee(Pee(self)),
            _ => panic!("Unknown clock configuration"),
        }
    }
}

impl<M> Drop for Mcg<M> {
    fn drop(&mut self) {
        LOCK.store(false, Ordering::Release);
    }
}

#[derive(PartialEq)]
enum OscSource {
    LockedLoop,
    Internal,
    External,
}

impl Into<u8> for OscSource {
    fn into(self) -> u8 {
        match self {
            Self::LockedLoop => 0,
            Self::Internal => 1,
            Self::External => 2,
        }
    }
}

impl From<u8> for OscSource {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::LockedLoop,
            1 => Self::Internal,
            2 => Self::External,
            3 => Self::LockedLoop,
            _ => panic!("Invalid OscSource value"),
        }
    }
}

/// The speed of the external oscillator.
#[derive(PartialEq, Copy, Clone)]
pub enum OscRange {
    /// 1kHz to 32kHz
    Low,

    /// 1MHz to 8MHz
    High,

    /// 8MHz to 32MHz
    VeryHigh,
}

impl Into<u8> for OscRange {
    fn into(self) -> u8 {
        match self {
            Self::Low => 0,
            Self::High => 1,
            Self::VeryHigh => 2,
        }
    }
}

impl<'a, M> Fei<'a, M> {
    /// Bypass the FLL and use the external reference.
    ///
    /// If `token` is [`Some`], the external reference will be
    /// configured using the internal oscillator. If it is [`None`],
    /// the external reference clock is used directly.
    pub fn use_external(
        self,
        divide: u32,
        range: OscRange,
        token: Option<OscToken>,
    ) -> Result<Fbe<'a, M>, Error> {
        self.0.regs.c2.update(|c2| {
            c2.set_bits(4..6, range.into());
            c2.set_bit(2, token.is_some());
        });

        if token.is_some() {
            // Wait for the oscillator to become enabled.
            while !self.0.regs.s.read().get_bit(1) {}
        }

        let frdiv = if range == OscRange::Low {
            match divide {
                1 => 0,
                2 => 1,
                4 => 2,
                8 => 3,
                16 => 4,
                32 => 5,
                64 => 6,
                128 => 7,
                _ => return Err(Error::InvalidDivider),
            }
        } else {
            match divide {
                32 => 0,
                64 => 1,
                128 => 2,
                256 => 3,
                512 => 4,
                1024 => 5,
                1280 => 6,
                1536 => 7,
                _ => return Err(Error::InvalidDivider),
            }
        };

        self.0.regs.c1.update(|c1| {
            c1.set_bits(6..8, OscSource::External.into());
            c1.set_bits(3..6, frdiv);
            c1.set_bit(2, false);
        });

        // Once we write to the control register, we need to wait for
        // the new clock to stabilize before we move on.
        // First: Wait for the FLL to be pointed at the crystal
        // Then: Wait for our clock source to be the crystal osc
        while self.0.regs.s.read().get_bit(4) {}
        while self.0.regs.s.read().get_bits(2..4) != OscSource::External.into() {}

        Ok(Fbe(self.0))
    }
}

macro_rules! fbe {
    ($m:ident) => {
        impl<'a> Fbe<'a, $m> {
            /// Enable the PLL and switch to PBE mode
            ///
            /// This method does not protect you from selecting clock
            /// frequencies which are outside of the acceptable range for the
            /// MCU. Be careful!
            pub fn enable_pll(self, numerator: u8, denominator: u8) -> Result<Pbe<'a, $m>, Error> {
                if !(24..=55).contains(&numerator) {
                    return Err(Error::InvalidDivider);
                }

                if !(1..=24).contains(&denominator) {
                    return Err(Error::InvalidDivider);
                }

                self.0.regs.c5.update(|c5| {
                    c5.set_bits(0..5, denominator - 1);
                });

                self.0.regs.c6.update(|c6| {
                    c6.set_bits(0..6, numerator - 24);
                    c6.set_bit(6, true);
                });

                // Wait for PLL to be enabled, using the crystal oscillator
                while !self.0.regs.s.read().get_bit(5) {}
                // Wait for the PLL to be "locked" and stable
                while !self.0.regs.s.read().get_bit(6) {}

                Ok(Pbe(self.0))
            }
        }
    };
}

fbe!(Mk20Dx128);
fbe!(Mk20Dx256);
fbe!(Mk64Fx512);
fbe!(Mkl26Z64);

impl<'a> Fbe<'a, Mk66Fx1M0> {
    /// Enable the PLL and switch to PBE mode
    ///
    /// This method does not protect you from selecting clock
    /// frequencies which are outside of the acceptable range for the
    /// MCU. Be careful!
    pub fn enable_pll(self, numerator: u8, denominator: u8) -> Result<Pbe<'a, Mk66Fx1M0>, Error> {
        if !(16..=47).contains(&numerator) {
            return Err(Error::InvalidDivider);
        }

        if !(1..=7).contains(&denominator) {
            return Err(Error::InvalidDivider);
        }

        self.0.regs.c5.update(|c5| {
            c5.set_bits(0..4, denominator - 1);
        });

        self.0.regs.c6.update(|c6| {
            c6.set_bits(0..5, numerator - 16);
            c6.set_bit(6, true);
        });

        // Wait for PLL to be enabled, using the crystal oscillator
        while !self.0.regs.s.read().get_bit(5) {}
        // Wait for the PLL to be "locked" and stable
        while !self.0.regs.s.read().get_bit(6) {}

        Ok(Pbe(self.0))
    }
}

impl<'a, M> Pbe<'a, M> {
    /// Switch the clock to the PLL
    pub fn use_pll(self) -> Pee<'a, M> {
        self.0.regs.c1.update(|c1| {
            c1.set_bits(6..8, OscSource::LockedLoop.into());
        });

        // mcg.c1 and mcg.s have slightly different behaviors.  In c1,
        // we use one value to indicate "Use whichever LL is
        // enabled". In s, it is differentiated between the FLL at 0,
        // and the PLL at 3. Instead of adding a value to OscSource
        // which would be invalid to set, we just check for the known
        // value "3" here.
        while self.0.regs.s.read().get_bits(2..4) != 3 {}

        Pee(self.0)
    }

    /// Disable the PLL
    ///
    /// Disabling the PLL is required before its dividers can be
    /// modified.
    pub fn disable_pll(self) -> Fbe<'a, M> {
        self.0.regs.c6.update(|c6| {
            c6.set_bit(6, false);
        });

        // Wait for FLL to be selected
        while self.0.regs.s.read().get_bit(5) {}
        Fbe(self.0)
    }
}

impl<'a, M> Pee<'a, M> {
    /// Bypass the PLL
    ///
    /// Bypassing and disabling the PLL is required before its
    /// dividers can be modified.
    pub fn bypass_pll(self) -> Pbe<'a, M> {
        self.0.regs.c1.update(|c1| {
            c1.set_bits(6..8, OscSource::External.into());
        });

        while self.0.regs.s.read().get_bits(2..4) != OscSource::External.into() {}
        Pbe(self.0)
    }
}
