use core::ptr::with_exposed_provenance;
use crate::{enum_ptr, enum_u32, println};
use crate::interrupt::IRQ_ENABLE_BASIC;
use crate::memory::dev_barrier;
use crate::timer::ARM_TIMER::ARM_TIMER_CONTROL;

const ARM_TIMER_BASE: u32 = 0x2000_B400;
const ARM_TIMER_IRQ: usize = 1 << 0;
const ARM_TIMER_CURRENT: *const u32 = with_exposed_provenance(0x2000_3004);

enum_ptr! {
    pub enum ARM_TIMER {
        ARM_TIMER_LOAD = ARM_TIMER_BASE + 0x0,
        ARM_TIMER_VALUE = ARM_TIMER_BASE + 0x4,
        ARM_TIMER_CONTROL = ARM_TIMER_BASE + 0x8,
        ARM_TIMER_IRQ_CLEAR = ARM_TIMER_BASE + 0x0C,
        ARM_TIMER_IRQ_RAW = ARM_TIMER_BASE + 0x10,
        ARM_TIMER_IRQ_MASKED = ARM_TIMER_BASE + 0x14,
        ARM_TIMER_RELOAD = ARM_TIMER_BASE + 0x18,
        ARM_TIMER_PREDIV = ARM_TIMER_BASE + 0x1C,
        ARM_TIMER_COUNTER = ARM_TIMER_BASE + 0x20,
    }
}

enum_u32! {
    pub enum ARM_TIMER_CTRL {
        ARM_TIMER_CTRL_ENABLE = (1 << 7),
        ARM_TIMER_CTRL_INT_ENABLE = (1 << 5),
        ARM_TIMER_CTRL_32BIT = (1 << 1),
        ARM_TIMER_CTRL_PRESCALE_1 = (0 << 2),
        ARM_TIMER_CTRL_PRESCALE_16 = (1 << 2),
        ARM_TIMER_CTRL_PRESCALE_256 = (2 << 2),
    }
}

unsafe fn timer_get_usec_raw() -> u32 {
    ARM_TIMER_CURRENT.read_volatile()
}

pub unsafe fn timer_get_usec() -> u32 {
    dev_barrier();
    let u = timer_get_usec_raw();
    dev_barrier();
    u
}

pub unsafe fn clear_irq() {
    ARM_TIMER::ARM_TIMER_IRQ_CLEAR.as_mut_ptr::<u32>().write_volatile(1);
}

pub unsafe fn timer_init(prescale: u32, ncycles: u32) {
    println!("timer init");

    dev_barrier();
    
    *IRQ_ENABLE_BASIC = ARM_TIMER_IRQ as u32;

    dev_barrier();

    ARM_TIMER::ARM_TIMER_LOAD.as_mut_ptr::<u32>().write_volatile(ncycles);

    let v = match prescale {
        1 => ARM_TIMER_CTRL::ARM_TIMER_CTRL_PRESCALE_1,
        16 => ARM_TIMER_CTRL::ARM_TIMER_CTRL_PRESCALE_16,
        256 => ARM_TIMER_CTRL::ARM_TIMER_CTRL_PRESCALE_256,
        _ => panic!("unsupported prescale"),
    };
    
    ARM_TIMER_CONTROL.as_mut_ptr::<u32>().write_volatile(ARM_TIMER_CTRL::ARM_TIMER_CTRL_32BIT | ARM_TIMER_CTRL::ARM_TIMER_CTRL_ENABLE | ARM_TIMER_CTRL::ARM_TIMER_CTRL_INT_ENABLE | v);

    dev_barrier();
}
