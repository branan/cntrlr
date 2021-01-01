// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! PLIC
//!
//! This PLIC module is tailored specifically to the PLIC found in the
//! FE310 series, and is not intended as a general-purpose RISC-V PLIC
//! driver.

use crate::register::{Register, Reserved};
use bit_field::BitField;

#[repr(C)]
struct PlicRegs {
    _reserved0: Reserved<u32>,
    priority: [Register<u32>; 52],
    _reserved1: [Reserved<u32>; 971],
    pending: [Register<u32>; 2],
    _reserved2: [Reserved<u32>; 1022],
    enable: [Register<u32>; 2],
    _reserved: [Reserved<u32>; 522238],
    threshold: Register<u32>,
    claim: Register<u32>,
}

/// The PLIC
pub struct Plic {
    regs: &'static mut PlicRegs,
}

impl Plic {
    /// Get the PLIC
    ///
    /// This is unsafe because the PLIC must be accessed from interupt
    /// handlers, and so requires special synchronization handling.
    ///
    /// # Safety
    /// There must only be one outstanding PLIC handle at any given time.
    pub unsafe fn get() -> Self {
        Self {
            regs: &mut *(0x0C00_0000 as *mut _),
        }
    }

    /// Mask all interrupt sources
    ///
    /// This sets the priority for all interrupt inputs to 0, and
    /// flags all interupts as disabled for hart 0.
    pub fn mask_all(&mut self) {
        for reg in &mut self.regs.enable {
            reg.write(0);
        }
        for reg in &mut self.regs.priority {
            reg.write(0);
        }
    }

    /// Enable an interrupt
    ///
    /// This enables the interrupt for hart 0
    pub fn enable(&mut self, intr: usize) {
        let reg = intr / 32;
        let bit = intr % 32;
        self.regs.enable[reg].update(|reg| {
            reg.set_bit(bit, true);
        });
    }

    /// Claim an interrupt
    pub fn claim(&mut self) -> u32 {
        self.regs.claim.read()
    }

    /// Complete a claimed interrupt
    pub fn complete(&mut self, intr: u32) {
        self.regs.claim.write(intr);
    }

    /// Set the interrupt threshold for hart 0
    ///
    /// Interrupts with a priority below this threshold will not be
    /// sent to the hart.
    pub fn set_threshold(&mut self, threshold: u32) {
        assert!(threshold < 8);
        self.regs.threshold.write(threshold);
    }

    /// Set the priority for an interrupt inpput.
    pub fn set_priority(&mut self, intr: usize, priority: u32) {
        assert!(priority < 8);
        self.regs.priority[intr - 1].write(priority);
    }
}
