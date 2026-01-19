#![no_std]
#![no_main]

mod gpio;

use crab_pi::libpi::delay_cycles;
use crab_pi::println;

use gpio::*;

#[unsafe(no_mangle)]
fn __user_main() {
    println!("Hello, world!");

    gpio_set_output(47);

    loop {
        gpio_set_on(47);
        delay_cycles(1000000);
        gpio_set_off(47);
        delay_cycles(1000000);
    }
}
