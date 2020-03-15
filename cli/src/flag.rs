pub(super) enum Flag {
    Help,
    Write,
    Read,
    Targets,
    Suggest,
    Unknown(String),
}

impl Flag {
    pub(super) fn short(&self) -> &'static str {
        match self {
            Flag::Write => "-w",
            Flag::Read => "-r",
            Flag::Targets => "-t",
            Flag::Suggest => "-s",
            Flag::Help => "-h",
            Flag::Unknown(_) => "",
        }
    }

    pub(super) fn long(&self) -> &'static str {
        match self {
            Flag::Write => "--write",
            Flag::Read => "--read",
            Flag::Targets => "--targets",
            Flag::Suggest => "--suggest",
            Flag::Help => "--help",
            Flag::Unknown(_) => "",
        }
    }

    pub(super) fn description(&self) -> &'static str {
        match self {
            Flag::Write => "Write saved configuration to stdout",
            Flag::Read => "Read configuration from stdin and save",
            Flag::Targets => "Write configured targets to stdout",
            Flag::Suggest => "Print a list of suggestions for the given input",
            Flag::Help => "Display usage message",
            Flag::Unknown(_) => "",
        }
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

pub(super) fn values() -> Vec<Flag> {
    vec![
        Flag::Write,
        Flag::Read,
        Flag::Targets,
        Flag::Suggest,
        Flag::Help,
    ]
}
