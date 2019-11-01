#![deny(warnings)]
#![warn(rust_2018_idioms)]

mod executors;
mod parser;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
enum VaiErrorType {
    NoArguments,
    EmptyArgument,
    NoQuery,
    NoTarget,
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
                VaiErrorType::EmptyArgument => "Empty argument specified",
                VaiErrorType::NoQuery => "No query specified",
                VaiErrorType::NoTarget => "No target specified",
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
    match args[0].as_str() {
        "-r" => executors::load_from_stdin()?.save_default(),
        "-w" => executors::load_default()?
            .to_json()
            .map(|json| println!("{}", json)),
        "-t" => Ok(executors::load_default()?
            .list_targets()
            .into_iter()
            .for_each(|target| println!("{}", target))),
        "-s" => {
            if args.len() < 2 {
                new_error(VaiErrorType::NoTarget)
            } else {
                let executors = executors::load_default()?;
                let target = executors
                    .find(&args[1])
                    .ok_or(VaiError(VaiErrorType::UnknownTarget))?;
                let query = match extract_query(args, 2) {
                    Ok(query) => query,
                    Err(_) => return Ok(()),
                };

                target
                    .suggest(&query)
                    .unwrap_or(vec![])
                    .into_iter()
                    .for_each(|entry| println!("{}", entry));
                target
                    .complete(&query)
                    .unwrap_or(vec![])
                    .into_iter()
                    .for_each(|entry| println!("{}", entry));
                Ok(())
            }
        }
        _ => new_error(VaiErrorType::UnknownCommand),
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
        if args[0].chars().next().ok_or(VaiError(VaiErrorType::EmptyArgument))? == '-' {
            support(args)
        } else {
            execute(args)
        }
    }
}
