// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! IO functionality specific to the Teensy LC board

use crate::{
    hw::{
        board::teensy_common::io::{Serial, SerialError},
        mcu::kinetis::{
            mkl26z64::{Pin, UartRx, UartTx},
            Mkl26Z64,
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

impl io::Serial for Serial<Mkl26Z64, Serial1Tx, Serial1Rx, 0> {
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
            super::PLL_FREQ.load(Ordering::Relaxed) / 2,
            &SERIAL_1_WAKERS,
        )?;

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
            super::BUS_FREQ.load(Ordering::Relaxed),
            &SERIAL_2_WAKERS,
        )?;
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
        Ok(())
    }

    fn disable(&mut self) -> Result<(), <Self as io::Serial>::Error> {
        self.0 = None;
        self.1 = None;
        Ok(())
    }
}

/// The first hardware serial port
pub fn serial_1() -> MutexGuard<'static, Serial<Mkl26Z64, Serial1Tx, Serial1Rx, 0>> {
    static SERIAL: Mutex<Serial<Mkl26Z64, Serial1Tx, Serial1Rx, 0>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

/// The second hardware serial port
pub fn serial_2() -> MutexGuard<'static, Serial<Mkl26Z64, Serial2Tx, Serial2Rx, 1>> {
    static SERIAL: Mutex<Serial<Mkl26Z64, Serial2Tx, Serial2Rx, 1>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

/// The third hardware serial port
pub fn serial_3() -> MutexGuard<'static, Serial<Mkl26Z64, Serial3Tx, Serial3Rx, 2>> {
    static SERIAL: Mutex<Serial<Mkl26Z64, Serial3Tx, Serial3Rx, 2>> = Mutex::new(Serial::new());
    SERIAL.lock()
}

static SERIAL_1_WAKERS: WakerSet = WakerSet::new();
static SERIAL_2_WAKERS: WakerSet = WakerSet::new();
static SERIAL_3_WAKERS: WakerSet = WakerSet::new();

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

const fn bitband_address<T>(addr: u32, bit: u32) -> *mut T {
    (0x4200_0000 + (addr - 0x4000_0000) * 32 + bit * 4) as _
}
