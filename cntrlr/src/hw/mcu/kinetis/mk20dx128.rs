// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! The NXP mk20dx128 MCU
//!
//! This is an ARM Cortex-M4 microcontroller produced by NXP. It is
//! used on the [`Teensy 3.0`](`crate::hw::board::teensy_30`) board.

pub use super::peripheral::mcg::OscRange;
pub use super::peripheral::osc::Osc;
pub use super::peripheral::port::{Cs, Sck, Sdi, Sdo, UartRx, UartTx};
pub use super::peripheral::sim::{PeripheralClockSource, UsbClockSource};
pub use super::peripheral::wdog::Watchdog;

/// The handle to the SysTick
pub type SysTick = super::peripheral::systick::SysTick<super::Mk20Dx128>;

/// The handle to the MCG
pub type Mcg = super::peripheral::mcg::Mcg<super::Mk20Dx128>;

/// The current mode of the system clock
pub type Clock<'a> = super::peripheral::mcg::Clock<'a, super::Mk20Dx128>;

/// A pin
pub type Pin<'a, const N: usize, const M: usize> =
    super::peripheral::port::Pin<'a, super::Mk20Dx128, N, M>;

/// A Port instance
pub type Port<const N: usize> = super::peripheral::port::Port<super::Mk20Dx128, N>;

/// The System Integration Module
pub type Sim = super::peripheral::sim::Sim<super::Mk20Dx128>;

/// A UART instance
pub type Uart<T, R, const N: usize> = super::peripheral::uart::Uart<super::Mk20Dx128, T, R, N>;
