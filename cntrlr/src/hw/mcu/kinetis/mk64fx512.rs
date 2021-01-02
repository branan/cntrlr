// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Th NXP mk64fx512 MCU
//!
//! This is an ARM Cortex-M4 microcontroller produced by NXP. It is
//! used on the [`Teensy 3.5`](`crate::hw::board::teensy_35`) board.

pub use super::peripheral::mcg::{Clock, Mcg, OscRange};
pub use super::peripheral::osc::Osc;
pub use super::peripheral::port::{UartRx, UartTx};
pub use super::peripheral::sim::{PeripheralClockSource, UsbClockSource};
pub use super::peripheral::systick::SysTick;
pub use super::peripheral::wdog::Watchdog;

/// a Pin
pub type Pin<'a, const N: usize, const M: usize> =
    super::peripheral::port::Pin<'a, super::Mk64Fx512, N, M>;

/// A Port instance
pub type Port<const N: usize> = super::peripheral::port::Port<super::Mk64Fx512, N>;

/// The System Integration Module
pub type Sim = super::peripheral::sim::Sim<super::Mk64Fx512>;

/// A UART instance.
pub type Uart<T, R, const N: usize> = super::peripheral::uart::Uart<super::Mk64Fx512, T, R, N>;
