// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Ports and Pins
//!
//! In the Kinetis familiy, the port module manges I/O pins and which
//! peripherals they are assigned to.

use super::{
    super::{Mk20Dx128, Mk20Dx256, Mk64Fx512, Mk66Fx1M0, Mkl26Z64},
    sim::{Gate, GatedPeripheral},
};
use crate::{digital::Pull, register::Register, sync::Flag};
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

impl<M, const N: usize, const P: usize> Pin<'_, M, N, P> {
    /// Use this in as a GPIO
    pub fn into_gpio(self) -> Gpio<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 1);
        });
        Gpio(self)
    }
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

impl Pin<'_, Mk20Dx128, 2, 0> {
    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
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

    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
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

    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
    }
}

impl Pin<'_, Mk20Dx128, 2, 5> {
    /// Use this pin as an SPI clock
    pub fn into_spi_sck(self) -> Sck<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sck(self)
    }
}

impl Pin<'_, Mk20Dx128, 2, 6> {
    /// Use this pin as an SPI output
    pub fn into_spi_sdo(self) -> Sdo<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sdo(self)
    }
}

impl Pin<'_, Mk20Dx128, 2, 7> {
    /// Use this pin as an SPI input
    pub fn into_spi_sdi(self) -> Sdi<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sdi(self)
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

impl Pin<'_, Mk20Dx128, 3, 5> {
    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
    }
}

impl Pin<'_, Mk20Dx128, 3, 6> {
    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
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

impl Pin<'_, Mk20Dx256, 2, 0> {
    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
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

    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
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

    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
    }
}

impl Pin<'_, Mk20Dx256, 2, 5> {
    /// Use this pin as an SPI clock
    pub fn into_spi_sck(self) -> Sck<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sck(self)
    }
}

impl Pin<'_, Mk20Dx256, 2, 6> {
    /// Use this pin as an SPI output
    pub fn into_spi_sdo(self) -> Sdo<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sdo(self)
    }
}

impl Pin<'_, Mk20Dx256, 2, 7> {
    /// Use this pin as an SPI input
    pub fn into_spi_sdi(self) -> Sdi<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sdi(self)
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

impl Pin<'_, Mk20Dx256, 3, 5> {
    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
    }
}

impl Pin<'_, Mk20Dx256, 3, 6> {
    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
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

    /// Use this pin as an SPI Chip Select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
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

    /// Use this pin as an SPI clock
    pub fn into_spi_sck(self) -> Sck<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sck(self)
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

    /// Use this pin as an SPI output
    pub fn into_spi_sdo(self) -> Sdo<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sdo(self)
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

    /// Use this pin as an SPI input
    pub fn into_spi_sdi(self) -> Sdi<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sdi(self)
    }
}

impl Pin<'_, Mk64Fx512, 1, 20> {
    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
    }
}

impl Pin<'_, Mk64Fx512, 1, 21> {
    /// Use this pin as an SPI clock
    pub fn into_spi_sck(self) -> Sck<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sck(self)
    }
}

impl Pin<'_, Mk64Fx512, 1, 22> {
    /// Use this pin as an SPI output
    pub fn into_spi_sdo(self) -> Sdo<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sdo(self)
    }
}

impl Pin<'_, Mk64Fx512, 1, 23> {
    /// Use this pin as an SPI input
    pub fn into_spi_sdi(self) -> Sdi<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sdi(self)
    }
}

impl Pin<'_, Mk64Fx512, 2, 0> {
    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
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

    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
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

    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
    }
}

impl Pin<'_, Mk64Fx512, 2, 5> {
    /// Use this pin as an SPI clock
    pub fn into_spi_sck(self) -> Sck<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sck(self)
    }
}

impl Pin<'_, Mk64Fx512, 2, 6> {
    /// Use this pin as an SPI output
    pub fn into_spi_sdo(self) -> Sdo<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sdo(self)
    }
}

impl Pin<'_, Mk64Fx512, 2, 7> {
    /// Use this pin as an SPI input
    pub fn into_spi_sdi(self) -> Sdi<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Sdi(self)
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

impl Pin<'_, Mk64Fx512, 3, 5> {
    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
    }
}

impl Pin<'_, Mk64Fx512, 3, 6> {
    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
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

impl Pin<'_, Mk64Fx512, 3, 15> {
    /// Use this pin as an SPI chip select
    pub fn into_spi_cs(self) -> Cs<Self> {
        self.reg.update(|ctl| {
            ctl.set_bits(8..11, 2);
        });
        Cs(self)
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

/// A pin which is configured as a GPIO
pub struct Gpio<P>(P);

/// A pin which is configured as an SPI SDO
pub struct Sdo<P>(P);

/// A pin which is configured as an SPI SDI
pub struct Sdi<P>(P);

/// A pin which is configured as an SPI clock
pub struct Sck<P>(P);

/// A pin which is configured as an SPI chip select
pub struct Cs<P>(P);

impl<M, const N: usize, const P: usize> Gpio<Pin<'_, M, N, P>> {
    /// Set this pin as high or low
    pub fn write(&mut self, value: bool) {
        unsafe {
            let pdor: &mut Register<u32> = &mut *bitband_address(0x400F_F000 + 0x40 * N, P);
            pdor.write(value.into());
        }
    }

    /// Read the status of this pin
    pub fn read(&self) -> bool {
        unsafe {
            let pdir: &Register<u32> = &*bitband_address(0x400F_F010 + 0x40 * N, P);
            pdir.read() != 0
        }
    }

    /// Set whether this pin is an output or an input
    pub fn set_output(&mut self, output: bool) {
        unsafe {
            let pddr: &mut Register<u32> = &mut *bitband_address(0x400F_F014 + 0x40 * N, P);
            pddr.write(output.into());
        }
    }

    /// Set pullup/down resistors on this pin
    pub fn set_pull(&mut self, pull: Option<Pull>) {
        self.0.reg.update(|pcr| {
            match pull {
                Some(pull) => {
                    match pull {
                        Pull::Up => pcr.set_bit(0, true),
                        Pull::Down => pcr.set_bit(0, false),
                    };
                    pcr.set_bit(1, true)
                }
                None => pcr.set_bit(1, false),
            };
        });
    }

    /// Set whether this pin is open-drain.
    pub fn set_open_drain(&mut self, open_drain: bool) {
        self.0.reg.update(|pcr| {
            pcr.set_bit(5, open_drain);
        });
    }
}

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

impl super::spi::Sdi<Mk20Dx128, 0> for Sdi<Pin<'_, Mk20Dx128, 2, 7>> {}
impl super::spi::Sdo<Mk20Dx128, 0> for Sdo<Pin<'_, Mk20Dx128, 2, 6>> {}
impl super::spi::Sck<Mk20Dx128, 0> for Sck<Pin<'_, Mk20Dx128, 2, 5>> {}
impl super::spi::Cs<Mk20Dx128, 0> for Cs<Pin<'_, Mk20Dx128, 2, 0>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 4
    }
}
impl super::spi::Cs<Mk20Dx128, 0> for Cs<Pin<'_, Mk20Dx128, 2, 3>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 1
    }
}
impl super::spi::Cs<Mk20Dx128, 0> for Cs<Pin<'_, Mk20Dx128, 2, 4>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 0
    }
}
impl super::spi::Cs<Mk20Dx128, 0> for Cs<Pin<'_, Mk20Dx128, 3, 5>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 2
    }
}
impl super::spi::Cs<Mk20Dx128, 0> for Cs<Pin<'_, Mk20Dx128, 3, 6>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 3
    }
}

impl super::spi::Sdi<Mk20Dx256, 0> for Sdi<Pin<'_, Mk20Dx256, 2, 7>> {}
impl super::spi::Sdo<Mk20Dx256, 0> for Sdo<Pin<'_, Mk20Dx256, 2, 6>> {}
impl super::spi::Sck<Mk20Dx256, 0> for Sck<Pin<'_, Mk20Dx256, 2, 5>> {}
impl super::spi::Cs<Mk20Dx256, 0> for Cs<Pin<'_, Mk20Dx256, 2, 0>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 4
    }
}
impl super::spi::Cs<Mk20Dx256, 0> for Cs<Pin<'_, Mk20Dx256, 2, 3>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 1
    }
}
impl super::spi::Cs<Mk20Dx256, 0> for Cs<Pin<'_, Mk20Dx256, 2, 4>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 0
    }
}
impl super::spi::Cs<Mk20Dx256, 0> for Cs<Pin<'_, Mk20Dx256, 3, 5>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 2
    }
}
impl super::spi::Cs<Mk20Dx256, 0> for Cs<Pin<'_, Mk20Dx256, 3, 6>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 3
    }
}

impl super::spi::Sdi<Mk64Fx512, 1> for Sdi<Pin<'_, Mk64Fx512, 1, 17>> {}
impl super::spi::Sdi<Mk64Fx512, 2> for Sdi<Pin<'_, Mk64Fx512, 1, 23>> {}
impl super::spi::Sdi<Mk64Fx512, 0> for Sdi<Pin<'_, Mk64Fx512, 2, 7>> {}
impl super::spi::Sdo<Mk64Fx512, 1> for Sdo<Pin<'_, Mk64Fx512, 1, 16>> {}
impl super::spi::Sdo<Mk64Fx512, 2> for Sdo<Pin<'_, Mk64Fx512, 1, 22>> {}
impl super::spi::Sdo<Mk64Fx512, 0> for Sdo<Pin<'_, Mk64Fx512, 2, 6>> {}
impl super::spi::Sck<Mk64Fx512, 1> for Sck<Pin<'_, Mk64Fx512, 1, 11>> {}
impl super::spi::Sck<Mk64Fx512, 2> for Sck<Pin<'_, Mk64Fx512, 1, 21>> {}
impl super::spi::Sck<Mk64Fx512, 0> for Sck<Pin<'_, Mk64Fx512, 2, 5>> {}
impl super::spi::Cs<Mk64Fx512, 1> for Cs<Pin<'_, Mk64Fx512, 1, 10>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 0
    }
}
impl super::spi::Cs<Mk64Fx512, 2> for Cs<Pin<'_, Mk64Fx512, 1, 20>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 0
    }
}
impl super::spi::Cs<Mk64Fx512, 0> for Cs<Pin<'_, Mk64Fx512, 2, 0>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 4
    }
}
impl super::spi::Cs<Mk64Fx512, 0> for Cs<Pin<'_, Mk64Fx512, 2, 3>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 1
    }
}
impl super::spi::Cs<Mk64Fx512, 0> for Cs<Pin<'_, Mk64Fx512, 2, 4>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 0
    }
}
impl super::spi::Cs<Mk64Fx512, 0> for Cs<Pin<'_, Mk64Fx512, 3, 5>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 2
    }
}
impl super::spi::Cs<Mk64Fx512, 0> for Cs<Pin<'_, Mk64Fx512, 3, 6>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 3
    }
}
impl super::spi::Cs<Mk64Fx512, 2> for Cs<Pin<'_, Mk64Fx512, 3, 15>> {
    fn cs_allowed(&self, bit: usize) -> bool {
        bit == 1
    }
}

unsafe impl GatedPeripheral<Mk20Dx128> for Port<Mk20Dx128, 0> {
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

unsafe impl GatedPeripheral<Mk20Dx128> for Port<Mk20Dx128, 1> {
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

unsafe impl GatedPeripheral<Mk20Dx128> for Port<Mk20Dx128, 2> {
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

unsafe impl GatedPeripheral<Mk20Dx128> for Port<Mk20Dx128, 3> {
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

unsafe impl GatedPeripheral<Mk20Dx128> for Port<Mk20Dx128, 4> {
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

unsafe impl GatedPeripheral<Mk20Dx256> for Port<Mk20Dx256, 0> {
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

unsafe impl GatedPeripheral<Mk20Dx256> for Port<Mk20Dx256, 1> {
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

unsafe impl GatedPeripheral<Mk20Dx256> for Port<Mk20Dx256, 2> {
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

unsafe impl GatedPeripheral<Mk20Dx256> for Port<Mk20Dx256, 3> {
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

unsafe impl GatedPeripheral<Mk20Dx256> for Port<Mk20Dx256, 4> {
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

unsafe impl GatedPeripheral<Mk64Fx512> for Port<Mk64Fx512, 0> {
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

unsafe impl GatedPeripheral<Mk64Fx512> for Port<Mk64Fx512, 1> {
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

unsafe impl GatedPeripheral<Mk64Fx512> for Port<Mk64Fx512, 2> {
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

unsafe impl GatedPeripheral<Mk64Fx512> for Port<Mk64Fx512, 3> {
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

unsafe impl GatedPeripheral<Mk64Fx512> for Port<Mk64Fx512, 4> {
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

unsafe impl GatedPeripheral<Mk66Fx1M0> for Port<Mk66Fx1M0, 0> {
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

unsafe impl GatedPeripheral<Mk66Fx1M0> for Port<Mk66Fx1M0, 1> {
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

unsafe impl GatedPeripheral<Mk66Fx1M0> for Port<Mk66Fx1M0, 2> {
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

unsafe impl GatedPeripheral<Mk66Fx1M0> for Port<Mk66Fx1M0, 3> {
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

unsafe impl GatedPeripheral<Mk66Fx1M0> for Port<Mk66Fx1M0, 4> {
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

unsafe impl GatedPeripheral<Mkl26Z64> for Port<Mkl26Z64, 0> {
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

unsafe impl GatedPeripheral<Mkl26Z64> for Port<Mkl26Z64, 1> {
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

unsafe impl GatedPeripheral<Mkl26Z64> for Port<Mkl26Z64, 2> {
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

unsafe impl GatedPeripheral<Mkl26Z64> for Port<Mkl26Z64, 3> {
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

unsafe impl GatedPeripheral<Mkl26Z64> for Port<Mkl26Z64, 4> {
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

unsafe fn bitband_address<T>(addr: usize, bit: usize) -> *mut T {
    (0x4200_0000 + (addr - 0x4000_0000) * 32 + bit * 4) as _
}
