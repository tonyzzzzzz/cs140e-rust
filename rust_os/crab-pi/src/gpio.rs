use macros::{enum_ptr, enum_u32};

const GPIO_BASE_ADDR: u32 = 0x2020_0000;
const GPIO_SET0_OFFSET: u32 = 0x1c;
const GPIO_CLR0_OFFSET: u32 = 0x28;
const GPIO_LEV0_OFFSET: u32 = 0x34;

enum_ptr! {
    pub enum GPIO_REG {
        GPIO_BASE = GPIO_BASE_ADDR,
        GPIO_SET0 = GPIO_BASE_ADDR + GPIO_SET0_OFFSET,
        GPIO_CLR0 = GPIO_BASE_ADDR + GPIO_CLR0_OFFSET,
        GPIO_LEV0 = GPIO_BASE_ADDR + GPIO_LEV0_OFFSET,
    }
}

enum_u32! {
    pub enum GPIO_FUNC {
        INPUT = 0,
        OUTPUT = 1,
        ALT_0 = 0b100,
        ALT_1 = 0b101,
        ALT_2 = 0b110,
        ALT_3 = 0b111,
        ALT_4 = 0b011,
        ALT_5 = 0b010,
    }
}

const GPIO_MAX_PIN: u32 = 53;

pub fn gpio_set_function(pin: u32, func: GPIO_FUNC) {
    if (pin > GPIO_MAX_PIN) {
        panic!("Invalid GPIO pin number");
    }

    let fsel_reg_num = pin / 10;
    let fsel_offset = 3 * (pin % 10);

    let fsel_reg = unsafe {
        GPIO_REG::GPIO_BASE.as_mut_ptr::<u32>().byte_offset((fsel_reg_num * 4) as isize)
    };

    unsafe {
        let mut cur_val = *fsel_reg;
        cur_val &= !(7 << fsel_offset);
        cur_val |= func.val() << fsel_offset;
        *fsel_reg = cur_val;
    }
}

pub fn gpio_set_output(pin: u32) {
    gpio_set_function(pin, GPIO_FUNC::OUTPUT);
}

pub fn gpio_set_input(pin: u32) {
    gpio_set_function(pin, GPIO_FUNC::INPUT);
}

pub fn gpio_set_on(mut pin: u32) {
    if (pin > GPIO_MAX_PIN) {
        panic!("Invalid GPIO pin number");
    }

    let mut gpio_set = GPIO_REG::GPIO_SET0.as_mut_ptr::<u32>();

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

    let mut gpio_clr = GPIO_REG::GPIO_CLR0.as_mut_ptr::<u32>();

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

    let mut gpio_lev = GPIO_REG::GPIO_LEV0.as_mut_ptr::<u32>();

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