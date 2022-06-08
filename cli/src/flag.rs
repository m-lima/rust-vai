use vai_core as core;

pub(super) enum Flag {
    Help,
    Write,
    Read,
    Targets,
    Suggest,
    Unknown(String),
}

impl Flag {
    fn short(&self) -> &'static str {
        match self {
            Flag::Write => "-w",
            Flag::Read => "-r",
            Flag::Targets => "-t",
            Flag::Suggest => "-s",
            Flag::Help => "-h",
            Flag::Unknown(_) => "",
        }
    }

    fn long(&self) -> &'static str {
        match self {
            Flag::Write => "--write",
            Flag::Read => "--read",
            Flag::Targets => "--targets",
            Flag::Suggest => "--suggest",
            Flag::Help => "--help",
            Flag::Unknown(_) => "",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Flag::Write => "Write saved configuration to stdout",
            Flag::Read => "Read configuration from stdin and save",
            Flag::Targets => "Write configured targets to stdout",
            Flag::Suggest => "Print a list of suggestions for the given input",
            Flag::Help => "Display usage message",
            Flag::Unknown(_) => "",
        }
    }

    fn values() -> Vec<Flag> {
        vec![
            Flag::Write,
            Flag::Read,
            Flag::Targets,
            Flag::Suggest,
            Flag::Help,
        ]
    }
}

impl std::convert::From<&str> for Flag {
    fn from(input: &str) -> Flag {
        match input {
            "-h" | "--help" => Flag::Help,
            "-r" | "--read" => Flag::Read,
            "-w" | "--write" => Flag::Write,
            "-t" | "--targets" => Flag::Targets,
            "-s" | "--suggest" => Flag::Suggest,
            command => Flag::Unknown(String::from(command)),
        }
    }
}

pub(super) fn print_usage() {
    let name = application_name();

    println!("Usage:");
    println!("    {} [target] [query]", name);
    println!("    {} <option>", name);
    println!();

    println!("Arguments:");
    println!(
        "    target          Which target to query{}",
        list_targets()
    );
    println!("    query           Query string for <target>");
    println!();

    println!("Options:");
    for flag in Flag::values() {
        println!(
            "    {}, {:<12}{}",
            flag.short(),
            flag.long(),
            flag.description()
        );
    }
    println!();

    println!("If no parameters are provided, the prompt user interface will be invoked");
}

fn list_targets() -> String {
    if let Some(targets) = core::executors::load_default()
        .ok()
        .map(|e| {
            e.list_targets()
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .filter(|t| !t.is_empty())
    {
        format!(" [{}]", targets.join(" "))
    } else {
        String::new()
    }
}

fn application_name() -> String {
    (|| {
        std::env::current_exe()
            .ok()?
            .file_stem()?
            .to_str()
            .map(String::from)
    })()
    .unwrap_or_else(|| String::from(env!("CARGO_PKG_NAME")))
}
