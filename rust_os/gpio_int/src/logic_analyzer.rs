#![feature(sync_unsafe_cell)]
#![no_std]
#![no_main]
#![allow(static_mut_refs)]

extern crate alloc;

use alloc::collections::VecDeque;
use core::cell::SyncUnsafeCell;
use core::time::Duration;
use crab_pi::cache::caches_enable;
use crab_pi::cycle_count::cycle_cnt_read;
use crab_pi::gpio::{gpio_int_falling_edge, gpio_int_rising_edge, gpio_interrupt_enable, gpio_interrupt_init, gpio_register_interrupt_handler, gpio_set_function, gpio_write, GPIOEvent, GPIO_FUNC};
use crab_pi::interrupt::{enable_interrupts, interrupt_init};
use crab_pi::memory::dev_barrier;
use crab_pi::println;
use crab_pi::timer::sleep;
use sw_uart::sw_uart::{baud_to_cycles, SwUart};

const OUT_PIN: u32 = 21;
const IN_PIN: u32 = 20;

// (Value, Time)
static mut BUFFER: SyncUnsafeCell<VecDeque<(u8, u32)>> = SyncUnsafeCell::new(VecDeque::new());

fn gpio_handler(pin: u32, event: GPIOEvent) {
    let time = cycle_cnt_read();
    let val: u8 = if matches!(event, GPIOEvent::RisingEdge) { 1 } else { 0 };

    unsafe { BUFFER.get_mut().push_back((val, time)) };
}

unsafe fn get_8() -> u8 {
    let (val, start_time) = BUFFER.get_mut().pop_front().unwrap();

    let mut last_elem = val;
    let mut last_time = start_time + baud_to_cycles(115200);

    let mut i = 0;

    let mut result: u8 = 0;

    for (v, t) in BUFFER.get_mut().iter() {
        let num_elms = (t - last_time) / baud_to_cycles(115200);

        for _ in 0..num_elms {
            result |= last_elem << i;
            i += 1;
        }

        last_time = *t;
        last_elem = *v;
    }

    while (i < 8) {
        result |= last_elem << i;
        i += 1;
    }

    BUFFER.get_mut().clear();

    result
}

#[unsafe(no_mangle)]
fn __user_main() {
    //Initialize Buffer
    unsafe {
        BUFFER.get_mut().reserve(16);
    }

    // Initialize GPIO pins
    let uart = SwUart::new(OUT_PIN, IN_PIN, 115200);
    gpio_int_falling_edge(IN_PIN);
    gpio_int_rising_edge(IN_PIN);

    println!("Cycles per bit: {}", uart.get_cycles_per_bit());


    // Initialize Interrupts
    unsafe{
        interrupt_init();
        caches_enable();

        gpio_register_interrupt_handler(IN_PIN, gpio_handler);
        gpio_interrupt_init();
        gpio_interrupt_enable();

        enable_interrupts();
    }


    for _ in 0..2 {
        uart.put_8(0b01010101);
        sleep(Duration::from_millis(100));

        let result = unsafe {get_8()};

        assert_eq!(result, 0b01010101);
    }
    
    println!("Done")
}
