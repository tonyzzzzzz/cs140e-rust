
#![allow(static_mut_refs)] // TODO: better ways
use alloc::boxed::Box;
use crab_pi::println;
use crab_pi::thread::{rpi_cur_thread_id, rpi_fork, rpi_thread_start};

static mut thread_count: usize = 0;
static mut thread_sum: usize = 0;

extern "C" fn thread_code(arg: *const u32) {
    let x = unsafe { *arg };

    println!("in thread tid = {} with x = {}", rpi_cur_thread_id(), x);

    assert_eq!(rpi_cur_thread_id() as u32, x+1);

    unsafe {
        thread_count += 1;
        thread_sum += x as usize;
    }
}

pub fn t5_test_implicit_exit(){
    let n = 30;

    let mut sum = 0;
    for i in 0..n {
        let x = Box::new(i);
        sum += i;
        rpi_fork(thread_code, Box::into_raw(x) as *const u32);
    }
    rpi_thread_start();

    unsafe {
        println!("count = {}, sum = {}", thread_count, thread_sum);
        assert_eq!(thread_count, n);
        assert_eq!(thread_sum, sum);
    }
    println!("SUCCESS");
}

