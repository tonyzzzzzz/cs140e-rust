use crab_pi::cycle_count::{cycle_cnt_read, wait_cycles, wait_until_cycle};
use crab_pi::gpio::{gpio_set_function, gpio_set_off, gpio_set_on, gpio_write, GPIO_FUNC};
use crab_pi::uart;

#[inline]
const fn baud_to_cycles(baud: u32) -> u32 {
    (700 * 1000 * 1000u32 ) / baud
}

#[inline]
const fn baud_to_usec(baud: u32) -> u32 {
    (1000 * 1000u32 ) / baud
}

pub struct SwUart {
    tx: u32,
    rx: u32,
    baud: u32,
    cycles_per_bit: u32,
    usec_per_bit: u32,
}

impl SwUart {
    pub fn new(tx: u32, rx: u32, baud: u32) -> Self {
       Self::new_impl(tx, rx, baud, baud_to_cycles(baud), baud_to_usec(baud))
    }

    fn new_impl(tx: u32, rx: u32, baud: u32, cycles_per_bit: u32, usec_per_bit: u32) -> Self {
        // Check sanity
        let mhz: u32 = 700 * 1000 * 1000;
        let derived = cycles_per_bit * baud;
        if !(mhz - baud) <= derived || !derived <= (mhz + baud) {
            panic!("Invalid baud rate");
        }

        // Pull high
        gpio_set_on(tx);

        gpio_set_function(tx, GPIO_FUNC::OUTPUT);
        gpio_set_function(rx, GPIO_FUNC::INPUT);


        SwUart {
            tx, rx, baud, cycles_per_bit, usec_per_bit
        }
    }

    pub fn put_8(&self, byte: u8) {
        // Start bit
        let mut cycle_now = cycle_cnt_read();
        gpio_set_off(self.tx);
        cycle_now += self.cycles_per_bit;
        wait_until_cycle(cycle_now);

        // For each bit, send
        for i in 0..8 {
            gpio_write(self.tx, ((byte >> i) & 1) != 0);
            cycle_now += self.cycles_per_bit;
            wait_until_cycle(cycle_now);
        }

        // Stop bit
        gpio_set_on(self.tx);
        cycle_now += self.cycles_per_bit;
        wait_until_cycle(cycle_now);
    }
}

