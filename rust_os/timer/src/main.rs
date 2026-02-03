#![no_std]
#![no_main]

use core::ptr::{addr_of, with_exposed_provenance_mut};
use crab_pi::interrupt::{enable_interrupts, interrupt_init, register_irq_basic_handler, IRQ_ENABLE_BASIC};
use crab_pi::memory::dev_barrier;
use crab_pi::{print, println};
use crab_pi::timer::{clear_irq, timer_get_usec, timer_init};

static mut cnt: u32 = 0;
static mut period: u32 = 0;
static mut period_sum: u32 = 0;
static mut last_clk: u32 = 0;

fn timer_interrupt_handler(pc: u32) {
    unsafe {
        clear_irq();

        dev_barrier();

        cnt += 1;
        let clk = timer_get_usec();
        period = if last_clk == 0 { 0 } else { clk - last_clk } ;
        last_clk = clk;
        period_sum += period;

        dev_barrier();
    }
}



#[unsafe(no_mangle)]
fn __user_main() {
    println!("Hello, world!");
    unsafe {
        interrupt_init();
        // Timer handler is at index 0, which is equal to 1 << 0.
        register_irq_basic_handler(0, timer_interrupt_handler);

        timer_init(16, 0x100);

        println!("enabling global ints");
        enable_interrupts();
        println!("Enabled");

        let start = timer_get_usec();

        let mut iter = 0;
        while (cnt < 20) {
            let cnt_local = addr_of!(cnt).read_volatile();
            let period_local = addr_of!(period).read_volatile();
            let sum_local = addr_of!(period_sum).read_volatile();
            println!("iter: {}, cnt: {}, period: {}, sum: {}", iter, cnt_local, period_local, sum_local);
            iter += 1;
        }

        let end = timer_get_usec();

        let tot = end - start;
        let tot_sec = tot / 1_000_000;
        let tot_ms = (tot / 1000) % 1000;
        let tot_usec = (tot % 1000);

        print!("-----------------------------------------\n");
        print!("summary:\n");
        print!("\t{}: total iterations\n", iter);
        print!("\t{}: tot interrupts\n", 20);
        print!("\t{}: iterations / interrupt\n", iter/20);
        print!("\t{}: average period\n\n", period_sum/(20-1));
        print!("total execution time: {}sec.{}ms.{}usec\n",
               tot_sec, tot_ms, tot_usec);
    }
}
