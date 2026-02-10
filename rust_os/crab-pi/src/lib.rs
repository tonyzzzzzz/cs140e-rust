#![feature(c_variadic)]
#![feature(allocator_api)]
#![feature(sync_unsafe_cell)]
#![feature(generic_const_exprs)]
#![no_std]
#![no_main]
extern crate alloc;

#[cfg(feature = "start-asm")]
mod start;

mod arch;
pub mod cache;
mod constant;
pub mod cycle_count;
pub mod gpio;
pub mod interrupt;
pub mod kmalloc;
pub mod libpi;
mod llvm_infra;
pub mod memory;
mod panic_infra;
pub mod print;
pub mod thread;
pub mod timer;
pub mod uart;
pub mod vector_base;
pub mod watchdog;
pub mod mailbox;
