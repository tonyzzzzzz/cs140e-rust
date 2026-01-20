use core::pin::Pin;
use crab_pi::println;

const GPIO_BASE: *mut u32 = ::core::ptr::with_exposed_provenance_mut(0x2020_0000);
const GPIO_SET0_OFFSET: u32 = 0x1c;
const GPIO_CLR0_OFFSET: u32 = 0x28;
const GPIO_LEV0_OFFSET: u32 = 0x34;

const GPIO_MAX_PIN: u32 = 53;

pub fn gpio_set_output(pin: u32) {
    if (pin > GPIO_MAX_PIN) {
        panic!("Invalid GPIO pin number");
    }

    let fsel_reg_num = pin / 10;
    let fsel_offset = 3 * (pin % 10);

    let fsel_reg = unsafe {
        GPIO_BASE.byte_offset((fsel_reg_num * 4) as isize)
    };

    unsafe {
        let mut cur_val = *fsel_reg;
        cur_val &= !(7 << fsel_offset);
        cur_val |= 1 << fsel_offset;
        *fsel_reg = cur_val;
    }
}

pub fn gpio_set_input(pin: u32) {
    if (pin > GPIO_MAX_PIN) {
        panic!("Invalid GPIO pin number");
    }

    let fsel_reg_num = pin / 10;
    let fsel_offset = 3 * (pin % 10);

    let fsel_reg = unsafe {
        GPIO_BASE.byte_offset((fsel_reg_num * 4) as isize)
    };

    unsafe {
        let mut cur_val = *fsel_reg;
        cur_val &= !(7 << fsel_offset);
        *fsel_reg = cur_val;
    }
}

pub fn gpio_set_on(mut pin: u32) {
    if (pin > GPIO_MAX_PIN) {
        panic!("Invalid GPIO pin number");
    }

    let mut gpio_set = unsafe {
        GPIO_BASE.byte_offset(GPIO_SET0_OFFSET as isize)
    };

    if (pin >= 32) {
        gpio_set = unsafe {
            gpio_set.byte_offset(4)
        };

        pin -= 32;
    }

    unsafe {
        *gpio_set = 1 << pin;
    }
}

pub fn gpio_set_off(mut pin: u32) {
    if (pin > GPIO_MAX_PIN) {
        panic!("Invalid GPIO pin number");
    }

    let mut gpio_clr = unsafe {
        GPIO_BASE.byte_offset(GPIO_CLR0_OFFSET as isize)
    };

    if (pin >= 32) {
        gpio_clr = unsafe {
            gpio_clr.byte_offset(4)
        };

        pin -= 32;
    }

    unsafe {
        *gpio_clr = 1 << pin;
    }
}

pub fn gpio_write(pin: u32, value: bool) {
    if (value) {
        gpio_set_on(pin);
    } else {
        gpio_set_off(pin);
    }
}

pub fn gpio_read(mut pin: u32) -> bool {
    if (pin > GPIO_MAX_PIN) {
        panic!("Invalid GPIO pin number");
    }

    let mut gpio_lev = unsafe {
        GPIO_BASE.byte_offset(GPIO_LEV0_OFFSET as isize)
    };

    if (pin >= 32) {
        gpio_lev = unsafe {
            gpio_lev.byte_offset(4)
        };

        pin -= 32;
    }

    unsafe {
        let lev_val = *gpio_lev;
        return (lev_val & (1 << pin)) != 0;
    }
}