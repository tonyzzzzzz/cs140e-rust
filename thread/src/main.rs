#![no_std]
#![no_main]
extern crate alloc;

mod t3_test_exit;

use crab_pi::kmalloc::KmallocAllocator;

#[global_allocator]
static GLOBAL: KmallocAllocator = KmallocAllocator;

/*
// run N trivial threads that explicitly call exit; no yield
#include "test-header.h"

void trivial(void* arg) {
    trace("trivial thread: arg=%d\n", (unsigned)arg);

    // manually call rpi_exit
    rpi_exit(0);
}

void notmain(void) {
    test_init();
    // make this > 1 to test
    int n = 10;
    for(int i = 0; i < n; i++)
        rpi_fork(trivial, (void*)i);
    rpi_thread_start();
    test_done();
    trace("SUCCESS\n");
}

 */
// extern "C"  fn thread_code(arg: *const u32) {
//     let cur_thread = rpi_cur_thread();
//     println!("in thread [{:p}], tid={} with x={}", cur_thread, cur_thread.thread_id, unsafe {*arg});
//     assert_eq!(cur_thread.thread_id, 1);
//     assert_eq!(unsafe {*arg}, 0xdeadbeef);
// 
//     restart();
// }

#[unsafe(no_mangle)]
fn __user_main() {
   t3_test_exit::t3_test_exit();
}
