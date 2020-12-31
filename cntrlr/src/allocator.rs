use crate::sync::Mutex;
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

struct Free {
    next: *mut Free,
    size: usize,
}

struct Allocator {
    brk: *mut u8,
    free: *mut Free,
}

unsafe impl Send for Allocator {}

impl Allocator {
    const fn new() -> Self {
        Self {
            brk: null_mut(),
            free: null_mut(),
        }
    }

    unsafe fn allocate(&mut self, layout: Layout) -> *mut u8 {
        if self.brk.is_null() {
            return null_mut();
        }

        if layout.align() > 4 {
            return null_mut();
        }

        let size = (layout.size() + 7) & !7;

        let mut smallest_prev: *mut Free = null_mut();
        let mut smallest: *mut Free = null_mut();
        let mut prev: *mut Free = null_mut();
        let mut cur = self.free;
        while !cur.is_null() {
            if smallest.is_null() || ((*cur).size < (*smallest).size && (*cur).size >= size) {
                smallest_prev = prev;
                smallest = cur;
            }
            prev = cur;
            cur = (*cur).next;
        }

        if smallest.is_null() {
            let out = self.brk;
            self.brk = self.brk.add(size);
            out
        } else if (*smallest).size == size {
            if smallest_prev.is_null() {
                self.free = (*smallest).next;
            } else {
                (*smallest_prev).next = (*smallest).next;
            }
            smallest as *mut u8
        } else {
            (*smallest).size -= size;
            (smallest as *mut u8).add((*smallest).size)
        }
    }

    unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        let size = (layout.size() + 7) & !7;
        let mut new = ptr as *mut Free;
        (*new).size = size;
        (*new).next = null_mut();

        let mut prev: *mut Free = null_mut();
        let mut cur = self.free;
        while !cur.is_null() {
            if (new as usize + (*new).size) < cur as usize {
                break;
            }
            if (new as usize + (*new).size) == cur as usize {
                // Remove entry and add to end of new entry
                if cur.is_null() {
                    self.free = (*cur).next;
                } else {
                    (*prev).next = (*cur).next;
                }

                (*new).size += (*cur).size;
                cur = (*cur).next;
                break;
            }

            if (cur as usize + (*cur).size) == new as usize {
                // Remove entry and add to the start of new entry
                if cur.is_null() {
                    self.free = (*cur).next;
                } else {
                    (*prev).next = (*cur).next;
                }

                (*cur).size += (*new).size;
                new = cur;
                cur = (*cur).next;

                // Keep going in case the new entry bridged the gap
                // between two entries and needs to be merged again.
                continue;
            }

            prev = cur;
            cur = (*cur).next;
        }

        (*new).next = cur;
        if prev.is_null() {
            self.free = new;
        } else {
            (*prev).next = new;
        }
    }
}

struct GlobalAllocator(Mutex<Allocator>);

impl GlobalAllocator {
    const fn new() -> Self {
        Self(Mutex::new(Allocator::new()))
    }

    unsafe fn init(&self, brk: *mut u8) {
        self.0.lock().brk = brk;
    }
}
unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().allocate(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().deallocate(ptr, layout)
    }
}

#[global_allocator]
static ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

pub unsafe fn init(brk: *mut u8) {
    ALLOCATOR.init(brk);
}

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Allocation error for {:?}", layout);
}
