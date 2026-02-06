use core::arch::asm;

#[inline(always)]
pub fn cycle_cnt_init() {
    unsafe {
        asm!(
        "mcr p15, 0, {val}, c15, c12, 0",
        val = in(reg) 1u32,
        options(nomem, nostack)
        );
    }
}


#[inline(always)]
pub fn cycle_cnt_read() -> u32 {
    let mut cnt: u32;
    unsafe { asm!("mrc p15, 0, {cnt}, c15, c12, 1", cnt = out(reg) cnt, options(nomem, nostack)) };
    cnt
}

pub fn wait_until_cycle(cycle: u32) {
    while cycle_cnt_read() < cycle {}
}

pub fn wait_cycles(cycles: u32) {
    wait_until_cycle(cycle_cnt_read() + cycles);
}