#![no_std]
#![no_main]
mod sw_uart;

use crab_pi::{println, uart};
use crab_pi::uart::{disable_uart, enable_uart};
use crate::sw_uart::SwUart;

#[unsafe(no_mangle)]
fn __user_main() {
    disable_uart();

    let uart = SwUart::new(14, 15, 115200);
    uart.put_8(b'H');
    uart.put_8(b'e');
    uart.put_8(b'l');
    uart.put_8(b'l');
    uart.put_8(b'o');
    uart.put_8(b'\n');
    uart.put_8(b'\n');
    uart.put_8(b'\n');

    unsafe {uart::init(115200)}

    println!("Done!");
}