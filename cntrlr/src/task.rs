// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Async task support for Cntrlr

use crate::sync::without_interrupts;
use alloc::{boxed::Box, vec::Vec};
use core::{
    cell::UnsafeCell,
    default::Default,
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, RawWaker, RawWakerVTable, Waker},
};

struct Task {
    wake: AtomicBool,
    future: Pin<Box<dyn Future<Output = !>>>,
}

/// Task Executor
#[derive(Default)]
pub struct Executor {
    tasks: Vec<Task>,
}

impl Executor {
    /// Create a new Executor
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a new task to this Executor
    pub fn add_task<F>(&mut self, task: F)
    where
        F: Future<Output = !> + 'static,
    {
        let task = Task {
            wake: AtomicBool::new(true),
            future: Box::pin(task),
        };
        self.tasks.push(task)
    }

    /// Hand control off to the Executor
    ///
    /// # Safety
    /// It must be safe for this function to enable interrupts.
    pub unsafe fn run(&mut self) -> ! {
        loop {
            // The execution loop is broken into two parts:
            //
            // 1. With interrupts disabled, check all tasks and sleep
            // if none are ready. On all supported architectures, this
            // sleep implicitly enables interrupts.
            //
            // 2. When we wake from sleep, run any tasks which have
            // been made ready.
            //
            // This two-phase check uses a few more cycles, but it
            // is a simple way to ensure we don't miss a wake.
            without_interrupts(|| {
                if self
                    .tasks
                    .iter()
                    .all(|task| !task.wake.load(Ordering::Acquire))
                {
                    // This is the same instruction with basically
                    // the same semantics on both ARM and RISC-V.
                    asm!("wfi")
                }
            });
            for task in &mut self.tasks {
                if task.wake.load(Ordering::Acquire) {
                    task.wake.store(false, Ordering::Relaxed);
                    let waker = waker_new(&task.wake);
                    let mut context = Context::from_waker(&waker);
                    let _ = task.future.as_mut().poll(&mut context);
                }
            }
        }
    }
}

/// Interrupt-safe waker management
///
/// This struct controls access to the underlying list of wakers using
/// critical sections. This is necessary since tpically a Cntrlr mutex
/// cannot be used from an interrupt handler.
pub struct WakerSet(UnsafeCell<Vec<Waker>>);

unsafe impl Send for WakerSet {}
unsafe impl Sync for WakerSet {}

impl WakerSet {
    /// Create a new WakerSet
    pub const fn new() -> Self {
        Self(UnsafeCell::new(Vec::new()))
    }

    /// Add a waker to this WakerSet
    pub fn add(&self, waker: Waker) {
        unsafe {
            without_interrupts(|| {
                (*self.0.get()).push(waker);
            })
        }
    }

    /// Wake all tasks blocked on this set, and clear the set
    pub fn wake(&self) {
        unsafe {
            without_interrupts(|| {
                for waker in (*self.0.get()).drain(..) {
                    waker.wake()
                }
            })
        }
    }
}

static WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake, waker_wake_by_ref, waker_drop);

unsafe fn waker_clone(waker: *const ()) -> RawWaker {
    RawWaker::new(waker, &WAKER_VTABLE)
}

unsafe fn waker_wake(waker: *const ()) {
    waker_wake_by_ref(waker);
}

unsafe fn waker_wake_by_ref(waker: *const ()) {
    let waker: *const AtomicBool = waker as _;
    (*waker).store(true, Ordering::Release);
}

unsafe fn waker_drop(_waker: *const ()) {}

unsafe fn waker_new(waker: &AtomicBool) -> Waker {
    Waker::from_raw(RawWaker::new(waker as *const _ as *const _, &WAKER_VTABLE))
}
