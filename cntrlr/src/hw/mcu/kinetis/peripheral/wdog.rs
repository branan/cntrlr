// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Watchdog

use crate::register::Register;
use bit_field::BitField;

#[repr(C)]
struct WatchdogRegs {
    stctrlh: Register<u16>,
    stctrll: Register<u16>,
    tovalh: Register<u16>,
    tovall: Register<u16>,
    winh: Register<u16>,
    winl: Register<u16>,
    refresh: Register<u16>,
    unlock: Register<u16>,
    tmrouth: Register<u16>,
    tmroutl: Register<u16>,
    rstcnt: Register<u16>,
    presc: Register<u16>,
}

/// The handle to the WDOG
pub struct Watchdog {
    regs: &'static mut WatchdogRegs,
}

impl Watchdog {
    /// Get the watchdog peripheral.
    ///
    /// Because the watchdog must be disabled early, accessing it
    /// cannot rely on any global variables. As such, we cannot
    /// provide a lock for the watchdog. This means it has to be
    /// unsafe to access.
    ///
    /// # Safety
    /// There must only be one outstanding reference to the watchdog.
    pub unsafe fn get() -> Watchdog {
        let regs = &mut *(0x4005_2000 as *mut _);
        Watchdog { regs }
    }

    /// Disable the watchdog
    pub fn disable(&mut self) {
        self.unlock();
        self.regs.stctrlh.update(|ctrl| {
            ctrl.set_bit(0, false);
        });
    }

    fn unlock(&mut self) {
        self.regs.unlock.write(0xC520);
        self.regs.unlock.write(0xD928);
        unsafe {
            // Wait one bus cycle for watchdog to unlock
            let mut dummy: u32 = 0;
            core::ptr::write_volatile(&mut dummy, 0);
        }
    }
}
