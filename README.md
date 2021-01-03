# Cntrlr - Simple, asynchronous embedded

[![Crates.io](https://img.shields.io/crates/v/cntrlr)](https://crates.io/crates/cntrlr)
[![Crates.io](https://img.shields.io/crates/l/cntrlr)](https://github.com/branan/cntrlr/blob/master/COPYING)
[![Docs.rs](https://docs.rs/cntrlr/badge.svg)](https://docs.rs/cntrlr)

Cntrlr is an all-in-one embedded platform for writing simple
asynchronous applications on top of common hobbyist development
boards.

```rust
#![no_std]
#![no_main]

use cntrlr::prelude::*;
use core::future::pending;

#[entry]
async fn main() -> ! {
    serial_1().enable(9600).unwrap();
    writeln!(serial_1(), \"Hello, World\").await.unwrap();

    // Hang forever once we've sent our message
    pending().await
}
```

## Getting Started

Cntrlr currently requires a recent nightly Rust compiler. The best way
to install nightly Rust is via `rustup`

To simplify building and flashing your application, `cargo-cntrlr` is
recommended.

```shell
cargo +nightly install cargo-cntrlr
cargo +nightly cntrlr new myapp
cd myapp
cargo +nightly cntrlr flash --board=<board>
```

For the list of supported boards, run `cargo cntrlr flash --board=help`

## Supported Boards

* PJRC Teensy 3.x family, based on NXP/Freescale Kinetis microcontrollers
    - Teensy 3.0
    - Teensy 3.1
    - Teensy 3.2
    - Teensy 3.5
    - Teensy 3.6
    - Teensy LC
* Sparkfun Red V, based on the SiFive Freedom microcontroller
    - The SiFive HiFive1 Rev B (which the Red V is based on) should
      also work, but is untested.
    - The original HiFive1 will mostly likely not work without being
      added as a dedicated board.

## Supported Functionality

* UART-based serial ports
* Simple digital GPIOs

## Future Work

Cntrlr will continue to be updated to add additional board support, as
well as expanding functionality for existing boards.

### Additional Board Support

Currently, support is planned for the following boards

* Teensy 4.0/4.1

Note that support for AVR-based boards is currently blocked on a
number of Rust compiler issues.

### Additional Feature Support

* SPI and I2C serial
* Analog read & write
* PWM
* SD Cards
* USB
