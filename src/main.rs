mod error;
mod executors;
//mod support;

fn support_mode() -> Result<(), error::Error> {
    match std::env::args().nth(2) {
        Some(arg) => match arg.as_str() {
            "r" => executors::load_from_stdin()?.save_default(),
            "w" => executors::load_default()?.to_json(),
            "t" => {
                executors::load_default()?.list_targets();
                Ok(())
            }
            "c" => Ok(()),
            cmd => Err(error::new(
                "main::support_mode",
                format!("Command not recognized: {}", cmd),
            )),
        },
        None => Err(error::new("main::support_mode", "No command given")),
    }
}

fn main() {
    if let Err(err) = if match std::env::args().nth(1) {
        Some(arg) => "-" == arg,
        None => false,
    } {
        support_mode()
    } else {
        executors::load_default().map(|_| ())
    } {
        eprintln!("{}", err);
    }
}
