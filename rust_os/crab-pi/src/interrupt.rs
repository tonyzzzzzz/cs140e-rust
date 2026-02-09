use crate::constant::INT_STACK_ADDR;
use crate::memory::{dev_barrier, gcc_mb};
use crate::println;
use crate::watchdog::clean_reboot;
use core::arch::{asm, global_asm};
use core::ptr::{with_exposed_provenance, with_exposed_provenance_mut};
use log::trace;
use macros::{enum_ptr, enum_u32};
use crate::cycle_count::cycle_cnt_read;
use crate::vector_base::vector_base_reset;

enum_u32! {
    pub enum SYS_MODE {
        USER = 0b10000,
        FIQ = 0b10001,
        IRQ = 0b10010,
        SVC = 0b10011,
        ABT = 0b10111,
        UND = 0b11011,
        SYS = 0b11111,
    }
}

/*
   POINTER DEFINITIONS
*/
const IRQ_BASE: u32 = 0x2000_b200;

enum_ptr! {
    pub enum IRQ_REG {
        BASIC_PENDING = IRQ_BASE,
        PENDING_1 = IRQ_BASE + 0x04,
        PENDING_2 = IRQ_BASE + 0x08,
        FIQ_CONTROL = IRQ_BASE + 0x0c,
        ENABLE_1 = IRQ_BASE + 0x10,
        ENABLE_2 = IRQ_BASE + 0x14,
        ENABLE_BASIC = IRQ_BASE + 0x18,
        DISABLE_1 = IRQ_BASE + 0x1c,
        DISABLE_2 = IRQ_BASE + 0x20,
        DISABLE_BASIC = IRQ_BASE + 0x24,
    }
}

/*
   REGISTRY
*/
pub type IrqHandler = fn(u32);
static mut IRQ_BASIC_HANDLERS: [Option<IrqHandler>; 32] = [None; 32];
static mut IRQ_1_HANDLERS: [Option<IrqHandler>; 32] = [None; 32];
static mut IRQ_2_HANDLERS: [Option<IrqHandler>; 32] = [None; 32];

global_asm!(
    include_str!("../asm/interrupts-asm.S"),
    INT_STACK_ADDR = const INT_STACK_ADDR,
    fast_interrupt_vector = sym fast_interrupt_vector,
    interrupt_vector = sym interrupt_vector,
    reset_vector = sym reset_vector,
    undefined_instruction_vector = sym undefined_instruction_vector,
    syscall_vector = sym syscall_vector,
    prefetch_abort_vector = sym prefetch_abort_vector,
    data_abort_vector = sym data_abort_vector
);

unsafe extern "C" {
    pub safe fn enable_interrupts();
    pub safe fn disable_interrupts();

    safe static _interrupt_table: [u32; 0];
}

#[unsafe(no_mangle)]
extern "C" fn fast_interrupt_vector(pc: u32) {
    panic!("not implemented yet");
}

#[inline]
pub fn cpsr_get() -> u32 {
    let mut cpsr: u32;
    unsafe {
        asm!("mrs {}, cpsr", out(reg) cpsr);
    }
    cpsr & 0b11111
}

#[inline]
pub fn spsr_get() -> u32 {
    let mut spsr: u32;
    unsafe {
        asm!("mrs {}, spsr", out(reg) spsr);
    }
    spsr & 0b11111
}

#[unsafe(no_mangle)]
unsafe extern "C" fn interrupt_vector(pc: u32) {
    dev_barrier();

    // Check BASIC IRQ
    let mut pending = IRQ_REG::BASIC_PENDING.as_ptr::<u32>().read_volatile();
    while pending != 0 {
        let i = pending.trailing_zeros();
        if let Some(handler) = IRQ_BASIC_HANDLERS[i as usize] {
            handler(pc);
        }
        pending &= !(1 << i);
    }

    // Check IRQ1
    let mut pending_irq1 = IRQ_REG::PENDING_1.as_ptr::<u32>().read_volatile();
    while pending_irq1 != 0 {
        let i = pending_irq1.trailing_zeros();
        if let Some(handler) = IRQ_1_HANDLERS[i as usize] {
            handler(pc);
        }
        pending_irq1 &= !(1 << i);
    }

    // Check IRQ2
    let mut pending_irq2 = IRQ_REG::PENDING_2.as_ptr::<u32>().read_volatile();
    while pending_irq2 != 0 {
        let i = pending_irq2.trailing_zeros();
        if let Some(handler) = IRQ_2_HANDLERS[i as usize] {
            handler(pc);
        }
        pending_irq2 &= !(1 << i);
    }
    dev_barrier();
}

#[unsafe(no_mangle)]
extern "C" fn reset_vector(pc: u32) {
    panic!("not implemented yet");
}

#[unsafe(no_mangle)]
extern "C" fn undefined_instruction_vector(pc: u32) {
    panic!("undefined instruction");
}

#[unsafe(no_mangle)]
extern "C" fn syscall_vector(pc: usize, r0: usize) -> i32 {
    let instruction = unsafe { *with_exposed_provenance::<u32>(pc) };
    let sys_num = instruction & 0x00ffffff;

    println!("syscall: {:x}", sys_num);

    #[cfg(debug_assertions)]
    unsafe {
        let prev_mode = spsr_get();

        if (prev_mode != SYS_MODE::USER.into()) {
            panic!("syscall in non-user mode: {:x}", sys_num);
        }

        println!("success: spsr is at user level: mode={:b}", prev_mode);
    }

    match sys_num {
        1 => {
            println!("syscall: hello world");
            0
        }
        2 => {
            println!("syscall: exit");
            clean_reboot()
        }
        _ => {
            println!("syscall: unknown: {:x}", sys_num);
            -1
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn prefetch_abort_vector(pc: u32) {
    panic!("prefetch abort");
}

#[unsafe(no_mangle)]
extern "C" fn data_abort_vector(pc: u32) {
    panic!("data abort");
}

pub unsafe fn interrupt_init() {
    println!("About to install interrupt handler");

    disable_interrupts();

    IRQ_REG::DISABLE_1
        .as_mut_ptr::<u32>()
        .write_volatile(0xffffffff);
    IRQ_REG::DISABLE_2
        .as_mut_ptr::<u32>()
        .write_volatile(0xffffffff);

    dev_barrier();

    let interrupt_table = &_interrupt_table as *const u32;

    println!(
        "Installed interrupt handler, interrupt table {:p}",
        interrupt_table
    );

    vector_base_reset(interrupt_table);
}

pub unsafe fn register_irq_basic_handler(irq: usize, handler: IrqHandler) {
    println!("Registered handler for IRQ {}", irq);
    IRQ_BASIC_HANDLERS[irq] = Some(handler);
}

pub unsafe fn register_irq_1_handler(index: usize, handler: IrqHandler) {
    println!("Registered handler for IRQ 1 index: {}", index);
    IRQ_1_HANDLERS[index] = Some(handler);
}

pub unsafe fn register_irq_2_handler(index: usize, handler: IrqHandler) {
    println!("Registered handler for IRQ 2 index: {}", index);
    IRQ_2_HANDLERS[index] = Some(handler);
}
