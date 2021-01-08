// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! SMC - System Mode Controllller

use super::super::Mk66Fx1M0;
use crate::{register::Register, sync::Flag};
use bit_field::BitField;
use core::{marker::PhantomData, sync::atomic::Ordering};

#[repr(C)]
struct SmcRegs {
    pmprot: Register<u8>,
    pmctrl: Register<u8>,
    stopctrl: Register<u8>,
    pmstat: Register<u8>,
}

/// The handle to the SMC
pub struct Smc<M> {
    regs: &'static mut SmcRegs,
    _mcu: PhantomData<M>,
}

static LOCK: Flag = Flag::new(false);

macro_rules! get {
    ($m:ident, $s:literal) => {
        #[cfg(any(doc, mcu = $s))]
        #[cfg_attr(feature = "doc-cfg", doc(cfg(mcu = $s)))]
        impl super::Peripheral for Smc<super::super::$m> {
            fn get() -> Option<Self> {
                unsafe {
                    if LOCK.swap(true, Ordering::Acquire) {
                        None
                    } else {
                        Some(Self {
                            regs: &mut *(0x4007_E000 as *mut _),
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

impl<M> Smc<M>
where
    Smc<M>: super::Peripheral,
{
    /// Get the handle to the SMC
    pub fn get() -> Option<Self> {
        super::Peripheral::get()
    }
}

impl Smc<Mk66Fx1M0> {
    /// Enable all run modes
    pub fn allow_all_modes(&mut self) {
        self.regs.pmprot.update(|pmprot| {
            pmprot.set_bit(1, true);
            pmprot.set_bit(3, true);
            pmprot.set_bit(5, true);
            pmprot.set_bit(7, true);
        });
    }

    /// Enter HSRUN mode
    ///
    /// HSRUN is required for clocks above 120MHz
    pub fn enter_hsrun(&mut self) {
        self.regs.pmctrl.update(|pmctrl| {
            pmctrl.set_bits(5..7, 3);
        });
        while self.regs.pmstat.read() != 0x80 {}
    }

    /// Exit HSRUN mode
    pub fn exit_hsrun(&mut self) {
        self.regs.pmctrl.update(|pmctrl| {
            pmctrl.set_bits(5..7, 0);
        });
        while self.regs.pmstat.read() != 0x01 {}
    }
}

impl<M> Drop for Smc<M> {
    fn drop(&mut self) {
        LOCK.store(false, Ordering::Release);
    }
}
