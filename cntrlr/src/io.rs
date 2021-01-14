// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! I/O functionality for Cntrlr boards

use alloc::string::String;
use cntrlr_macros::board_fn;
use core::{fmt::Debug, future::Future, ops::DerefMut};

/// Error type for [`ReadExt::read_line`]
#[derive(Debug)]
pub enum LineError<E> {
    /// Error from the underlying implementation
    Read(E),

    /// Error decoding the read bytes as UTF-8
    Utf8(core::str::Utf8Error),
}

/// Serial configuration options
#[non_exhaustive]
pub enum SerialOption {
    /// Invert the serial polarity
    Invert(bool),
}

/// SPI configuration options
#[non_exhaustive]
pub enum SpiOption {
    /// Use a hardware chip select for the given pin
    HardwareCs(usize),
}

/// Allows reading bytes from a source
pub trait Read {
    /// The error type
    type Error: Debug;

    /// The future for [`Self::read()`]
    type Future<'a>: Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

    /// Read bytes from the device
    ///
    /// This reads as many bytes as are currently available, up to
    /// `buf.len()`, and returns the number of bytes written.
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::Future<'a>
    where
        Self: 'a;
}

/// Allows writing bytes to  a sink
pub trait Write {
    /// The error type
    type Error: Debug;

    /// The future for [`Self::write()`]
    type Future<'a>: Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

    /// The future for [`Self::flush()`]
    type FlushFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Write bytes to the device
    ///
    /// This writes as many bytes as possible, up to `buf.len()`, and
    /// returns the number of bytes written.
    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::Future<'a>
    where
        Self: 'a;

    /// Ensure all written bytes have been transmitted
    ///
    /// On devices which do not support this operation, this function
    /// does nothing.
    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a>
    where
        Self: 'a;
}

/// Extended functions for reading bytes
pub trait ReadExt: Read {
    /// The future for [`Self::read_exact()`]
    type ExactFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// The future for [`Self::read_line()`]
    type LineFuture<'a>: Future<Output = Result<(), LineError<Self::Error>>> + 'a
    where
        Self: 'a;

    /// Read bytes from the device
    ///
    /// This reads exactly `buf.len()` bytes from the device.
    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ExactFuture<'a>
    where
        Self: 'a;

    /// Read a line from the device
    ///
    /// This reads bytes from the device into the passed
    /// [`String`] until it reaches a newline
    /// (0x0A). The newline will also be included in the output
    /// string.
    fn read_line<'a>(&'a mut self, buf: &'a mut String) -> Self::LineFuture<'a>
    where
        Self: 'a;
}

impl<T: Read> ReadExt for T {
    #[rustfmt::skip]
    type ExactFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;
    #[rustfmt::skip]
    type LineFuture<'a> where Self: 'a = impl Future<Output = Result<(), LineError<Self::Error>>> + 'a;

    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ExactFuture<'a>
    where
        Self: 'a,
    {
        async move {
            let mut buf = buf;
            while !buf.is_empty() {
                let read = self.read(buf).await?;
                buf = &mut buf[read..];
            }
            Ok(())
        }
    }

    fn read_line<'a>(&'a mut self, buf: &'a mut String) -> Self::LineFuture<'a>
    where
        Self: 'a,
    {
        async move {
            let buf = unsafe { buf.as_mut_vec() };
            loop {
                let mut byte = [0];
                self.read(&mut byte).await.map_err(LineError::Read)?;
                buf.push(byte[0]);
                if byte[0] == b'\n' {
                    break;
                }
            }
            let _ = core::str::from_utf8(&buf).map_err(LineError::Utf8)?;
            Ok(())
        }
    }
}

/// Extended functions for writing bytes
pub trait WriteExt: Write {
    /// The future for [`Self::write_all()`]
    type AllFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// The future for [`Self::write_fmt()`]
    type FmtFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Write bytes to the device
    ///
    /// This writes exactly `buf.len()` bytes to the device.
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> Self::AllFuture<'a>
    where
        Self: 'a;

    /// Write a formatted message to the device
    ///
    /// Internally, this will allocate a [`String`] to hold the
    /// formatted output.
    fn write_fmt<'a>(&'a mut self, fmt: core::fmt::Arguments<'a>) -> Self::FmtFuture<'a>
    where
        Self: 'a;
}

impl<T: Write> WriteExt for T {
    #[rustfmt::skip]
    type AllFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    #[rustfmt::skip]
    type FmtFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> Self::AllFuture<'a>
    where
        Self: 'a,
    {
        async move {
            let mut buf = buf;
            while !buf.is_empty() {
                let written = self.write(buf).await?;
                buf = &buf[written..];
            }
            Ok(())
        }
    }

    fn write_fmt<'a>(&'a mut self, fmt: core::fmt::Arguments<'a>) -> Self::FmtFuture<'a>
    where
        Self: 'a,
    {
        async move {
            use alloc::format;

            let formatted = format!("{}", fmt);
            self.write_all(formatted.as_bytes()).await
        }
    }
}

/// Trait for RS232/UART-style serial devices
pub trait Serial: Read + Write {
    /// The error type
    type Error: Debug;

    /// Enable the serial port at the specified baud rate
    fn enable(&mut self, baud: usize) -> Result<(), <Self as Serial>::Error> {
        self.enable_with_options(baud, &[])
    }

    /// Enable the serial port at the specified baud rate, with the selected options
    fn enable_with_options(
        &mut self,
        baud: usize,
        options: &[SerialOption],
    ) -> Result<(), <Self as Serial>::Error>;

    /// Disable the serial port.
    fn disable(&mut self) -> Result<(), <Self as Serial>::Error>;
}

/// Trait for SPI devices
///
/// SPI is a synchronous, bidirectional protocol. See
/// [`SpiTransfer::transfer()`] for details of how this
/// bidirectionality is handled in the API.
pub trait Spi {
    /// The error type
    type Error: Debug;

    /// The associated transfer type
    type Transfer<'a>: SpiTransfer<Error = Self::Error>
        + Read<Error = Self::Error>
        + Write<Error = Self::Error>
        + 'a
    where
        Self: 'a;

    /// The Future for starting a transfr
    type TransferFuture<'a>: Future<Output = Result<Self::Transfer<'a>, Self::Error>> + 'a
    where
        Self: 'a;

    /// The future for the `flush()` function
    type FlushFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
    where
        Self: 'a;

    /// Enable the SPI port
    fn enable(&mut self) -> Result<(), <Self as Spi>::Error> {
        self.enable_with_options(&[])
    }

    /// Enable the SPI port with the selected options
    fn enable_with_options(&mut self, options: &[SpiOption]) -> Result<(), <Self as Spi>::Error>;

    /// Disable the spi port
    fn disable(&mut self) -> Result<(), <Self as Spi>::Error>;

    /// Begin a transfer on this SPI port
    ///
    /// A transfer is any series of SPI packets to the same device, at
    /// a single baud rate. The actual baud rate may vary from the one
    /// requested, and will be the closest possible rate below the
    /// requested one.
    fn transfer<'a>(
        &'a mut self,
        baud: usize,
        cs: usize,
        packet: usize,
    ) -> Self::TransferFuture<'a>
    where
        Self: 'a;

    /// Flush any outstanding writes to this SPI port
    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a>
    where
        Self: 'a;
}

/// An SPI transfer to or from a device
///
/// All SPI transfers are bidirectional. See
/// [`SpiTransfer::transfer()`] for details.
pub trait SpiTransfer {
    /// The error type
    type Error: Debug;

    /// The future for a transfer
    type TransferFuture<'a>: Future<Output = Result<usize, Self::Error>> + 'a
    where
        Self: 'a;

    /// Perform a bidirectional SPI transfer
    ///
    /// SPI transfers are always done in a multiple of the packet
    /// size. If `buf_in` is below the packet size, the packet will be
    /// padded with zeroes. If `buf_out` is below the packet size, any
    /// additional read data will be discarded..
    ///
    /// At least one of `buf_in` or `buf_out` must be large enough for
    /// an entire packet to be written or read, respectively.
    fn transfer<'a>(
        &'a mut self,
        buf_in: &'a [u8],
        buf_out: &'a mut [u8],
    ) -> Self::TransferFuture<'a>
    where
        Self: 'a;
}

/// The serial connection to a host PC
///
/// On some boards, this is an alias for the serial port at
/// [`serial_1`]. If you intend to use the serial port for off-board
/// communication, you should use [`serial_1`] for compatibility with
/// boards which differentiate the two serial ports.
#[board_fn(io, red_v)]
pub fn pc_serial() -> impl DerefMut<Target = impl Serial> {}

/// The first hardware serial port
///
/// This port is typically on pins 0 and 1
///
/// On some boards, this is an alias for the serial port at
/// [`pc_serial`]. If you intend to use the serial port for PC
/// communication, you should use [`pc_serial`] for compatibility with
/// boards which differentiate the two serial ports.
#[board_fn(io, red_v, teensy_30, teensy_32, teensy_35, teensy_36, teensy_lc)]
pub fn serial_1() -> impl DerefMut<Target = impl Serial> {}

/// The second hardware serial port
#[board_fn(io, red_v, teensy_30, teensy_32, teensy_35, teensy_36, teensy_lc)]
pub fn serial_2() -> impl DerefMut<Target = impl Serial> {}

/// The third hardware serial port
#[board_fn(io, teensy_30, teensy_32, teensy_35, teensy_36, teensy_lc)]
pub fn serial_3() -> impl DerefMut<Target = impl Serial> {}

/// The fourth hardware serial port
#[board_fn(io, teensy_35, teensy_36)]
pub fn serial_4() -> impl DerefMut<Target = impl Serial> {}

/// The fifth hardware serial port
#[board_fn(io, teensy_35, teensy_36)]
pub fn serial_5() -> impl DerefMut<Target = impl Serial> {}

/// The sixth hardware serial port
#[board_fn(io, teensy_35)]
pub fn serial_6() -> impl DerefMut<Target = impl Serial> {}

/// The first hardware SPI port
///
/// Pinouts can vary, but on most boards, the following pinout is
/// used:
/// * 11: Data Out
/// * 12: Data In
/// * 13: Clock
///
/// Some boards support hardware chip selects. See the documentation
/// for your board for details.
#[board_fn(io, teensy_30, teensy_32, teensy_35, teensy_36)]
pub fn spi_1() -> impl DerefMut<Target = impl Spi> {}

/// The second hardware SPI port
#[board_fn(io, teensy_35, teensy_36)]
pub fn spi_2() -> impl DerefMut<Target = impl Spi> {}

/// The third hardware SPI port
#[board_fn(io, teensy_35, teensy_36)]
pub fn spi_3() -> impl DerefMut<Target = impl Spi> {}
