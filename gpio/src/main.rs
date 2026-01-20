#![no_std]
#![no_main]

mod gpio;

use crab_pi::libpi::delay_cycles;
use crab_pi::println;

use gpio::*;

const LED_1: u32 = 20;
const LED_2: u32 = 27;
const ACT: u32 = 47;
const OUTPUT: u32 = 9;
const INPUT: u32 = 8;

fn _1_blink() {
    gpio_set_output(LED_1);

    loop {
        gpio_set_on(LED_1);
        delay_cycles(1000000);
        gpio_set_off(LED_1);
        delay_cycles(1000000);
    }
}

fn _2_blink() {
    gpio_set_output(LED_1);
    gpio_set_output(LED_2);

    loop {
        gpio_set_on(LED_1);
        gpio_set_off(LED_2);
        delay_cycles(3000000);
        gpio_set_off(LED_1);
        gpio_set_on(LED_2);
        delay_cycles(3000000);
    }
}

fn _3_loopback() {
    gpio_set_output(LED_1);
    gpio_set_output(LED_2);
    gpio_set_input(INPUT);
    gpio_set_output(OUTPUT);
    
    let mut v = false;
    
    loop {
        gpio_write(LED_1, v);
        
        gpio_write(OUTPUT, v);
        gpio_write(LED_2, gpio_read(INPUT));
        
        delay_cycles(1000000);
        
        v = !v;
    }
}

fn _4_act_blink() {
    gpio_set_output(ACT);

    loop {
        gpio_set_on(ACT);
        delay_cycles(1000000);
        gpio_set_off(ACT);
        delay_cycles(1000000);
    }
}

fn _5_all() {
    gpio_set_output(LED_1);
    gpio_set_output(LED_2);
    gpio_set_output(ACT);

    loop {
        gpio_set_off(ACT);
        gpio_set_on(LED_1);
        gpio_set_on(LED_2);
        delay_cycles(1000000);
        gpio_set_on(ACT);
        gpio_set_off(LED_1);
        gpio_set_off(LED_2);
        delay_cycles(1000000);
    }
}

#[unsafe(no_mangle)]
fn __user_main() {
    _3_loopback();
}

