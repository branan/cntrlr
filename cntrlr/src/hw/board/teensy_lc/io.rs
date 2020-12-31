use crate::{
    hw::{
        board::teensy_common::io::{Serial, SerialError},
        mcu::kinetis::{
            mkl26z64::{Pin, Sim, Uart, UartRx, UartTx},
            Mkl26Z64,
        },
    },
    io,
    sync::{Mutex, MutexGuard},
    task::WakerSet,
};
use core::{ptr::write_volatile, sync::atomic::Ordering};

pub type Serial1Rx = UartRx<Pin<'static, 1, 16>>;
pub type Serial1Tx = UartTx<Pin<'static, 1, 17>>;
pub type Serial2Rx = UartRx<Pin<'static, 2, 3>>;
pub type Serial2Tx = UartTx<Pin<'static, 2, 4>>;
pub type Serial3Rx = UartRx<Pin<'static, 3, 2>>;
pub type Serial3Tx = UartTx<Pin<'static, 3, 3>>;

impl io::Serial for Serial<Mkl26Z64, Serial1Tx, Serial1Rx, 0> {
    type Error = SerialError;

    fn enable(&mut self, baud: usize) -> Result<(), <Self as io::Serial>::Error> {
        let divisor = (super::CPU_FREQ.load(Ordering::Relaxed) as usize * 32) / (baud * 16);
        let mut uart = Sim::get().enable_peripheral::<Uart<(), (), 0>>();
        uart.set_divisor(divisor);

        let tx = super::digital::port_b()
            .pin::<17>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_tx();
        let rx = super::digital::port_b()
            .pin::<16>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_rx();
        let uart = uart.enable_tx(tx).enable_rx(rx);
        self.0 = Some(uart);
        self.1 = Some(&SERIAL_1_WAKERS);
        Ok(())
    }

    fn disable(&mut self) -> Result<(), <Self as io::Serial>::Error> {
        self.0 = None;
        self.1 = None;
        Ok(())
    }
}

impl io::Serial for Serial<Mkl26Z64, Serial2Tx, Serial2Rx, 1> {
    type Error = SerialError;

    fn enable(&mut self, baud: usize) -> Result<(), <Self as io::Serial>::Error> {
        let divisor = (super::CPU_FREQ.load(Ordering::Relaxed) as usize * 32) / (baud * 16);
        let mut uart = Sim::get().enable_peripheral::<Uart<(), (), 1>>();
        uart.set_divisor(divisor);

        let tx = super::digital::port_c()
            .pin::<4>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_tx();
        let rx = super::digital::port_c()
            .pin::<3>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_rx();
        let uart = uart.enable_tx(tx).enable_rx(rx);
        self.0 = Some(uart);
        self.1 = Some(&SERIAL_2_WAKERS);
        Ok(())
    }

    fn disable(&mut self) -> Result<(), <Self as io::Serial>::Error> {
        self.0 = None;
        self.1 = None;
        Ok(())
    }
}

impl io::Serial for Serial<Mkl26Z64, Serial3Tx, Serial3Rx, 2> {
    type Error = SerialError;

    fn enable(&mut self, baud: usize) -> Result<(), <Self as io::Serial>::Error> {
        let divisor = (super::BUS_FREQ.load(Ordering::Relaxed) as usize * 32) / (baud * 16);
        let mut uart = Sim::get().enable_peripheral::<Uart<(), (), 2>>();
        uart.set_divisor(divisor);

        let tx = super::digital::port_d()
            .pin::<3>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_tx();
        let rx = super::digital::port_d()
            .pin::<2>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_rx();
        let uart = uart.enable_tx(tx).enable_rx(rx);
        self.0 = Some(uart);
        self.1 = Some(&SERIAL_3_WAKERS);
        Ok(())
    }

    fn disable(&mut self) -> Result<(), <Self as io::Serial>::Error> {
        self.0 = None;
        self.1 = None;
        Ok(())
    }
}

pub fn serial_1() -> MutexGuard<'static, Serial<Mkl26Z64, Serial1Tx, Serial1Rx, 0>> {
    static SERIAL: Mutex<Serial<Mkl26Z64, Serial1Tx, Serial1Rx, 0>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

pub fn serial_2() -> MutexGuard<'static, Serial<Mkl26Z64, Serial2Tx, Serial2Rx, 1>> {
    static SERIAL: Mutex<Serial<Mkl26Z64, Serial2Tx, Serial2Rx, 1>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

pub fn serial_3() -> MutexGuard<'static, Serial<Mkl26Z64, Serial3Tx, Serial3Rx, 2>> {
    static SERIAL: Mutex<Serial<Mkl26Z64, Serial3Tx, Serial3Rx, 2>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

static SERIAL_1_WAKERS: WakerSet = WakerSet::new();
static SERIAL_2_WAKERS: WakerSet = WakerSet::new();
static SERIAL_3_WAKERS: WakerSet = WakerSet::new();

pub extern "C" fn serial_1_intr() {
    unsafe {
        const UART_TX_INTR: *mut u8 = bitband_address(0x4006_A003, 7);
        const UART_TC_INTR: *mut u8 = bitband_address(0x4006_A003, 6);
        const UART_RX_INTR: *mut u8 = bitband_address(0x4006_A003, 5);
        write_volatile(UART_TX_INTR, 0);
        write_volatile(UART_TC_INTR, 0);
        write_volatile(UART_RX_INTR, 0);
        SERIAL_1_WAKERS.wake();
    }
}

pub extern "C" fn serial_2_intr() {
    unsafe {
        const UART_TX_INTR: *mut u8 = bitband_address(0x4006_B003, 7);
        const UART_TC_INTR: *mut u8 = bitband_address(0x4006_B003, 6);
        const UART_RX_INTR: *mut u8 = bitband_address(0x4006_B003, 5);
        write_volatile(UART_TX_INTR, 0);
        write_volatile(UART_TC_INTR, 0);
        write_volatile(UART_RX_INTR, 0);
        SERIAL_2_WAKERS.wake();
    }
}

pub extern "C" fn serial_3_intr() {
    unsafe {
        const UART_TX_INTR: *mut u8 = bitband_address(0x4006_C003, 7);
        const UART_TC_INTR: *mut u8 = bitband_address(0x4006_C003, 6);
        const UART_RX_INTR: *mut u8 = bitband_address(0x4006_C003, 5);
        write_volatile(UART_TX_INTR, 0);
        write_volatile(UART_TC_INTR, 0);
        write_volatile(UART_RX_INTR, 0);
        SERIAL_3_WAKERS.wake();
    }
}

const fn bitband_address<T>(addr: u32, bit: u32) -> *mut T {
    (0x4200_0000 + (addr - 0x4000_0000) * 32 + bit * 4) as _
}
