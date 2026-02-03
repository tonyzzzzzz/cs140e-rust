#![allow(static_mut_refs)]

use alloc::boxed::Box;
use crab_pi::println;
use crab_pi::thread::{rpi_exit, rpi_fork, rpi_thread_start, RUN_Q};

extern "C" fn trivial(arg: *const u32) {
    println!("trivial thread: arg={}", unsafe { *arg });

    rpi_exit(0);
}

pub fn t3_test_exit() {

    for i in 0..10 {
        let arg = Box::new(i);
        rpi_fork(trivial, Box::into_raw(arg) as *const u32);
    }
    rpi_thread_start();

    println!("SUCCESS");
}