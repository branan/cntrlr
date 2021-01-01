// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! System Integration Module

use crate::{
    register::{Register, Reserved},
    sync::Flag,
};
use bit_field::BitField;
use core::{marker::PhantomData, sync::atomic::Ordering};

/// A peripheral
///
/// This trait indicates that the implementing peripheral handle is
/// clock-gated in the SIM.
pub unsafe trait Peripheral<T> {
    /// The clock gate that controls this peripheral
    const GATE: (usize, usize);

    /// Get the instance of this peripheral, gated by `gate`.
    unsafe fn new(gate: Gate) -> Self;
}

#[repr(C)]
struct SimRegs {
    sopt1: Register<u32>,
    sopt1_cfg: Register<u32>,
    _reserved_0: [Reserved<u32>; 1023],
    sopt2: Register<u32>,
    _reserved_1: Reserved<u32>,
    sopt4: Register<u32>,
    sopt5: Register<u32>,
    _reserved_2: Reserved<u32>,
    sopt7: Register<u32>,
    _reserved_3: [Reserved<u32>; 2],
    sdid: Register<u32>,
    scgc: [Register<u32>; 7],
    clkdiv: [Register<u32>; 2],
    fcfg: [Register<u32>; 2],
    uid: [Register<u32>; 4],
}

/// The handle to the SIM
pub struct Sim<M> {
    regs: &'static mut SimRegs,
    _mcu: PhantomData<M>,
}

/// The clock used for USB
#[derive(PartialEq)]
pub enum UsbClockSource {
    /// An External USB clock
    UsbClkIn,

    /// The enebled PLL or FLL
    PllFll,
}

/// The clock used for certain peripherals
#[derive(PartialEq)]
pub enum PeripheralClockSource {
    /// The FLL
    Fll,

    /// The PLL
    Pll,
}

static LOCK: Flag = Flag::new(false);

impl<M> Sim<M> {
    /// Get the handle to the SIM
    ///
    /// # Panics
    /// This method will panic if there is an outstanding handle to the
    /// SIM
    pub fn get() -> Self {
        unsafe {
            if LOCK.swap(true, Ordering::Acquire) {
                panic!("Lock contention");
            }
            Self {
                regs: &mut *(0x4004_7000 as *mut _),
                _mcu: PhantomData,
            }
        }
    }

    /// Set the main system dividers
    ///
    /// This method does not verify that the divider values set clock
    /// rates which are within the MCU's specifications.
    pub fn set_dividers(&mut self, core: u32, bus: u32, flash: u32) {
        self.regs.clkdiv[0].update(|clkdiv| {
            clkdiv.set_bits(28..32, core - 1);
            clkdiv.set_bits(24..28, bus - 1);
            clkdiv.set_bits(16..20, flash - 1);
        });
    }

    /// Set the USB dividers.
    ///
    /// This method does not verify that the divider values set clock
    /// rates which are within the MCU's specifications.
    pub fn set_usb_dividers(&mut self, numerator: u32, denominator: u32) {
        self.regs.clkdiv[1].update(|clkdiv| {
            clkdiv.set_bits(0..1, numerator - 1);
            clkdiv.set_bits(1..4, denominator - 1);
        })
    }

    /// Set the USB clock source
    pub fn set_usb_source(&mut self, source: UsbClockSource) {
        self.regs.sopt2.update(|sopt2| {
            sopt2.set_bit(18, source == UsbClockSource::PllFll);
        })
    }

    /// Set the peripheral clock source
    pub fn set_peripheral_source(&mut self, source: PeripheralClockSource) {
        self.regs.sopt2.update(|sopt2| {
            sopt2.set_bit(16, source == PeripheralClockSource::Pll);
        });
    }

    /// Enable  a peripheral
    ///
    /// Enable a clock-gated peripheral, returning its handle.
    ///
    /// # Panics
    /// This method will panic if there is an outstanding handle to the
    /// requested peripheral
    pub fn enable_peripheral<P: Peripheral<M>>(&mut self) -> P {
        unsafe {
            let gate = bitband_address(self.regs.scgc.as_mut_ptr().add(P::GATE.0 - 1), P::GATE.1);
            if core::ptr::read_volatile(gate) != 0 {
                panic!("Peripheral at {:?} is already enabled", P::GATE);
            }

            core::ptr::write_volatile(gate, 1);
            P::new(Gate(gate))
        }
    }
}

impl<M> Drop for Sim<M> {
    fn drop(&mut self) {
        LOCK.store(false, Ordering::Release);
    }
}

/// A handle to an enabled clock gate.
///
/// This disables the held clock gate when it is dropped.
pub struct Gate(*mut u32);

unsafe impl Send for Gate {}

impl Drop for Gate {
    fn drop(&mut self) {
        unsafe {
            core::ptr::write_volatile(self.0, 0);
        }
    }
}

unsafe fn bitband_address<T>(addr: *mut Register<T>, bit: usize) -> *mut T {
    (0x4200_0000 + (addr as usize - 0x4000_0000) * 32 + bit * 4) as _
}
