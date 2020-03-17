#![deny(warnings)]
#![deny(clippy::pedantic)]
#![warn(rust_2018_idioms)]

use vai_core as core;

mod flag;
mod prompt;

type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
enum Error {
    NoQuery,
    UnknownTarget,
    UnknownCommand(String),
    Core(core::error::Error),
}

impl std::error::Error for Error {}

impl std::convert::From<core::error::Error> for Error {
    fn from(error: core::error::Error) -> Self {
        Self::Core(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Error::NoQuery => write!(fmt, "No query specified"),
            Error::UnknownTarget => write!(fmt, "Unrecognized target"),
            Error::UnknownCommand(command) => write!(fmt, "Unrecognized command: {}", command),
            Error::Core(err) => write!(fmt, "{}", err),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Mode {
    Interactive,
    Command(Vec<String>),
    Execute(Vec<String>),
}

fn extract_query(args: Vec<String>, index: usize) -> Result<String> {
    if args.len() <= index {
        Err(Error::NoQuery)
    } else {
        Ok(args
            .into_iter()
            .skip(index)
            .collect::<Vec<String>>()
            .join(" "))
    }
}

fn application_name() -> String {
    (|| {
        std::env::current_exe()
            .ok()?
            .file_name()?
            .to_str()
            .map(String::from)
    })()
    .unwrap_or(String::from(env!("CARGO_PKG_NAME")))
}

fn print_usage() -> Result {
    let name = application_name();

    println!("Usage:          {} [target] [query]", name);
    println!("                {} [option]", name);
    println!();
    println!("Arguments:");
    print!("target          Which target to query");

    match core::executors::load_default() {
        Ok(executors) => {
            print!(" [ ");
            for executor in executors.list_targets() {
                print!("{} ", executor);
            }
            println!("]");
        }
        Err(_) => println!(),
    }

    println!("query           Query string for <target>");
    println!();
    println!("Options:");

    for flag in flag::values() {
        println!(
            "{}, {:<12}{}",
            flag.short(),
            flag.long(),
            flag.description()
        );
    }

    Ok(())
}

fn print_targets() -> Result {
    core::executors::load_default()?
        .list_targets()
        .into_iter()
        .for_each(|target| println!("{}", target));
    Ok(())
}

fn support(args: Vec<String>) -> Result {
    match args[0].as_str().into() {
        flag::Flag::Help => print_usage(),
        flag::Flag::Read => core::executors::load_default()?
            .to_json()
            .map(|json| println!("{}", json))
            .map_err(Error::from),
        flag::Flag::Write => core::executors::load_from_stdin()?
            .save_default()
            .map_err(Error::from),
        flag::Flag::Targets => print_targets(),
        flag::Flag::Suggest => {
            if args.len() < 2 {
                print_targets()
            } else {
                let executors = core::executors::load_default()?;
                match executors.find(&args[1]) {
                    Some(target) => {
                        let query = match extract_query(args, 2) {
                            Ok(query) => query,
                            Err(_) => return Ok(()),
                        };

                        target
                            .suggest(&query)
                            .unwrap_or_else(|_| vec![])
                            .into_iter()
                            .for_each(|entry| println!("{}", entry));
                        target
                            .complete(&query)
                            .unwrap_or_else(|_| vec![])
                            .into_iter()
                            .for_each(|entry| println!("{}", entry));
                        Ok(())
                    }
                    None => {
                        let possible_targets = executors
                            .list_targets()
                            .into_iter()
                            .filter(|target| target.starts_with(&args[1]))
                            .collect::<Vec<_>>();
                        if possible_targets.is_empty() {
                            Err(Error::UnknownTarget)
                        } else {
                            for possible_target in possible_targets {
                                println!("{}", possible_target);
                            }
                            Ok(())
                        }
                    }
                }
            }
        }
        flag::Flag::Unknown(command) => Err(Error::UnknownCommand(String::from(command))),
    }
}

pub(crate) fn execute(args: Vec<String>) -> Result {
    core::executors::load_default()?
        .find(&args[0])
        .ok_or_else(|| Error::UnknownTarget)?
        .execute(&extract_query(args, 1)?)
        .map_err(Error::from)
}

fn select_mode<I: std::iter::Iterator<Item = String>>(input: I) -> Mode {
    let args = input
        .skip(1)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    if args.is_empty() {
        Mode::Interactive
    } else if args[0].starts_with('-') {
        Mode::Command(args)
    } else {
        Mode::Execute(args)
    }
}

fn main() {
    match select_mode(std::env::args()) {
        Mode::Interactive => {
            if atty::is(atty::Stream::Stdout) {
                prompt::run(application_name())
            } else {
                unimplemented!("No GUI yet");
            }
        }
        Mode::Command(args) => support(args),
        Mode::Execute(args) => execute(args),
    }
    .unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(-1);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! args {
        ($($x:expr),*) => (
            <[String]>::into_vec(Box::new([String::from("vai"), $(String::from($x)),*])).into_iter()
        );
    }

    fn param(param: &str) -> Vec<String> {
        vec![String::from(param)]
    }

    #[test]
    fn test_prompt_mode() {
        assert_eq!(select_mode(args!["", ""]), Mode::Interactive);
        assert_eq!(select_mode(args!["     "]), Mode::Interactive);
        assert_eq!(select_mode(args!["     ", "", ""]), Mode::Interactive);
    }

    #[test]
    fn test_command_mode() {
        assert_eq!(select_mode(args!["", "-"]), Mode::Command(param("-")));
        assert_eq!(select_mode(args!["  -  "]), Mode::Command(param("-")));
        assert_eq!(select_mode(args!["  ", "", "-"]), Mode::Command(param("-")));
    }

    #[test]
    fn test_execution_mode() {
        assert_eq!(select_mode(args!["", "a"]), Mode::Execute(param("a")));
        assert_eq!(select_mode(args!["  a  "]), Mode::Execute(param("a")));
        assert_eq!(select_mode(args!["  ", "", "a"]), Mode::Execute(param("a")));
    }
}
