#![no_std]
#![no_main]

extern crate crab_pi;

use core::arch::{asm, global_asm};
use core::ptr::with_exposed_provenance_mut;
use core::time::Duration;
use crab_pi::{println, uart, watchdog};
use constants::BOOT_OP;
use crab_pi::cycle_count::cycle_cnt_init;
use crab_pi::memory::gcc_mb;
use crab_pi::timer::sleep;
use crab_pi::uart::{can_read_timeout, read_bytes, write_bytes};

global_asm!(include_str!("../asm/boot.S"));

unsafe extern "C" {
    // We declare these as `[u32; 0]` so that they have an alignment of 4 but a size of zero. This
    // is to prevent aliasing, since otherwise producing mutable references to anything in the BSS
    // would be undefined behaviour.

    static __bss_start__: [u32; 0];
    static __bss_end__: [u32; 0];

    fn BRANCHTO(addr: u32) -> !;
}

unsafe fn zero_out_bss() {
    gcc_mb();
    let bss_start_ptr = &__bss_start__;
    let bss_end_ptr = &__bss_end__;
    let bss_size = bss_end_ptr as *const u32 as usize - bss_start_ptr as *const u32 as usize;
    let bss_start = bss_start_ptr as *const u32 as *mut u32;
    unsafe {
        core::ptr::write_bytes(bss_start, 0, bss_size / 4);
    }
    gcc_mb();
}

#[unsafe(no_mangle)]
extern "C" fn _cstart() {
    unsafe {
        zero_out_bss();

        uart::init(115200);

        cycle_cnt_init();

        main();
    }

    watchdog::clean_reboot();
}

#[unsafe(no_mangle)]
extern "C" fn rpi_reboot() {
    watchdog::clean_reboot();
}

pub fn wait_for_data(timeout: Option<Duration>) {
    let packet = BOOT_OP::GET_PROG_INFO.val().to_le_bytes();
    loop {
        uart::write_bytes(&packet);
        if can_read_timeout(Duration::from_millis(300)) {
            break;
        }
    }
}

fn debug_print(s: &str) {
    let s_bytes = s.as_bytes();

    let mut write_buf = [0u8; 4 + 4];
    write_buf[..4].copy_from_slice(&BOOT_OP::PRINT_STRING.val().to_le_bytes());
    write_buf[4..].copy_from_slice(&s_bytes.len().to_le_bytes());
    write_bytes(&write_buf);
    write_bytes(s_bytes);
}


unsafe fn main() {
    wait_for_data(None);

    // Got something to read, read and get the program info
    // [PUT_PROG_INFO, addr, nbytes, cksum] = 4x4 bytes
    let mut buf = [0u8; 16];
    read_bytes(&mut buf);

    let mut chunks = buf.chunks(4);
    let op = u32::from_le_bytes(chunks.next().unwrap().try_into().unwrap());
    let addr = u32::from_le_bytes(chunks.next().unwrap().try_into().unwrap());
    let nbytes = u32::from_le_bytes(chunks.next().unwrap().try_into().unwrap());
    let cksum = u32::from_le_bytes(chunks.next().unwrap().try_into().unwrap());
    assert_eq!(op, BOOT_OP::PUT_PROG_INFO.val());

    // Check Collision
    if addr + nbytes > 0x200000 {
        let write_buf = BOOT_OP::BOOT_ERROR.val().to_le_bytes();
        write_bytes(&write_buf);
        return;
    }

    // Echo back GET_CODE
    let mut write_buf = [0u8; 8];
    write_buf[..4].copy_from_slice(&BOOT_OP::GET_CODE.val().to_le_bytes());
    write_buf[4..].copy_from_slice(&cksum.to_le_bytes());
    write_bytes(&write_buf);

    // Expect [PUT_CODE, CODE]
    let mut op_buf = [0u8; 4];
    read_bytes(&mut op_buf);
    assert_eq!(u32::from_le_bytes(op_buf), BOOT_OP::PUT_CODE.val());

    let code_begin_ptr = with_exposed_provenance_mut::<u8>(addr as usize);
    let bytes_array_ptr = core::slice::from_raw_parts_mut(code_begin_ptr, nbytes as usize);
    read_bytes(bytes_array_ptr);

    let recv_crc32 = crc32fast::hash(bytes_array_ptr);

    if recv_crc32 != cksum {
        let write_buf = BOOT_OP::BOOT_ERROR.val().to_le_bytes();
        write_bytes(&write_buf);
        return;
    }

    // Return boot success
    let write_buf = BOOT_OP::BOOT_SUCCESS.val().to_le_bytes();
    write_bytes(&write_buf);

    // Add name
    let name = "Jiaye Zou: BootLoader starting.\n";
    write_bytes(name.as_bytes());

    // Flush
    uart::flush();

    BRANCHTO(addr);
}
