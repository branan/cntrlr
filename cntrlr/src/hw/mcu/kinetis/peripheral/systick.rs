// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! ARM System Tick timer
//!
//! Not actually kinetis-specific, but for now this is the only ARM
//! family we support.

use crate::{register::Register, sync::Flag};
use bit_field::BitField;
use core::{marker::PhantomData, sync::atomic::Ordering};

#[repr(C)]
struct SysTickRegs {
    ctrl: Register<u32>,
    load: Register<u32>,
    val: Register<u32>,
    calib: Register<u32>,
}

/// The handle to the SYSTICK
pub struct SysTick<M> {
    regs: &'static mut SysTickRegs,
    _mcu: PhantomData<M>,
}

static LOCK: Flag = Flag::new(false);

macro_rules! get {
    ($m:ident, $s:literal) => {
        #[cfg(any(doc, mcu = $s))]
        #[cfg_attr(feature = "doc-cfg", doc(cfg(mcu = $s)))]
        impl super::Peripheral for SysTick<super::super::$m> {
            fn get() -> Option<Self> {
                unsafe {
                    if LOCK.swap(true, Ordering::Acquire) {
                        None
                    } else {
                        Some(Self {
                            regs: &mut *(0xE000_E010 as *mut _),
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

impl<M> SysTick<M>
where
    SysTick<M>: super::Peripheral,
{
    /// Get the handle to the SysTick
    ///
    /// Returns `None` if the SysTick is already in use.
    pub fn get() -> Option<Self> {
        super::Peripheral::get()
    }
}

impl<M> SysTick<M> {
    /// Set the value which is reloaded when the timer reaches zero
    pub fn set_reload_value(&mut self, value: u32) {
        self.regs.load.write(value);
    }

    /// Set the current value of the countdown timer
    pub fn set_current_value(&mut self, value: u32) {
        self.regs.val.write(value);
    }

    /// Enable or disable the systick timer
    pub fn enable(&mut self, enabled: bool) {
        self.regs.ctrl.update(|ctrl| {
            ctrl.set_bit(0, enabled);
        });
    }

    /// Enable or disable the interrupt for the systick timer
    pub fn enable_interrupt(&mut self, enabled: bool) {
        self.regs.ctrl.update(|ctrl| {
            ctrl.set_bit(1, enabled);
        });
    }

    /// Set whether the systick uses the core clock or the external
    /// reference clock.
    pub fn use_core_clock(&mut self, core: bool) {
        self.regs.ctrl.update(|ctrl| {
            ctrl.set_bit(2, core);
        });
    }
}

impl<M> Drop for SysTick<M> {
    fn drop(&mut self) {
        LOCK.store(false, Ordering::Release);
    }
}
