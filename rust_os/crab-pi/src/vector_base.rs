use core::ptr::{addr_of, null};
use macros::{cp_asm_get, cp_asm_set};

cp_asm_get!(vbar, p15, 0, c12, c0, 0);
cp_asm_set!(vbar, p15, 0, c12, c0, 0);

pub fn vector_base_get() -> *const u32 {
    vbar_get() as *const u32
}

// Check non-null and first 5 bits cleared (aligned)
fn vector_base_check(vector_base: *const u32) -> bool {
    let addr_u32 = vector_base as usize;
    addr_u32 != 0 && addr_u32 & 0b11111 == 0
}

pub fn vector_base_set(vector_base: *const u32) {
    if !vector_base_check(vector_base) {
        panic!("invalid vector base");
    }

    // Check old vc
    let old_vc = vector_base_get();
    if old_vc == vector_base {
        return;
    }
    if old_vc != null() {
        panic!("vector base already set");
    }
    vbar_set(vector_base as u32);

    let new_vc = vector_base_get();
    assert_eq!(new_vc, vector_base);
}

// Sets new vector base, returns old vector base
pub fn vector_base_reset(vector_base: *const u32) -> *const u32 {
    if !vector_base_check(vector_base) {
        panic!("invalid vector base");
    }
    let old_vc = vector_base_get();
    vbar_set(vector_base as u32);

    // Check
    let new_vc = vector_base_get();
    assert_eq!(new_vc, vector_base);

    old_vc
}
