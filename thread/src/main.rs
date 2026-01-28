#![no_std]
#![no_main]
extern crate alloc;

mod t3_test_exit;
mod t4_test_yield;
mod t5_test_implicit_exit;
mod t7_realtime_yield;

#[unsafe(no_mangle)]
fn __user_main() {
   // t3_test_exit::t3_test_exit();
   // t4_test_yield::t4_test_yield();
   // t5_test_implicit_exit::t5_test_implicit_exit();
   t7_realtime_yield::t7_realtime_yield();
}
