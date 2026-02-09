#![no_std]
#![no_main]

global_asm!(include_str!("../asm/interrupt-asm.S"));

use core::arch::{asm, global_asm};
use crab_pi::cache::caches_enable;
use crab_pi::cycle_count::cycle_cnt_read;
use crab_pi::println;
use crab_pi::vector_base::{vector_base_reset, vector_base_set};

unsafe extern "C" {
    fn sys_plus1(x: u32) -> u32;
    static _interrupt_vector_orig: [u32; 0];
    static _interrupt_vector_slow: [u32; 0];
    static _interrupt_vector_fast: [u32; 0];
}

#[unsafe(no_mangle)]
fn __user_main() {
    unsafe {
        test_swi(
            "orig lab 5 vector(no cache)",
            &_interrupt_vector_orig as *const u32,
        );
        test_swi(
            "better relocation (no cache)",
            &_interrupt_vector_slow as *const u32,
        );
        test_swi(
            "fast relocation (no cache)",
            &_interrupt_vector_fast as *const u32,
        );
    }

    caches_enable();
    unsafe {
        test_swi(
            "orig lab 5 vector(no cache)",
            &_interrupt_vector_orig as *const u32,
        );
        test_swi(
            "better relocation (icache enabled)",
            &_interrupt_vector_slow as *const u32,
        );
        test_swi(
            "fast relocation (icache cache)",
            &_interrupt_vector_fast as *const u32,
        );
    }
}

unsafe fn test_swi(name: &str, vector_base: *const u32) {
    vector_base_reset(vector_base);

    unsafe {
        // Align to 16 bytes (2^4) to reduce prefetch buffer effects
        asm!(".align 4", options(nomem, nostack, preserves_flags));
    }
    let s = cycle_cnt_read();
    sys_plus1(1);
    let t = cycle_cnt_read() - s;
    println!("{}: single call took {} cycles", name, t);

    unsafe {
        // Align to 16 bytes (2^4) to reduce prefetch buffer effects
        asm!(".align 4", options(nomem, nostack, preserves_flags));
    }
    let s = cycle_cnt_read();
    sys_plus1(1);
    let t = cycle_cnt_read() - s;
    println!("{}: single call took {} cycles", name, t);

    unsafe {
        // Align to 16 bytes (2^4) to reduce prefetch buffer effects
        asm!(".align 4", options(nomem, nostack, preserves_flags));
    }
    let s = cycle_cnt_read();
    sys_plus1(1);
    let t = cycle_cnt_read() - s;
    println!("{}: single call took {} cycles", name, t);

    unsafe {
        // Align to 16 bytes (2^4) to reduce prefetch buffer effects
        asm!(".align 4", options(nomem, nostack, preserves_flags));
    }
    let s = cycle_cnt_read();
    sys_plus1(1);
    sys_plus1(1);
    sys_plus1(1);
    sys_plus1(1);
    sys_plus1(1);
    sys_plus1(1);
    sys_plus1(1);
    sys_plus1(1);
    sys_plus1(1);
    sys_plus1(1);
    let t = cycle_cnt_read() - s;
    println!("{}: 10 calls took {} cycles", name, t);
}
