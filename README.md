# Cntrlr - Simple, asynchronous embedded

[![Crates.io](https://img.shields.io/crates/v/cntrlr)](https://crates.io/crates/cntrlr)
[![Crates.io](https://img.shields.io/crates/l/cntrlr)](https://github.com/branan/cntrlr/blob/main/COPYING)
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

## Project Goals

The primary goal of Cntrlr is to provide a prototyping environment for
hobbyists. This generally means that functionality should be simple
and automatic, but provide escape hatches to directly access hardware
for more complex uses.

A secondary goal is to explore the API space afforded by Rust's type
system. This also ties into the first goal, since high-level APIs must
be designed in a way that allows for low-level access to the hardware
by user applications.

It is explicitly a non-goal to move away from nightly Rust. Cntrlr's
goals mean it needs the flexibility to bring in new nightly features
when it makes sense for the API.

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

### The nighty requirement

Cntrlr uses a number of rust nightly features. Some of these are
required for the functionality the crate provides, while others are
used to create a more ergonomic API.

## Supported Boards

* PJRC Teensy 3.x family, based on NXP/Freescale Kinetis microcontrollers
    - Teensy 3.0
    - Teensy 3.1
    - Teensy 3.2
    - Teensy 3.5
    - Teensy 3.6
    - Possibly Teensy LC. The bringup code looks good compared to the
      Teensyduino core, but I don't own an LC to test with at the
      moment. It may not work.
* Sparkfun Red V, based on the SiFive Freedom E310 microcontroller
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
