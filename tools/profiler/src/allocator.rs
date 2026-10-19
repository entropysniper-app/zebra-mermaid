use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};

pub struct Counted;
pub static BLOCKS: AtomicUsize = AtomicUsize::new(0);
pub static LIVE: AtomicUsize = AtomicUsize::new(0);
pub static PEAK: AtomicUsize = AtomicUsize::new(0);
pub static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
pub static REALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
pub static BYTES: AtomicUsize = AtomicUsize::new(0);
pub static TRACK: AtomicBool = AtomicBool::new(false);
fn allocated(size: usize, reallocation: bool) {
    if !reallocation {
        BLOCKS.fetch_add(1, Relaxed);
    }
    let live = LIVE.fetch_add(size, Relaxed) + size;
    if TRACK.load(Relaxed) {
        PEAK.fetch_max(live, Relaxed);
        if reallocation {
            REALLOCATIONS.fetch_add(1, Relaxed);
        } else {
            ALLOCATIONS.fetch_add(1, Relaxed);
        }
        BYTES.fetch_add(size, Relaxed);
    }
}
// SAFETY: all allocations are delegated unchanged to System; counters never dereference
// pointers, allocate memory, or alter sizes/alignments. Failed allocation/reallocation
// leaves counters unchanged. This executable is isolated from the production library.
unsafe impl GlobalAlloc for Counted {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            allocated(layout.size(), false);
        }
        p
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            allocated(layout.size(), false);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Relaxed);
        BLOCKS.fetch_sub(1, Relaxed);
        unsafe { System.dealloc(p, layout) };
    }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(p, layout, size) };
        if !next.is_null() {
            LIVE.fetch_sub(layout.size(), Relaxed);
            allocated(size, true);
        }
        next
    }
}
