#![no_std]
#![allow(incomplete_features)]
#![feature(
    alloc_error_handler,
    asm,
    cfg_target_has_atomic,
    future_poll_fn,
    generic_associated_types,
    min_const_generics,
    naked_functions,
    never_type,
    type_alias_impl_trait
)]
#![cfg_attr(feature = "doc-cfg", feature(doc_cfg))]

extern crate alloc;

pub mod hw;
pub mod io;
pub mod sync;
pub mod task;

pub mod macros {
    pub use cntrlr_macros::{entry, reset};
}

pub mod prelude {
    pub use crate::io::{Read, ReadExt, Serial, Write, WriteExt};
    use cntrlr_macros::prelude_fn;

    #[prelude_fn(arduino_uno, red_v)]
    pub use crate::io::pc_serial;

    #[prelude_fn(
        arduino_uno,
        arduino_leonardo,
        teensy_30,
        teensy_32,
        teensy_35,
        teensy_36,
        teensy_40,
        teensy_41,
        teensy_lc,
        red_v
    )]
    pub use crate::io::serial_1;

    #[prelude_fn(
        teensy_30, teensy_32, teensy_35, teensy_36, teensy_40, teensy_41, teensy_lc, red_v
    )]
    pub use crate::io::serial_2;

    #[prelude_fn(
        teensy_30, teensy_32, teensy_35, teensy_36, teensy_40, teensy_41, teensy_lc
    )]
    pub use crate::io::serial_3;

    #[prelude_fn(teensy_35, teensy_36, teensy_40, teensy_41)]
    pub use crate::io::serial_4;

    #[prelude_fn(teensy_35, teensy_36, teensy_40, teensy_41)]
    pub use crate::io::serial_5;

    #[prelude_fn(teensy_35, teensy_40, teensy_41)]
    pub use crate::io::serial_6;

    #[prelude_fn(teensy_40, teensy_41)]
    pub use crate::io::serial_7;

    #[prelude_fn(teensy_41)]
    pub use crate::io::serial_8;

    pub use crate::macros::entry;
}

mod allocator;
mod register;
mod runtime;
