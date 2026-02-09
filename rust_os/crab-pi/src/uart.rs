use crate::gpio::{GPIO_FUNC, gpio_set_function};
use crate::memory::{dev_barrier, dsb};
use crate::timer::timer_get_usec;
use core::time::Duration;
use macros::enum_ptr;

const AUX_BASE_ADDR: u32 = 0x2021_5000;

enum_ptr! {
    pub enum AUX_REG {
        AUX_IRQ = AUX_BASE_ADDR, /* size = 3 */
        AUX_ENABLES = AUX_BASE_ADDR + 0x4, /* size = 3 */
        AUX_MU_IO_REG = AUX_BASE_ADDR + 0x40, /* size = 8 */
        AUX_MU_IER_REG = AUX_BASE_ADDR + 0x44, /* size = 8 */
        AUX_MU_IIR_REG = AUX_BASE_ADDR + 0x48, /* size = 8 */
        AUX_MU_LCR_REG = AUX_BASE_ADDR + 0x4c, /* size = 8 */
        AUX_MU_MCR_REG = AUX_BASE_ADDR + 0x50, /* size = 8 */
        AUX_MU_CNTL_REG = AUX_BASE_ADDR + 0x60, /* size = 8 */
        AUX_MU_STAT_REG = AUX_BASE_ADDR + 0x64, /* size = 32 */
        AUX_MU_BAUD_REG = AUX_BASE_ADDR + 0x68, /* size = 16 */
    }
}

pub unsafe fn init(baud_rate: u32) {
    /*
    INITIALIZE GPIO PIN 14-15 to ALT5
    */
    dsb();
    gpio_set_function(14, GPIO_FUNC::ALT_5);
    gpio_set_function(15, GPIO_FUNC::ALT_5);
    dsb();

    /*
    Enable AUX Mini UART
     */
    let aux_reg = AUX_REG::AUX_ENABLES.as_mut_ptr::<u32>();
    let mut aux_value = aux_reg.read_volatile();
    aux_value |= 0x1;
    aux_reg.write_volatile(aux_value);
    dsb();

    /*
    Mini UART Extra Control = 0
     */
    AUX_REG::AUX_MU_CNTL_REG
        .as_mut_ptr::<u32>()
        .write_volatile(0);

    /*
    Mini UART Interrupt Identify[0x48] = 0x6
     */
    AUX_REG::AUX_MU_IIR_REG
        .as_mut_ptr::<u32>()
        .write_volatile(0x6);

    /*
    Mini UART Line Control[0x4c] = 0x3
     */
    AUX_REG::AUX_MU_LCR_REG
        .as_mut_ptr::<u32>()
        .write_volatile(0x3);

    /*
    Mini UART Modem Control[0x50] = 0
     */
    AUX_REG::AUX_MU_MCR_REG
        .as_mut_ptr::<u32>()
        .write_volatile(0);

    /*
    Mini UART Baud Rate[0x68] = baud_rate (hard coded to 0x10e)
     */
    AUX_REG::AUX_MU_BAUD_REG
        .as_mut_ptr::<u32>()
        .write_volatile(0x10e);

    /*
    Mini UART Interrupt Enable = 0
     */
    AUX_REG::AUX_MU_IER_REG
        .as_mut_ptr::<u32>()
        .write_volatile(0);

    /*
    Mini UART Extra Control = 3
     */
    AUX_REG::AUX_MU_CNTL_REG
        .as_mut_ptr::<u32>()
        .write_volatile(0x3);

    dsb();
}

pub fn disable_uart() {
    flush();
    dev_barrier();
    unsafe {
        let aux_reg = AUX_REG::AUX_ENABLES.as_mut_ptr::<u32>();
        *aux_reg &= !0x1;
    }
    dev_barrier();
}

pub fn enable_uart() {
    dev_barrier();
    unsafe {
        let aux_reg = AUX_REG::AUX_ENABLES.as_mut_ptr::<u32>();
        *aux_reg |= 0x1;
    }
    dev_barrier();
}

pub fn flush() {
    dsb();
    let stat_reg = AUX_REG::AUX_MU_STAT_REG.as_ptr::<u32>();
    unsafe { while (stat_reg.read_volatile() & 0x200 == 0x0) {} }
    dsb();
}

#[inline(always)]
pub fn can_write() -> bool {
    let state_reg = AUX_REG::AUX_MU_STAT_REG.as_ptr::<u32>();
    unsafe { (state_reg.read_volatile() & 0x2) != 0 }
}

#[inline(always)]
pub fn can_read() -> bool {
    let state_reg = AUX_REG::AUX_MU_STAT_REG.as_ptr::<u32>();
    unsafe { (state_reg.read_volatile() & 0x1) != 0 }
}

pub fn can_read_timeout(timeout: Duration) -> bool {
    let state_reg = AUX_REG::AUX_MU_STAT_REG.as_ptr::<u32>();
    unsafe {
        let time_now = timer_get_usec();
        while timer_get_usec() - time_now < timeout.as_micros() as u32 {
            if (state_reg.read_volatile() & 0x1) != 0 {
                return true;
            }
        }
    }
    false
}

// TODO: Implement STD IO
pub fn write_bytes(bytes: &[u8]) {
    dsb();
    let io_reg = AUX_REG::AUX_MU_IO_REG.as_mut_ptr::<u32>();
    unsafe {
        for byte in bytes {
            while !can_write() {}
            io_reg.write_volatile(*byte as u32);
        }
    }
    dsb();
}

pub fn read_bytes(bytes: &mut [u8]) {
    dsb();
    let io_reg = AUX_REG::AUX_MU_IO_REG.as_ptr::<u32>();
    unsafe {
        for byte in bytes {
            while !can_read() {}
            *byte = io_reg.read_volatile() as u8;
        }
    }
    dsb();
}
