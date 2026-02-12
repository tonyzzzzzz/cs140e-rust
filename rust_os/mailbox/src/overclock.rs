#![no_std]
#![no_main]

use core::arch::asm;
use core::ptr::addr_of;
use crab_pi::cache::caches_enable;
use crab_pi::cycle_count::cycle_cnt_read;
use crab_pi::gpio::GPIO_REG;
use crab_pi::mailbox::{rpi_clock_hz_set, RpiClockType};
use crab_pi::memory::{dev_barrier, dmb};
use crab_pi::println;
use crab_pi::timer::{timer_get_usec, timer_get_usec_raw};

#[macro_export]
macro_rules! time_cyc {
    ($fn:expr) => {{
        let start = cycle_cnt_read();
        let result = $fn;
        let elapsed = cycle_cnt_read() - start;
        (elapsed, result)
    }};
}

#[macro_export]
macro_rules! time_cyc_print {
    ($msg:expr, $fn:expr) => {{
        let (cycles, result) = $crate::time_cyc!($fn);
        println!("{} \t\t= {} cycles <{}>", $msg, cycles, stringify!($fn));
        result
    }};
}

unsafe extern "C" {
    fn PUT32(addr: u32, val: u32);
}


fn cycles_per_second() -> u32 {
    let cycle_start = cycle_cnt_read();

    let s = timer_get_usec();
    while (timer_get_usec() - s) < 1000_000 {}

    let cycle_end = cycle_cnt_read();
    cycle_end - cycle_start
}

fn measure_put32(ptr: *mut u32) -> u32 {
    unsafe {
        asm!(".align 5",  options(nomem, nostack, preserves_flags));
        let s = cycle_cnt_read();
        PUT32(ptr as u32, 0);
        let e = cycle_cnt_read();
        e - s
    }
}

fn measure_get32(ptr: *const u32) -> u32 {
    unsafe {
        asm!(".align 5", options(nomem, nostack, preserves_flags));
        let s = cycle_cnt_read();
        ptr.read_volatile();
        let e = cycle_cnt_read();
        e - s
    }
}

fn measure_ptr_write(ptr: *mut u32) -> u32 {
    unsafe {
        asm!(".align 5", options(nomem, nostack, preserves_flags));
        let s = cycle_cnt_read();
        ptr.write_volatile(0);
        let e = cycle_cnt_read();
        e - s
    }
}

fn measure_ptr_read(ptr: *const u32) -> u32 {
    unsafe {
        asm!(".align 5", options(nomem, nostack, preserves_flags));
        let s = cycle_cnt_read();
        ptr.read_volatile();
        let e = cycle_cnt_read();
        e - s
    }
}

unsafe fn measure(msg: &str) {
    println!("--------------------------");
    println!("measuring: {}", msg);

    let x: u32 = 0;
    let set_0 = GPIO_REG::SET0.as_mut_ptr::<u32>();
    let level_0 = GPIO_REG::LEV0.as_ptr::<u32>();


    println!("call to put 32 \t= {} cycles", measure_put32(addr_of!(x) as *mut u32));
    println!("call to get 32 \t= {} cycles", measure_get32(addr_of!(x)));
    println!("ptr write \t= {} cycles", measure_ptr_write(addr_of!(x) as *mut u32));
    println!("ptr read \t= {} cycles", measure_ptr_read(addr_of!(x)));
    println!("periph write \t= {} cycles", measure_ptr_write(set_0));
    println!("periph read \t= {} cycles", measure_ptr_read(level_0));

    println!("--------------------------");
    asm!(".align 5", options(nomem, nostack, preserves_flags));

    time_cyc_print!("read/write barrier", dev_barrier());
    time_cyc_print!("read barrier", dmb());
    time_cyc_print!("safe timer", timer_get_usec());
    time_cyc_print!("unsafe timer", timer_get_usec_raw());
    time_cyc_print!("cycle cnt read", cycle_cnt_read());

}

#[unsafe(no_mangle)]
fn __user_main() {
    println!("Cycles per second = {}", cycles_per_second());
    unsafe { measure("700Mhz") } ;

    rpi_clock_hz_set(RpiClockType::CPU, 1000*1000*1000);

    println!("Cycles per second now = {}", cycles_per_second());
    unsafe {
        measure("1GHz");
    }

    caches_enable();
    unsafe {
        measure("1Ghz + Cache: first run");
    }
    unsafe {
        measure("1Ghz + Cache: second run");
    }
}
