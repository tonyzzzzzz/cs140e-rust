#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use core::time::Duration;
use crab_pi::gpio::{gpio_int_falling_edge, gpio_int_rising_edge, gpio_interrupt_enable, gpio_interrupt_init, gpio_read, gpio_register_interrupt_handler, gpio_set_function, gpio_write, GPIOEvent, GPIO_FUNC};
use crab_pi::interrupt::{enable_interrupts, interrupt_init, register_irq_basic_handler};
use crab_pi::memory::{dev_barrier, dmb};
use crab_pi::println;
use crab_pi::timer::{clear_irq, sleep, timer_init};

const OUT_PIN: u32 = 21;
const IN_PIN: u32 = 20;

static mut rising_edge_count: u32 = 0;
static mut falling_edge_count: u32 = 0;

fn gpio_handler(pin: u32, event: GPIOEvent) {
    match event {
        GPIOEvent::RisingEdge => unsafe { rising_edge_count += 1 },
        GPIOEvent::FallingEdge => unsafe { falling_edge_count += 1 },
    }
}

fn timer_interrupt_handler(pc: u32) {
    unsafe {
        dev_barrier();

        clear_irq();

        dev_barrier();
    }
}

#[unsafe(no_mangle)]
fn __user_main() {
    // Initialize GPIO pins
    gpio_set_function(OUT_PIN, GPIO_FUNC::OUTPUT);
    gpio_set_function(IN_PIN, GPIO_FUNC::INPUT);
    gpio_write(OUT_PIN, true);
    gpio_int_falling_edge(IN_PIN);
    gpio_int_rising_edge(IN_PIN);


    // Initialize Interrupts
    unsafe{
        interrupt_init();

        gpio_register_interrupt_handler(IN_PIN, gpio_handler);
        gpio_interrupt_init();
        gpio_interrupt_enable();

        register_irq_basic_handler(0, timer_interrupt_handler);
        timer_init(1, 0x100);

        enable_interrupts();
    }

    let N = 1024*32;

    for i in 1..= N {

        gpio_write(OUT_PIN, false);
        unsafe {assert_eq!(i, falling_edge_count)};

        gpio_write(OUT_PIN, true);
        dev_barrier();

        unsafe {
            assert_eq!(i, rising_edge_count);
            assert_eq!(falling_edge_count, rising_edge_count);
        };

        if(i % 1024 == 0){
            println!("{}/{}", i, N);
        }
    }
}
