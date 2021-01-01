// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! The NXP mk66fx1m0 MCU
//!
//! This is an ARM Cortex-M4F microcontroller produced by NXP. It is
//! used on the [`Teensy 3.6`](`crate::hw::board::teensy_36`) board.

pub use super::peripheral::mcg::{Clock, Mcg, OscRange}; // TODO: MK66 has a different MCG
pub use super::peripheral::osc::Osc;
pub use super::peripheral::port::{UartRx, UartTx};
pub use super::peripheral::systick::SysTick;
pub use super::peripheral::wdog::Watchdog;

/// A Pin
pub type Pin<'a, const N: usize, const M: usize> =
    super::peripheral::port::Pin<'a, super::Mk66Fx1M0, N, M>;

/// A Port instance
pub type Port<const N: usize> = super::peripheral::port::Port<super::Mk66Fx1M0, N>;

/// The System Integration Module
pub type Sim = super::peripheral::sim::Sim<super::Mk66Fx1M0>;

/// A UART instance
pub type Uart<T, R, const N: usize> = super::peripheral::uart::Uart<super::Mk66Fx1M0, T, R, N>;
