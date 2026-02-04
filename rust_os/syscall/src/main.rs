#![no_std]
#![no_main]

use crab_pi::interrupt::{cpsr_get, interrupt_init, SYS_MODE};
use core::arch::global_asm;
use crab_pi::println;

global_asm!(r#"
.global syscall_hello
syscall_hello:
    @ we are already at system level: running this will trash
    @ the lr, so we need to save it.
    push {{lr}}
    swi 1
    pop {{lr}}
    bx lr

.global syscall_illegal
syscall_illegal:
    @ your code should reject this call with an error.
    push {{lr}}
    swi 2
    pop {{lr}}
    bx lr

.global run_user_code
run_user_code:
    @ 1. switch to user mode
    cps {USER_MODE}

    @ 2. set sp
    mov sp, r1

    @ jump to address
    blx r0
"#, USER_MODE = const SYS_MODE::USER.val());

unsafe extern "C" {
    fn syscall_hello();
    fn syscall_illegal();
    fn run_user_code(f: extern "C" fn(), sp: *mut u32);
}

static N: usize = 1024 * 64;
static USER_STACK: [u128; N / 16usize] = [0; N / 16usize];

extern "C" fn user_fn() {
    println!("Hello from user mode!");

    println!("Checking that stack got switched");
    let var: u128 = 0;
    unsafe {
        assert!(&var as *const _ >= &USER_STACK as *const _);
        assert!(&var as *const _ < USER_STACK.as_ptr().add(N) as *const _);
    }

    let current_mode = cpsr_get();

    if current_mode != SYS_MODE::USER.into() {
        panic!("syscall in non-user mode: mode={:b}", current_mode);
    }

    println!("cpsr is at user level");

    println!("Calling syscall_hello from user mode");
    unsafe {syscall_hello()};

    println!("Calling syscall_illegal from user mode");
    unsafe {syscall_illegal()};

    unreachable!()
}

#[unsafe(no_mangle)]
fn __user_main() {
    unsafe {
        interrupt_init();

        println!("Calling user_fn with stack={:p}", USER_STACK.as_ptr().add(N));

        run_user_code(user_fn, USER_STACK.as_ptr().add(N) as *const _ as *mut u32);

        unreachable!();
    }

}
