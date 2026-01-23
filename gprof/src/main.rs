#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::mem::MaybeUninit;
use core::ptr::addr_of;
use crab_pi::interrupt::{enable_interrupts, interrupt_init, register_irq_basic_handler, IrqHandler};
use crab_pi::kmalloc::KmallocAllocator;
use crab_pi::memory::dev_barrier;
use crab_pi::println;
use crab_pi::timer::{clear_irq, timer_get_usec, timer_init};

#[global_allocator]
static GLOBAL: KmallocAllocator = KmallocAllocator;

static mut cnt: u32 = 0;
static mut period: u32 = 0;
static mut last_clk: u32 = 0;

unsafe extern "C" {
    safe static __code_start__: [u32; 0];
    safe static __code_end__: [u32; 0];
}

static mut GPROF_COUNTS: *mut u32 = unsafe { MaybeUninit::<*mut u32>::zeroed().assume_init() };
static mut CODE_SIZE: u32 = 0;

fn gprof_init() {
    unsafe {
        let code_start = &__code_start__ as *const u32;
        let code_end = &__code_end__  as *const u32;
        let mut code_size = code_end as usize - code_start as usize;
        code_size /= 4;
        println!("code size: {}", code_size);
        let mut v: Vec<u32> = vec![0; code_size];
        CODE_SIZE = code_size as u32;

        GPROF_COUNTS = v.as_mut_ptr();
    }
}

unsafe fn gprof_inc(pc: u32) {
    // println!("pc: {:08x}", pc);

    let code_start = &__code_start__ as *const u32 as usize as u32;
    let code_location = (pc - code_start) / 4;
    *GPROF_COUNTS.add(code_location as usize) += 1;
}

unsafe fn gprof_dump(min_val: u32) {
    for i in 0..CODE_SIZE {
        let val = GPROF_COUNTS.add(i as usize).read_volatile();
        if (val >= min_val) {
            let loc = i * 4 + (&__code_start__ as *const u32 as u32);
            println!("{:08x}: {}", loc , val);
        }
    }
}

fn timer_interrupt_handler(pc: u32) {
    unsafe {
        dev_barrier();

        clear_irq();

        cnt += 1;
        gprof_inc(pc);

        // Old counter
        let clk = timer_get_usec();
        period = if last_clk == 0 { 0 } else { clk - last_clk } ;
        last_clk = clk;

        dev_barrier();
    }
}

#[unsafe(no_mangle)]
 fn __user_main() {
    unsafe {
        gprof_init();

        interrupt_init();

        register_irq_basic_handler(0, timer_interrupt_handler);

        timer_init(16, 0x100);

        enable_interrupts();

        println!("Kmalloc initialized. code start: {:p}, code end: {:p}", &__code_start__, &__code_end__);
    }


    unsafe {
        let mut iter = 0;
        while (cnt < 1000) {
            // println!("iter={}; cnt={}, period={}", iter, addr_of!(cnt).read_volatile(), addr_of!(period).read_volatile());
            iter+=1;
            // if (iter % 10 == 0){
            //     gprof_dump(2);
            // }
        }
        gprof_dump(1);

    }


}
