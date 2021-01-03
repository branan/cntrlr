// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! The UART

use super::super::Fe310G002;
use crate::{register::Register, sync::Flag};
use bit_field::BitField;
use core::{marker::PhantomData, sync::atomic::Ordering};

#[repr(C)]
struct UartRegs {
    txdata: Register<u32>,
    rxdata: Register<u32>,
    txctrl: Register<u32>,
    rxctrl: Register<u32>,
    ie: Register<u32>,
    ip: Register<u32>,
    div: Register<u32>,
}

/// A UART
pub struct Uart<M, T, R, const N: usize> {
    regs: &'static mut UartRegs,
    tx: T,
    rx: R,
    mcu: PhantomData<M>,
}

/// A GPIO pin which can be used as a UART tansmit pin
pub trait UartTx<M, const N: usize>: Unpin {}

/// A GPIO pin which can be used as a UART ecieve pin
pub trait UartRx<M, const N: usize>: Unpin {}

static LOCKS: [Flag; 2] = [Flag::new(false), Flag::new(false)];

impl Uart<Fe310G002, (), (), 0> {
    /// Get UART instance 0
    pub fn get() -> Option<Self> {
        unsafe { Self::do_get(0x1001_3000) }
    }
}

impl Uart<Fe310G002, (), (), 1> {
    /// Get UARt instance 1
    pub fn get() -> Option<Self> {
        unsafe { Self::do_get(0x1002_3000) }
    }
}

impl<M, const N: usize> Uart<M, (), (), N> {
    unsafe fn do_get(addr: usize) -> Option<Self> {
        if LOCKS[N].swap(true, Ordering::Acquire) {
            None
        } else {
            Some(Self {
                regs: &mut *(addr as *mut _),
                tx: (),
                rx: (),
                mcu: PhantomData,
            })
        }
    }

    /// Set the UART divisor
    ///
    /// The Uart is clocked at CPU_FREQ/div. If the UART is used as a
    /// reciever, this value must be at least 16.
    pub fn set_divisor(&mut self, div: usize) {
        assert!(div >= 1);
        self.regs.div.write(div as u32 - 1);
    }
}

impl<M, T, const N: usize> Uart<M, T, (), N> {
    /// Enable this UART as a reciever
    pub fn enable_rx<R: UartRx<M, N>>(self, rx: R) -> Uart<M, T, R, N> {
        self.regs.txctrl.update(|rxctrl| {
            rxctrl.set_bit(0, true);
        });
        Uart {
            regs: self.regs,
            tx: self.tx,
            rx,
            mcu: self.mcu,
        }
    }
}

impl<M, R, const N: usize> Uart<M, (), R, N> {
    /// Enable this UART as a transmitter
    pub fn enable_tx<T: UartTx<M, N>>(self, tx: T) -> Uart<M, T, R, N> {
        self.regs.txctrl.update(|txctrl| {
            txctrl.set_bit(0, true);
        });
        Uart {
            regs: self.regs,
            tx,
            rx: self.rx,
            mcu: self.mcu,
        }
    }
}

impl<M, T, R: UartRx<M, N>, const N: usize> Uart<M, T, R, N> {
    /// Read a byte from the UART.
    ///
    /// Retuns `None` if no data is available to be read
    pub fn read_data(&mut self) -> Option<u8> {
        let data = self.regs.rxdata.read();
        if data.get_bit(31) {
            None
        } else {
            Some(data.get_bits(0..8) as u8)
        }
    }

    /// Enable the UART to interrupt the CPU when the recieve FIFO is
    /// above the watermark.
    pub fn enable_rx_intr(&mut self) {
        self.regs.ie.update(|ie| {
            ie.set_bit(1, true);
        });
    }
}

impl<M, T: UartTx<M, N>, R, const N: usize> Uart<M, T, R, N> {
    /// Write a byte to the UART
    ///
    /// Returns `false` if the byte cannot be written
    pub fn write_data(&mut self, data: u8) -> bool {
        if self.regs.txdata.read().get_bit(31) {
            false
        } else {
            self.regs.txdata.write(data as u32);
            true
        }
    }

    /// Enable the UART to interrupt the CPU when the transmit FIFO is
    /// below the watermark.
    pub fn enable_tx_intr(&mut self) {
        self.regs.ie.update(|ie| {
            ie.set_bit(0, true);
        });
    }
}

impl<M, T, R, const N: usize> Uart<M, T, R, N> {
    /// Set the FIFO interrupt watermarks. Both the TX and RX
    /// watermarks must be in the range `0..8`.
    pub fn set_watermarks(&mut self, tx: u32, rx: u32) {
        assert!(tx <= 7);
        assert!(rx <= 7);

        self.regs.txctrl.update(|txctrl| {
            txctrl.set_bits(16..19, tx);
        });
        self.regs.rxctrl.update(|rxctrl| {
            rxctrl.set_bits(16..19, rx);
        });
    }
}
