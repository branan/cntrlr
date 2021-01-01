//! The NXP mkl26z64 microcontroller
//!
//! This is an ARM Cortex-M0 microcontroller produced by NXP. It is
//! used on the [`Teensy LC`](`crate::hw::board::teensy_lc`) board.

pub use super::peripheral::mcg::{Clock, Mcg, OscRange};
pub use super::peripheral::osc::Osc;
pub use super::peripheral::port::{UartRx, UartTx};
pub use super::peripheral::wdog::Watchdog;

/// A Pin
pub type Pin<'a, const N: usize, const M: usize> =
    super::peripheral::port::Pin<'a, super::Mkl26Z64, N, M>;

/// A Port instance
pub type Port<const N: usize> = super::peripheral::port::Port<super::Mkl26Z64, N>;

/// The System Integration Module
pub type Sim = super::peripheral::sim::Sim<super::Mkl26Z64>;

/// A UART instance
pub type Uart<T, R, const N: usize> = super::peripheral::uart::Uart<super::Mkl26Z64, T, R, N>;
