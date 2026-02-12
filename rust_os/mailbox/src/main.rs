#![no_std]
#![no_main]

use crab_pi::mailbox::RpiClockType;
use crab_pi::{mailbox, println};

#[unsafe(no_mangle)]
fn __user_main() {
    println!(
        "mailbox serial number = {:x}",
        mailbox::mbox_get_serial_num()
    );
    println!("mailbox model = {:x}", mailbox::mbox_get_model());
    println!(
        "mailbox board revision = {:x}",
        mailbox::mbox_get_revision()
    );
    println!(
        "mailbox board memory = {}MB",
        mailbox::mbox_get_memory() / 1000000
    );
    println!(
        "mailbox temperature = {}degrees",
        mailbox::mbox_get_temperature() / 1000
    );

    println!(
        "mailbox current hz = {}",
        mailbox::rpi_clock_current_hz_get(RpiClockType::CPU)
    );
    println!(
        "mailbox max hz = {}",
        mailbox::rpi_clock_max_hz_get(RpiClockType::CPU)
    );
    println!(
        "mailbox min hz = {}",
        mailbox::rpi_clock_min_hz_get(RpiClockType::CPU)
    );
}
