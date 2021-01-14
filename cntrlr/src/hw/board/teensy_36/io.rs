// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! IO functinoality specific to the Teensy 3.6 board

use crate::{
    hw::{
        board::teensy_common::io::{Serial, SerialError, Spi, SpiBoard, SpiError},
        mcu::kinetis::{
            mk66fx1m0::{Cs, Pin, Sck, Sdi, Sdo, UartRx, UartTx},
            Mk66Fx1M0,
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

/// The pin used to recieve for serial 4
pub type Serial4Rx = UartRx<Pin<'static, 1, 10>>;

/// The pin used to transmit for serial 4
pub type Serial4Tx = UartTx<Pin<'static, 1, 11>>;

/// The pin used to recieve for serial 5
pub type Serial5Rx = UartRx<Pin<'static, 4, 25>>;

/// The pin used to transmit for serial 5
pub type Serial5Tx = UartTx<Pin<'static, 4, 24>>;

/// The pin used as SDI for SPI 1
pub type Spi1Sdi = Sdi<Pin<'static, 2, 7>>;

/// The pin used as SDO for SPI 1
pub type Spi1Sdo = Sdo<Pin<'static, 2, 6>>;

/// The pin used as SCK for SPI 1
pub type Spi1Sck = Sck<Pin<'static, 2, 5>>;

/// The pin used as CS0 for SPI 1
pub type Spi1Cs0 = Cs<Pin<'static, 2, 4>>;

/// The pin used as CS1 for SPI 1
pub type Spi1Cs1 = Cs<Pin<'static, 2, 3>>;

/// The pin used as CS2 for SPI 1
pub type Spi1Cs2 = Cs<Pin<'static, 3, 5>>;

/// The pin usd as CS3 for SPI 1
pub type Spi1Cs3 = Cs<Pin<'static, 3, 6>>;

/// The pin used as CS4 for SPI 1
pub type Spi1Cs4 = Cs<Pin<'static, 2, 0>>;

/// The Chip Selects for SPI 1
pub type Spi1Cs = (
    Option<Spi1Cs0>,
    Option<Spi1Cs1>,
    Option<Spi1Cs2>,
    Option<Spi1Cs3>,
    Option<Spi1Cs4>,
);

/// The pin used as SDI for SPI 2
pub type Spi2Sdi = Sdi<Pin<'static, 1, 17>>;

/// The pin used as SDO for SPI 2
pub type Spi2Sdo = Sdo<Pin<'static, 1, 16>>;

/// The pin used as SCK for SPI 2
pub type Spi2Sck = Sck<Pin<'static, 1, 11>>;

/// The pin used as CS for SPI 2
pub type Spi2Cs = Option<Cs<Pin<'static, 1, 10>>>;

/// The pin used as SDI for SPI 3a
pub type Spi3Sdi = Sdi<Pin<'static, 1, 23>>;

/// The pin used as SDO for SPI 3
pub type Spi3Sdo = Sdo<Pin<'static, 1, 22>>;

/// The pin used as SCK for SPI 3
pub type Spi3Sck = Sck<Pin<'static, 1, 21>>;

/// The pin used as CS0 for SPI 3
pub type Spi3Cs0 = Cs<Pin<'static, 1, 20>>;

/// The pin used as CS for SPI 3
pub type Spi3Cs1 = Cs<Pin<'static, 3, 15>>;

/// The Chip Selects for SPI 3
pub type Spi3Cs = (Option<Spi3Cs0>, Option<Spi3Cs1>);

impl io::Serial for Serial<Mk66Fx1M0, Serial1Tx, Serial1Rx, 0> {
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

impl io::Serial for Serial<Mk66Fx1M0, Serial2Tx, Serial2Rx, 1> {
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

impl io::Serial for Serial<Mk66Fx1M0, Serial3Tx, Serial3Rx, 2> {
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

impl io::Serial for Serial<Mk66Fx1M0, Serial4Tx, Serial4Rx, 3> {
    type Error = SerialError;

    fn enable_with_options(
        &mut self,
        baud: usize,
        options: &[io::SerialOption],
    ) -> Result<(), <Self as io::Serial>::Error> {
        let tx = super::digital::port_b()
            .ok_or(SerialError::PortInUse)?
            .pin::<11>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_tx();
        let rx = super::digital::port_b()
            .ok_or(SerialError::PortInUse)?
            .pin::<10>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_rx();

        self.do_enable(
            baud,
            options,
            tx,
            rx,
            super::BUS_FREQ.load(Ordering::Relaxed),
            &SERIAL_4_WAKERS,
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

impl io::Serial for Serial<Mk66Fx1M0, Serial5Tx, Serial5Rx, 4> {
    type Error = SerialError;

    fn enable_with_options(
        &mut self,
        baud: usize,
        options: &[io::SerialOption],
    ) -> Result<(), <Self as io::Serial>::Error> {
        let tx = super::digital::port_e()
            .ok_or(SerialError::PortInUse)?
            .pin::<24>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_tx();
        let rx = super::digital::port_e()
            .ok_or(SerialError::PortInUse)?
            .pin::<25>()
            .ok_or(SerialError::PinInUse)?
            .into_uart_rx();

        self.do_enable(
            baud,
            options,
            tx,
            rx,
            super::BUS_FREQ.load(Ordering::Relaxed),
            &SERIAL_5_WAKERS,
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

impl SpiBoard<Spi1Sdi, Spi1Sdo, Spi1Sck, Spi1Cs>
    for Spi<Mk66Fx1M0, Spi1Sdi, Spi1Sdo, Spi1Sck, Spi1Cs, 0>
{
    fn sdi() -> Result<Spi1Sdi, SpiError> {
        super::digital::port_c()
            .ok_or(SpiError::PortInUse)
            .and_then(|port| port.pin::<7>().ok_or(SpiError::PinInUse))
            .map(|pin| pin.into_spi_sdi())
    }

    fn sdo() -> Result<Spi1Sdo, SpiError> {
        super::digital::port_c()
            .ok_or(SpiError::PortInUse)
            .and_then(|port| port.pin::<6>().ok_or(SpiError::PinInUse))
            .map(|pin| pin.into_spi_sdo())
    }

    fn sck() -> Result<Spi1Sck, SpiError> {
        super::digital::port_c()
            .ok_or(SpiError::PortInUse)
            .and_then(|port| port.pin::<5>().ok_or(SpiError::PinInUse))
            .map(|pin| pin.into_spi_sck())
    }

    fn cs(options: &[SpiOption]) -> Result<Spi1Cs, SpiError> {
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
        &SPI_1_WAKERS
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

impl SpiBoard<Spi2Sdi, Spi2Sdo, Spi2Sck, Spi2Cs>
    for Spi<Mk66Fx1M0, Spi2Sdi, Spi2Sdo, Spi2Sck, Spi2Cs, 1>
{
    fn sdi() -> Result<Spi2Sdi, SpiError> {
        super::digital::port_b()
            .ok_or(SpiError::PortInUse)
            .and_then(|port| port.pin::<17>().ok_or(SpiError::PinInUse))
            .map(|pin| pin.into_spi_sdi())
    }

    fn sdo() -> Result<Spi2Sdo, SpiError> {
        super::digital::port_b()
            .ok_or(SpiError::PortInUse)
            .and_then(|port| port.pin::<16>().ok_or(SpiError::PinInUse))
            .map(|pin| pin.into_spi_sdo())
    }

    fn sck() -> Result<Spi2Sck, SpiError> {
        super::digital::port_b()
            .ok_or(SpiError::PortInUse)
            .and_then(|port| port.pin::<11>().ok_or(SpiError::PinInUse))
            .map(|pin| pin.into_spi_sck())
    }

    fn cs(options: &[SpiOption]) -> Result<Spi2Cs, SpiError> {
        let mut cs = None;

        for option in options {
            match option {
                SpiOption::HardwareCs(31) => {
                    let pin = super::digital::port_b()
                        .ok_or(SpiError::PortInUse)?
                        .pin::<10>()
                        .ok_or(SpiError::PinInUse)?
                        .into_spi_cs();
                    cs = Some(pin);
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
        &SPI_2_WAKERS
    }

    fn hardware_cs(cs: usize) -> Option<usize> {
        match cs {
            31 => Some(0),
            _ => None,
        }
    }
}

impl SpiBoard<Spi3Sdi, Spi3Sdo, Spi3Sck, Spi3Cs>
    for Spi<Mk66Fx1M0, Spi3Sdi, Spi3Sdo, Spi3Sck, Spi3Cs, 2>
{
    fn sdi() -> Result<Spi3Sdi, SpiError> {
        super::digital::port_b()
            .ok_or(SpiError::PortInUse)
            .and_then(|port| port.pin::<23>().ok_or(SpiError::PinInUse))
            .map(|pin| pin.into_spi_sdi())
    }

    fn sdo() -> Result<Spi3Sdo, SpiError> {
        super::digital::port_b()
            .ok_or(SpiError::PortInUse)
            .and_then(|port| port.pin::<22>().ok_or(SpiError::PinInUse))
            .map(|pin| pin.into_spi_sdo())
    }

    fn sck() -> Result<Spi3Sck, SpiError> {
        super::digital::port_b()
            .ok_or(SpiError::PortInUse)
            .and_then(|port| port.pin::<21>().ok_or(SpiError::PinInUse))
            .map(|pin| pin.into_spi_sck())
    }

    fn cs(options: &[SpiOption]) -> Result<Spi3Cs, SpiError> {
        let mut cs = (None, None);

        for option in options {
            match option {
                SpiOption::HardwareCs(43) => {
                    let pin = super::digital::port_b()
                        .ok_or(SpiError::PortInUse)?
                        .pin::<20>()
                        .ok_or(SpiError::PinInUse)?
                        .into_spi_cs();
                    cs.0 = Some(pin);
                }
                SpiOption::HardwareCs(54) => {
                    let pin = super::digital::port_d()
                        .ok_or(SpiError::PortInUse)?
                        .pin::<15>()
                        .ok_or(SpiError::PinInUse)?
                        .into_spi_cs();
                    cs.1 = Some(pin);
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
        &SPI_3_WAKERS
    }

    fn hardware_cs(cs: usize) -> Option<usize> {
        match cs {
            43 => Some(0),
            54 => Some(1),
            _ => None,
        }
    }
}

/// The first hardware serial port
pub fn serial_1() -> MutexGuard<'static, Serial<Mk66Fx1M0, Serial1Tx, Serial1Rx, 0>> {
    static SERIAL: Mutex<Serial<Mk66Fx1M0, Serial1Tx, Serial1Rx, 0>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

/// The second hardware serial port
pub fn serial_2() -> MutexGuard<'static, Serial<Mk66Fx1M0, Serial2Tx, Serial2Rx, 1>> {
    static SERIAL: Mutex<Serial<Mk66Fx1M0, Serial2Tx, Serial2Rx, 1>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

/// The third hardware serial port
pub fn serial_3() -> MutexGuard<'static, Serial<Mk66Fx1M0, Serial3Tx, Serial3Rx, 2>> {
    static SERIAL: Mutex<Serial<Mk66Fx1M0, Serial3Tx, Serial3Rx, 2>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

/// The fourth hardware serial port
pub fn serial_4() -> MutexGuard<'static, Serial<Mk66Fx1M0, Serial4Tx, Serial4Rx, 3>> {
    static SERIAL: Mutex<Serial<Mk66Fx1M0, Serial4Tx, Serial4Rx, 3>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

/// The fifth hardware serial port
pub fn serial_5() -> MutexGuard<'static, Serial<Mk66Fx1M0, Serial5Tx, Serial5Rx, 4>> {
    static SERIAL: Mutex<Serial<Mk66Fx1M0, Serial5Tx, Serial5Rx, 4>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

/// The first hardware spi port
///
/// On the Teensy 3.6, the SPI uses the following pins:
/// * 11: Data Out
/// * 12: Data In
/// * 13: Clock
/// * Hardware chip selects on pins 9, 10, 15, 20, and 21
pub fn spi_1() -> MutexGuard<'static, Spi<Mk66Fx1M0, Spi1Sdi, Spi1Sdo, Spi1Sck, Spi1Cs, 0>> {
    static SPI: Mutex<Spi<Mk66Fx1M0, Spi1Sdi, Spi1Sdo, Spi1Sck, Spi1Cs, 0>> =
        Mutex::new(Spi::new());
    SPI.lock()
}

/// The second hardware spi port
///
/// On the Teensy 3.6, the SPI uses the following pins:
/// * 0: Data Out
/// * 1: Data In
/// * 32: Clock
/// * A single hardware chip select on pin 31
pub fn spi_2() -> MutexGuard<'static, Spi<Mk66Fx1M0, Spi2Sdi, Spi2Sdo, Spi2Sck, Spi2Cs, 1>> {
    static SPI: Mutex<Spi<Mk66Fx1M0, Spi2Sdi, Spi2Sdo, Spi2Sck, Spi2Cs, 1>> =
        Mutex::new(Spi::new());
    SPI.lock()
}

/// The third hardware spi port
///
/// On the Teensy 3.6, the SPI uses the following pins:
/// * 44: Data Out
/// * 45: Data In
/// * 46: Clock
/// * Hardware chip selects on pins 43 and 54
pub fn spi_3() -> MutexGuard<'static, Spi<Mk66Fx1M0, Spi3Sdi, Spi3Sdo, Spi3Sck, Spi3Cs, 2>> {
    static SPI: Mutex<Spi<Mk66Fx1M0, Spi3Sdi, Spi3Sdo, Spi3Sck, Spi3Cs, 2>> =
        Mutex::new(Spi::new());
    SPI.lock()
}

static SERIAL_1_WAKERS: WakerSet = WakerSet::new();
static SERIAL_2_WAKERS: WakerSet = WakerSet::new();
static SERIAL_3_WAKERS: WakerSet = WakerSet::new();
static SERIAL_4_WAKERS: WakerSet = WakerSet::new();
static SERIAL_5_WAKERS: WakerSet = WakerSet::new();
static SPI_1_WAKERS: WakerSet = WakerSet::new();
static SPI_2_WAKERS: WakerSet = WakerSet::new();
static SPI_3_WAKERS: WakerSet = WakerSet::new();

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

/// The interrupt function for serial 4
pub extern "C" fn serial_4_intr() {
    unsafe {
        const UART_TX_INTR: *mut u8 = bitband_address(0x4006_D003, 7);
        const UART_TC_INTR: *mut u8 = bitband_address(0x4006_D003, 6);
        const UART_RX_INTR: *mut u8 = bitband_address(0x4006_D003, 5);
        write_volatile(UART_TX_INTR, 0);
        write_volatile(UART_TC_INTR, 0);
        write_volatile(UART_RX_INTR, 0);
        SERIAL_1_WAKERS.wake();
    }
}

/// The interrupt function for serial 5
pub extern "C" fn serial_5_intr() {
    unsafe {
        const UART_TX_INTR: *mut u8 = bitband_address(0x400E_A003, 7);
        const UART_TC_INTR: *mut u8 = bitband_address(0x400E_A003, 6);
        const UART_RX_INTR: *mut u8 = bitband_address(0x400E_A003, 5);
        write_volatile(UART_TX_INTR, 0);
        write_volatile(UART_TC_INTR, 0);
        write_volatile(UART_RX_INTR, 0);
        SERIAL_2_WAKERS.wake();
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
        SPI_1_WAKERS.wake();
    }
}

/// The interrupt function for spi 2
pub extern "C" fn spi_2_intr() {
    unsafe {
        const SPI_TX_INTR: *mut u32 = bitband_address(0x4002_D030, 25);
        const SPI_RX_INTR: *mut u32 = bitband_address(0x4002_D030, 17);
        const SPI_TC_INTR: *mut u32 = bitband_address(0x4002_D030, 31);
        write_volatile(SPI_TX_INTR, 0);
        write_volatile(SPI_RX_INTR, 0);
        write_volatile(SPI_TC_INTR, 0);
        SPI_2_WAKERS.wake();
    }
}

/// The interrupt function for spi 3
pub extern "C" fn spi_3_intr() {
    unsafe {
        const SPI_TX_INTR: *mut u32 = bitband_address(0x400A_C030, 25);
        const SPI_RX_INTR: *mut u32 = bitband_address(0x400A_C030, 17);
        const SPI_TC_INTR: *mut u32 = bitband_address(0x400A_C030, 31);
        write_volatile(SPI_TX_INTR, 0);
        write_volatile(SPI_RX_INTR, 0);
        write_volatile(SPI_TC_INTR, 0);
        SPI_3_WAKERS.wake();
    }
}

const fn bitband_address<T>(addr: u32, bit: u32) -> *mut T {
    (0x4200_0000 + (addr - 0x4000_0000) * 32 + bit * 4) as _
}
