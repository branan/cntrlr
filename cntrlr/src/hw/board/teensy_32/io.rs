// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! IO functionality specific to the Teensy 3.2 board

use crate::{
    hw::{
        board::teensy_common::io::{Serial, SerialError, Spi, SpiBoard, SpiError},
        mcu::kinetis::{
            mk20dx256::{Cs, Miso, Mosi, Pin, Sck, UartRx, UartTx},
            Mk20Dx256,
        },
    },
    io::{self, SpiOption},
    sync::{Mutex, MutexGuard},
    task::WakerSet,
};
use core::{ptr::write_volatile, sync::atomic::Ordering};

/// The pin used to recieve for serial 1
pub type Serial1Rx = UartRx<Pin<'static, 1, 16>>;

/// The pin used to transmit for serial 1
pub type Serial1Tx = UartTx<Pin<'static, 1, 17>>;

/// The pin used to recieve for serial 2
pub type Serial2Rx = UartRx<Pin<'static, 2, 3>>;

/// The pin used to transmit for serial 2
pub type Serial2Tx = UartTx<Pin<'static, 2, 4>>;

/// The pin used to recieve for serial 3
pub type Serial3Rx = UartRx<Pin<'static, 3, 2>>;

/// The pin used to transmit for serial 3
pub type Serial3Tx = UartTx<Pin<'static, 3, 3>>;

/// The pin used as MISO for the SPI
pub type SpiMiso = Miso<Pin<'static, 2, 7>>;

/// The pin used as MOSI for the SPI
pub type SpiMosi = Mosi<Pin<'static, 2, 6>>;

/// The pin used as SCK for the SPI
pub type SpiSck = Sck<Pin<'static, 2, 5>>;

/// The pin used as CS0 for the SPI
pub type SpiCs0 = Cs<Pin<'static, 2, 4>>;

/// The pin used as CS1 for the SPI
pub type SpiCs1 = Cs<Pin<'static, 2, 3>>;

/// The pin used as CS2 for the SPI
pub type SpiCs2 = Cs<Pin<'static, 3, 5>>;

/// The pin usd as CS3 for the SPI
pub type SpiCs3 = Cs<Pin<'static, 3, 6>>;

/// The pin used as CS4 for the SPI
pub type SpiCs4 = Cs<Pin<'static, 2, 0>>;

/// The Chip Selects for the SPI
pub type SpiCs = (
    Option<SpiCs0>,
    Option<SpiCs1>,
    Option<SpiCs2>,
    Option<SpiCs3>,
    Option<SpiCs4>,
);

impl io::Serial for Serial<Mk20Dx256, Serial1Tx, Serial1Rx, 0> {
    type Error = SerialError;

    fn enable_with_options(
        &mut self,
        baud: usize,
        options: &[io::SerialOption],
    ) -> Result<(), <Self as io::Serial>::Error> {
        let tx = super::digital::port_b()
            .ok_or(SerialError::PortInUse)?
            .pin::<17>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_tx();
        let rx = super::digital::port_b()
            .ok_or(SerialError::PortInUse)?
            .pin::<16>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_rx();

        self.do_enable(
            baud,
            options,
            tx,
            rx,
            super::CPU_FREQ.load(Ordering::Relaxed),
            &SERIAL_1_WAKERS,
        )?;

        if let Some(uart) = self.0.as_mut() {
            uart.enable_tx_fifo(8, 7);
            uart.enable_rx_fifo(8, 1);
        }
        Ok(())
    }

    fn disable(&mut self) -> Result<(), <Self as io::Serial>::Error> {
        self.0 = None;
        self.1 = None;
        Ok(())
    }
}

impl io::Serial for Serial<Mk20Dx256, Serial2Tx, Serial2Rx, 1> {
    type Error = SerialError;

    fn enable_with_options(
        &mut self,
        baud: usize,
        options: &[io::SerialOption],
    ) -> Result<(), <Self as io::Serial>::Error> {
        let tx = super::digital::port_c()
            .ok_or(SerialError::PortInUse)?
            .pin::<4>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_tx();
        let rx = super::digital::port_c()
            .ok_or(SerialError::PortInUse)?
            .pin::<3>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_rx();

        self.do_enable(
            baud,
            options,
            tx,
            rx,
            super::CPU_FREQ.load(Ordering::Relaxed),
            &SERIAL_2_WAKERS,
        )?;

        if let Some(uart) = self.0.as_mut() {
            uart.enable_tx_fifo(8, 7);
            uart.enable_rx_fifo(8, 1);
        }
        Ok(())
    }

    fn disable(&mut self) -> Result<(), <Self as io::Serial>::Error> {
        self.0 = None;
        self.1 = None;
        Ok(())
    }
}

impl io::Serial for Serial<Mk20Dx256, Serial3Tx, Serial3Rx, 2> {
    type Error = SerialError;

    fn enable_with_options(
        &mut self,
        baud: usize,
        options: &[io::SerialOption],
    ) -> Result<(), <Self as io::Serial>::Error> {
        let tx = super::digital::port_d()
            .ok_or(SerialError::PortInUse)?
            .pin::<3>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_tx();
        let rx = super::digital::port_d()
            .ok_or(SerialError::PortInUse)?
            .pin::<2>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_rx();

        self.do_enable(
            baud,
            options,
            tx,
            rx,
            super::BUS_FREQ.load(Ordering::Relaxed),
            &SERIAL_3_WAKERS,
        )?;

        if let Some(uart) = self.0.as_mut() {
            uart.enable_tx_fifo(1, 0);
            uart.enable_rx_fifo(1, 1);
        }
        Ok(())
    }

    fn disable(&mut self) -> Result<(), <Self as io::Serial>::Error> {
        self.0 = None;
        self.1 = None;
        Ok(())
    }
}

impl SpiBoard<SpiMiso, SpiMosi, SpiSck, SpiCs>
    for Spi<Mk20Dx256, SpiMiso, SpiMosi, SpiSck, SpiCs, 0>
{
    fn miso() -> Result<SpiMiso, SpiError> {
        super::digital::port_c()
            .ok_or(SpiError::PortInUse)
            .and_then(|port| port.pin::<7>().ok_or(SpiError::PinInUse))
            .map(Pin::into_spi_miso)
    }

    fn mosi() -> Result<SpiMosi, SpiError> {
        super::digital::port_c()
            .ok_or(SpiError::PortInUse)
            .and_then(|port| port.pin::<6>().ok_or(SpiError::PinInUse))
            .map(Pin::into_spi_mosi)
    }

    fn sck() -> Result<SpiSck, SpiError> {
        super::digital::port_c()
            .ok_or(SpiError::PortInUse)
            .and_then(|port| port.pin::<5>().ok_or(SpiError::PinInUse))
            .map(Pin::into_spi_sck)
    }

    fn cs(options: &[SpiOption]) -> Result<SpiCs, SpiError> {
        let mut cs = (None, None, None, None, None);

        for option in options {
            match option {
                SpiOption::HardwareCs(10) => {
                    let pin = super::digital::port_c()
                        .ok_or(SpiError::PortInUse)?
                        .pin::<4>()
                        .ok_or(SpiError::PinInUse)?
                        .into_spi_cs();
                    cs.0 = Some(pin);
                }
                SpiOption::HardwareCs(9) => {
                    let pin = super::digital::port_c()
                        .ok_or(SpiError::PortInUse)?
                        .pin::<3>()
                        .ok_or(SpiError::PinInUse)?
                        .into_spi_cs();
                    cs.1 = Some(pin);
                }
                SpiOption::HardwareCs(20) => {
                    let pin = super::digital::port_d()
                        .ok_or(SpiError::PortInUse)?
                        .pin::<5>()
                        .ok_or(SpiError::PinInUse)?
                        .into_spi_cs();
                    cs.2 = Some(pin);
                }
                SpiOption::HardwareCs(21) => {
                    let pin = super::digital::port_d()
                        .ok_or(SpiError::PortInUse)?
                        .pin::<6>()
                        .ok_or(SpiError::PinInUse)?
                        .into_spi_cs();
                    cs.3 = Some(pin);
                }
                SpiOption::HardwareCs(15) => {
                    let pin = super::digital::port_c()
                        .ok_or(SpiError::PortInUse)?
                        .pin::<0>()
                        .ok_or(SpiError::PinInUse)?
                        .into_spi_cs();
                    cs.4 = Some(pin);
                }
                SpiOption::HardwareCs(_) => return Err(SpiError::InvalidOption),
            }
        }
        Ok(cs)
    }

    fn clock_source() -> usize {
        super::BUS_FREQ.load(Ordering::Relaxed)
    }

    fn wakers() -> &'static WakerSet {
        &SPI_WAKERS
    }

    fn hardware_cs(cs: usize) -> Option<usize> {
        match cs {
            9 => Some(1),
            10 => Some(0),
            15 => Some(4),
            20 => Some(2),
            21 => Some(3),
            _ => None,
        }
    }
}

/// The first hardware serial port
pub fn serial_1() -> MutexGuard<'static, Serial<Mk20Dx256, Serial1Tx, Serial1Rx, 0>> {
    static SERIAL: Mutex<Serial<Mk20Dx256, Serial1Tx, Serial1Rx, 0>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

/// The second hardware serial port
pub fn serial_2() -> MutexGuard<'static, Serial<Mk20Dx256, Serial2Tx, Serial2Rx, 1>> {
    static SERIAL: Mutex<Serial<Mk20Dx256, Serial2Tx, Serial2Rx, 1>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

/// The third hardware serial port
pub fn serial_3() -> MutexGuard<'static, Serial<Mk20Dx256, Serial3Tx, Serial3Rx, 2>> {
    static SERIAL: Mutex<Serial<Mk20Dx256, Serial3Tx, Serial3Rx, 2>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

/// The first hardware spi port
///
/// On the Teensy 3.2, the SPI uses the following pins:
/// * 11: Data Out
/// * 12: Data In
/// * 13: Clock
/// * Hardware chip selects on pins 9, 10, 15, 20, and 21
pub fn spi_1() -> MutexGuard<'static, Spi<Mk20Dx256, SpiMiso, SpiMosi, SpiSck, SpiCs, 0>> {
    static SPI: Mutex<Spi<Mk20Dx256, SpiMiso, SpiMosi, SpiSck, SpiCs, 0>> = Mutex::new(Spi::new());
    SPI.lock()
}

static SERIAL_1_WAKERS: WakerSet = WakerSet::new();
static SERIAL_2_WAKERS: WakerSet = WakerSet::new();
static SERIAL_3_WAKERS: WakerSet = WakerSet::new();
static SPI_WAKERS: WakerSet = WakerSet::new();

/// The interrupt function for serial 1
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

/// The interrupt function for serial 2
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

/// The interrupt function for serial 3
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

/// The interrupt function for spi 1
pub extern "C" fn spi_1_intr() {
    unsafe {
        const SPI_TX_INTR: *mut u32 = bitband_address(0x4002_C030, 25);
        const SPI_RX_INTR: *mut u32 = bitband_address(0x4002_C030, 17);
        const SPI_TC_INTR: *mut u32 = bitband_address(0x4002_C030, 31);
        write_volatile(SPI_TX_INTR, 0);
        write_volatile(SPI_RX_INTR, 0);
        write_volatile(SPI_TC_INTR, 0);
        SPI_WAKERS.wake();
    }
}

const fn bitband_address<T>(addr: u32, bit: u32) -> *mut T {
    (0x4200_0000 + (addr - 0x4000_0000) * 32 + bit * 4) as _
}
