#![no_std]
#![no_main]
pub use paste::paste;

pub mod cp;
pub use cp::*;

#[macro_export]
macro_rules! enum_ptr {
    ($(#[$m:meta])* $vis:vis enum $Name:ident { $($V:ident = $n:expr,)* }) => {
        $(#[$m])*
        #[repr(u32)]
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        $vis enum $Name { $($V = $n,)* }

        impl $Name {
            pub const fn from_u32(v: u32) -> Option<Self> {
                match v {
                    $(
                        _ if v == ($n) => Some(Self::$V),
                    )*
                    _ => None,
                }
            }

            #[inline(always)]
            pub const fn addr(self) -> usize {
                self as usize
            }

            #[inline(always)]
            pub const fn as_mut_ptr<T>(self) -> *mut T {
                ::core::ptr::with_exposed_provenance_mut(self.addr())
            }

            #[inline(always)]
            pub const fn as_ptr<T>(self) -> *const T {
                ::core::ptr::with_exposed_provenance(self.addr())
            }
        }
    };
}

#[macro_export]
macro_rules! enum_u32 {
    ($(#[$m:meta])* $vis:vis enum $Name:ident { $($V:ident = $n:expr,)* }) => {
        $(#[$m])*
        #[repr(u32)]
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        $vis enum $Name { $($V = $n,)* }

        impl $Name {
            pub const fn from_u32(v: u32) -> Option<Self> {
                match v {
                    $(
                        _ if v == ($n) => Some(Self::$V),
                    )*
                    _ => None,
                }
            }

            #[inline(always)]
            pub const fn val(self) -> u32 {
                self as u32
            }
        }

        impl From<$Name> for u32 {
            fn from(v: $Name) -> Self {
                v.val()
            }
        }

        impl ::core::ops::BitOr for $Name {
            type Output = u32;
            fn bitor(self, rhs: Self) -> Self::Output {
                self.val() | rhs.val()
            }
        }

        impl ::core::ops::BitOr<u32> for $Name {
            type Output = u32;
            fn bitor(self, rhs: u32) -> Self::Output {
                self.val() | rhs
            }
        }

        impl ::core::ops::BitOr<$Name> for u32 {
            type Output = u32;
            fn bitor(self, rhs: $Name) -> Self::Output {
                self | rhs.val()
            }
        }
    };
}
