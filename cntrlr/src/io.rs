use cntrlr_macros::board_fn;
use core::{fmt::Debug, future::Future, ops::DerefMut};

#[derive(Debug)]
pub enum LineError<E> {
    Read(E),
    Utf8(core::str::Utf8Error),
}

pub trait Read {
    type Error: Debug;
    type Future<'a>: Future<Output = Result<usize, Self::Error>> + 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::Future<'a>;
}

pub trait Write {
    type Error: Debug;
    type Future<'a>: Future<Output = Result<usize, Self::Error>> + 'a;
    type FlushFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::Future<'a>;
    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a>;
}

pub trait ReadExt: Read {
    type ExactFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;
    type LineFuture<'a>: Future<Output = Result<(), LineError<Self::Error>>> + 'a;
    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ExactFuture<'a>;
    fn read_line<'a>(&'a mut self, buf: &'a mut alloc::string::String) -> Self::LineFuture<'a>;
}

impl<T: Read + 'static> ReadExt for T {
    type ExactFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
    type LineFuture<'a> = impl Future<Output = Result<(), LineError<Self::Error>>> + 'a;

    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ExactFuture<'a> {
        async move {
            let mut buf = buf;
            while buf.len() > 0 {
                let read = self.read(buf).await?;
                buf = &mut buf[read..];
            }
            Ok(())
        }
    }

    fn read_line<'a>(&'a mut self, buf: &'a mut alloc::string::String) -> Self::LineFuture<'a> {
        async move {
            let buf = unsafe { buf.as_mut_vec() };
            loop {
                let mut byte = [0];
                self.read(&mut byte).await.map_err(|e| LineError::Read(e))?;
                buf.push(byte[0]);
                if byte[0] == b'\n' {
                    break;
                }
            }
            let _ = core::str::from_utf8(&buf).map_err(|e| LineError::Utf8(e))?;
            Ok(())
        }
    }
}

pub trait WriteExt: Write {
    type AllFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;
    type FmtFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a;

    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> Self::AllFuture<'a>;
    fn write_fmt<'a>(&'a mut self, fmt: core::fmt::Arguments<'a>) -> Self::FmtFuture<'a>;
}

impl<T: Write + 'static> WriteExt for T {
    type AllFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
    type FmtFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> Self::AllFuture<'a> {
        async move {
            let mut buf = buf;
            while buf.len() > 0 {
                let written = self.write(buf).await?;
                buf = &buf[written..];
            }
            Ok(())
        }
    }

    fn write_fmt<'a>(&'a mut self, fmt: core::fmt::Arguments<'a>) -> Self::FmtFuture<'a> {
        async move {
            use alloc::format;

            let formatted = format!("{}", fmt);
            self.write_all(formatted.as_bytes()).await
        }
    }
}

pub trait Serial: Read + Write {
    type Error: Debug;

    fn enable(&mut self, baud: usize) -> Result<(), <Self as Serial>::Error>;
    fn disable(&mut self) -> Result<(), <Self as Serial>::Error>;
}

#[derive(Clone, Copy, PartialEq)]
pub enum Parity {
    Even,
    Odd,
}

/// The serial connection to a host PC
///
/// On some boards, this is an alias for the serial port at
/// [`serial_1`]. If you intend to use the serial port for off-board
/// communication, you should use [`serial_1`] for compatibility with
/// boards which differentiate the two serial ports.
#[board_fn(io, arduino_uno, red_v)]
pub fn pc_serial() -> impl DerefMut<Target = impl Serial> {}

/// The first serial port
///
/// This port is typically on pins 0 and 1
///
/// On some boards, this is an alias for the serial port at
/// [`pc_serial`]. If you intend to use the serial port for PC
/// communication, you should use [`pc_serial`] for compatibility with
/// boards which differentiate the two serial ports.
#[board_fn(
    io,
    arduino_uno,
    arduino_leonardo,
    teensy_30,
    teensy_32,
    teensy_35,
    teensy_36,
    teensy_40,
    teensy_41,
    teensy_lc,
    red_v
)]
pub fn serial_1() -> impl DerefMut<Target = impl Serial> {}

#[board_fn(
    io, teensy_30, teensy_32, teensy_35, teensy_36, teensy_40, teensy_41, teensy_lc, red_v
)]
pub fn serial_2() -> impl DerefMut<Target = impl Serial> {}

#[board_fn(
    io, teensy_30, teensy_32, teensy_35, teensy_36, teensy_40, teensy_41, teensy_lc
)]
pub fn serial_3() -> impl DerefMut<Target = impl Serial> {}

#[board_fn(io, teensy_35, teensy_36, teensy_40, teensy_41)]
pub fn serial_4() -> impl DerefMut<Target = impl Serial> {}

#[board_fn(io, teensy_35, teensy_36, teensy_40, teensy_41)]
pub fn serial_5() -> impl DerefMut<Target = impl Serial> {}

#[board_fn(io, teensy_35, teensy_40, teensy_41)]
pub fn serial_6() -> impl DerefMut<Target = impl Serial> {}

#[board_fn(io, teensy_40, teensy_41)]
pub fn serial_7() -> impl DerefMut<Target = impl Serial> {}

#[board_fn(io, teensy_41)]
pub fn serial_8() -> impl DerefMut<Target = impl Serial> {}
