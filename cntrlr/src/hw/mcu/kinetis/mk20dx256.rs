//! The NXP mk20dx256 MCU
//!
//! This is an ARM Cortex-M4 microcontroller produced by NXP. It is
//! used on the [`Teensy 3.1 and 3.2`](`crate::hw::board::teensy_32`)
//! boards.

pub use super::peripheral::mcg::{Clock, Mcg, OscRange};
pub use super::peripheral::osc::Osc;
pub use super::peripheral::port::{UartRx, UartTx};
pub use super::peripheral::sim::{PeripheralClockSource, UsbClockSource};
pub use super::peripheral::wdog::Watchdog;

/// A pin
pub type Pin<'a, const N: usize, const M: usize> =
    super::peripheral::port::Pin<'a, super::Mk20Dx256, N, M>;

/// A Port instance
pub type Port<const N: usize> = super::peripheral::port::Port<super::Mk20Dx256, N>;

/// The System Integration Module
pub type Sim = super::peripheral::sim::Sim<super::Mk20Dx256>;

/// A UART instance
pub type Uart<T, R, const N: usize> = super::peripheral::uart::Uart<super::Mk20Dx256, T, R, N>;
