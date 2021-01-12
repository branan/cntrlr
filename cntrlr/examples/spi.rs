// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

// Writes a simple string to the first SPI port.
//
// Typically the SPI output is pin 11, and the SPI clock is pin
// 13. Pin 10 is available as a hardware chip select on most boards.

#![no_std]
#![no_main]
#![allow(dead_code)]

const CHIP_SELECT: usize = 10;
const MESSAGE: &str = "Hello, World";
const ERROR: usize = 0;

use cntrlr::prelude::*;

#[cfg(any(board = "teensy_30", board = "teensy_32", board = "teensy_35"))]
#[entry]
async fn main() -> ! {
    pin_mode(CHIP_SELECT, PinMode::Output);
    spi_1()
        .enable()
        //       .enable_with_options(&[SpiOption::HardwareCs(CHIP_SELECT)])
        .expect("Error enabling SPI");
    loop {
        spi_1()
            .transfer(2_000_000, CHIP_SELECT, 12 * 8)
            .await
            .expect("Error starting SPI transfer")
            .write_all(MESSAGE.as_bytes())
            .await
            .expect("Error writing SPI message");

        let mut msg_in: [u8; 12] = [0xFF; 12];
        spi_1()
            .transfer(2_000_000, CHIP_SELECT, 12 * 8)
            .await
            .expect("Error starting SPI transfer")
            .read_exact(&mut msg_in)
            .await
            .expect("Error reading SPI message");

        if !msg_in.iter().all(|byte| *byte == 0) {
            pin_mode(ERROR, PinMode::Output);
            digital_write(ERROR, true);
        }
    }
}

#[cfg(not(any(board = "teensy_30", board = "teensy_32", board = "teensy_35")))]
#[entry]
async fn main() -> ! {
    core::future::pending().await
}
