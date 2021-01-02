// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! ARM System Tick timer
//!
//! Not actually kinetis-specific, but for now this is the only ARM
//! family we support.

use crate::{register::Register, sync::Flag};
use bit_field::BitField;
use core::sync::atomic::Ordering;

#[repr(C)]
struct SysTickRegs {
    ctrl: Register<u32>,
    load: Register<u32>,
    val: Register<u32>,
    calib: Register<u32>,
}

/// The handle to the SYSTICK
pub struct SysTick {
    regs: &'static mut SysTickRegs,
}

static SYSTICK_LOCK: Flag = Flag::new(false);

impl SysTick {
    /// Get the handle to the SysTick
    ///
    /// Returns `None` if the SysTick is already in use.
    pub fn get() -> Option<Self> {
        unsafe {
            if SYSTICK_LOCK.swap(true, Ordering::AcqRel) {
                None
            } else {
                Some(Self {
                    regs: &mut *(0xE000E010 as *mut _),
                })
            }
        }
    }

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

impl Drop for SysTick {
    fn drop(&mut self) {
        SYSTICK_LOCK.store(false, Ordering::Release);
    }
}
