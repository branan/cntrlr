// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! System Integration Module

use super::super::{Mk20Dx128, Mk20Dx256, Mk64Fx512, Mk66Fx1M0, Mkl26Z64};
use crate::{
    register::{Register, Reserved},
    sync::Flag,
};
use bit_field::BitField;
use core::{marker::PhantomData, sync::atomic::Ordering};

/// A clock-gated peripheral
///
/// This trait indicates that the implementing peripheral handle is
/// clock-gated in the SIM.
pub unsafe trait GatedPeripheral<T> {
    /// The clock gate that controls this peripheral
    const GATE: (usize, usize);

    /// Get the instance of this peripheral, gated by `gate`.
    ///
    /// # Safety
    /// The gate must be enabled, and no other references to this
    /// peripheral may be outstanding.
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
    _reserved_4: [Reserved<u32>; 40],
    copc: Register<u32>,
    srvcop: Register<u32>,
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

/// The clock used for UARTs
#[derive(PartialEq)]
pub enum UartClockSource {
    /// The enebled PLL or FLL
    PllFll,

    /// The external reference clock
    Oscer,

    /// The internal reference clock
    Mcgir,
}

/// The clock used for certain peripherals
#[derive(PartialEq)]
pub enum PeripheralClockSource {
    /// The FLL
    Fll,

    /// The PLL
    Pll,

    /// The 48MHz reference oscillator
    Irc48,
}

static LOCK: Flag = Flag::new(false);

macro_rules! get {
    ($m:ident, $s:literal) => {
        #[cfg(any(doc, mcu = $s))]
        #[cfg_attr(feature = "doc-cfg", doc(cfg(mcu = $s)))]
        impl super::Peripheral for Sim<$m> {
            fn get() -> Option<Self> {
                unsafe {
                    if LOCK.swap(true, Ordering::Acquire) {
                        None
                    } else {
                        Some(Self {
                            regs: &mut *(0x4004_7000 as *mut _),
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

impl<M> Sim<M>
where
    Sim<M>: super::Peripheral,
{
    /// Get the handle to the SIM
    ///
    /// Returns 'None' if the SIM is already in use.
    pub fn get() -> Option<Self> {
        super::Peripheral::get()
    }
}

impl<M> Sim<M> {
    /// Set the USB clock source
    pub fn set_usb_source(&mut self, source: UsbClockSource) {
        self.regs.sopt2.update(|sopt2| {
            sopt2.set_bit(18, source == UsbClockSource::PllFll);
        })
    }

    /// Set the peripheral clock source
    pub fn set_peripheral_source(&mut self, source: PeripheralClockSource) {
        let source = match source {
            PeripheralClockSource::Fll => 0,
            PeripheralClockSource::Pll => 1,
            PeripheralClockSource::Irc48 => 3,
        };
        self.regs.sopt2.update(|sopt2| {
            sopt2.set_bits(16..18, source);
        });
    }

    /// Enable  a peripheral
    ///
    /// Enable a clock-gated peripheral, returning its handle. Returns
    /// `None` if the peripheral is already active.
    pub fn enable_peripheral<P: GatedPeripheral<M>>(&mut self) -> Option<P> {
        unsafe {
            let gate = bitband_address(self.regs.scgc.as_mut_ptr().add(P::GATE.0 - 1), P::GATE.1);
            if core::ptr::read_volatile(gate) != 0 {
                None
            } else {
                core::ptr::write_volatile(gate, 1);
                Some(P::new(Gate(gate)))
            }
        }
    }
}

impl Sim<Mk20Dx128> {
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
}

impl Sim<Mk20Dx256> {
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
}

impl Sim<Mk64Fx512> {
    /// Set the main system dividers
    ///
    /// This method does not verify that the divider values set clock
    /// rates which are within the MCU's specifications.
    pub fn set_dividers(&mut self, core: u32, bus: u32, flex: u32, flash: u32) {
        self.regs.clkdiv[0].update(|clkdiv| {
            clkdiv.set_bits(28..32, core - 1);
            clkdiv.set_bits(24..28, bus - 1);
            clkdiv.set_bits(20..24, flex - 1);
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
}

impl Sim<Mk66Fx1M0> {
    /// Set the main system dividers
    ///
    /// This method does not verify that the divider values set clock
    /// rates which are within the MCU's specifications.
    pub fn set_dividers(&mut self, core: u32, bus: u32, flex: u32, flash: u32) {
        self.regs.clkdiv[0].update(|clkdiv| {
            clkdiv.set_bits(28..32, core - 1);
            clkdiv.set_bits(24..28, bus - 1);
            clkdiv.set_bits(20..24, flex - 1);
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
}

impl Sim<Mkl26Z64> {
    /// Set the main system dividers
    ///
    /// This method does not verify that the divider values set clock
    /// rates which are within the MCU's specifications.
    pub fn set_dividers(&mut self, core: u32, flash: u32) {
        self.regs.clkdiv[0].update(|clkdiv| {
            clkdiv.set_bits(28..32, core - 1);
            clkdiv.set_bits(16..20, flash - 1);
        });
    }

    /// Disable the watchdog
    pub fn disable_cop(&mut self) {
        self.regs.copc.update(|copc| {
            copc.set_bits(2..4, 0);
        });
    }

    /// Set the Uart0 clock source
    pub fn set_uart0_source(&mut self, source: Option<UartClockSource>) {
        let source = match source {
            None => 0,
            Some(UartClockSource::PllFll) => 1,
            Some(UartClockSource::Oscer) => 2,
            Some(UartClockSource::Mcgir) => 3,
        };
        self.regs.sopt2.update(|sopt2| {
            sopt2.set_bits(26..28, source);
        });
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
