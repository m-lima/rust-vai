use vai_core as core;

use crate::flag;
use crate::{Error, Result};

pub(super) fn support(args: Vec<String>) -> Result {
    match args[0].as_str().into() {
        flag::Flag::Help => {
            flag::print_usage();
            Ok(())
        }
        flag::Flag::Write => core::executors::load_default()?
            .to_json()
            .map(|json| println!("{}", json))
            .map_err(Error::from),
        flag::Flag::Read => core::executors::load_from_stdin()?
            .save_default()
            .map_err(Error::from),
        flag::Flag::Targets => print_targets(),
        flag::Flag::Suggest => {
            if args.len() < 2 {
                print_targets()
            } else {
                let executors = core::executors::load_default()?;
                if let Some(target) = executors.find(&args[1]) {
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
                        .fuzzy_history(&query, 10)
                        .unwrap_or_else(|_| vec![])
                        .into_iter()
                        .for_each(|entry| println!("{}", entry));
                    Ok(())
                } else {
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
        flag::Flag::Unknown(command) => Err(Error::UnknownCommand(command)),
    }
}

fn print_targets() -> Result {
    core::executors::load_default()?
        .list_targets()
        .into_iter()
        .for_each(|target| println!("{}", target));
    Ok(())
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
