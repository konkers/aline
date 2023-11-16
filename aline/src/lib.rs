#![cfg_attr(not(feature = "std"), no_std)]
#![feature(async_fn_in_trait)]

use core::str::FromStr;

use num_traits::Num;

pub use aline_macro::CommandParser;

mod command;
mod dyner;

#[derive(Debug, PartialEq)]
pub enum Error {
    TooFewArguments,
    UnusedArguments,
    ArgumentParseError,
    InvalidIntegerPrefix,
    IntegerParseError,
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait CommandParser {
    fn name(&self) -> &str;
    fn parse(&mut self, args: &[&str]) -> Result<()>;
}

pub mod internal {
    use crate::{Error, Result};

    pub fn next_arg<'a>(args: &'a [&'a str]) -> Result<(&'a [&'a str], &'a str)> {
        if args.is_empty() {
            return Err(Error::TooFewArguments);
        }

        Ok((&args[1..], args[0]))
    }
}

#[derive(Debug, PartialEq)]
pub struct PrefixedNum<T: Num> {
    pub inner: T,
}

impl<T: Num> PrefixedNum<T> {
    pub fn new(value: T) -> Self {
        Self { inner: value }
    }
}

impl<T: Num> FromStr for PrefixedNum<T> {
    type Err = Error;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let value = if let Some(s) = s.strip_prefix("0x") {
            T::from_str_radix(s, 16).map_err(|_| Error::IntegerParseError)?
        } else if let Some(s) = s.strip_prefix("0o") {
            T::from_str_radix(s, 8).map_err(|_| Error::IntegerParseError)?
        } else if let Some(s) = s.strip_prefix("0b") {
            T::from_str_radix(s, 2).map_err(|_| Error::IntegerParseError)?
        } else {
            T::from_str_radix(s, 10).map_err(|_| Error::IntegerParseError)?
        };
        Ok(Self { inner: value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Needed to allow proc macros to work
    use crate as aline;

    #[test]
    fn all_bases_parse_correctly() {
        #[derive(CommandParser, Debug, PartialEq)]
        struct TestCommand {
            hex: PrefixedNum<u32>,
            octal: PrefixedNum<u32>,
            binary: PrefixedNum<u32>,
            decimal: PrefixedNum<u32>,
        }

        let mut test = TestCommand {
            hex: PrefixedNum::new(0),
            octal: PrefixedNum::new(0),
            binary: PrefixedNum::new(0),
            decimal: PrefixedNum::new(0),
        };

        test.parse(&["0xff", "0o7", "0b1010101", "1234"]).unwrap();

        assert_eq!(
            test,
            TestCommand {
                hex: PrefixedNum::new(0xff),
                octal: PrefixedNum::new(0o7),
                binary: PrefixedNum::new(0b1010101),
                decimal: PrefixedNum::new(1234),
            }
        );
    }

    #[test]
    fn too_few_arguments_returns_error() {
        #[derive(CommandParser, Debug, PartialEq)]
        struct TestCommand {
            a: u32,
            b: u32,
        }

        let mut test = TestCommand { a: 0, b: 0 };

        assert_eq!(test.parse(&["1"]), Err(Error::TooFewArguments));
    }

    #[test]
    fn too_many_arguments_returns_error() {
        #[derive(CommandParser, Debug, PartialEq)]
        struct TestCommand {
            a: u32,
        }

        let mut test = TestCommand { a: 0 };

        assert_eq!(test.parse(&["1", "2"]), Err(Error::UnusedArguments));
    }

    #[test]
    fn heapless_string_parses_correctly() {
        #[derive(CommandParser, Debug, PartialEq)]
        struct TestCommand {
            s: heapless::String<16>,
        }

        let mut test = TestCommand {
            s: heapless::String::new(),
        };

        test.parse(&["test"]).unwrap();

        assert_eq!(
            test,
            TestCommand {
                s: heapless::String::try_from("test").unwrap()
            }
        );
    }

    #[test]
    fn heapless_string_overflow_returns_error() {
        #[derive(CommandParser, Debug, PartialEq)]
        struct TestCommand {
            s: heapless::String<2>,
        }

        let mut test = TestCommand {
            s: heapless::String::new(),
        };

        assert_eq!(test.parse(&["test"]), Err(Error::ArgumentParseError));
    }
}
