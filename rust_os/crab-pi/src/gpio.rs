use crate::interrupt::{IRQ_REG, IrqHandler, register_irq_2_handler};
use crate::memory::dev_barrier;
use crate::println;
use core::cell::SyncUnsafeCell;
use macros::{enum_ptr, enum_u32};

const GPIO_BASE_ADDR: u32 = 0x2020_0000;

enum_ptr! {
    pub enum GPIO_REG {
        FSEL0 = GPIO_BASE_ADDR,
        FSEL1 = GPIO_BASE_ADDR + 0x04,
        FSEL2 = GPIO_BASE_ADDR + 0x08,
        FSEL3 = GPIO_BASE_ADDR + 0x0c,
        FSEL4 = GPIO_BASE_ADDR + 0x10,
        FSEL5 = GPIO_BASE_ADDR + 0x14,
        SET0 = GPIO_BASE_ADDR + 0x1c,
        SET1 = GPIO_BASE_ADDR + 0x20,
        CLR0 = GPIO_BASE_ADDR + 0x28,
        CLR1 = GPIO_BASE_ADDR + 0x2c,
        LEV0 = GPIO_BASE_ADDR + 0x34,
        LEV1 = GPIO_BASE_ADDR + 0x38,
        EDS0 = GPIO_BASE_ADDR + 0x40,
        EDS1 = GPIO_BASE_ADDR + 0x44,
        REN0 = GPIO_BASE_ADDR + 0x4c,
        REN1 = GPIO_BASE_ADDR + 0x50,
        FEN0 = GPIO_BASE_ADDR + 0x58,
        FEN1 = GPIO_BASE_ADDR + 0x5c,
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
        GPIO_REG::FSEL0
            .as_mut_ptr::<u32>()
            .byte_offset((fsel_reg_num * 4) as isize)
    };

    unsafe {
        let mut cur_val = fsel_reg.read_volatile();
        cur_val &= !(7 << fsel_offset);
        cur_val |= func.val() << fsel_offset;
        fsel_reg.write_volatile(cur_val);
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

    let gpio_set = if (pin >= 32) {
        pin -= 32;
        GPIO_REG::SET1
    } else {
        GPIO_REG::SET0
    }
    .as_mut_ptr::<u32>();

    unsafe {
        gpio_set.write_volatile(1 << pin);
    }
}

pub fn gpio_set_off(mut pin: u32) {
    if (pin > GPIO_MAX_PIN) {
        panic!("Invalid GPIO pin number");
    }

    let gpio_clr = if (pin >= 32) {
        pin -= 32;
        GPIO_REG::CLR1
    } else {
        GPIO_REG::CLR0
    }
    .as_mut_ptr::<u32>();

    unsafe {
        gpio_clr.write_volatile(1 << pin);
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

    let gpio_lev = if (pin >= 32) {
        pin -= 32;
        GPIO_REG::LEV1
    } else {
        GPIO_REG::LEV0
    }
    .as_mut_ptr::<u32>();

    unsafe {
        let lev_val = gpio_lev.read_volatile();
        lev_val & (1 << pin) != 0
    }
}

const GPIO0_IRQ_MASK: u32 = 0x1 << 17;

type GPIOHandlerFn = fn(u32, GPIOEvent);
#[derive(Debug)]
pub enum GPIOEvent {
    RisingEdge,
    FallingEdge,
}
static GPIO_INT_HANDLER: SyncUnsafeCell<[fn(u32, GPIOEvent); 32]> =
    SyncUnsafeCell::new([default_gpio_handler; 32]);

pub fn gpio_has_interrupt() -> bool {
    dev_barrier();
    let has_interrupt = unsafe { IRQ_REG::PENDING_2.as_ptr::<u32>().read_volatile() };
    dev_barrier();

    has_interrupt & GPIO0_IRQ_MASK != 0
}

pub fn gpio_int_rising_edge(pin: u32) {
    if pin >= 32 {
        panic!("Invalid GPIO pin number");
    }

    dev_barrier();
    let gpio_ren_reg = GPIO_REG::REN0.as_mut_ptr::<u32>();

    unsafe {
        gpio_ren_reg.write_volatile(gpio_ren_reg.read_volatile() | (1 << pin));
    }

    dev_barrier();
}

pub fn gpio_int_falling_edge(pin: u32) {
    if pin >= 32 {
        panic!("Invalid GPIO pin number");
    }

    dev_barrier();
    let gpio_fen_reg = GPIO_REG::FEN0.as_mut_ptr::<u32>();

    unsafe {
        gpio_fen_reg.write_volatile(gpio_fen_reg.read_volatile() | (1 << pin));
    }

    dev_barrier();
}

pub fn gpio_event_detected(pin: u32) -> bool {
    if pin >= 32 {
        panic!("Invalid GPIO pin number");
    }

    dev_barrier();

    let gpio_eds_reg = GPIO_REG::EDS0.as_ptr::<u32>();

    let result = unsafe {
        let eds_val = gpio_eds_reg.read_volatile();
        eds_val & (1 << pin) != 0
    };

    dev_barrier();

    result
}

pub fn gpio_event_clear(pin: u32) {
    if pin >= 32 {
        panic!("Invalid GPIO pin number");
    }

    dev_barrier();
    let gpio_eds_reg = GPIO_REG::EDS0.as_mut_ptr::<u32>();

    unsafe {
        gpio_eds_reg.write_volatile(1 << pin);
    }
    dev_barrier();
}

pub fn gpio_interrupt_init() {
    unsafe { register_irq_2_handler(17, gpio_irq_handler) }
}

pub fn gpio_interrupt_enable() {
    dev_barrier();
    unsafe {
        IRQ_REG::ENABLE_2
            .as_mut_ptr::<u32>()
            .write_volatile(GPIO0_IRQ_MASK);
    }
    dev_barrier();
}

pub fn gpio_interrupt_disable() {
    dev_barrier();
    unsafe {
        IRQ_REG::DISABLE_2
            .as_mut_ptr::<u32>()
            .write_volatile(GPIO0_IRQ_MASK);
    }
    dev_barrier();
}

fn gpio_irq_handler(pc: u32) {
    let mut eds_val = unsafe { GPIO_REG::EDS0.as_ptr::<u32>().read_volatile() };

    let handlers = unsafe { &*GPIO_INT_HANDLER.get() };

    while eds_val != 0 {
        let i = eds_val.trailing_zeros();
        let event = if gpio_read(i as u32) {
            GPIOEvent::RisingEdge
        } else {
            GPIOEvent::FallingEdge
        };

        handlers[i as usize](i as u32, event);

        eds_val &= !(1 << i);

        gpio_event_clear(i as u32);
    }

    // for (i, handler) in handlers.iter().enumerate() {
    //     if eds_val & (1 << i) != 0 {
    //         let event = if gpio_read(i as u32) {
    //             GPIOEvent::RisingEdge
    //         } else {
    //             GPIOEvent::FallingEdge
    //         };
    //
    //         handler(i as u32, event);
    //
    //         gpio_event_clear(i as u32);
    //     }
    // }
}

pub fn gpio_register_interrupt_handler(pin: u32, handler: GPIOHandlerFn) {
    if pin >= 32 {
        panic!("Invalid GPIO pin number");
    }

    let handlers = unsafe { &mut *GPIO_INT_HANDLER.get() };
    handlers[pin as usize] = handler;
}

fn default_gpio_handler(pin: u32, event: GPIOEvent) {
    println!("Unhandled GPIO Event: {:?} at pin: {}", event, pin);
}
