pub fn delay_cycles(mut ticks: u32) {
    while (ticks > 0) {
        ticks -= 1;
    }
}
