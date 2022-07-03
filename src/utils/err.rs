use colored::Colorize;
use std::{fmt, process::exit};

pub trait FormatUnpack<T, E> {
    fn fup(self, pretty: bool) -> T;
}

impl<T, E: fmt::Display> FormatUnpack<T, E> for Result<T, E> {
    #[cfg(not(feature = "panic_immediate_abort"))]
    fn fup(self, no_pretty: bool) -> T {
        match self {
            Ok(x) => x,
            Err(y) => {
                if no_pretty {
                    panic!("{}", y)
                }
                pretty_error(y);
            }
        }
    }
}

pub fn pretty_error<T: fmt::Display>(message: T) -> ! {
    println!("{}: {}", "Error".red(), message);
    exit(1)
}
