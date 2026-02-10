#![no_std]
#![no_main]
pub mod sw_uart;

use constants::UART_BAUD_RATE;
use crate::sw_uart::SwUart;
use crab_pi::uart::{disable_uart, enable_uart};
use crab_pi::{println, uart};

#[unsafe(no_mangle)]
fn __user_main() {
    disable_uart();

    let uart = SwUart::new(14, 15, UART_BAUD_RATE);
    uart.put_8(b'H');
    uart.put_8(b'e');
    uart.put_8(b'l');
    uart.put_8(b'l');
    uart.put_8(b'o');
    uart.put_8(b'\n');
    uart.put_8(b'\n');
    uart.put_8(b'\n');

    unsafe { uart::init(UART_BAUD_RATE) }

    println!("Done!");
}
