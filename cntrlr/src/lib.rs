// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! An Arduino-style library for simple asynchronous embedded programming
//!
//! ```
//! use cntrlr::prelude::*;
//! use core::futures::pending;
//!
//! #[entry]
//! async fn main() -> ! {
//!    serial_1().enable(9600);
//!    writeln!(serial_1(), "Hello, World").await.expect("Failed to message");
//!    pending().await
//! }
//! ```

#![no_std]
#![allow(incomplete_features)]
#![deny(missing_docs)]
#![feature(
    alloc_error_handler,
    asm,
    cfg_target_has_atomic,
    future_poll_fn,
    generic_associated_types,
    naked_functions,
    never_type,
    type_alias_impl_trait
)]
#![cfg_attr(feature = "doc-cfg", feature(doc_cfg))]

extern crate alloc;

pub mod digital;
pub mod hw;
pub mod io;
pub mod sync;
pub mod task;

/// Support Macros
pub mod macros {
    pub use cntrlr_macros::{entry, raw_entry, reset};
}

/// Common functions and traits for using Cntrlr
pub mod prelude {
    pub use crate::digital::{PinMode, Pull};
    pub use crate::io::{Read, ReadExt, Serial, Write, WriteExt};
    use cntrlr_macros::prelude_fn;

    #[prelude_fn(red_v)]
    pub use crate::io::pc_serial;

    #[prelude_fn(red_v, teensy_30, teensy_32, teensy_35, teensy_36, teensy_lc)]
    pub use crate::io::serial_1;

    #[prelude_fn(red_v, teensy_30, teensy_32, teensy_35, teensy_36, teensy_lc)]
    pub use crate::io::serial_2;

    #[prelude_fn(teensy_30, teensy_32, teensy_35, teensy_36, teensy_lc)]
    pub use crate::io::serial_3;

    #[prelude_fn(teensy_35, teensy_36)]
    pub use crate::io::serial_4;

    #[prelude_fn(teensy_35, teensy_36)]
    pub use crate::io::serial_5;

    #[prelude_fn(teensy_35)]
    pub use crate::io::serial_6;

    #[prelude_fn(red_v, teensy_30, teensy_32, teensy_35, teensy_36, teensy_lc)]
    pub use crate::digital::{digital_read, digital_write, pin_mode};

    pub use crate::macros::entry;
}

mod allocator;
mod register;
mod runtime;
