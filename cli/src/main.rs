#![deny(warnings)]
#![deny(clippy::pedantic)]
#![warn(rust_2018_idioms)]

mod prompt;

use vai_core as core;

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
    Prompt,
    Command(Vec<String>),
    Execute(Vec<String>),
}

// struct Flag {
//     short: &'static str,
//     long: &'static str,
//     description: &'static str,
// }
//
// impl Flag {
//     const Help: Flag = Flag {
//         short: "-h",
//         long: "--help",
//         description: "Display usage message",
//     };
//
//     const Write: Flag = Flag {
//         short: "-w",
//         long: "--write",
//         description: "Write saved configuration to stdout",
//     };
//
//     const Read: Flag = Flag {
//         short: "-r",
//         long: "--read",
//         description: "Read configuration from stdin and save",
//     };
//
//     const Targets: Flag = Flag {
//         short: "-t",
//         long: "--targets",
//         description: "Write configured targets to stdout",
//     };
//
//     const Suggest: Flag = Flag {
//         short: "-s",
//         long: "--suggest",
//         description: "Prints a list of suggestions for the given input",
//     };
//
//     fn parse(arg: &str) -> Option<Self> {
//         match arg {
//             "-h" | "--help" => Some(Flag::Help),
//             "-r" | "--read" => Some(Flag::Read),
//             "-w" | "--write" => Some(Flag::Write),
//             "-t" | "--targets" => Some(Flag::Targets),
//             "-s" | "--suggest" => Some(Flag::Suggest),
//             _ => None,
//         }
//     }
// }

// enum Flag {
//     Help,
//     Write,
//     Read,
//     Targets,
//     Suggest,
//     Unknown,
// }

// impl std::convert::From<&str> for Flag {
//
// impl std::convert::From<&str> for Flag {
//     fn from(input: &str) -> Flag {
//         match input {
//             "-h" | "--help" => Flag::Help,
//             "-r" | "--read" => Flag::Read,
//             "-w" | "--write" => Flag::Write,
//             "-t" | "--targets" => Flag::Targets,
//             "-s" | "--suggest" => Flag::Suggest,
//             _ => Flag::Unknown,
//         }
//     }
// }

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

fn print_usage() -> ! {
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
    println!("-r, --read      Read configuration from stdin and save");
    println!("-w, --write     Write saved configuration to stdout");
    println!("-t, --targets   Write configured targets to stdout");
    println!("-s, --suggest   Prints a list of suggestions for the given input");
    println!("-h, --help      Display this help message");
    println!();

    std::process::exit(0);
}

fn print_targets() -> Result {
    core::executors::load_default()?
        .list_targets()
        .into_iter()
        .for_each(|target| println!("{}", target));
    Ok(())
}

fn support(args: Vec<String>) -> Result {
    match args[0].as_str() {
        "-h" | "--help" => print_usage(),
        "-r" | "--read" => core::executors::load_from_stdin()?
            .save_default()
            .map_err(Error::from),
        "-w" | "--write" => core::executors::load_default()?
            .to_json()
            .map(|json| println!("{}", json))
            .map_err(Error::from),
        "-t" | "--targets" => print_targets(),
        "-s" | "--suggest" => {
            if args.len() < 2 {
                print_targets()
            } else {
                let executors = core::executors::load_default()?;
                let target = executors.find(&args[1]).ok_or(Error::UnknownTarget)?;
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
        }
        command => Err(Error::UnknownCommand(String::from(command))),
    }
}

fn execute(args: Vec<String>) -> Result {
    core::executors::load_default()?
        .find(&args[0])
        .ok_or_else(|| Error::UnknownTarget)?
        .execute(&extract_query(args, 1)?)
        .map_err(Error::from)
}

fn select_mode<I: std::iter::Iterator<Item = String>>(input: I) -> Mode {
    let args: Vec<String> = input
        .skip(1)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if args.is_empty() {
        Mode::Prompt
    } else if args[0].starts_with('-') {
        Mode::Command(args)
    } else {
        Mode::Execute(args)
    }
}

fn main() {
    match select_mode(std::env::args()) {
        Mode::Prompt => prompt::run(),
        Mode::Command(args) => support(args),
        Mode::Execute(args) => execute(args),
    }
    .unwrap_or_else(|err| eprintln!("{}", err));
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
        assert_eq!(select_mode(args!["", ""]), Mode::Prompt);
        assert_eq!(select_mode(args!["     "]), Mode::Prompt);
        assert_eq!(select_mode(args!["     ", "", ""]), Mode::Prompt);
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
