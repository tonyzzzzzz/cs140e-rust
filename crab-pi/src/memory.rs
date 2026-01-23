use core::arch::{asm, global_asm};

global_asm!(include_str!("../asm/mem-barrier.S"));

unsafe extern "C" {
    pub safe fn dev_barrier();
    pub safe fn dmb();
    pub safe fn dsb();
}

#[inline]
pub unsafe fn gcc_mb() {
    asm!("", options(nostack));
}