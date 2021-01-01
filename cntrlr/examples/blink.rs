// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

#![no_std]
#![no_main]

use cntrlr::prelude::*;

#[entry]
async fn main() -> ! {
    pin_mode(2, PinMode::Output);
    loop {
        digital_write(2, true);
        sleep_millis(500).await;
        digital_write(2, false);
        sleep_millis(500).await;
    }
}
