// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! IO functionality shared between the various Teensy 3.x boards

use crate::{
    hw::mcu::kinetis::peripheral::{
        sim::{GatedPeripheral, Sim},
        uart::{Uart, UartRx, UartTx},
        Peripheral,
    },
    io::{self, SerialOption},
    task::WakerSet,
};
use core::{
    future::{poll_fn, Future},
    task::Poll,
};

/// An error from a serial instance
#[derive(Debug)]
#[non_exhaustive]
pub enum SerialError {
    /// The serial port cannot be read or written because it is disabled
    NotEnabled,

    /// The serial port cannot be enabled because its TX or RX pin is in use
    PinInUse,

    /// The serial port cannot be enabled because its PORT is in use
    PortInUse,

    /// The serial port cannot be enabled because its UART is in use
    UartInUse,

    /// The serial port cannot be enabled because the SIM is in use
    SimInUse,

    /// The serial port cannot be enabled because the selected baud rate is invalid
    InvalidBaud,
}

/// A serial instance
///
/// This wraps a UART and provides application-level functionality.
pub struct Serial<M, T, R, const N: usize>(
    pub(crate) Option<Uart<M, T, R, N>>,
    pub(crate) Option<&'static WakerSet>,
);

impl<M, T, R, const N: usize> Serial<M, T, R, N>
where
    T: UartTx<M, N>,
    R: UartRx<M, N>,
    Uart<M, (), (), N>: GatedPeripheral<M>,
    Sim<M>: Peripheral,
{
    pub(crate) fn do_enable(
        &mut self,
        baud: usize,
        options: &[SerialOption],
        tx: T,
        rx: R,
        source_clock: usize,
        wakers: &'static WakerSet,
    ) -> Result<(), SerialError> {
        let divisor = (source_clock * 32) / (baud * 16);
        if divisor < 32 {
            return Err(SerialError::InvalidBaud);
        }
        let mut uart = Sim::<M>::get()
            .ok_or(SerialError::SimInUse)?
            .enable_peripheral::<Uart<M, (), (), N>>()
            .ok_or(SerialError::UartInUse)?;
        uart.set_divisor(divisor);

        for option in options {
            match option {
                SerialOption::Invert(invert) => uart.invert(*invert),
            }
        }

        self.0 = Some(uart.enable_tx(tx).enable_rx(rx));
        self.1 = Some(wakers);
        Ok(())
    }
}

impl<M, T, R, const N: usize> io::Read for Serial<M, T, R, N>
where
    M: 'static,
    T: 'static,
    R: UartRx<M, N> + 'static,
{
    type Error = SerialError;
    type Future<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::Future<'a> {
        poll_fn(move |ctx| {
            if buf.is_empty() {
                return Poll::Ready(Ok(0));
            }

            let mut count = 0;
            let uart = self.0.as_mut().ok_or(SerialError::NotEnabled)?;
            while let Some(byte) = uart.read_data() {
                buf[count] = byte;
                count += 1;
                if count >= buf.len() {
                    break;
                }
            }
            if count > 0 {
                Poll::Ready(Ok(count))
            } else {
                if let Some(wakers) = self.1.as_ref() {
                    wakers.add(ctx.waker().clone());
                }
                uart.enable_rx_intr();
                Poll::Pending
            }
        })
    }
}

impl<M, T, R, const N: usize> io::Write for Serial<M, T, R, N>
where
    M: 'static,
    T: UartTx<M, N> + 'static,
    R: 'static,
{
    type Error = SerialError;
    type Future<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a;
    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::Future<'a> {
        poll_fn(move |ctx| {
            if buf.is_empty() {
                return Poll::Ready(Ok(0));
            }

            let mut count = 0;
            let mut buf = buf;
            let uart = self.0.as_mut().ok_or(SerialError::NotEnabled)?;
            while uart.write_data(buf[0]) {
                count += 1;
                buf = &buf[1..];
                if buf.is_empty() {
                    break;
                }
            }
            if count > 0 {
                Poll::Ready(Ok(count))
            } else {
                if let Some(wakers) = self.1.as_ref() {
                    wakers.add(ctx.waker().clone());
                }
                uart.enable_tx_intr();
                Poll::Pending
            }
        })
    }

    #[allow(clippy::needless_lifetimes)] // This lint is incorrect for GATs
    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        poll_fn(move |ctx| {
            let uart = self.0.as_mut().ok_or(SerialError::NotEnabled)?;
            if uart.is_transmit_complete() {
                Poll::Ready(Ok(()))
            } else {
                if let Some(wakers) = self.1.as_ref() {
                    wakers.add(ctx.waker().clone());
                }
                uart.enable_complete_intr();
                Poll::Pending
            }
        })
    }
}

impl<M, T, R, const N: usize> Serial<M, T, R, N> {
    /// Create a new instance of a serial port, in a disabled state.
    pub const fn new() -> Self {
        Self(None, None)
    }
}
