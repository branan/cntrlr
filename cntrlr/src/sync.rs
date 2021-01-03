// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>

//! Synchronization primitives

use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(not(mcu = "fe310g002"))]
use core::sync::atomic::AtomicBool;

/// A true or false flag
///
/// This provides [`AtomicBool`](core::sync::atomic::AtomicBool)-like
/// semantics across all supported MCUs. On MCUs without atomic
/// support, it is implemented as a critical section.
#[cfg(not(mcu = "fe310g002"))]
#[derive(Default)]
pub struct Flag(AtomicBool);

/// A true or false flag
///
/// This provides [`AtomicBool`](core::sync::atomic::AtomicBool)-like
/// semantics across all supported MCUs. On MCUs without atomic
/// support, it is implemented as a critical section.
#[cfg(mcu = "fe310g002")]
#[derive(Default)]
pub struct Flag(AtomicUsize);

unsafe impl Send for Flag {}
unsafe impl Sync for Flag {}

impl Flag {
    /// Create a new Flag
    pub const fn new(value: bool) -> Self {
        #[cfg(mcu = "fe310g002")]
        {
            Self(AtomicUsize::new(if value { 1 } else { 0 }))
        }

        #[cfg(not(mcu = "fe310g002"))]
        {
            Self(AtomicBool::new(value))
        }
    }

    /// Store a value to the flag.
    ///
    /// See [`core::sync::atomic::AtomicBool::store`]
    #[allow(clippy::useless_conversion)] // Not useless on every target
    pub fn store(&self, value: bool, ordering: Ordering) {
        self.0.store(value.into(), ordering)
    }
}

#[cfg(any(doc, target_has_atomic = "8"))]
impl Flag {
    /// Stores a value, returning the previous value
    ///
    /// See [`core::sync::atomic::AtomicBool::swap`]
    #[allow(clippy::useless_conversion)] // Not useless on every target
    pub fn swap(&self, value: bool, ordering: Ordering) -> bool {
        let out = self.0.swap(value.into(), ordering);
        #[cfg(mcu = "fe310g002")]
        {
            out != 0
        }

        #[cfg(not(mcu = "fe310g002"))]
        {
            out
        }
    }
}

#[cfg(not(any(doc, target_has_atomic = "8")))]
impl Flag {
    /// Stores a value, returning the previous value
    ///
    /// See [`core::sync::atomic::AtomicBool::swap`]
    #[allow(clippy::useless_conversion)] // Not useless on every target
    pub fn swap(&self, value: bool, ordering: Ordering) -> bool {
        let (load_ordering, store_ordering) = match ordering {
            Ordering::Relaxed => (Ordering::Relaxed, Ordering::Relaxed),
            Ordering::Acquire => (Ordering::Acquire, Ordering::Relaxed),
            Ordering::Release => (Ordering::Relaxed, Ordering::Release),
            Ordering::AcqRel => (Ordering::Acquire, Ordering::Release),
            Ordering::SeqCst => (Ordering::SeqCst, Ordering::SeqCst),
            _ => panic!("Unsupported swap ordering"),
        };
        without_interrupts(|| {
            let out = self.0.load(load_ordering);
            self.0.store(value.into(), store_ordering);
            out
        })
    }
}

/// An atomic pointer-sized value
///
/// This provides [`AtomicUsize`](core::sync::atomic::AtomicUsize)-like
/// semantics across all supported MCUs. On MCUs without atomic
/// support, it is implemented as a critical section.
pub struct Value(AtomicUsize);

unsafe impl Send for Value {}
unsafe impl Sync for Value {}

impl Value {
    /// Create a new value
    pub const fn new(value: usize) -> Self {
        Self(AtomicUsize::new(value))
    }

    /// Store a value
    ///
    /// See [`core::sync::atomic::AtomicUsize::store`]
    pub fn store(&self, value: usize, ordering: Ordering) {
        self.0.store(value, ordering)
    }

    /// Load a value
    ///
    /// See [`core::sync::atomic::AtomicUsize::load`]
    pub fn load(&self, ordering: Ordering) -> usize {
        self.0.load(ordering)
    }
}

#[cfg(any(doc, target_has_atomic = "32"))]
impl Value {
    /// Stores a value, returning the previous value
    ///
    /// See [`core::sync::atomic::AtomicUsize::swap`]
    pub fn swap(&self, value: usize, ordering: Ordering) -> usize {
        self.0.swap(value, ordering)
    }
}

#[cfg(not(any(doc, target_has_atomic = "32")))]
impl Value {
    /// Stores a value, returning the previous value
    ///
    /// See [`core::sync::atomic::AtomicUsize::swap`]
    pub fn swap(&self, value: usize, ordering: Ordering) -> usize {
        let (load_ordering, store_ordering) = match ordering {
            Ordering::Relaxed => (Ordering::Relaxed, Ordering::Relaxed),
            Ordering::Acquire => (Ordering::Acquire, Ordering::Relaxed),
            Ordering::Release => (Ordering::Relaxed, Ordering::Release),
            Ordering::AcqRel => (Ordering::Acquire, Ordering::Release),
            Ordering::SeqCst => (Ordering::SeqCst, Ordering::SeqCst),
            _ => panic!("Unsupported swap ordering"),
        };
        without_interrupts(|| {
            let out = self.0.load(load_ordering);
            self.0.store(value, store_ordering);
            out
        })
    }
}

/// A value which can be initalized only once
pub struct Once<T> {
    state: Value,
    value: UnsafeCell<Option<T>>,
}

const UNINIT: usize = 0;
const IN_PROGRESS: usize = 1;
const INIT: usize = 2;

unsafe impl<T: Send> Send for Once<T> {}
unsafe impl<T> Sync for Once<T> {}

impl<T> Once<T> {
    /// Create a new, uninitialized Once
    pub const fn new() -> Self {
        Self {
            state: Value::new(UNINIT),
            value: UnsafeCell::new(None),
        }
    }

    /// Initialize the stored value if needed, then return it.
    pub fn get_or_try_init<F>(&self, f: F) -> Option<&T>
    where
        F: FnOnce() -> Option<T>,
    {
        unsafe {
            match self.state.load(Ordering::Acquire) {
                IN_PROGRESS => panic!("Lock contention"),
                UNINIT => match self.state.swap(IN_PROGRESS, Ordering::Acquire) {
                    IN_PROGRESS => panic!("Lock contention"),
                    UNINIT => {
                        if let Some(value) = f() {
                            *self.value.get() = Some(value);
                            self.state.store(INIT, Ordering::Release);
                        } else {
                            self.state.store(UNINIT, Ordering::Release);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
            (&*self.value.get()).as_ref()
        }
    }
}

/// A simple lock
///
/// This lock does not disable interrupts, so attempting to use a
/// mutex in an interrupt context may deadlock. In interrupt contexts,
/// prefer a dedicated synchronization primitive based around
/// [`without_interrupts`]
pub struct Mutex<T> {
    lock: Flag,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Create a new Mutex
    pub const fn new(item: T) -> Self {
        Self {
            lock: Flag::new(false),
            value: UnsafeCell::new(item),
        }
    }

    /// Acquire this mutex
    pub fn lock(&self) -> MutexGuard<T> {
        while self.lock.swap(true, Ordering::AcqRel) {}
        MutexGuard(self)
    }

    unsafe fn unlock(&self) {
        self.lock.store(false, Ordering::Release)
    }
}

/// An RAII guard for a [`Mutex`]
pub struct MutexGuard<'a, T>(&'a Mutex<T>);

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.value.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.value.get() }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            self.0.unlock();
        }
    }
}

struct InterruptGate {
    count: UnsafeCell<usize>,
    enable: UnsafeCell<bool>,
}

unsafe impl Sync for InterruptGate {}

static INTERRUPTS: InterruptGate = InterruptGate::new();

impl InterruptGate {
    const fn new() -> Self {
        Self {
            count: UnsafeCell::new(0),
            enable: UnsafeCell::new(false),
        }
    }

    fn enable(&self) {
        unsafe {
            if !arch::interrupts_enabled() {
                *self.count.get() -= 1;
                if *self.count.get() == 0 && *self.enable.get() {
                    *self.enable.get() = false;
                    arch::enable_interrupts();
                }
            }
        }
    }

    fn disable(&self) {
        unsafe {
            if arch::interrupts_enabled() {
                arch::disable_interrupts();
                *self.enable.get() = true;
            }
            *self.count.get() += 1;
        }
    }

    fn flag_enable(&self) {
        unsafe {
            self.disable();
            *self.enable.get() = true;
            self.enable();
        }
    }
}

/// Runs the passed closure without interrupts, in a critical section.
///
/// This can be used as the underlying locking mechanism for
/// situations where [`Mutex`] is not appropriate.
pub fn without_interrupts<T, F>(f: F) -> T
where
    F: FnOnce() -> T,
{
    INTERRUPTS.disable();
    let out = f();
    INTERRUPTS.enable();
    out
}

/// Enable interrupts
///
/// Ensure that interrupts will be enabled once the outermost
/// [`without_interrupts`] call is completed.
pub fn enable_interrupts() {
    INTERRUPTS.flag_enable()
}

#[cfg(target_arch = "riscv32")]
mod arch {
    use bit_field::BitField;
    use core::sync::atomic::{compiler_fence, Ordering};

    pub fn interrupts_enabled() -> bool {
        unsafe {
            let mut mstatus: u32;
            asm!("csrr {0}, mstatus", out(reg) mstatus);
            mstatus.get_bit(3)
        }
    }

    pub fn disable_interrupts() {
        unsafe {
            let mut mstatus: u32;
            asm!("csrr {0}, mstatus", out(reg) mstatus);
            mstatus.set_bit(3, false);
            asm!("csrw mstatus, {0}", in(reg) mstatus);
        }
        compiler_fence(Ordering::Acquire);
    }

    pub unsafe fn enable_interrupts() {
        compiler_fence(Ordering::Release);
        let mut mstatus: u32;
        asm!("csrr {0}, mstatus", out(reg) mstatus);
        mstatus.set_bit(3, true);
        asm!("csrw mstatus, {0}", in(reg) mstatus);
    }
}

#[cfg(target_arch = "arm")]
mod arch {
    use core::sync::atomic::{compiler_fence, Ordering};

    #[inline]
    pub fn interrupts_enabled() -> bool {
        unsafe {
            let primask: u32;
            asm!("mrs {}, primask", out(reg) primask);
            primask & 1 == 0
        }
    }

    #[inline]
    pub unsafe fn disable_interrupts() {
        asm!(
            "cpsid i
             dmb
             dsb
             isb"
        );
        compiler_fence(Ordering::Acquire);
    }

    #[inline]
    pub unsafe fn enable_interrupts() {
        compiler_fence(Ordering::Release);
        asm!(
            "dmb
             dsb
             isb
             cpsie i"
        );
    }
}
