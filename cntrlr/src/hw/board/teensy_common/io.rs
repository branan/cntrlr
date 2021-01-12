// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! IO functionality shared between the various Teensy 3.x boards

use crate::{
    hw::mcu::kinetis::peripheral::{
        sim::{GatedPeripheral, Sim},
        spi::{self, Cs, Fifo, Sck, Sdi, Sdo},
        uart::{Uart, UartRx, UartTx},
        Peripheral,
    },
    io::{self, SerialOption, SpiOption},
    task::WakerSet,
};
use bit_field::BitField;
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

/// An error from a SPI instance
#[derive(Debug)]
#[non_exhaustive]
pub enum SpiError {
    /// The SPI cannot be read or written because it is disabled
    NotEnabled,

    /// The SPI cannot be enabled because its TX or RX pin is in use
    PinInUse,

    /// The SPI cannot be enabled because its PORT is in use
    PortInUse,

    /// The SPI cannot be enabled because the SPI is in use
    SpiInUse,

    /// The SPI cannot be enabled because the SIM is in use
    SimInUse,

    /// The SPI cannot be enabled because the selected baud rate is invalid
    InvalidBaud,

    /// The SPI cannot be enabled because a selected option is invalid
    InvalidOption,

    /// The SPI packet cannot be written because not enough bytes were provided
    InsufficientData,
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

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::Future<'a>
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

impl<M, T, R, const N: usize> io::Write for Serial<M, T, R, N>
where
    M: 'static,
    T: UartTx<M, N> + 'static,
    R: 'static,
{
    type Error = SerialError;
    type Future<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a;
    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::Future<'a>
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

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a>
    where
        Self: 'a,
    {
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

/// An SPI
pub struct Spi<M, I, O, C, CS, const N: usize> {
    pub(crate) spi: Option<spi::Spi<M, I, O, C, CS, N>>,
    pub(crate) wakers: Option<&'static WakerSet>,
    transfer_count: u16,
    trailing_frames: usize,
}

/// An active SPI transfer
pub struct SpiTransfer<'a, M, I, O, C, CS, const N: usize> {
    spi: &'a mut Spi<M, I, O, C, CS, N>,
    packet_len: usize,
    tail_len: usize,
    tail_bit_len: usize,
    cs: usize,
}

impl<M, I, O, C, CS, const N: usize> SpiTransfer<'_, M, I, O, C, CS, N>
where
    I: Sdi<M, N>,
    O: Sdo<M, N>,
    C: Sck<M, N>,
    CS: Cs<M, N>,
    Spi<M, I, O, C, CS, N>: SpiBoard<I, O, C, CS>,
    spi::Spi<M, I, O, C, CS, N>: Fifo,
{
    /// Transfer one or more packets to and from the SPI
    pub async fn transfer(
        &mut self,
        mut buf_in: &[u8],
        mut buf_out: &mut [u8],
    ) -> Result<usize, SpiError> {
        if buf_in.len() < self.packet_len && buf_out.len() < self.packet_len {
            return Err(SpiError::InsufficientData);
        }
        let spi = self.spi.spi.as_mut().ok_or(SpiError::NotEnabled)?;
        let wakers = self.spi.wakers.as_mut().ok_or(SpiError::NotEnabled)?;
        let transfer_count = &mut self.spi.transfer_count;

        let hardware_cs = Spi::<M, I, O, C, CS, N>::hardware_cs(self.cs).and_then(|cs| {
            if spi.cs_allowed(cs) {
                Some(cs)
            } else {
                None
            }
        });

        let has_tail = self.tail_len != 0;
        let main_fifo_entries = (self.packet_len - self.tail_len) / 2;
        let required_fifo_entries = main_fifo_entries + if has_tail { 1 } else { 0 };

        let mut trailing_frames = self.spi.trailing_frames;
        let mut frames_read = 0;
        let mut frames_written = 0;
        let mut written = 0;
        loop {
            // Send as many packets as we can without awaiting, but at least 1
            if hardware_cs.is_none() {
                poll_fn(|ctx| {
                    if *transfer_count == spi.transfer_count() {
                        spi.clear_transfer_complete();
                        Poll::Ready(())
                    } else {
                        wakers.add(ctx.waker().clone());
                        spi.enable_tc_intr();
                        Poll::Pending
                    }
                })
                .await;
                crate::digital::digital_write(self.cs, false);
            }

            while frames_written < main_fifo_entries {
                while let Some(word) = spi.read() {
                    frames_read += 1;
                    if frames_read > trailing_frames {
                        if buf_out.len() == 1 {
                            buf_out[0] = word.get_bits(8..16) as u8;
                            buf_out = &mut buf_out[1..];
                        } else if !buf_out.is_empty() {
                            buf_out[0] = word.get_bits(8..16) as u8;
                            buf_out[1] = word.get_bits(0..8) as u8;
                            buf_out = &mut buf_out[2..];
                        }
                    }
                }
                let word = if buf_in.is_empty() {
                    0
                } else if buf_in.len() == 1 {
                    let word = (buf_in[0] as u16) << 8;
                    buf_in = &buf_in[1..];
                    word
                } else {
                    let word = (buf_in[0] as u16) << 8 | (buf_in[1] as u16);
                    buf_in = &buf_in[2..];
                    word
                };
                let hold_cs = (frames_written + 1) < required_fifo_entries;
                while !spi.write(0, hardware_cs, hold_cs, word) {
                    poll_fn(|ctx| {
                        if spi.tx_fifo_available() > 0 {
                            Poll::Ready(())
                        } else {
                            wakers.add(ctx.waker().clone());
                            spi.enable_tx_intr();
                            Poll::Pending
                        }
                    })
                    .await;
                }
                spi.clear_transfer_complete();
                spi.clear_tx_ready();
                frames_written += 1;
            }

            if has_tail {
                let word = if self.tail_bit_len > 8 {
                    let (hi, lo) = if buf_in.is_empty() {
                        (0, 0)
                    } else if buf_in.len() == 1 {
                        let hi = buf_in[0];
                        buf_in = &buf_in[1..];
                        (hi, 0)
                    } else {
                        let hi = buf_in[0];
                        let lo = buf_in[1];
                        buf_in = &buf_in[2..];
                        (hi, lo)
                    };
                    (hi as u16) << (self.tail_bit_len - 8)
                        | (lo.get_bits(0..(self.tail_bit_len - 8)) as u16)
                } else {
                    let byte = if buf_in.is_empty() {
                        0
                    } else {
                        let byte = buf_in[0];
                        buf_in = &buf_in[1..];
                        byte
                    };
                    byte.get_bits(0..self.tail_bit_len) as u16
                };
                while !spi.write(1, hardware_cs, false, word) {
                    poll_fn(|ctx| {
                        if spi.tx_fifo_available() > 0 {
                            Poll::Ready(())
                        } else {
                            wakers.add(ctx.waker().clone());
                            spi.enable_tx_intr();
                            Poll::Pending
                        }
                    })
                    .await;
                }
                spi.clear_transfer_complete();
                spi.clear_tx_ready();
                frames_written += 1;
            }

            while (frames_read < trailing_frames
                || (frames_read - trailing_frames) < main_fifo_entries)
                && !buf_out.is_empty()
            {
                if let Some(word) = spi.read() {
                    frames_read += 1;
                    if frames_read > trailing_frames {
                        if buf_out.len() == 1 {
                            buf_out[0] = word.get_bits(8..16) as u8;
                            buf_out = &mut buf_out[1..];
                        } else if !buf_out.is_empty() {
                            buf_out[0] = word.get_bits(8..16) as u8;
                            buf_out[1] = word.get_bits(0..8) as u8;
                            buf_out = &mut buf_out[2..];
                        }
                    }
                    spi.clear_rx_ready();
                } else {
                    poll_fn(|ctx| {
                        if spi.rx_fifo_available() > 0 {
                            Poll::Ready(())
                        } else {
                            wakers.add(ctx.waker().clone());
                            spi.enable_rx_intr();
                            Poll::Pending
                        }
                    })
                    .await;
                }
            }

            if has_tail && !buf_out.is_empty() {
                loop {
                    if let Some(word) = spi.read() {
                        if self.tail_bit_len > 8 {
                            if buf_out.len() > 1 {
                                buf_out[0] =
                                    word.get_bits((self.tail_bit_len - 8)..self.tail_bit_len) as u8;
                                buf_out[1] = word.get_bits(0..(self.tail_bit_len - 8)) as u8;
                                buf_out = &mut buf_out[2..];
                            } else {
                                buf_out[0] =
                                    word.get_bits((self.tail_bit_len - 8)..self.tail_bit_len) as u8;
                                buf_out = &mut buf_out[1..];
                            }
                        } else {
                            buf_out[0] = word.get_bits(0..self.tail_bit_len) as u8;
                            buf_out = &mut buf_out[1..];
                        }
                        spi.clear_rx_ready();
                        break;
                    } else {
                        poll_fn(|ctx| {
                            if spi.rx_fifo_available() > 0 {
                                Poll::Ready(())
                            } else {
                                wakers.add(ctx.waker().clone());
                                spi.enable_rx_intr();
                                Poll::Pending
                            }
                        })
                        .await;
                    }
                }
            }

            written += self.packet_len;
            *transfer_count = transfer_count.wrapping_add(frames_written as u16);
            frames_read = 0;
            frames_written = 0;
            trailing_frames = trailing_frames + frames_written - frames_read;

            // Ensue we don't desync due to FIFO overrun
            while trailing_frames > spi.fifo_depth() {
                if spi.read().is_some() {
                    trailing_frames -= 1;
                    spi.clear_rx_ready();
                } else {
                    poll_fn(|ctx| {
                        if spi.rx_fifo_available() > 0 {
                            Poll::Ready(())
                        } else {
                            wakers.add(ctx.waker().clone());
                            spi.enable_rx_intr();
                            Poll::Pending
                        }
                    })
                    .await;
                }
            }

            if hardware_cs.is_none() {
                poll_fn(|ctx| {
                    if *transfer_count == spi.transfer_count() {
                        spi.clear_transfer_complete();
                        Poll::Ready(())
                    } else {
                        wakers.add(ctx.waker().clone());
                        spi.enable_tc_intr();
                        Poll::Pending
                    }
                })
                .await;
                crate::digital::digital_write(self.cs, true);
            }

            // If we can't send another packet without awaiting, we're done
            if spi.tx_fifo_available() < required_fifo_entries
                || (buf_out.len() < self.packet_len && buf_in.len() < self.packet_len)
            {
                // Update the number of frames to discard at the next transaction
                self.spi.trailing_frames = trailing_frames;
                return Ok(written);
            }
        }
    }
}

impl<M, I, O, C, CS, const N: usize> io::Read for SpiTransfer<'_, M, I, O, C, CS, N>
where
    I: Sdi<M, N>,
    O: Sdo<M, N>,
    C: Sck<M, N>,
    CS: Cs<M, N>,
    Spi<M, I, O, C, CS, N>: SpiBoard<I, O, C, CS>,
    spi::Spi<M, I, O, C, CS, N>: Fifo,
{
    type Error = SpiError;
    #[rustfmt::skip]
    type Future<'a> where Self: 'a = impl Future<Output = Result<usize, Self::Error>> + 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::Future<'a>
    where
        Self: 'a,
    {
        self.transfer(&[], buf)
    }
}

impl<M, I, O, C, CS, const N: usize> io::Write for SpiTransfer<'_, M, I, O, C, CS, N>
where
    I: Sdi<M, N>,
    O: Sdo<M, N>,
    C: Sck<M, N>,
    CS: Cs<M, N>,
    Spi<M, I, O, C, CS, N>: SpiBoard<I, O, C, CS>,
    spi::Spi<M, I, O, C, CS, N>: Fifo,
{
    type Error = SpiError;
    #[rustfmt::skip]
    type Future<'a> where Self: 'a = impl Future<Output = Result<usize, Self::Error>> + 'a;

    #[rustfmt::skip]
    type FlushFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>> + 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::Future<'a>
    where
        Self: 'a,
    {
        self.transfer(buf, &mut [])
    }

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a>
    where
        Self: 'a,
    {
        poll_fn(move |ctx| {
            let spi = self.spi.spi.as_mut().ok_or(SpiError::NotEnabled)?;
            if self.spi.transfer_count == spi.transfer_count() {
                spi.clear_transfer_complete();
                Poll::Ready(Ok(()))
            } else {
                if let Some(wakers) = self.spi.wakers {
                    wakers.add(ctx.waker().clone());
                }
                spi.enable_tc_intr();
                Poll::Pending
            }
        })
    }
}

impl<M, I, O, C, CS, const N: usize> io::SpiTransfer for SpiTransfer<'_, M, I, O, C, CS, N>
where
    I: Sdi<M, N>,
    O: Sdo<M, N>,
    C: Sck<M, N>,
    CS: Cs<M, N>,
    Spi<M, I, O, C, CS, N>: SpiBoard<I, O, C, CS>,
    spi::Spi<M, I, O, C, CS, N>: Fifo,
{
    type Error = SpiError;
    #[rustfmt::skip]
    type TransferFuture<'a> where Self: 'a = impl Future<Output = Result<usize, Self::Error>> + 'a;

    fn transfer<'a>(
        &'a mut self,
        buf_in: &'a [u8],
        buf_out: &'a mut [u8],
    ) -> Self::TransferFuture<'a>
    where
        Self: 'a,
    {
        self.transfer(buf_in, buf_out)
    }
}

impl<M, I, O, C, CS, const N: usize> Spi<M, I, O, C, CS, N> {
    /// Create a new instance of an SPI, in a disabled state.
    pub const fn new() -> Self {
        Self {
            spi: None,
            wakers: None,
            transfer_count: 0,
            trailing_frames: 0,
        }
    }
}

impl<M, I, O, C, CS, const N: usize> io::Spi for Spi<M, I, O, C, CS, N>
where
    I: Sdi<M, N>,
    O: Sdo<M, N>,
    C: Sck<M, N>,
    CS: Cs<M, N>,
    spi::Spi<M, (), (), (), (), N>: GatedPeripheral<M> + Fifo,
    spi::Spi<M, I, O, C, CS, N>: Fifo,
    Sim<M>: Peripheral,
    Spi<M, I, O, C, CS, N>: SpiBoard<I, O, C, CS>,
{
    type Error = SpiError;
    type Transfer<'a>
    where
        Self: 'a,
    = SpiTransfer<'a, M, I, O, C, CS, N>;
    #[rustfmt::skip]
    type TransferFuture<'a> where Self: 'a = impl Future<Output = Result<Self::Transfer<'a>, Self::Error>>;
    #[rustfmt::skip]
    type FlushFuture<'a> where Self: 'a = impl Future<Output = Result<(), Self::Error>>;

    fn enable_with_options(&mut self, options: &[SpiOption]) -> Result<(), Self::Error> {
        let sdi = Self::sdi()?;
        let sdo = Self::sdo()?;
        let sck = Self::sck()?;
        let cs = Self::cs(options)?;

        let spi = Sim::<M>::get()
            .ok_or(SpiError::SimInUse)?
            .enable_peripheral::<spi::Spi<M, (), (), (), (), N>>()
            .ok_or(SpiError::SpiInUse)?;

        self.transfer_count = spi.transfer_count();
        self.spi = Some(spi.enable(sdi, sdo, sck, cs));
        self.wakers = Some(Self::wakers());
        Ok(())
    }

    fn disable(&mut self) -> Result<(), SpiError> {
        self.spi = None;
        self.wakers = None;
        self.trailing_frames = 0;
        Ok(())
    }

    fn transfer<'a>(&'a mut self, baud: usize, cs: usize, packet: usize) -> Self::TransferFuture<'a>
    where
        Self: 'a,
    {
        let clock_source = Self::clock_source();
        let mut selected_pbr = 0;
        let mut selected_br = 0;
        let mut error = usize::MAX;

        // TODO: Should we check possibilities for DBR as well? Do the
        // asymmetric clocks that generates matter? Is an option best?
        for pbr in &[2, 3, 5, 7] {
            for br in &[
                2, 4, 6, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768,
            ] {
                let calculated_baud = clock_source / (pbr * br);
                if calculated_baud > baud {
                    continue;
                }
                let calculated_error = baud - calculated_baud;
                if calculated_error < error {
                    selected_pbr = *pbr;
                    selected_br = *br;
                    error = calculated_error;
                }
                // Higher divisors will just be larger errors at this point - check the next PBR
                break;
            }
        }

        let pbr = selected_pbr;
        let br = selected_br;

        let packet_len = if packet % 8 != 0 {
            (packet / 8) + 1
        } else {
            packet / 8
        };

        let tail_bit_len = packet % 16;

        let (sz_0, sz_1) = if tail_bit_len != 0 {
            (16, Some(tail_bit_len))
        } else {
            (16, None)
        };

        let tail_len = if let Some(sz_1) = sz_1 {
            if sz_1 > 8 {
                2
            } else {
                1
            }
        } else {
            0
        };

        // TODO: There are many other timing/delay parameters that are
        // tunable. How should they be exposed/calculated here?

        async move {
            let spi = self.spi.as_mut().ok_or(SpiError::NotEnabled)?;
            let transfer_count = &mut self.transfer_count;
            let wakers = &self.wakers;
            let (current_pbr_0, current_br_0, current_sz_0) = spi.read_ctar0();
            let need_flush = current_pbr_0 != pbr
                || current_br_0 != br
                || current_sz_0 != sz_0
                || sz_1
                    .map(|sz_1| {
                        let (current_pbr_1, current_br_1, current_sz_1) = spi.read_ctar1();
                        current_pbr_1 != pbr || current_br_1 != br || current_sz_1 != sz_1
                    })
                    .unwrap_or(false);
            if need_flush {
                poll_fn(|ctx| {
                    if *transfer_count == spi.transfer_count() {
                        spi.clear_transfer_complete();
                        Poll::Ready(())
                    } else {
                        if let Some(wakers) = wakers {
                            wakers.add(ctx.waker().clone());
                        }
                        spi.enable_tc_intr();
                        Poll::Pending
                    }
                })
                .await;
            }
            spi.set_ctar0(pbr, br, sz_0);
            if let Some(sz_1) = sz_1 {
                spi.set_ctar1(pbr, br, sz_1);
            }
            Ok(SpiTransfer {
                spi: self,
                packet_len,
                tail_len,
                tail_bit_len,
                cs,
            })
        }
    }

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a>
    where
        Self: 'a,
    {
        poll_fn(move |ctx| {
            let spi = self.spi.as_mut().ok_or(SpiError::NotEnabled)?;
            if self.transfer_count == spi.transfer_count() {
                spi.clear_transfer_complete();
                Poll::Ready(Ok(()))
            } else {
                if let Some(wakers) = self.wakers {
                    wakers.add(ctx.waker().clone());
                }
                spi.enable_tc_intr();
                Poll::Pending
            }
        })
    }
}

#[allow(missing_docs)]
pub trait SpiBoard<I, O, C, CS> {
    fn sdi() -> Result<I, SpiError>;
    fn sdo() -> Result<O, SpiError>;
    fn sck() -> Result<C, SpiError>;
    fn cs(options: &[SpiOption]) -> Result<CS, SpiError>;
    fn clock_source() -> usize;
    fn wakers() -> &'static WakerSet;
    fn hardware_cs(cs: usize) -> Option<usize>;
}
