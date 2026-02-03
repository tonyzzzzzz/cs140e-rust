pub struct UartProxy;

impl ::core::fmt::Write for UartProxy {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        crate::uart::write_bytes(s.as_bytes());
        Ok(())
    }
}

#[unsafe(no_mangle)]
#[feature(c_varadic)]
pub unsafe extern "C" fn printk(mut args: ...) -> i32 {
    0
}

#[macro_export]
macro_rules! print {
    ($($args:tt)*) => {
        {
            #[allow(unused)]
            use ::core::fmt::Write;
            let _ = ::core::write!(&mut $crate::print::UartProxy, $($args)*);
            $crate::uart::flush();
        }
    }
}
#[macro_export]
macro_rules! println {
    ($($args:tt)*) => {
        {
            #[allow(unused)]
            use ::core::fmt::Write;
            let _ = ::core::writeln!(&mut $crate::print::UartProxy, $($args)*);
            $crate::uart::flush();
        }
    }
}
