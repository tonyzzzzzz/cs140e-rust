#![allow(static_mut_refs)] // TODO: better ways

use crate::println;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use core::arch::global_asm;
use core::ptr::null;

type RPIThreadExecFn = extern "C" fn(*const u32);

const THREAD_MAX_STACKSIZE: usize = (1024 * 8 / 4);

static mut THREAD_ID_COUNTER: usize = 1;

pub static mut RUN_Q: VecDeque<Box<RPIThread>> = VecDeque::new();
static mut FREE_Q: VecDeque<Box<RPIThread>> = VecDeque::new();

static mut CUR_THREAD: Option<Box<RPIThread>> = None;
static mut SCHEDULER_THREAD: Option<Box<RPIThread>> = None;

global_asm!(include_str!("../asm/rpi-thread-asm.S"));

unsafe extern "C" {
    fn rpi_init_trampoline();
    fn rpi_cswitch(old_sp_save: *mut *const u32, new_sp: *const u32);
    fn rpi_get_sp() -> *const u32;
}

#[repr(align(8))]
#[derive(Clone)]
struct Align8<T>(pub T);

// #[derive(Clone)]
pub struct RPIThread {
    saved_sp: *const u32,

    pub thread_id: usize,

    annot: String,

    stack: Align8<[u32; THREAD_MAX_STACKSIZE]>,
}

pub fn rpi_thread_start() {
    unsafe {
        if RUN_Q.is_empty() {
            println!("no thread to run");
            return;
        }

        // Initialize scheduler thread if needed
        if SCHEDULER_THREAD.is_none() {
            let sched_thread = RPIThread {
                saved_sp: null(),
                thread_id: 0,
                annot: "scheduler".to_string(),
                stack: Align8([u32::MAX; THREAD_MAX_STACKSIZE]), // Scheduler thread does not need a stack
            };

            SCHEDULER_THREAD = Some(Box::new(sched_thread));
        }

        println!("&RUN_Q={:p}", core::ptr::addr_of!(RUN_Q));
        println!("RUN_Q size: {}", RUN_Q.len());

        let next_thread = RUN_Q.pop_front().unwrap();
        println!("next_thread");
        let sched = SCHEDULER_THREAD.as_mut().unwrap();
        let sched_saved_sp_addr: *mut *const u32 = &mut sched.saved_sp;

        println!("Scheduler stack pointer: {:p}", rpi_get_sp());

        let next_thread_sp = next_thread.saved_sp;
        CUR_THREAD = Some(next_thread);
        rpi_cswitch(sched_saved_sp_addr, next_thread_sp)
    }
}

pub fn rpi_fork(f: RPIThreadExecFn, arg: *const u32) {
    println!("\n\nFORKING....");
    let mut new_thread = Box::new(RPIThread {
        saved_sp: null(),
        thread_id: unsafe { THREAD_ID_COUNTER },
        annot: "".to_string(),
        stack: Align8([u32::MAX; THREAD_MAX_STACKSIZE]),
    });

    unsafe {
        THREAD_ID_COUNTER += 1;
        let mut sp_now = new_thread.stack.0.as_mut_ptr().add(THREAD_MAX_STACKSIZE);

        // Save the trampoline routine to LR
        sp_now = sp_now.sub(1);
        println!("sp_now = {:p}", sp_now);
        sp_now.write_volatile(rpi_init_trampoline as *const () as u32);

        // Move code to r4, arg to r5
        sp_now = sp_now.sub(7);
        println!("sp_now = {:p}", sp_now);
        sp_now.write_volatile(arg as u32);

        sp_now = sp_now.sub(1);
        println!("sp_now = {:p}", sp_now);
        sp_now.write_volatile(f as u32);

        new_thread.as_mut().saved_sp = sp_now;
        println!(
            "rpi_fork: tid={}, code={:p}, arg={:p}, saved_sp={:p}",
            new_thread.thread_id, f, arg, new_thread.saved_sp
        );

        println!("&RUN_Q={:p}", core::ptr::addr_of!(RUN_Q));
        RUN_Q.push_back(new_thread);

        println!("sp[1] = {:x}", sp_now.read_volatile());
        println!("sp[2] = {:x}", sp_now.add(1).read_volatile());
        println!("lr = {:x}", sp_now.add(9).read_volatile());
        println!(
            "lr = {:x}, trampoline = {:p}",
            sp_now.add(9).read_volatile(),
            rpi_init_trampoline as *const ()
        );
    }
}

pub fn rpi_cur_thread_id() -> usize {
    unsafe { CUR_THREAD.as_ref().unwrap().thread_id }
}

// pub fn rpi_cur_thread() -> Box<RPIThread> {
//     unsafe { CUR_THREAD.unwrap() }
// }

#[unsafe(no_mangle)]
pub extern "C" fn rpi_exit(exit_code: i32) {
    unsafe {
        println!("Thread exiting with code {}", exit_code);

        let previous_thread = CUR_THREAD.as_mut().unwrap();
        let previous_thread_sp = &mut previous_thread.saved_sp;

        let next_thread_sp = match RUN_Q.pop_front() {
            Some(x) => {
                let next_sp = x.saved_sp.clone();
                CUR_THREAD = Some(x);
                next_sp
            }
            None => SCHEDULER_THREAD.as_mut().unwrap().saved_sp.clone(),
        };

        rpi_cswitch(previous_thread_sp, next_thread_sp)
    }
}

pub fn rpi_yield() {
    unsafe {
        if RUN_Q.is_empty() {
            return;
        }

        let mut previous_thread = CUR_THREAD.take().unwrap();
        let previous_thread_sp = &raw mut previous_thread.saved_sp;

        RUN_Q.push_back(previous_thread);

        let next_thread = RUN_Q.pop_front().unwrap();
        let next_thread_sp = next_thread.saved_sp;
        CUR_THREAD = Some(next_thread);

        rpi_cswitch(previous_thread_sp, next_thread_sp)
    }
}
