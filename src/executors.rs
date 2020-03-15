use super::parser;
use super::Result;

use serde::{Deserialize, Serialize};

const HISTORY_PREFIX: &str = "history_";
const CONFIG_FILE: &str = "config";

#[derive(Debug, Clone)]
enum Error {
    Path,
    Fetch(u16),
}
impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Error::Path => write!(fmt, "Could not infer HOME directory"),
            Error::Fetch(status) => write!(fmt, "Got status code {} from suggest query", status),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct FuzzyMatch(i64, String);

impl FuzzyMatch {
    fn new(choice: String, pattern: &str) -> Option<Self> {
        Some(Self(
            fuzzy_matcher::skim::fuzzy_match(choice.as_str(), pattern)?,
            choice,
        ))
    }

    #[inline]
    fn matched(self) -> String {
        self.1
    }
}

impl Ord for FuzzyMatch {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for FuzzyMatch {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Executor {
    name: String,
    command: String,
    suggestion: String,
    parser: parser::Parser,
}

impl Executor {
    fn clean_up_name(self) -> Self {
        Self {
            name: self.name.to_lowercase(),
            command: self.command,
            suggestion: self.suggestion,
            parser: self.parser,
        }
    }

    fn save_history(&self, query: &str) -> Result {
        let path =
            default_path().map(|path| path.join(format!("{}{}", HISTORY_PREFIX, self.name)))?;
        let history = std::fs::read_to_string(&path)
            .ok()
            .map_or_else(String::new, |string| {
                string
                    .lines()
                    .filter(|line| *line != query)
                    .collect::<Vec<&str>>()
                    .join("\n")
            });
        let data = format!("{}\n{}\n", query, history);
        std::fs::write(&path, data).map_err(std::convert::Into::into)
    }

    pub fn execute(&self, query: &str) -> Result {
        let url = format!("{}{}", &self.command, query);
        webbrowser::open(url.as_str())?;
        self.save_history(query)
    }

    pub fn complete(&self, query: &str) -> Result<Vec<String>> {
        use std::io::BufRead;

        let path =
            default_path().map(|path| path.join(format!("{}{}", HISTORY_PREFIX, self.name)))?;
        std::fs::OpenOptions::new()
            .write(false)
            .read(true)
            .open(path)
            .map(std::io::BufReader::new)
            .map(std::io::BufReader::lines)
            .map(|lines| {
                let mut completions: Vec<FuzzyMatch> = lines
                    .filter_map(std::result::Result::ok)
                    .filter_map(|line| FuzzyMatch::new(line, &query))
                    .collect();
                completions.sort_unstable();
                completions
                    .into_iter()
                    .take(10)
                    .map(FuzzyMatch::matched)
                    .collect()
            })
            .or_else(|_| Ok(vec![]))
    }

    pub fn suggest(&self, query: &str) -> Result<Vec<String>> {
        if query.len() < 3 || self.suggestion.is_empty() || self.parser == parser::Parser::NONE {
            return Ok(vec![]);
        }

        let result = {
            let response = ureq::get(format!("{}{}", self.suggestion, query).as_str()).call();
            if !response.ok() {
                return Err(Error::Fetch(response.status()).into());
            }
            response.into_string()?
        };

        parser::parse(&self.parser, &result)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Executors(Vec<Executor>);

pub fn load_default() -> Result<Executors> {
    default_path()
        .map(|path| path.join(CONFIG_FILE))
        .and_then(load)
}

fn default_path() -> Result<std::path::PathBuf> {
    std::env::var("VAI_CONFIG")
        .map(std::path::PathBuf::from)
        .or_else(|_| match dirs::config_dir() {
            Some(path) => Ok(path.join("vai")),
            None => Err(Error::Path.into()),
        })
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

impl Executors {
    #[inline]
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

    pub fn find(&self, name: &str) -> Option<&Executor> {
        let lower_case_name = name.to_lowercase();
        for executor in self.executors() {
            if executor.name == lower_case_name {
                return Some(executor);
            }
        }
        None
    }
}
