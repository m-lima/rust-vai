#![deny(warnings)]
#![warn(rust_2018_idioms)]

mod completer;
mod error;
mod executors;
//mod support;

fn extract_query(index: usize) -> Result<String, error::Error> {
    if std::env::args().len() <= index {
        Err(error::new("main::extract_query", "No query specified"))
    } else {
        Ok(std::env::args()
            .skip(index)
            .collect::<Vec<String>>()
            .join(" "))
    }
}

fn support() -> Result<(), error::Error> {
    match std::env::args().nth(2) {
        Some(arg) => match arg.as_str() {
            "r" => executors::load_from_stdin()?.save_default(),
            "w" => executors::load_default()?.to_json(),
            "t" => executors::load_default()?.list_targets(),
            "s" => match std::env::args().nth(3) {
                Some(target) => executors::load_default()?
                    .find(target)?
                    .suggest(extract_query(4)?),
                None => Err(error::new("main::support", "No target provided")),
            },
            cmd => Err(error::new(
                "main::support",
                format!("Command not recognized: {}", cmd),
            )),
        },
        None => Err(error::new("main::support", "No command given")),
    }
}

fn execute() -> Result<(), error::Error> {
    match std::env::args().nth(1) {
        Some(target) => executors::load_default()?
            .find(target)?
            .execute(extract_query(2)?),
        None => Err(error::new("main::execute", "Invalid target specified")),
    }
}

fn main() {
    if let Err(err) = if match std::env::args().nth(1) {
        Some(arg) => "-" == arg,
        None => false,
    } {
        support()
    } else {
        execute()
    } {
        eprintln!("{}", err);
        std::process::exit(-1);
    }
}
