#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use cntrlr::prelude::*;
use core::future::pending;

#[entry]
async fn main() -> ! {
    let mut name = String::new();
    serial_1().enable(9600).unwrap();
    write!(serial_1(), "Enter your name: ").await.unwrap();
    serial_1().read_line(&mut name).await.unwrap();
    writeln!(serial_1(), "Hello, {}", name).await.unwrap();
    pending().await
}
