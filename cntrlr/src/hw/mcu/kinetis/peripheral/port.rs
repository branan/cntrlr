// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Ports and Pins
//!
//! In the Kinetis familiy, the port module manges I/O pins and which
//! peripherals they are assigned to.

use super::{
    super::{Mk20Dx128, Mk20Dx256, Mk64Fx512, Mk66Fx1M0, Mkl26Z64},
    sim::{Gate, Peripheral},
};
use crate::{register::Register, sync::Flag};
use bit_field::BitField;
use core::{default::Default, marker::PhantomData, sync::atomic::Ordering};

/// The handle to a PORT
///
/// Port A is `Port<0>`, Port B is `Port<1>`, etc.
pub struct Port<M, const N: usize> {
    pins: [Flag; 32],
    base: *mut Register<u32>,
    _gate: Gate,
    _mcu: PhantomData<M>,
}

unsafe impl<M, const N: usize> Send for Port<M, N> {}
unsafe impl<M, const N: usize> Sync for Port<M, N> {}

impl<M, const N: usize> Port<M, N> {
    /// Get a pin from this port
    ///
    /// Returns `None` if the pin is already in use
    pub fn pin<const P: usize>(&self) -> Option<Pin<M, N, P>> {
        unsafe {
            if P >= 32 || self.pins[P].swap(true, Ordering::Acquire) {
                None
            } else {
                Some(Pin {
                    reg: &mut *self.base.add(P),
                    port: self,
                })
            }
        }
    }
}

/// A pin from a port
pub struct Pin<'a, M, const N: usize, const P: usize> {
    reg: &'static mut Register<u32>,
    port: &'a Port<M, N>,
}

impl<M, const N: usize, const P: usize> Drop for Pin<'_, M, N, P> {
    fn drop(&mut self) {
        self.port.pins[P].store(false, Ordering::Release);
    }
}

impl Pin<'_, Mk20Dx128, 1, 16> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk20Dx128, 1, 17> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk20Dx128, 2, 3> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk20Dx128, 2, 4> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk20Dx128, 3, 2> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk20Dx128, 3, 3> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk20Dx256, 1, 16> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk20Dx256, 1, 17> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk20Dx256, 2, 3> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk20Dx256, 2, 4> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk20Dx256, 3, 2> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk20Dx256, 3, 3> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk64Fx512, 1, 10> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk64Fx512, 1, 11> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk64Fx512, 1, 16> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk64Fx512, 1, 17> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk64Fx512, 2, 3> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk64Fx512, 2, 4> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk64Fx512, 3, 2> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk64Fx512, 3, 3> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk64Fx512, 3, 8> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk64Fx512, 3, 9> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk64Fx512, 4, 24> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk64Fx512, 4, 25> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk66Fx1M0, 1, 10> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk66Fx1M0, 1, 11> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk66Fx1M0, 1, 16> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk66Fx1M0, 1, 17> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk66Fx1M0, 2, 3> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk66Fx1M0, 2, 4> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk66Fx1M0, 3, 2> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk66Fx1M0, 3, 3> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk66Fx1M0, 3, 8> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk66Fx1M0, 3, 9> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mk66Fx1M0, 4, 24> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mk66Fx1M0, 4, 25> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mkl26Z64, 1, 16> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mkl26Z64, 1, 17> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mkl26Z64, 2, 3> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mkl26Z64, 2, 4> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

impl Pin<'_, Mkl26Z64, 3, 2> {
    /// Use this pin as a UART recieve pin
    pub fn into_uart_rx(self) -> UartRx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartRx(self)
    }
}

impl Pin<'_, Mkl26Z64, 3, 3> {
    /// Use this pin as a UART transmit pin
    pub fn into_uart_tx(self) -> UartTx<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 3);
        });
        UartTx(self)
    }
}

/// A pin which is configured as a UART reciever
pub struct UartRx<P>(P);

/// A pin which is configured as a UART transmitter
pub struct UartTx<P>(P);

impl super::uart::UartRx<Mk20Dx128, 0> for UartRx<Pin<'_, Mk20Dx128, 1, 16>> {}
impl super::uart::UartRx<Mk20Dx128, 1> for UartRx<Pin<'_, Mk20Dx128, 2, 3>> {}
impl super::uart::UartRx<Mk20Dx128, 2> for UartRx<Pin<'_, Mk20Dx128, 3, 2>> {}
impl super::uart::UartTx<Mk20Dx128, 0> for UartTx<Pin<'_, Mk20Dx128, 1, 17>> {}
impl super::uart::UartTx<Mk20Dx128, 1> for UartTx<Pin<'_, Mk20Dx128, 2, 4>> {}
impl super::uart::UartTx<Mk20Dx128, 2> for UartTx<Pin<'_, Mk20Dx128, 3, 3>> {}

impl super::uart::UartRx<Mk20Dx256, 0> for UartRx<Pin<'_, Mk20Dx256, 1, 16>> {}
impl super::uart::UartRx<Mk20Dx256, 1> for UartRx<Pin<'_, Mk20Dx256, 2, 3>> {}
impl super::uart::UartRx<Mk20Dx256, 2> for UartRx<Pin<'_, Mk20Dx256, 3, 2>> {}
impl super::uart::UartTx<Mk20Dx256, 0> for UartTx<Pin<'_, Mk20Dx256, 1, 17>> {}
impl super::uart::UartTx<Mk20Dx256, 1> for UartTx<Pin<'_, Mk20Dx256, 2, 4>> {}
impl super::uart::UartTx<Mk20Dx256, 2> for UartTx<Pin<'_, Mk20Dx256, 3, 3>> {}

impl super::uart::UartRx<Mk64Fx512, 3> for UartRx<Pin<'_, Mk64Fx512, 1, 10>> {}
impl super::uart::UartRx<Mk64Fx512, 0> for UartRx<Pin<'_, Mk64Fx512, 1, 16>> {}
impl super::uart::UartRx<Mk64Fx512, 1> for UartRx<Pin<'_, Mk64Fx512, 2, 3>> {}
impl super::uart::UartRx<Mk64Fx512, 2> for UartRx<Pin<'_, Mk64Fx512, 3, 2>> {}
impl super::uart::UartRx<Mk64Fx512, 5> for UartRx<Pin<'_, Mk64Fx512, 3, 9>> {}
impl super::uart::UartRx<Mk64Fx512, 4> for UartRx<Pin<'_, Mk64Fx512, 4, 25>> {}
impl super::uart::UartTx<Mk64Fx512, 3> for UartTx<Pin<'_, Mk64Fx512, 1, 11>> {}
impl super::uart::UartTx<Mk64Fx512, 0> for UartTx<Pin<'_, Mk64Fx512, 1, 17>> {}
impl super::uart::UartTx<Mk64Fx512, 1> for UartTx<Pin<'_, Mk64Fx512, 2, 4>> {}
impl super::uart::UartTx<Mk64Fx512, 2> for UartTx<Pin<'_, Mk64Fx512, 3, 3>> {}
impl super::uart::UartTx<Mk64Fx512, 5> for UartTx<Pin<'_, Mk64Fx512, 3, 8>> {}
impl super::uart::UartTx<Mk64Fx512, 4> for UartTx<Pin<'_, Mk64Fx512, 4, 24>> {}

impl super::uart::UartRx<Mk66Fx1M0, 3> for UartRx<Pin<'_, Mk66Fx1M0, 1, 10>> {}
impl super::uart::UartRx<Mk66Fx1M0, 0> for UartRx<Pin<'_, Mk66Fx1M0, 1, 16>> {}
impl super::uart::UartRx<Mk66Fx1M0, 1> for UartRx<Pin<'_, Mk66Fx1M0, 2, 3>> {}
impl super::uart::UartRx<Mk66Fx1M0, 2> for UartRx<Pin<'_, Mk66Fx1M0, 3, 2>> {}
impl super::uart::UartRx<Mk66Fx1M0, 4> for UartRx<Pin<'_, Mk66Fx1M0, 4, 25>> {}
impl super::uart::UartTx<Mk66Fx1M0, 3> for UartTx<Pin<'_, Mk66Fx1M0, 1, 11>> {}
impl super::uart::UartTx<Mk66Fx1M0, 0> for UartTx<Pin<'_, Mk66Fx1M0, 1, 17>> {}
impl super::uart::UartTx<Mk66Fx1M0, 1> for UartTx<Pin<'_, Mk66Fx1M0, 2, 4>> {}
impl super::uart::UartTx<Mk66Fx1M0, 2> for UartTx<Pin<'_, Mk66Fx1M0, 3, 3>> {}
impl super::uart::UartTx<Mk66Fx1M0, 4> for UartTx<Pin<'_, Mk66Fx1M0, 4, 24>> {}

impl super::uart::UartRx<Mkl26Z64, 0> for UartRx<Pin<'_, Mkl26Z64, 1, 16>> {}
impl super::uart::UartRx<Mkl26Z64, 1> for UartRx<Pin<'_, Mkl26Z64, 2, 3>> {}
impl super::uart::UartRx<Mkl26Z64, 2> for UartRx<Pin<'_, Mkl26Z64, 3, 2>> {}
impl super::uart::UartTx<Mkl26Z64, 0> for UartTx<Pin<'_, Mkl26Z64, 1, 17>> {}
impl super::uart::UartTx<Mkl26Z64, 1> for UartTx<Pin<'_, Mkl26Z64, 2, 4>> {}
impl super::uart::UartTx<Mkl26Z64, 2> for UartTx<Pin<'_, Mkl26Z64, 3, 3>> {}

unsafe impl Peripheral<Mk20Dx128> for Port<Mk20Dx128, 0> {
    const GATE: (usize, usize) = (5, 9);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_9000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx128> for Port<Mk20Dx128, 1> {
    const GATE: (usize, usize) = (5, 10);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_A000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx128> for Port<Mk20Dx128, 2> {
    const GATE: (usize, usize) = (5, 11);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_B000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx128> for Port<Mk20Dx128, 3> {
    const GATE: (usize, usize) = (5, 12);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_C000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx128> for Port<Mk20Dx128, 4> {
    const GATE: (usize, usize) = (5, 13);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_D000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx256> for Port<Mk20Dx256, 0> {
    const GATE: (usize, usize) = (5, 9);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_9000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx256> for Port<Mk20Dx256, 1> {
    const GATE: (usize, usize) = (5, 10);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_A000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx256> for Port<Mk20Dx256, 2> {
    const GATE: (usize, usize) = (5, 11);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_B000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx256> for Port<Mk20Dx256, 3> {
    const GATE: (usize, usize) = (5, 12);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_C000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk20Dx256> for Port<Mk20Dx256, 4> {
    const GATE: (usize, usize) = (5, 13);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_D000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk64Fx512> for Port<Mk64Fx512, 0> {
    const GATE: (usize, usize) = (5, 9);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_9000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk64Fx512> for Port<Mk64Fx512, 1> {
    const GATE: (usize, usize) = (5, 10);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_A000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk64Fx512> for Port<Mk64Fx512, 2> {
    const GATE: (usize, usize) = (5, 11);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_B000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk64Fx512> for Port<Mk64Fx512, 3> {
    const GATE: (usize, usize) = (5, 12);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_C000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk64Fx512> for Port<Mk64Fx512, 4> {
    const GATE: (usize, usize) = (5, 13);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_D000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk66Fx1M0> for Port<Mk66Fx1M0, 0> {
    const GATE: (usize, usize) = (5, 9);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_9000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk66Fx1M0> for Port<Mk66Fx1M0, 1> {
    const GATE: (usize, usize) = (5, 10);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_A000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk66Fx1M0> for Port<Mk66Fx1M0, 2> {
    const GATE: (usize, usize) = (5, 11);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_B000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk66Fx1M0> for Port<Mk66Fx1M0, 3> {
    const GATE: (usize, usize) = (5, 12);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_C000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mk66Fx1M0> for Port<Mk66Fx1M0, 4> {
    const GATE: (usize, usize) = (5, 13);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_D000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mkl26Z64> for Port<Mkl26Z64, 0> {
    const GATE: (usize, usize) = (5, 9);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_9000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mkl26Z64> for Port<Mkl26Z64, 1> {
    const GATE: (usize, usize) = (5, 10);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_A000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mkl26Z64> for Port<Mkl26Z64, 2> {
    const GATE: (usize, usize) = (5, 11);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_B000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mkl26Z64> for Port<Mkl26Z64, 3> {
    const GATE: (usize, usize) = (5, 12);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_C000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}

unsafe impl Peripheral<Mkl26Z64> for Port<Mkl26Z64, 4> {
    const GATE: (usize, usize) = (5, 13);

    unsafe fn new(gate: Gate) -> Self {
        Self {
            pins: Default::default(),
            base: 0x4004_D000 as *mut _,
            _gate: gate,
            _mcu: PhantomData,
        }
    }
}
