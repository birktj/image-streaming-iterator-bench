use std::alloc::{System, GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering::SeqCst};

pub struct Counter;

static ALLOCATED: AtomicUsize = ATOMIC_USIZE_INIT;
static ALLOCATED_MAX: AtomicUsize = ATOMIC_USIZE_INIT;

unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            let val = ALLOCATED.fetch_add(layout.size(), SeqCst) + layout.size();
            loop {
                let max = ALLOCATED_MAX.load(SeqCst);
                if val > max {
                    let new = ALLOCATED_MAX.compare_and_swap(max, val, SeqCst);
                    if new == val {
                        break;
                    }
                }
                else {
                    break;
                }
            }
        }
        return ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), SeqCst);
    }
}

pub fn get_allocated() -> usize {
    ALLOCATED.load(SeqCst) 
}

pub fn get_max_allocated() -> usize {
    ALLOCATED_MAX.load(SeqCst) 
}

pub fn reset_allocator() {
    ALLOCATED_MAX.store(0, SeqCst);
}
