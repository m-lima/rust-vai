#![deny(warnings)]
#![warn(rust_2018_idioms)]

mod completer;
mod executors;
mod parser;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
enum VaiErrorType {
    NoArguments,
    NoQuery,
    NoTarget,
    NoCommand,
    UnknownTarget,
    UnknownCommand,
}

#[derive(Debug, Clone)]
struct VaiError(VaiErrorType);
impl std::error::Error for VaiError {}
impl std::fmt::Display for VaiError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}",
            match self.0 {
                VaiErrorType::NoArguments => "No arguments specified",
                VaiErrorType::NoQuery => "No query specified",
                VaiErrorType::NoTarget => "No target specified",
                VaiErrorType::NoCommand => "No command specified",
                VaiErrorType::UnknownTarget => "Unrecognized target",
                VaiErrorType::UnknownCommand => "Unrecognized command",
            }
        )
    }
}

#[inline(always)]
fn new_error<T>(error: VaiErrorType) -> Result<T> {
    Err(VaiError(error).into())
}

fn extract_query(args: Vec<String>, index: usize) -> Result<String> {
    if args.len() <= index {
        new_error(VaiErrorType::NoQuery)
    } else {
        Ok(args
            .into_iter()
            .skip(index)
            .collect::<Vec<String>>()
            .join(" "))
    }
}

fn support(args: Vec<String>) -> Result {
    if args.len() < 2 {
        new_error(VaiErrorType::NoCommand)
    } else {
        match args[1].as_str() {
            "r" => executors::load_from_stdin()?.save_default(),
            "w" => executors::load_default()?
                .to_json()
                .map(|json| println!("{}", json)),
            "t" => Ok(executors::load_default()?
                .list_targets()
                .into_iter()
                .for_each(|target| println!("{}", target))),
            "s" => {
                if args.len() < 3 {
                    new_error(VaiErrorType::NoTarget)
                } else {
                    let executors = executors::load_default()?;
                    let target = executors
                        .find(&args[2])
                        .ok_or(VaiError(VaiErrorType::UnknownTarget))?;
                    let query = &extract_query(args, 3)?;
                    completer::complete(query, target)?
                        .into_iter()
                        .for_each(|entry| println!("{}", entry));
                    target
                        .suggest(query)?
                        .into_iter()
                        .for_each(|entry| println!("{}", entry));
                    Ok(())
                }
            }
            _ => new_error(VaiErrorType::UnknownCommand),
        }
    }
}

fn execute(args: Vec<String>) -> Result {
    executors::load_default()?
        .find(&args[0])
        .ok_or(VaiError(VaiErrorType::UnknownTarget))?
        .execute(&extract_query(args, 1)?)
}

fn main() -> Result {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        new_error(VaiErrorType::NoArguments)
    } else {
        if args[0] == "-" {
            support(args)
        } else {
            execute(args)
        }
    }
}
