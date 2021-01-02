// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! PRCI
//!
///! The PRCI is responsible for generating the clocks used on the
///! FE310 series.
use super::super::Fe310G002;
use crate::{register::Register, sync::Flag};
use bit_field::BitField;
use core::{marker::PhantomData, sync::atomic::Ordering};

#[repr(C)]
struct PrciRegs {
    hfrosccfg: Register<u32>,
    hfxosccfg: Register<u32>,
    pllcfg: Register<u32>,
    plloutdiv: Register<u32>,
}

/// A PRCI
pub struct Prci<M> {
    regs: &'static mut PrciRegs,
    _mcu: PhantomData<M>,
}

static LOCK: Flag = Flag::new(false);

impl Prci<Fe310G002> {
    /// Get the PRCI instance
    pub fn get() -> Option<Self> {
        unsafe {
            if LOCK.swap(true, Ordering::Acquire) {
                None
            } else {
                Some(Self {
                    regs: &mut *(0x1000_8000 as *mut _),
                    _mcu: PhantomData,
                })
            }
        }
    }
}

impl<M> Prci<M> {
    /// Enable the PLL, with the selected set of dividers
    ///
    /// The final CPU_CLOCK will be equal to `(XTAL_IN * f) / (q * r *
    /// div)`
    ///
    /// * `r` must be in the range `1..=4`
    /// * `f` must be an even numbe in the range `2..=128`
    /// * `q` must be one of 2, 4, or 8
    /// * `div` must be 1 or be an even number in the range `2..=128`
    pub fn use_pll(&mut self, r: u32, f: u32, q: u32, div: u32) {
        // TODO: This should be broken down into a nice state machine like
        // the Kinetis MCG
        assert!(r >= 1 && r <= 4);
        let r = r - 1;
        assert!(f >= 2 && f <= 128 && f % 2 == 0);
        let f = f / 2 - 2;
        let q = match q {
            2 => 0b01,
            4 => 0b10,
            8 => 0b11,
            _ => panic!("Invalid value for Q"),
        };
        assert!(div == 1 || (div % 2) == 0);
        if self.regs.pllcfg.read().get_bit(16) {
            // Make sure the ring oscillator is enabled and switch to it
            self.regs.hfrosccfg.update(|hfrosccfg| {
                hfrosccfg.set_bit(30, true);
            });
            while !self.regs.hfrosccfg.read().get_bit(31) {}
            self.regs.pllcfg.update(|pllcfg| {
                pllcfg.set_bit(16, false);
            });
        }
        self.regs.pllcfg.update(|pll| {
            pll.set_bit(18, true); // bypass PLL
        });

        // Set final PLL dividers
        if div == 1 {
            self.regs.plloutdiv.write(1 << 8);
        } else {
            self.regs.plloutdiv.write(div / 2 - 1);
        }

        // Enable crystal oscillator
        self.regs.hfxosccfg.update(|hfxosccfg| {
            hfxosccfg.set_bit(30, true);
        });
        while !self.regs.hfxosccfg.read().get_bit(31) {}

        // Set PLL dividers and wait for lock
        self.regs.pllcfg.update(|pllcfg| {
            pllcfg.set_bits(0..2, r);
            pllcfg.set_bits(4..10, f);
            pllcfg.set_bits(10..12, q);
            pllcfg.set_bit(17, true);
            pllcfg.set_bit(18, false);
        });
        while !self.regs.pllcfg.read().get_bit(31) {}

        // Switch to PLL as main clock source
        self.regs.pllcfg.update(|pllcfg| {
            pllcfg.set_bit(16, true);
        });
    }
}

impl<M> Drop for Prci<M> {
    fn drop(&mut self) {
        LOCK.store(false, Ordering::Release);
    }
}
