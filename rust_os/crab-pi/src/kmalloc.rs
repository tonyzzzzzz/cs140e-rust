/*
/*******************************************************************************
 * simple memory allocation: no free, just have to reboot().
 */

// returns 0-filled memory.
void *kmalloc(unsigned nbytes) ;
void *kmalloc_notzero(unsigned nbytes) ;
void *kmalloc_aligned(unsigned nbytes, unsigned alignment);

// initialize and set where the heap starts and give a maximum
// size in bytes
void kmalloc_init_set_start(void *addr, unsigned max_nbytes);
static inline void kmalloc_init(void) {
    unsigned long MB = 1024*1024;
    kmalloc_init_set_start((void*)MB, 64*MB);
}

// return pointer to the first free byte.  used for
// bounds checking.
void *kmalloc_heap_ptr(void);
// pointer to initial start of heap
void *kmalloc_heap_start(void);
// pointer to end of heap
void *kmalloc_heap_end(void);
 */
use core::alloc::{AllocError, Allocator, GlobalAlloc, Layout};
use core::ptr::{with_exposed_provenance_mut, NonNull};
use core::sync::atomic::{AtomicBool, Ordering};
use crate::println;

unsafe extern "C" {
    static __heap_start__: [u8; 0];
}

unsafe extern "C" {
    fn kmalloc(nbytes: usize) -> *mut u8;
    pub fn kmalloc_notzero(nbytes: usize) -> *mut u8;
    pub fn kmalloc_aligned(nbytes: usize, alignment: usize) -> *mut u8;

    pub fn kmalloc_init_set_start(addr: *mut u8, max_nbytes: usize);
    pub fn kmalloc_heap_ptr() -> *mut u8;
    pub fn kmalloc_heap_start() -> *mut u8;
    pub fn kmalloc_heap_end() -> *mut u8;

}

pub fn kmalloc_alloc<T>() -> *mut T {
    unsafe { kmalloc(size_of::<T>()) as *mut T }
}

static KMALLOC_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub struct KmallocAllocator;

impl KmallocAllocator {
    pub fn new(start: *mut u8, max_nbytes: usize) -> Self {
        unsafe { kmalloc_init_set_start(start, max_nbytes) };
        Self
    }
}

impl Default for KmallocAllocator {
    fn default() -> Self {
        let MB: usize = 1024 * 1024;

        unsafe { kmalloc_init_set_start(with_exposed_provenance_mut(MB), 64*MB) };
        Self
    }
}

unsafe impl Allocator for KmallocAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let alignment = layout.align();
        let start = unsafe {kmalloc_aligned(layout.size(), alignment) };
        let non_null_arr = unsafe {NonNull::new_unchecked(start)};
        Ok(NonNull::slice_from_raw_parts(non_null_arr, layout.size()))
    }

    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // NOP, does not support deallocation
    }

    unsafe fn grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let alignment = new_layout.align();
        let start = unsafe {kmalloc_aligned(new_layout.size(), alignment) };
        for i in 0..old_layout.size() {
            start.add(i).write(ptr.as_ptr().add(i).read());
        }
        let non_null_arr = unsafe {NonNull::new_unchecked(start)};

        Ok(NonNull::slice_from_raw_parts(non_null_arr, new_layout.size()))
    }

    unsafe fn grow_zeroed(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.allocate_zeroed(new_layout)
    }

    unsafe fn shrink(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let alignment = new_layout.align();
        let start = unsafe {kmalloc_aligned(new_layout.size(), alignment) };
        for i in 0..new_layout.size() {
            start.add(i).write(ptr.as_ptr().add(i).read());
        }
        let non_null_arr = unsafe {NonNull::new_unchecked(start)};

        Ok(NonNull::slice_from_raw_parts(non_null_arr, new_layout.size()))
    }

    fn by_ref(&self) -> &Self
    where
        Self: Sized
    {
        self
    }
}

#[inline(always)]
unsafe fn ensure_init_default() {
    if !KMALLOC_INITIALIZED.load(Ordering::Acquire) {
        // Default heap placement: start at 1 MiB, size 64 MiB.
        let mb: usize = 1024 * 1024;
        kmalloc_init_set_start(&__heap_start__ as *const u8 as *mut u8, 64 * mb);
        KMALLOC_INITIALIZED.store(true, Ordering::Release);
    }
}

unsafe impl GlobalAlloc for KmallocAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        println!("Allocating {} size", layout.size());
        ensure_init_default();
        let ptr = kmalloc_aligned(layout.size(), layout.align());
        println!("Allocated at {:p}", ptr);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        println!("Allocating {} size", layout.size());
        ensure_init_default();
        println!("Deallocating, NOP")
        // NOP, does not support deallocation
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        println!("Allocating {} size", layout.size());
        ensure_init_default();
        let ptr = kmalloc_aligned(layout.size(), layout.align());
        println!("Allocated at {:p}", ptr);
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        println!("Reallocating from {} to {} bytes", layout.size(), new_size);
        ensure_init_default();
        let new_ptr = kmalloc_aligned(new_size, layout.align());
        // Copy old data to new allocation
        let copy_size = layout.size().min(new_size);
        core::ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
        println!("Reallocated from {:p} to {:p}", ptr, new_ptr);
        new_ptr
    }
}