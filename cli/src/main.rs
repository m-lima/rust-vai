#![deny(warnings)]
#![deny(clippy::pedantic)]
#![warn(rust_2018_idioms)]

use vai_core as core;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
enum Error {
    NoArguments,
    EmptyArgument,
    NoQuery,
    NoTarget,
    UnknownTarget,
    UnknownCommand,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Error::NoArguments => write!(fmt, "No arguments specified"),
            Error::EmptyArgument => write!(fmt, "Empty argument specified"),
            Error::NoQuery => write!(fmt, "No query specified"),
            Error::NoTarget => write!(fmt, "No target specified"),
            Error::UnknownTarget => write!(fmt, "Unrecognized target"),
            Error::UnknownCommand => write!(fmt, "Unrecognized command"),
        }
    }
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

#[inline]
fn new_error<T>(error: Error) -> Result<T> {
    Err(error.into())
}

fn extract_query(args: Vec<String>, index: usize) -> Result<String> {
    if args.len() <= index {
        new_error(Error::NoQuery)
    } else {
        Ok(args
            .into_iter()
            .skip(index)
            .collect::<Vec<String>>()
            .join(" "))
    }
}

trait FirstCharacter {
    fn first_char(&self) -> Result<char>;
}

impl FirstCharacter for Vec<String> {
    fn first_char(&self) -> Result<char> {
        self[0]
            .chars()
            .next()
            .ok_or(Error::EmptyArgument)
            .map_err(std::convert::Into::into)
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
    println!("Configuration file is stored at:");
    println!(" - $VAI_CONFIG (if defined)");

    if let Some(path) = dirs::config_dir() {
        println!(" - {}", path.join("vai").display());
    }

    std::process::exit(0);
}

fn support(args: Vec<String>) -> Result {
    match args[0].as_str() {
        "-h" | "--help" => print_usage(),
        "-r" | "--read" => core::executors::load_from_stdin()?.save_default(),
        "-w" | "--write" => core::executors::load_default()?
            .to_json()
            .map(|json| println!("{}", json)),
        "-t" | "--targets" => {
            core::executors::load_default()?
                .list_targets()
                .into_iter()
                .for_each(|target| println!("{}", target));
            Ok(())
        }
        "-s" | "--suggest" => {
            if args.len() < 2 {
                new_error(Error::NoTarget)
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
        _ => new_error(Error::UnknownCommand),
    }
}

fn execute(args: Vec<String>) -> Result {
    core::executors::load_default()?
        .find(&args[0])
        .ok_or_else(|| Error::UnknownTarget)?
        .execute(&extract_query(args, 1)?)
}

fn run() -> Result {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        new_error(Error::NoArguments)
    } else if args.first_char()? == '-' {
        support(args)
    } else {
        execute(args)
    }
}

fn main() {
    run().unwrap_or_else(|err| eprintln!("{}", err));
}
