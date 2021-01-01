// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Digital pin support specific to the Sparkfun Red V

use crate::{hw::mcu::sifive::fe310g002::Gpio, sync::Once};

/// The GPIO
///
/// The global instance of the GPIO, used to share ownership among
/// different board modules.
pub fn gpio() -> &'static Gpio<0> {
    static PORT: Once<Gpio<0>> = Once::new();
    PORT.get_or_init(|| Gpio::get())
}
