// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! IO functinoality specific to the Teensy 3.6 board

use crate::{
    hw::{
        board::teensy_common::io::{Serial, SerialError},
        mcu::kinetis::{
            mk66fx1m0::{Pin, UartRx, UartTx},
            Mk66Fx1M0,
        },
    },
    io,
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

static SERIAL_1_WAKERS: WakerSet = WakerSet::new();
static SERIAL_2_WAKERS: WakerSet = WakerSet::new();
static SERIAL_3_WAKERS: WakerSet = WakerSet::new();
static SERIAL_4_WAKERS: WakerSet = WakerSet::new();
static SERIAL_5_WAKERS: WakerSet = WakerSet::new();

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

const fn bitband_address<T>(addr: u32, bit: u32) -> *mut T {
    (0x4200_0000 + (addr - 0x4000_0000) * 32 + bit * 4) as _
}
