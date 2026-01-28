use alloc::boxed::Box;
use crab_pi::println;
use crab_pi::thread::{rpi_cur_thread_id, rpi_exit, rpi_fork, rpi_thread_start, rpi_yield, RUN_Q};

extern "C" fn trivial(arg: *const u32) {
    println!("thread {} yielding", rpi_cur_thread_id());
    rpi_yield();
    println!("thread {} exiting", rpi_cur_thread_id());
    rpi_exit(0);
}

pub fn t4_test_yield(){

    for i in 0..10 {
        let arg = Box::new(i);
        rpi_fork(trivial, Box::into_raw(arg) as *const u32);
    }
    rpi_thread_start();

    println!("SUCCESS");
}