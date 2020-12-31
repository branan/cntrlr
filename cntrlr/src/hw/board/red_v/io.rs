use crate::{
    hw::mcu::sifive::{
        fe310g002::{Pin, Uart, UartRx, UartTx},
        peripheral::uart,
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

#[derive(Debug)]
#[non_exhaustive]
pub enum SerialError {
    NotEnabled,
    PinInUse,
    InvalidBaud,
}

pub struct Serial<T, R, const N: usize>(Option<Uart<T, R, N>>, Option<&'static WakerSet>);

impl<T, R, const N: usize> Read for Serial<T, R, N>
where
    T: 'static,
    R: uart::UartRx<Fe310G002, N> + 'static,
{
    type Error = SerialError;
    type Future<'a> = impl Future<Output = Result<usize, Self::Error>>;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> <Self as Read>::Future<'a> {
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

impl<T, R, const N: usize> Write for Serial<T, R, N>
where
    T: uart::UartTx<Fe310G002, N> + 'static,
    R: 'static,
{
    type Error = SerialError;
    type Future<'a> = impl Future<Output = Result<usize, Self::Error>>;
    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> <Self as Write>::Future<'a> {
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

    fn flush<'a>(&'a mut self) -> <Self as Write>::FlushFuture<'a> {
        // Not possible to flush on this hardware - just say we're
        // flushed.
        ready(Ok(()))
    }
}

pub type Serial1Rx = UartRx<Pin<'static, 0, 16>>;
pub type Serial1Tx = UartTx<Pin<'static, 0, 17>>;
pub type Serial2Rx = UartRx<Pin<'static, 0, 23>>;
pub type Serial2Tx = UartTx<Pin<'static, 0, 18>>;

impl io::Serial for Serial<Serial1Tx, Serial1Rx, 0> {
    type Error = SerialError;
    fn enable(&mut self, baud: usize) -> Result<(), <Self as io::Serial>::Error> {
        let divisor = super::CPU_FREQ.load(Ordering::Relaxed) as usize / baud;
        if baud < 16 {
            return Err(SerialError::InvalidBaud);
        }

        let mut uart = Uart::<(), (), 0>::get();
        uart.set_divisor(divisor);
        uart.set_watermarks(7, 0);

        let tx = super::digital::gpio()
            .pin::<17>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_tx();
        let rx = super::digital::gpio()
            .pin::<16>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_rx();
        self.0 = Some(uart.enable_tx(tx).enable_rx(rx));
        self.1 = Some(&SERIAL_1_WAKERS);
        Ok(())
    }

    fn disable(&mut self) -> Result<(), <Self as io::Serial>::Error> {
        self.0 = None;
        self.1 = None;
        Ok(())
    }
}

impl io::Serial for Serial<Serial2Tx, Serial2Rx, 1> {
    type Error = SerialError;
    fn enable(&mut self, baud: usize) -> Result<(), <Self as io::Serial>::Error> {
        let divisor = super::CPU_FREQ.load(Ordering::Relaxed) as usize / baud;
        if baud < 16 {
            return Err(SerialError::InvalidBaud);
        }

        let mut uart = Uart::<(), (), 1>::get();
        uart.set_divisor(divisor);
        uart.set_watermarks(7, 0);

        let tx = super::digital::gpio()
            .pin::<18>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_tx();
        let rx = super::digital::gpio()
            .pin::<23>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_rx();
        self.0 = Some(uart.enable_tx(tx).enable_rx(rx));
        self.1 = Some(&SERIAL_2_WAKERS);
        Ok(())
    }

    fn disable(&mut self) -> Result<(), <Self as io::Serial>::Error> {
        self.0 = None;
        self.1 = None;
        Ok(())
    }
}

pub fn pc_serial() -> MutexGuard<'static, Serial<Serial1Tx, Serial1Rx, 0>> {
    serial_1()
}

pub fn serial_1() -> MutexGuard<'static, Serial<Serial1Tx, Serial1Rx, 0>> {
    static SERIAL: Mutex<Serial<Serial1Tx, Serial1Rx, 0>> = Mutex::new(Serial(None, None));
    SERIAL.lock()
}

pub fn serial_2() -> MutexGuard<'static, Serial<Serial2Tx, Serial2Rx, 1>> {
    static SERIAL: Mutex<Serial<Serial2Tx, Serial2Rx, 1>> = Mutex::new(Serial(None, None));
    SERIAL.lock()
}

static SERIAL_1_WAKERS: WakerSet = WakerSet::new();
static SERIAL_2_WAKERS: WakerSet = WakerSet::new();

pub extern "C" fn serial_1_intr() {
    #[cfg(board = "red_v")]
    unsafe {
        const UART_IE: *mut u32 = 0x1001_3010 as _;
        asm!("amoand.w {}, {}, ({})", out(reg) _, in(reg) 0xFFFF_FFFCu32, in(reg) UART_IE);
        SERIAL_1_WAKERS.wake();
    }
}

pub extern "C" fn serial_2_intr() {
    #[cfg(board = "red_v")]
    unsafe {
        const UART_IE: *mut u32 = 0x1002_3010 as _;
        asm!("amoand.w {}, {}, ({})", out(reg) _, in(reg) 0xFFFF_FFFCu32, in(reg) UART_IE);
        SERIAL_2_WAKERS.wake();
    }
}
