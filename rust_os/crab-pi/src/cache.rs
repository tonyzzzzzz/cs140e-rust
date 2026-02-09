use macros::{cp_asm_get, cp_asm_set};

cp_asm_get!(control_reg, p15, 0, c1, c0, 0);
cp_asm_set!(control_reg, p15, 0, c1, c0, 1);

pub fn caches_enable() {
    let mut control_reg = control_reg_get();
    control_reg |= (1 << 12);
    control_reg |= (1 << 11);
    control_reg_set(control_reg);
}

pub fn caches_disable() {
    let mut control_reg = control_reg_get();
    control_reg &= !(1 << 12);
    control_reg &= !(1 << 11);
    control_reg_set(control_reg);
}

pub fn is_caches_enabled() -> bool {
    let control_reg = control_reg_get();

    (control_reg & (1 << 12)) != 0 && (control_reg & (1 << 11)) != 0
}
