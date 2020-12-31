use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(not(mcu = "fe310g002"))]
use core::sync::atomic::AtomicBool;

#[cfg(not(mcu = "fe310g002"))]
#[derive(Default)]
pub struct Flag(AtomicBool);

#[cfg(mcu = "fe310g002")]
#[derive(Default)]
pub struct Flag(AtomicUsize);

unsafe impl Send for Flag {}
unsafe impl Sync for Flag {}

impl Flag {
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

    pub fn store(&self, value: bool, ordering: Ordering) {
        self.0.store(value.into(), ordering)
    }
}

#[cfg(any(doc, target_has_atomic = "8"))]
impl Flag {
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

pub struct Value(AtomicUsize);

unsafe impl Send for Value {}
unsafe impl Sync for Value {}

impl Value {
    pub const fn new(value: usize) -> Self {
        Self(AtomicUsize::new(value))
    }

    pub fn store(&self, value: usize, ordering: Ordering) {
        self.0.store(value, ordering)
    }

    pub fn load(&self, ordering: Ordering) -> usize {
        self.0.load(ordering)
    }
}

#[cfg(any(doc, target_has_atomic = "32"))]
impl Value {
    pub fn swap(&self, value: usize, ordering: Ordering) -> usize {
        self.0.swap(value, ordering)
    }
}

#[cfg(not(any(doc, target_has_atomic = "32")))]
impl Value {
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
    pub const fn new() -> Self {
        Self {
            state: Value::new(UNINIT),
            value: UnsafeCell::new(None),
        }
    }

    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        unsafe {
            match self.state.load(Ordering::Acquire) {
                IN_PROGRESS => panic!("Lock contention"),
                UNINIT => match self.state.swap(IN_PROGRESS, Ordering::Acquire) {
                    IN_PROGRESS => panic!("Lock contention"),
                    UNINIT => {
                        let value = f();
                        *self.value.get() = Some(value);
                        self.state.store(INIT, Ordering::Release);
                    }
                    _ => {}
                },
                _ => {}
            }
            (&*self.value.get()).as_ref().unwrap()
        }
    }
}

pub struct Mutex<T> {
    lock: Flag,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(item: T) -> Self {
        Self {
            lock: Flag::new(false),
            value: UnsafeCell::new(item),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        while self.lock.swap(true, Ordering::AcqRel) {}
        MutexGuard(self)
    }

    unsafe fn unlock(&self) {
        self.lock.store(false, Ordering::Release)
    }
}

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

pub fn without_interrupts<T, F>(f: F) -> T
where
    F: FnOnce() -> T,
{
    INTERRUPTS.disable();
    let out = f();
    INTERRUPTS.enable();
    out
}

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