#![deny(warnings)]
#![deny(clippy::pedantic)]
#![warn(rust_2018_idioms)]

use vai_core as core;

mod executor;
mod flag;
mod support;

type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug)]
enum Error {
    NoQuery,
    UnknownTarget,
    UnknownCommand(String),
    Core(core::error::Error),
    Rucline(rucline::Error),
}

impl std::error::Error for Error {}

impl std::convert::From<core::error::Error> for Error {
    fn from(error: core::error::Error) -> Self {
        Self::Core(error)
    }
}

impl std::convert::From<rucline::Error> for Error {
    fn from(error: rucline::Error) -> Self {
        Self::Rucline(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Error::NoQuery => write!(fmt, "No query specified"),
            Error::UnknownTarget => write!(fmt, "Unrecognized target"),
            Error::UnknownCommand(command) => write!(fmt, "Unrecognized command: {}", command),
            Error::Core(err) => write!(fmt, "{}", err),
            Error::Rucline(err) => write!(fmt, "{}", err),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Mode {
    Support(Vec<String>),
    Execute(Vec<String>),
}

fn select_mode<I: std::iter::Iterator<Item = String>>(input: I) -> Mode {
    let args = input
        .skip(1)
        .map(|s| String::from(s.trim()))
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    if !args.is_empty() && args[0].starts_with('-') {
        Mode::Support(args)
    } else {
        Mode::Execute(args)
    }
}

fn main() {
    match select_mode(std::env::args()) {
        Mode::Support(args) => support::support(args),
        Mode::Execute(args) => executor::execute(args),
    }
    .unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(-1);
    });
}

#[cfg(test)]
mod tests {
    use super::select_mode;
    use super::Mode::{Execute, Support};

    macro_rules! args {
        ($($x:expr),*) => (
            <[String]>::into_vec(Box::new([String::from("vai"), $(String::from($x)),*])).into_iter()
        );
    }

    macro_rules! param {
        ($($l:literal),*) => {
            vec![$($l.to_string()),*]
        };
    }

    #[test]
    fn test_prompt_mode() {
        assert_eq!(select_mode(args!["", ""]), Execute(param!()));
        assert_eq!(select_mode(args!["     "]), Execute(param!()));
        assert_eq!(select_mode(args!["     ", "", ""]), Execute(param!()));
    }

    #[test]
    fn test_command_mode() {
        assert_eq!(select_mode(args!["", "-"]), Support(param!("-")));
        assert_eq!(select_mode(args!["  -  "]), Support(param!("-")));
        assert_eq!(select_mode(args!["  ", "", "-"]), Support(param!("-")));
    }

    #[test]
    fn test_execution_mode() {
        assert_eq!(select_mode(args!["", "a"]), Execute(param!("a")));
        assert_eq!(select_mode(args!["  a  "]), Execute(param!("a")));
        assert_eq!(select_mode(args!["  ", "", "a"]), Execute(param!("a")));
    }
}
