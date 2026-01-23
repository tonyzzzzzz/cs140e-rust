#![no_std]
#![no_main]

use core::ptr::{addr_of, with_exposed_provenance_mut};
use crab_pi::interrupt::{enable_interrupts, interrupt_init, register_irq_basic_handler, IRQ_ENABLE_BASIC};
use crab_pi::memory::dev_barrier;
use crab_pi::{print, println};

const ARM_TIMER_IRQ: usize = 1 << 0;

const ARM_TIMER_BASE: usize = 0x2000_B400;
const ARM_TIMER_LOAD: *mut u32 = with_exposed_provenance_mut(ARM_TIMER_BASE + 0x0);
const ARM_TIMER_VALUE: *mut u32 = with_exposed_provenance_mut(ARM_TIMER_BASE + 0x4);
const ARM_TIMER_CONTROL: *mut u32 = with_exposed_provenance_mut(ARM_TIMER_BASE + 0x8);
const ARM_TIMER_IRQ_CLEAR: *mut u32 = with_exposed_provenance_mut(ARM_TIMER_BASE + 0x0C);
const ARM_TIMER_IRQ_RAW: *mut u32 = with_exposed_provenance_mut(ARM_TIMER_BASE + 0x10);
const ARM_TIMER_IRQ_MASKED: *mut u32 = with_exposed_provenance_mut(ARM_TIMER_BASE + 0x14);
const ARM_TIMER_RELOAD: *mut u32 = with_exposed_provenance_mut(ARM_TIMER_BASE + 0x18);
const ARM_TIMER_PREDIV: *mut u32 = with_exposed_provenance_mut(ARM_TIMER_BASE + 0x1C);
const ARM_TIMER_COUNTER: *mut u32 = with_exposed_provenance_mut(ARM_TIMER_BASE + 0x20);

pub struct ArmTimerCtrl(u32);
impl ArmTimerCtrl {
    pub const ARM_TIMER_CTRL_32BIT        : u32 = ( 1 << 1 );
    pub const ARM_TIMER_CTRL_PRESCALE_1   : u32 = ( 0 << 2 );
    pub const ARM_TIMER_CTRL_PRESCALE_16  : u32 = ( 1 << 2 );
    pub const ARM_TIMER_CTRL_PRESCALE_256 : u32 = ( 2 << 2 );
    pub const ARM_TIMER_CTRL_INT_ENABLE   : u32 = ( 1 << 5 );
    pub const ARM_TIMER_CTRL_ENABLE       : u32 = ( 1 << 7 );
}

static mut cnt: u32 = 0;
static mut period: u32 = 0;
static mut period_sum: u32 = 0;
static mut last_clk: u32 = 0;

unsafe fn timer_get_usec_raw() -> u32 {
    return *with_exposed_provenance_mut(0x2000_3004);
}

unsafe fn timer_get_usec() -> u32 {
    dev_barrier();
    let u = timer_get_usec_raw();
    dev_barrier();
    u
}

unsafe fn timer_init(prescale: u32, ncycles: u32) {
    println!("timer init");

    dev_barrier();

    // Timer handler is at index 0, which is equal to 1 << 0.
    register_irq_basic_handler(0, timer_interrupt_handler);

    *IRQ_ENABLE_BASIC = ARM_TIMER_IRQ as u32;

    dev_barrier();

    *ARM_TIMER_LOAD = ncycles;

    let v = match prescale {
        1 => ArmTimerCtrl::ARM_TIMER_CTRL_PRESCALE_1,
        16 => ArmTimerCtrl::ARM_TIMER_CTRL_PRESCALE_16,
        256 => ArmTimerCtrl::ARM_TIMER_CTRL_PRESCALE_256,
        _ => panic!("unsupported prescale"),
    };

    *ARM_TIMER_CONTROL = ArmTimerCtrl::ARM_TIMER_CTRL_32BIT | ArmTimerCtrl::ARM_TIMER_CTRL_ENABLE | ArmTimerCtrl::ARM_TIMER_CTRL_INT_ENABLE | v;

    dev_barrier();
}

fn timer_interrupt_handler() {
    unsafe {
        *ARM_TIMER_IRQ_CLEAR = 1;

        dev_barrier();

        cnt += 1;
        let clk = timer_get_usec_raw();
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
