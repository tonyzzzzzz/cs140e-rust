#![no_std]
#![no_main]

use crab_pi::println;

#[unsafe(no_mangle)]
fn __user_main() {
    println!("Hello, world!");
}
