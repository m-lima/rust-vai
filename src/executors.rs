use super::Result;

use serde::{Deserialize, Serialize};

const HISTORY_PREFIX: &'static str = "history_";
const CONFIG_FILE: &'static str = "config";

#[derive(Debug, Clone)]
struct PathError;
impl std::error::Error for PathError {}
impl std::fmt::Display for PathError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "Could not infer HOME directory")
    }
}

fn default_path() -> Result<std::path::PathBuf> {
    std::env::var("VAI_CONFIG")
        .map(|var| std::path::PathBuf::from(var))
        .or_else(|_| match dirs::config_dir() {
            Some(path) => Ok(path.join("vai")),
            None => Err(PathError.into()),
        })
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Executor {
    name: String,
    command: String,
    suggestion: String,
    completer: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Executors(Vec<Executor>);

pub fn load_default() -> Result<Executors> {
    default_path()
        .map(|path| path.join(CONFIG_FILE))
        .and_then(load)
}

pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Executors> {
    bincode::deserialize(std::fs::read(&path)?.as_slice())
        .map(Executors)
        .map_err(std::convert::Into::into)
}

pub fn load_from_stdin() -> Result<Executors> {
    let executors: Vec<Executor> = serde_json::from_reader(std::io::stdin())?;
    Ok(Executors(
        executors.into_iter().map(Executor::clean_up_name).collect(),
    ))
}

impl Executor {
    fn clean_up_name(self) -> Executor {
        Executor {
            name: self.name.to_lowercase(),
            command: self.command,
            suggestion: self.suggestion,
            completer: self.completer,
        }
    }

    fn save_history(&self, query: &String) -> Result {
        let path =
            default_path().map(|path| path.join(format!("{}{}", HISTORY_PREFIX, self.name)))?;
        let history = std::fs::read_to_string(&path)
            .map(|string| {
                string
                    .lines()
                    .filter(|line| line != query)
                    .collect::<Vec<&str>>()
                    .join("\n")
            })
            .unwrap_or(String::new());
        let data = format!("{}\n{}\n", query, history);
        std::fs::write(&path, data).map_err(std::convert::Into::into)
    }

    pub fn execute(&self, query: &String) -> Result {
        let url = format!("{}{}", &self.command, query);
        webbrowser::open(url.as_str())?;
        self.save_history(query)
    }

    pub fn suggest(&self, query: &String) -> Result<Vec<String>> {
        use std::io::BufRead;
        let path =
            default_path().map(|path| path.join(format!("{}{}", HISTORY_PREFIX, self.name)))?;
        std::fs::OpenOptions::new()
            .write(false)
            .read(true)
            .open(path)
            .map(std::io::BufReader::new)
            .map(std::io::BufReader::lines)
            .map(|lines| lines
                .filter_map(std::result::Result::ok)
                .filter(|line| line.starts_with(query))
                .collect())
            .or(Ok(vec![]))
    }
}

impl Executors {
    #[inline(always)]
    fn executors(&self) -> &Vec<Executor> {
        &self.0
    }

    pub fn list_targets(&self) -> Vec<&String> {
        self.executors()
            .iter()
            .map(|executor| &executor.name)
            .collect()
    }

    pub fn save_default(&self) -> Result {
        default_path()
            .map(|path| path.join(CONFIG_FILE))
            .and_then(|path| self.save(path))
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(&parent)?;
        }
        let bytes = bincode::serialize(self.executors())?;
        std::fs::write(&path, bytes).map_err(std::convert::Into::into)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self).map_err(std::convert::Into::into)
    }

    pub fn find(&self, name: &String) -> Option<&Executor> {
        let lower_case_name = name.to_lowercase();
        for executor in self.executors() {
            if executor.name == lower_case_name {
                return Some(executor);
            }
        }
        None
    }
}
