#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use cntrlr::prelude::*;
use core::future::pending;

#[entry]
async fn main() -> ! {
    let mut name = String::new();
    serial_1().enable(9600).expect("Could not initalize serial");
    write!(serial_1(), "Enter your name: ")
        .await
        .expect("Could not write prompt to serial");
    serial_1()
        .read_line(&mut name)
        .await
        .expect("Could not read name from serial");
    writeln!(serial_1(), "Hello, {}", name)
        .await
        .expect("Could not write response to serial");
    pending().await
}
