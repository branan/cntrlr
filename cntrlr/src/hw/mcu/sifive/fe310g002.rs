// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! The SiFive fe310g002 MCU
//!
//! This is a risc-v microcontroller produced by SiFive. It is used on
//! the [`Sparkfun Red V`](`crate::hw::board::red_v`) board.

pub use super::{
    peripheral::gpio::{UartRx, UartTx},
    peripheral::plic::Plic,
    Fe310G002,
};

/// A GPIO instance
pub type Gpio<const N: usize> = super::peripheral::gpio::Gpio<Fe310G002, N>;

/// GPIO Pin
pub type Pin<'a, const N: usize, const P: usize> =
    super::peripheral::gpio::Pin<'a, Fe310G002, N, P>;

/// The PRCI
pub type Prci = super::peripheral::prci::Prci<Fe310G002>;

/// An SPI instance
pub type Spi<T, R, const N: usize> = super::peripheral::spi::Spi<Fe310G002, T, R, N>;

/// A UART instance
pub type Uart<T, R, const N: usize> = super::peripheral::uart::Uart<Fe310G002, T, R, N>;
