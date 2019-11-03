#![deny(warnings)]
#![deny(clippy::pedantic)]
#![warn(rust_2018_idioms)]

mod executors;
mod parser;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
enum ErrorType {
    NoArguments,
    EmptyArgument,
    NoQuery,
    NoTarget,
    UnknownTarget,
    UnknownCommand,
}

#[derive(Debug, Clone)]
struct VaiError(ErrorType);
impl std::error::Error for VaiError {}
impl std::fmt::Display for VaiError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}",
            match self.0 {
                ErrorType::NoArguments => "No arguments specified",
                ErrorType::EmptyArgument => "Empty argument specified",
                ErrorType::NoQuery => "No query specified",
                ErrorType::NoTarget => "No target specified",
                ErrorType::UnknownTarget => "Unrecognized target",
                ErrorType::UnknownCommand => "Unrecognized command",
            }
        )
    }
}

#[inline]
fn new_error<T>(error: ErrorType) -> Result<T> {
    Err(VaiError(error).into())
}

fn extract_query(args: Vec<String>, index: usize) -> Result<String> {
    if args.len() <= index {
        new_error(ErrorType::NoQuery)
    } else {
        Ok(args
            .into_iter()
            .skip(index)
            .collect::<Vec<String>>()
            .join(" "))
    }
}

fn support(args: Vec<String>) -> Result {
    match args[0].as_str() {
        "-r" => executors::load_from_stdin()?.save_default(),
        "-w" => executors::load_default()?
            .to_json()
            .map(|json| println!("{}", json)),
        "-t" => {
            executors::load_default()?
                .list_targets()
                .into_iter()
                .for_each(|target| println!("{}", target));
            Ok(())
        }
        "-s" => {
            if args.len() < 2 {
                new_error(ErrorType::NoTarget)
            } else {
                let executors = executors::load_default()?;
                let target = executors
                    .find(&args[1])
                    .ok_or(VaiError(ErrorType::UnknownTarget))?;
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
        _ => new_error(ErrorType::UnknownCommand),
    }
}

fn execute(args: Vec<String>) -> Result {
    executors::load_default()?
        .find(&args[0])
        .ok_or_else(|| VaiError(ErrorType::UnknownTarget))?
        .execute(&extract_query(args, 1)?)
}

fn main() -> Result {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        new_error(ErrorType::NoArguments)
    } else if args[0]
        .chars()
        .next()
        .ok_or(VaiError(ErrorType::EmptyArgument))?
        == '-'
    {
        support(args)
    } else {
        execute(args)
    }
}
