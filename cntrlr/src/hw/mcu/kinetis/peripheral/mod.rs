// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Shared peripherals for Kinetis family microcontrollers.

pub mod mcg;
pub mod osc;
pub mod port;
pub mod sim;
pub mod smc;
pub mod spi;
pub mod systick;
pub mod uart;
pub mod wdog;

/// A Kinetis peripheral
///
/// Trait for peripherals which are always enabled.
pub trait Peripheral: Sized {
    /// Get the instance of this perihperal
    ///
    /// Returns `None` if the peripheral is already in use.
    fn get() -> Option<Self>;
}
