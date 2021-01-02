// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! The GPIO on an FE310 microcontroller.

use super::super::Fe310G002;
use crate::sync::Flag;
use bit_field::BitField;
use core::{cell::UnsafeCell, marker::PhantomData, ptr::read_volatile, sync::atomic::Ordering};

struct GpioReg(UnsafeCell<u32>);

unsafe impl Send for GpioReg {}
unsafe impl Sync for GpioReg {}

impl GpioReg {
    fn set<const N: usize>(&self, value: bool) {
        assert!(N < 32);
        let _ = value; // Silence warning when not building for this MCU
        #[cfg(mcu = "fe310g002")]
        unsafe {
            if value {
                asm!("amoor.w {}, {}, ({})", out(reg) _, in(reg) 1 << N, in(reg) self.0.get());
            } else {
                asm!("amoand.w {}, {}, ({})", out(reg) _, in(reg) !(1 << N), in(reg) self.0.get());
            }
        }
    }

    fn get<const N: usize>(&self) -> bool {
        assert!(N < 32);
        unsafe { read_volatile(self.0.get()).get_bit(N) }
    }
}

#[repr(C)]
struct GpioRegs {
    input_val: GpioReg,
    input_en: GpioReg,
    output_en: GpioReg,
    output_val: GpioReg,
    pue: GpioReg,
    ds: GpioReg,
    rise_ie: GpioReg,
    fall_ie: GpioReg,
    fall_ip: GpioReg,
    low_ie: GpioReg,
    low_ip: GpioReg,
    iof_en: GpioReg,
    iof_sel: GpioReg,
    out_xor: GpioReg,
}

/// A GPIO instance
pub struct Gpio<M, const N: usize> {
    pins: [Flag; 32],
    regs: &'static GpioRegs,
    _mcu: PhantomData<M>,
}

/// A single pin from a GPIO instance.
pub struct Pin<'a, M, const N: usize, const P: usize> {
    port: &'a Gpio<M, N>,
}

static LOCK: Flag = Flag::new(false);

impl Gpio<Fe310G002, 0> {
    /// Get GPIO instance 0
    pub fn get() -> Option<Self> {
        unsafe {
            if LOCK.swap(true, Ordering::Acquire) {
                None
            } else {
                Some(Self {
                    pins: Default::default(),
                    regs: &*(0x1001_2000 as *const _),
                    _mcu: PhantomData,
                })
            }
        }
    }
}

impl<M, const N: usize> Gpio<M, N> {
    /// Get a pin from this GPIO
    pub fn pin<const P: usize>(&self) -> Option<Pin<M, N, P>> {
        if P >= 32 || self.pins[P].swap(true, Ordering::Acquire) {
            None
        } else {
            Some(Pin { port: self })
        }
    }
}

impl<M, const N: usize, const P: usize> Pin<'_, M, N, P> {
    /// Use this pin as a GPIO
    pub fn into_gpio(self) -> GpioPin<Self> {
        self.port.regs.iof_en.set::<P>(false);
        GpioPin(self)
    }
}

impl<M, const N: usize, const P: usize> Drop for Pin<'_, M, N, P> {
    fn drop(&mut self) {
        self.port.pins[P].store(false, Ordering::Release);
    }
}
impl Pin<'_, Fe310G002, 0, 16> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.port.regs.iof_sel.set::<16>(false);
        self.port.regs.iof_en.set::<16>(true);
        UartRx(self)
    }
}

impl Pin<'_, Fe310G002, 0, 17> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.port.regs.iof_sel.set::<17>(false);
        self.port.regs.iof_en.set::<17>(true);
        UartTx(self)
    }
}

impl Pin<'_, Fe310G002, 0, 18> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.port.regs.iof_sel.set::<18>(false);
        self.port.regs.iof_en.set::<18>(true);
        UartTx(self)
    }
}

impl Pin<'_, Fe310G002, 0, 23> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.port.regs.iof_sel.set::<23>(false);
        self.port.regs.iof_en.set::<23>(true);
        UartRx(self)
    }
}

/// A GPIO pin which is configured for UART recieve
pub struct UartRx<P>(P);

/// A GPIO pin whic is configured for UART transmit
pub struct UartTx<P>(P);

/// A GPIO pin which is configured as a GPIO
pub struct GpioPin<P>(P);

impl<M, const N: usize, const P: usize> GpioPin<Pin<'_, M, N, P>> {
    /// Set this pin as high or low
    pub fn write(&mut self, value: bool) {
        self.0.port.regs.output_val.set::<P>(value);
    }

    /// Read the status of this pin
    pub fn read(&self) -> bool {
        self.0.port.regs.input_val.get::<P>()
    }

    /// Set whether this pin is an output or an input
    pub fn set_output(&mut self, output: bool) {
        self.0.port.regs.output_en.set::<P>(output);
        self.0.port.regs.input_en.set::<P>(!output);
    }

    /// Enable the pull-up resistor
    pub fn enable_pullup(&mut self, pullup: bool) {
        self.0.port.regs.pue.set::<P>(pullup);
    }
}

impl super::uart::UartRx<Fe310G002, 0> for UartRx<Pin<'_, Fe310G002, 0, 16>> {}
impl super::uart::UartRx<Fe310G002, 1> for UartRx<Pin<'_, Fe310G002, 0, 23>> {}
impl super::uart::UartTx<Fe310G002, 0> for UartTx<Pin<'_, Fe310G002, 0, 17>> {}
impl super::uart::UartTx<Fe310G002, 1> for UartTx<Pin<'_, Fe310G002, 0, 18>> {}
