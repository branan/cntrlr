// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Time functionality specific to the Red-V board

use crate::{sync::Value, task::WakerSet};
use core::{
    future::{poll_fn, Future},
    sync::atomic::Ordering,
    task::Poll,
};

/// Retreive the number of milliseconds the device has been running
///
/// This is a wrapping counter. On the Red-V board, it is 32-bits
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
pub fn sleep_millis(duration: usize) -> impl Future<Output = ()> {
    let current = millis();
    let target = current.wrapping_add(duration);
    poll_fn(move |ctx| {
        if target < current {
            // We wrapped - first wait until we loop past zero
            if millis() >= current {
                TIMER_WAKERS.add(ctx.waker().clone());
                Poll::Pending
            } else {
                if millis() >= target {
                    Poll::Ready(())
                } else {
                    TIMER_WAKERS.add(ctx.waker().clone());
                    Poll::Pending
                }
            }
        } else {
            if millis() >= target {
                Poll::Ready(())
            } else {
                TIMER_WAKERS.add(ctx.waker().clone());
                Poll::Pending
            }
        }
    })
}

static MILLIS: Value = Value::new(0);
static FRACT: Value = Value::new(0);
static TIMER_WAKERS: WakerSet = WakerSet::new();

/// Interrupt functino for the clint timer
pub extern "C" fn timer_intr() {
    #[cfg(mcu = "fe310g002")]
    unsafe {
        // We don't bother with MTIMEHI - if we overflow that far
        // things are pretty messed up anyway.
        const MTIMELO: *mut u32 = 0x0200_BFF8 as _;
        asm!("amoadd.w {}, {}, ({})", out(reg) _, in(reg) -33, in(reg) MTIMELO);
    }
    let mut millis = MILLIS.load(Ordering::Relaxed);
    millis = millis.wrapping_add(1);
    let mut fract = FRACT.load(Ordering::Relaxed);
    fract += 33000 - 32768;
    if fract >= 1000 {
        millis = millis.wrapping_add(1);
        fract -= 1000;
    }
    FRACT.store(fract, Ordering::Relaxed);
    MILLIS.store(millis, Ordering::Relaxed);
    TIMER_WAKERS.wake();
}
