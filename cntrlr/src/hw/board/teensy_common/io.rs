use crate::{
    hw::mcu::kinetis::peripheral::uart::{Uart, UartRx, UartTx},
    io,
    task::WakerSet,
};
use core::{
    future::{poll_fn, Future},
    task::Poll,
};

#[derive(Debug)]
#[non_exhaustive]
pub enum SerialError {
    NotEnabled,
    PinInUse,
}

pub struct Serial<M, T, R, const N: usize>(
    pub(crate) Option<Uart<M, T, R, N>>,
    pub(crate) Option<&'static WakerSet>,
);

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
            if buf.len() == 0 {
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
            if buf.len() == 0 {
                return Poll::Ready(Ok(0));
            }

            let mut count = 0;
            let mut buf = buf;
            let uart = self.0.as_mut().ok_or(SerialError::NotEnabled)?;
            while let Some(_) = uart.write_data(buf[0]) {
                count += 1;
                buf = &buf[1..];
                if buf.len() == 0 {
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
    pub const fn new() -> Self {
        Self(None, None)
    }
}