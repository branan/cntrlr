// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Peripherals found on SiFive Freedom Everywhere microcontrollers.

pub mod gpio;
pub mod plic;
pub mod prci;
pub mod spi;
pub mod uart;

/// A Sifive peripheral
pub trait Peripheral: Sized {
    /// Get the instance of this perihperal
    ///
    /// Returns `None` if the peripheral is already in use.
    fn get() -> Option<Self>;
}
