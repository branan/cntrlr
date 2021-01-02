// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Time functionality shared between the various Teensy 3.x boards

use crate::{sync::Value, task::WakerSet};
use core::{
    future::{poll_fn, Future},
    sync::atomic::Ordering,
    task::Poll,
};

/// Retreive the number of milliseconds the device has been running
///
/// This is a wrapping counter. On the Teensy boards, it is 32-bits
///
/// # Note
/// This count may become inaccurate if the sytem clock is modified
/// after startup.
pub fn millis() -> usize {
    MILLIS.load(Ordering::Relaxed)
}

/// Sleep this task for some number of milliseconds
///
/// This task will be slept, and awoken once the number of
/// milliseconds has pased.
pub fn sleep_millis(mut duration: usize) -> impl Future<Output = ()> {
    let mut start = millis();
    poll_fn(move |ctx| {
        let current = millis();
        let elapsed = current.wrapping_sub(start);
        if elapsed >= duration {
            Poll::Ready(())
        } else {
            duration -= elapsed;
            start = current;
            SYSTICK_WAKERS.add(ctx.waker().clone());
            Poll::Pending
        }
    })
}

static MILLIS: Value = Value::new(0);
static SYSTICK_WAKERS: WakerSet = WakerSet::new();

/// Interrupt function for the ARM systick
pub extern "C" fn systick_intr() {
    let millis = MILLIS.load(Ordering::Relaxed);
    let millis = millis.wrapping_add(1);
    MILLIS.store(millis, Ordering::Relaxed);
    SYSTICK_WAKERS.wake();
}
