#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use crab_pi::kmalloc::KmallocAllocator;
use crab_pi::println;

#[global_allocator]
static GLOBAL: KmallocAllocator = KmallocAllocator;

#[unsafe(no_mangle)]
fn __user_main() {
    println!("Kmalloc initialized.");

    unsafe {
        let content = Box::new(100);
        println!("{:p}", content);
        println!("{}", content);
    }

}
