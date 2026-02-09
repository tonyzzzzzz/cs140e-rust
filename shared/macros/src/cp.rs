#[inline(always)]
pub fn prefetch_flush() {
    unsafe {
        core::arch::asm!(
        "mcr p15, 0, {r}, c7, c5, 4",
        r = in(reg) 0u32,
        options(nomem, nostack)
        );
    }
}

#[macro_export]
macro_rules! cp_asm_set_raw {
    ($fn_name:ident, $coproc:ident, $opcode_1:literal, $crn:ident, $crm:ident, $opcode_2:literal) => {
        $crate::paste! {
            #[inline(always)]
            pub fn [<$fn_name _set_raw>](v: u32) {
                unsafe {
                    core::arch::asm!(
                        concat!(
                            "mcr ", stringify!($coproc), ", ",
                            stringify!($opcode_1), ", ",
                            "{v}, ",
                            stringify!($crn), ", ",
                            stringify!($crm), ", ",
                            stringify!($opcode_2)
                        ),
                        v = in(reg) v,
                        options(nomem, nostack)
                    );
                }
            }
        }
    };
}

#[macro_export]
macro_rules! cp_asm_set {
    ($fn_name:ident, $coproc:ident, $opcode_1:literal, $crn:ident, $crm:ident, $opcode_2:literal) => {
        $crate::cp_asm_set_raw!($fn_name, $coproc, $opcode_1, $crn, $crm, $opcode_2);

        $crate::paste! {
            #[inline(always)]
            pub fn [<$fn_name _set>](v: u32) {
                [<$fn_name _set_raw>](v);
                $crate::prefetch_flush();
            }
        }
    };
}

#[macro_export]
macro_rules! cp_asm_get {
    ($fn_name:ident, $coproc:ident, $opcode_1:literal, $crn:ident, $crm:ident, $opcode_2:literal) => {
        $crate::paste! {
            #[inline(always)]
            pub fn [<$fn_name _get>]() -> u32 {
                let ret: u32;
                unsafe {
                    core::arch::asm!(
                        concat!(
                            "mrc ", stringify!($coproc), ", ",
                            stringify!($opcode_1), ", ",
                            "{ret}, ",
                            stringify!($crn), ", ",
                            stringify!($crm), ", ",
                            stringify!($opcode_2)
                        ),
                        ret = out(reg) ret,
                        options(nomem, nostack)
                    );
                }
                ret
            }
        }
    };
}

#[macro_export]
macro_rules! cp_asm {
    ($fn_name:ident, $coproc:ident, $opcode_1:literal, $crn:ident, $crm:ident, $opcode_2:literal) => {
        $crate::cp_asm_set!($fn_name, $coproc, $opcode_1, $crn, $crm, $opcode_2);
        $crate::cp_asm_get!($fn_name, $coproc, $opcode_1, $crn, $crm, $opcode_2);
    };
}

#[macro_export]
macro_rules! cp_asm_raw {
    ($fn_name:ident, $coproc:ident, $opcode_1:literal, $crn:ident, $crm:ident, $opcode_2:literal) => {
        $crate::cp_asm_set_raw!($fn_name, $coproc, $opcode_1, $crn, $crm, $opcode_2);
        $crate::cp_asm_get!($fn_name, $coproc, $opcode_1, $crn, $crm, $opcode_2);
    };
}
