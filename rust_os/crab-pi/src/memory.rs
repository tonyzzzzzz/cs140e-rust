use crate::kmalloc::KmallocAllocator;
use core::arch::{asm, global_asm};

#[global_allocator]
static GLOBAL: KmallocAllocator = KmallocAllocator;

global_asm!(include_str!("../asm/mem-barrier.S"));

unsafe extern "C" {
    pub safe fn dev_barrier();
    pub safe fn dmb();
    pub safe fn dsb();
}

#[inline]
pub fn gcc_mb() {
    unsafe {asm!("", options(nostack))};
}
