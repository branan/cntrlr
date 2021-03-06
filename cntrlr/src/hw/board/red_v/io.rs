// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! IO functionality specific to the Sparkfun Red V board.

use crate::{
    hw::mcu::sifive::{
        fe310g002::{Pin, Uart, UartRx, UartTx},
        peripheral::{uart, Peripheral},
        Fe310G002,
    },
    io::{self, Read, Write},
    sync::{Mutex, MutexGuard},
    task::WakerSet,
};
use core::{
    future::{poll_fn, ready, Future},
    sync::atomic::Ordering,
    task::Poll,
};

/// An error from a serial interface
#[derive(Debug)]
#[non_exhaustive]
pub enum SerialError {
    /// The serial port cannot be read or written because it is disabled
    NotEnabled,

    /// The serial port cannot be enabled because its TX or RX pin is in use
    PinInUse,

    /// The serial port cannot be enabled because its UART is in use
    UartInUse,

    /// The serial port cannot be enabled because the GPIO is in use
    GpioInUse,

    /// The serial port cannot be enabled because the selected baud rate is invalid
    InvalidBaud,

    /// The serial port cannot be enabled because a requested option is invalid
    InvalidOption,
}

/// A serial interface
///
/// This wraps a UART and provides application-level functionality.
pub struct Serial<T, R, const N: usize>(Option<Uart<T, R, N>>, Option<&'static WakerSet>);

impl<T, R, const N: usize> Read for Serial<T, R, N>
where
    T: 'static,
    R: uart::UartRx<Fe310G002, N> + 'static,
{
    type Error = SerialError;
    type Future<'a> = impl Future<Output = Result<usize, Self::Error>>;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> <Self as Read>::Future<'a>
    where
        Self: 'a,
    {
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

impl<T, R, const N: usize> Write for Serial<T, R, N>
where
    T: uart::UartTx<Fe310G002, N> + 'static,
    R: 'static,
{
    type Error = SerialError;
    type Future<'a> = impl Future<Output = Result<usize, Self::Error>>;
    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> <Self as Write>::Future<'a>
    where
        Self: 'a,
    {
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

    fn flush<'a>(&'a mut self) -> <Self as Write>::FlushFuture<'a>
    where
        Self: 'a,
    {
        // Not possible to flush on this hardware - just say we're
        // flushed.
        ready(Ok(()))
    }
}

/// The pin used to recieve for serial 1
pub type Serial1Rx = UartRx<Pin<'static, 0, 16>>;

/// The pin used to transmit for serial 1
pub type Serial1Tx = UartTx<Pin<'static, 0, 17>>;

/// The pin used to recieve for serial 2
pub type Serial2Rx = UartRx<Pin<'static, 0, 23>>;

/// The pin used to transmit for serial 2
pub type Serial2Tx = UartTx<Pin<'static, 0, 18>>;

impl<T, R, const N: usize> Serial<T, R, N>
where
    T: uart::UartTx<Fe310G002, N>,
    R: uart::UartRx<Fe310G002, N>,
    Uart<(), (), N>: Peripheral,
{
    fn do_enable(
        &mut self,
        baud: usize,
        options: &[io::SerialOption],
        tx: T,
        rx: R,
        wakers: &'static WakerSet,
    ) -> Result<(), SerialError> {
        let divisor = super::CPU_FREQ.load(Ordering::Relaxed) as usize / baud;
        if divisor < 16 {
            return Err(SerialError::InvalidBaud);
        }
        if !options.is_empty() {
            return Err(SerialError::InvalidOption);
        }
        let mut uart = Uart::<(), (), N>::get().ok_or(SerialError::UartInUse)?;
        uart.set_divisor(divisor);
        uart.set_watermarks(7, 0);
        self.0 = Some(uart.enable_tx(tx).enable_rx(rx));
        self.1 = Some(wakers);
        Ok(())
    }
}

impl io::Serial for Serial<Serial1Tx, Serial1Rx, 0> {
    type Error = SerialError;
    fn enable_with_options(
        &mut self,
        baud: usize,
        options: &[io::SerialOption],
    ) -> Result<(), <Self as io::Serial>::Error> {
        let tx = super::digital::gpio()
            .ok_or(SerialError::GpioInUse)?
            .pin::<17>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_tx();
        let rx = super::digital::gpio()
            .ok_or(SerialError::GpioInUse)?
            .pin::<16>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_rx();
        self.do_enable(baud, options, tx, rx, &SERIAL_1_WAKERS)
    }

    fn disable(&mut self) -> Result<(), <Self as io::Serial>::Error> {
        self.0 = None;
        self.1 = None;
        Ok(())
    }
}

impl io::Serial for Serial<Serial2Tx, Serial2Rx, 1> {
    type Error = SerialError;
    fn enable_with_options(
        &mut self,
        baud: usize,
        options: &[io::SerialOption],
    ) -> Result<(), <Self as io::Serial>::Error> {
        let tx = super::digital::gpio()
            .ok_or(SerialError::GpioInUse)?
            .pin::<18>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_tx();
        let rx = super::digital::gpio()
            .ok_or(SerialError::GpioInUse)?
            .pin::<23>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_rx();
        self.do_enable(baud, options, tx, rx, &SERIAL_2_WAKERS)
    }

    fn disable(&mut self) -> Result<(), <Self as io::Serial>::Error> {
        self.0 = None;
        self.1 = None;
        Ok(())
    }
}

/// The serial connection to a host PC
///
/// On this board, this is an alias for [`serial_1`]. If you intend to
/// use the serial port to communicate with outside hardware via pins
/// 0 and 1, you should prefer to use [`serial_1`] for compatibility
/// with board which differentiate those serial ports.
pub fn pc_serial() -> MutexGuard<'static, Serial<Serial1Tx, Serial1Rx, 0>> {
    serial_1()
}

/// The first hardware serial port
///
/// On this board, this is an alias for [`pc_serial`]. If you intend
/// to use the serial port to communicate with a hose PC, you should
/// prefer to use [`pc_serial`] for compatibility with boards which
/// differentiate those serial ports.
pub fn serial_1() -> MutexGuard<'static, Serial<Serial1Tx, Serial1Rx, 0>> {
    static SERIAL: Mutex<Serial<Serial1Tx, Serial1Rx, 0>> = Mutex::new(Serial(None, None));
    SERIAL.lock()
}

/// The second hardware serial port
pub fn serial_2() -> MutexGuard<'static, Serial<Serial2Tx, Serial2Rx, 1>> {
    static SERIAL: Mutex<Serial<Serial2Tx, Serial2Rx, 1>> = Mutex::new(Serial(None, None));
    SERIAL.lock()
}

static SERIAL_1_WAKERS: WakerSet = WakerSet::new();
static SERIAL_2_WAKERS: WakerSet = WakerSet::new();

/// The interrupt function for serial 1
pub extern "C" fn serial_1_intr() {
    #[cfg(board = "red_v")]
    unsafe {
        const UART_IE: *mut u32 = 0x1001_3010 as _;
        asm!("amoand.w {}, {}, ({})", out(reg) _, in(reg) 0xFFFF_FFFCu32, in(reg) UART_IE);
        SERIAL_1_WAKERS.wake();
    }
}

/// The interrupt function for serial 2
pub extern "C" fn serial_2_intr() {
    #[cfg(board = "red_v")]
    unsafe {
        const UART_IE: *mut u32 = 0x1002_3010 as _;
        asm!("amoand.w {}, {}, ({})", out(reg) _, in(reg) 0xFFFF_FFFCu32, in(reg) UART_IE);
        SERIAL_2_WAKERS.wake();
    }
}
