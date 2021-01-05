// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! The UART

use crate::{register::Register, sync::Flag};
use bit_field::BitField;
use core::{marker::PhantomData, mem::ManuallyDrop, sync::atomic::Ordering};

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
    regs: ManuallyDrop<&'static mut UartRegs>,
    tx: ManuallyDrop<T>,
    rx: ManuallyDrop<R>,
    mcu: PhantomData<M>,
}

/// A GPIO pin which can be used as a UART tansmit pin
pub trait UartTx<M, const N: usize>: Unpin {}

/// A GPIO pin which can be used as a UART ecieve pin
pub trait UartRx<M, const N: usize>: Unpin {}

static LOCKS: [Flag; 2] = [Flag::new(false), Flag::new(false)];

macro_rules! get {
    ($i:literal, $a:literal) => {
        #[cfg(any(doc, mcu = "fe310g002"))]
        #[cfg_attr(feature = "doc-cfg", doc(cfg(mcu = "fe310g002")))]
        impl super::Peripheral for Uart<super::super::Fe310G002, (), (), $i> {
            fn get() -> Option<Self> {
                unsafe {
                    if LOCKS[$i].swap(true, Ordering::Acquire) {
                        None
                    } else {
                        Some(Self {
                            regs: ManuallyDrop::new(&mut *($a as *mut _)),
                            tx: ManuallyDrop::new(()),
                            rx: ManuallyDrop::new(()),
                            mcu: PhantomData,
                        })
                    }
                }
            }
        }
    };
}

get!(0, 0x1001_3000);
get!(1, 0x1002_3000);

impl<M, const N: usize> Uart<M, (), (), N>
where
    Uart<M, (), (), N>: super::Peripheral,
{
    /// Get the handle to a UART
    ///
    /// Returns 'None' if the UART is already in use.
    pub fn get() -> Option<Self> {
        super::Peripheral::get()
    }
}

impl<M, const N: usize> Uart<M, (), (), N> {
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
    pub fn enable_rx<R: UartRx<M, N>>(mut self, rx: R) -> Uart<M, T, R, N> {
        self.regs.txctrl.update(|rxctrl| {
            rxctrl.set_bit(0, true);
        });
        unsafe {
            let regs = ManuallyDrop::new(ManuallyDrop::take(&mut self.regs));
            let tx = ManuallyDrop::new(ManuallyDrop::take(&mut self.tx));
            let rx = ManuallyDrop::new(rx);
            let mcu = self.mcu;
            ManuallyDrop::drop(&mut self.rx);
            core::mem::forget(self);
            Uart { regs, tx, rx, mcu }
        }
    }
}

impl<M, R, const N: usize> Uart<M, (), R, N> {
    /// Enable this UART as a transmitter
    pub fn enable_tx<T: UartTx<M, N>>(mut self, tx: T) -> Uart<M, T, R, N> {
        self.regs.txctrl.update(|txctrl| {
            txctrl.set_bit(0, true);
        });
        unsafe {
            let regs = ManuallyDrop::new(ManuallyDrop::take(&mut self.regs));
            let tx = ManuallyDrop::new(tx);
            let rx = ManuallyDrop::new(ManuallyDrop::take(&mut self.rx));
            let mcu = self.mcu;
            ManuallyDrop::drop(&mut self.tx);
            core::mem::forget(self);
            Uart { regs, tx, rx, mcu }
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

impl<M, T, R, const N: usize> Drop for Uart<M, T, R, N> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.tx);
            ManuallyDrop::drop(&mut self.rx);
            LOCKS[N].store(false, Ordering::Release);
        }
    }
}
