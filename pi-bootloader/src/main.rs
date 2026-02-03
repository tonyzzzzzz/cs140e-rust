#![no_std]
#![no_main]

extern crate crab_pi;

use core::arch::{asm, global_asm};
use core::time::Duration;
use crab_pi::{println, uart, watchdog};
use constants::BOOT_OP;
use crab_pi::memory::gcc_mb;
use crab_pi::timer::sleep;

global_asm!(include_str!("../asm/boot.S"));

unsafe extern "C" {
    // We declare these as `[u32; 0]` so that they have an alignment of 4 but a size of zero. This
    // is to prevent aliasing, since otherwise producing mutable references to anything in the BSS
    // would be undefined behaviour.

    safe static __bss_start__: [u32; 0];
    safe static __bss_end__: [u32; 0];
}

unsafe fn zero_out_bss() {
    gcc_mb();
    let bss_start_ptr = &__bss_start__;
    let bss_end_ptr = &__bss_end__;
    let bss_size = bss_end_ptr as *const u32 as usize - bss_start_ptr as *const u32 as usize;
    let bss_start = bss_start_ptr as *const u32 as *mut u32;
    unsafe {
        core::ptr::write_bytes(bss_start, 0, bss_size / 4);
    }
    gcc_mb();
}

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



#[unsafe(no_mangle)]
extern "C" fn _cstart() {
    unsafe {
        zero_out_bss();

        uart::init(115200);

        cycle_cnt_init();
    }

    main();

    watchdog::clean_reboot();
}

#[unsafe(no_mangle)]
extern "C" fn rpi_reboot() {

}



pub fn wait_for_data(timeout: Option<Duration>) {
    let packet = BOOT_OP::GET_PROG_INFO.val().to_le_bytes();
    loop {
        uart::write_bytes(&packet);
        sleep(Duration::from_millis(300));
    }
}




fn main() {
    wait_for_data(None)
}
