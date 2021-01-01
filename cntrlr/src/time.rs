// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Time functionality for Cntrlr boards

use cntrlr_macros::board_fn;
use core::future::Future;

/// Retrieve the number of milliseconds the device has been running.
///
/// This is a wrapping counter. Its size is dependent on the board
/// used.
#[board_fn(time, red_v, teensy_30, teensy_32, teensy_35, teensy_36, teensy_lc)]
pub fn millis() -> usize {}

/// Sleep this task for some number of milliseconds
///
/// This task will be slept, and awoken once the nymber of
/// milliseconds has passed.
#[board_fn(time, red_v, teensy_30, teensy_32, teensy_35, teensy_36, teensy_lc)]
pub fn sleep_millis(duration: usize) -> impl Future<Output = ()> {}
