#![no_std]
#![no_main]

use macros::enum_u32;

pub const ARM_BASE: u32 = 0x8000;

enum_u32! {
    pub enum BOOT_OP {
        // the weird numbers are to try to help with debugging
        // when you drop a byte, flip them, corrupt one, etc.
        BOOT_START      = 0xFFFF0000,

        GET_PROG_INFO   = 0x11112222,       // pi sends
        PUT_PROG_INFO   = 0x33334444,       // unix sends

        GET_CODE        = 0x55556666,       // pi sends
        PUT_CODE        = 0x77778888,       // unix sends

        BOOT_SUCCESS    = 0x9999AAAA,       // pi sends on success
        BOOT_ERROR      = 0xBBBBCCCC,       // pi sends on failure.

        PRINT_STRING    = 0xDDDDEEEE,       // pi sends to print a string.
    }
}
