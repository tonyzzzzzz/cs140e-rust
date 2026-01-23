use core::arch::global_asm;
use core::ptr::with_exposed_provenance_mut;
use crate::constant::INT_STACK_ADDR;
use crate::memory::{dev_barrier, gcc_mb};
use crate::println;

/*
    POINTER DEFINITIONS
 */
const IRQ_BASE: usize = 0x2000_b200;
const IRQ_BASIC_PENDING: *mut u32 = with_exposed_provenance_mut(IRQ_BASE + 0x00);
const IRQ_PENDING_1: *mut u32 = with_exposed_provenance_mut(IRQ_BASE + 0x04);
const IRQ_PENDING_2: *mut u32 = with_exposed_provenance_mut(IRQ_BASE + 0x08);
const IRQ_FIQ_CONTROL: *mut u32 = with_exposed_provenance_mut(IRQ_BASE + 0x0c);
const IRQ_ENABLE_1: *mut u32 = with_exposed_provenance_mut(IRQ_BASE + 0x10);
const IRQ_ENABLE_2: *mut u32 = with_exposed_provenance_mut(IRQ_BASE + 0x14);
pub const IRQ_ENABLE_BASIC: *mut u32 = with_exposed_provenance_mut(IRQ_BASE + 0x18);
const IRQ_DISABLE_1: *mut u32 = with_exposed_provenance_mut(IRQ_BASE + 0x1c);
const IRQ_DISABLE_2: *mut u32 = with_exposed_provenance_mut(IRQ_BASE + 0x20);
const IRQ_DISABLE_BASIC: *mut u32 = with_exposed_provenance_mut(IRQ_BASE + 0x24);

/*
    REGISTRY
 */
pub type IrqHandler = fn();
static mut IRQ_BASIC_HANDLERS: [Option<IrqHandler>; 32] = [None; 32];


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
    safe static _interrupt_table_end: [u32; 0];
}

extern "C" fn fast_interrupt_vector(pc: u32) {
    panic!("not implemented yet");
}

unsafe extern "C" fn interrupt_vector(pc: u32) {
    dev_barrier();

    let pending = *IRQ_BASIC_PENDING;

    for i in 0..32 {
        if pending & (1 << i) != 0 {
            if let Some(handler) = IRQ_BASIC_HANDLERS[i] {
                handler();
            }
        }
    }

    dev_barrier();
}

extern "C" fn reset_vector(pc: u32) {
    panic!("not implemented yet");
}

extern "C" fn undefined_instruction_vector(pc: u32) {
    panic!("undefined instruction");
}

extern "C" fn syscall_vector(pc: u32) {
    panic!("syscall");
}

extern "C" fn prefetch_abort_vector(pc: u32) {
    panic!("prefetch abort");
}

extern "C" fn data_abort_vector(pc: u32) {
    panic!("data abort");
}

pub unsafe fn interrupt_init() {
    println!("About to install interrupt handler");

    disable_interrupts();

    *IRQ_DISABLE_1 = 0xffffffff;
    *IRQ_DISABLE_2 = 0xffffffff;

    dev_barrier();

    let interrupt_table =  &_interrupt_table as *const u32;
    let interrupt_table_end = &_interrupt_table_end as *const u32;

    println!("Installed interrupt handler, interrupt table {:p}, end {:p}", interrupt_table, interrupt_table_end);

    let src: *const u32 = interrupt_table;
    let dst: *mut u32 = with_exposed_provenance_mut(0);
    let n = interrupt_table_end as usize - interrupt_table as usize;

    gcc_mb();

    for i in 0..n {
        unsafe { dst.add(i).write_volatile(src.add(i).read_volatile()) };
    }
    gcc_mb();
}

pub unsafe fn register_irq_basic_handler(irq: usize, handler: IrqHandler) {
    println!("Registered handler for IRQ {}", irq);
    IRQ_BASIC_HANDLERS[irq] = Some(handler);
}