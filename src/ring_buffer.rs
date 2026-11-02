//! Pure-Rust Lock-Free Ring Buffer without Heap Allocation
//! Zero allocations on the hot path, cacheline-aligned (64 bytes) to prevent false sharing.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::UnsafeCell;

pub const BUFFER_CAPACITY: usize = 1024;

#[repr(align(64))]
pub struct LockFreeRingBuffer<T: Copy + Default, const N: usize = BUFFER_CAPACITY> {
    head: AtomicUsize,
    _pad1: [u8; 56],
    tail: AtomicUsize,
    _pad2: [u8; 56],
    slots: [UnsafeCell<T>; N],
}

unsafe impl<T: Copy + Default + Send, const N: usize> Sync for LockFreeRingBuffer<T, N> {}
unsafe impl<T: Copy + Default + Send, const N: usize> Send for LockFreeRingBuffer<T, N> {}

impl<T: Copy + Default, const N: usize> LockFreeRingBuffer<T, N> {
    pub const fn new() -> Self {
        assert!(N > 0 && (N & (N - 1)) == 0, "Capacity must be a power of two");
        
        // Safety: UnsafeCell array initialized safely with default representation
        let slots = unsafe {
            let arr: [UnsafeCell<T>; N] = std::mem::zeroed();
            arr
        };

        Self {
            head: AtomicUsize::new(0),
            _pad1: [0; 56],
            tail: AtomicUsize::new(0),
            _pad2: [0; 56],
            slots,
        }
    }

    #[inline(always)]
    pub fn push(&self, item: T) -> Result<(), &'static str> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if head.wrapping_sub(tail) >= N {
            return Err("Ring buffer full");
        }

        let idx = head & (N - 1);
        unsafe {
            *self.slots[idx].get() = item;
        }

        self.head.store(head.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    #[inline(always)]
    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if tail == head {
            return None;
        }

        let idx = tail & (N - 1);
        let item = unsafe { *self.slots[idx].get() };

        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(item)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        head.wrapping_sub(tail)
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
