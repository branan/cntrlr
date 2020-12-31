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

#[derive(Default)]
pub struct Executor {
    tasks: Vec<Task>,
}

impl Executor {
    pub fn new() -> Self {
        Default::default()
    }

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

    pub fn run(&mut self) -> ! {
        loop {
            for task in &mut self.tasks {
                if task.wake.load(Ordering::Acquire) {
                    task.wake.store(false, Ordering::Relaxed);
                    let waker = unsafe { waker_new(&task.wake) };
                    let mut context = Context::from_waker(&waker);
                    let _ = task.future.as_mut().poll(&mut context);
                }
            }
        }
    }
}

pub struct WakerSet(UnsafeCell<Vec<Waker>>);

unsafe impl Send for WakerSet {}
unsafe impl Sync for WakerSet {}

impl WakerSet {
    pub const fn new() -> Self {
        Self(UnsafeCell::new(Vec::new()))
    }

    pub fn add(&self, waker: Waker) {
        unsafe {
            without_interrupts(|| {
                (*self.0.get()).push(waker);
            })
        }
    }

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
