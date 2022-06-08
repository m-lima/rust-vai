use serde::{Deserialize, Serialize};

use super::error;
use super::parser;
use super::Result;

const HISTORY_PREFIX: &str = "history_";
const CONFIG_FILE: &str = "config";

#[derive(Debug, PartialEq, Eq)]
struct FuzzyMatch(i64, String);

impl FuzzyMatch {
    fn new(
        choice: String,
        pattern: &str,
        fuzzy: &impl fuzzy_matcher::FuzzyMatcher,
    ) -> Option<Self> {
        Some(Self(fuzzy.fuzzy_match(choice.as_str(), pattern)?, choice))
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

/// Represents a target for querying
///
/// Must contain a URL to be called by the browser
///
/// May contain a URL for querying for suggestions, along with it parser
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Executor {
    name: String,
    alias: String,
    command: String,
    suggestion: String,
    parser: parser::Parser,
}

impl Executor {
    fn clean_up_name(self) -> Self {
        Self {
            name: self.name.to_lowercase(),
            alias: self.alias.to_lowercase(),
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
        std::fs::write(&path, data).map_err(|e| error::Error::Write(path, e))
    }

    /// Returns the name associated with this executor
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Executes the query by calling the default browser
    ///
    /// # Arguments
    ///
    /// * `query` - Query string to to execute on `target`
    ///
    /// # Errors
    ///
    /// * If `webbrowser::open(&str)` fails, then [`Error(Browser)`](../error/struct.Error.html)
    /// * If the history cannot be saved, then [`Error(Write)`](../error/struct.Error.html)
    pub fn execute(&self, query: &str) -> Result {
        let url = format!("{}{}", &self.command, query);
        webbrowser::open(url.as_str()).map_err(error::Error::Browser)?;
        self.save_history(query)
    }

    /// Returns the complete history from file
    ///
    /// # Errors
    ///
    /// * If the path for the history cannot be created, then [`Error(Path)`](../error/struct.Error.html)
    pub fn history(&self) -> Result<Vec<String>> {
        use std::io::BufRead;

        let path =
            default_path().map(|path| path.join(format!("{}{}", HISTORY_PREFIX, self.name)))?;
        std::fs::OpenOptions::new()
            .write(false)
            .read(true)
            .open(path)
            .map(std::io::BufReader::new)
            .map(std::io::BufReader::lines)
            .map(|lines| lines.filter_map(std::result::Result::ok).collect())
            .or_else(|_| Ok(vec![]))
    }

    /// Suggest up to `count` queries based on fuzzy matching of the history
    ///
    /// # Arguments
    ///
    /// * `query` - Query string to to get historic completions for
    /// * `count` - Maximum number of items to return
    ///
    /// # Errors
    ///
    /// * If the path for the history cannot be created, then [`Error(Path)`](../error/struct.Error.html)
    pub fn fuzzy_history(&self, query: &str, count: usize) -> Result<Vec<String>> {
        use std::io::BufRead;

        let path =
            default_path().map(|path| path.join(format!("{}{}", HISTORY_PREFIX, self.name)))?;
        let fuzzy = fuzzy_matcher::skim::SkimMatcherV2::default();
        std::fs::OpenOptions::new()
            .write(false)
            .read(true)
            .open(path)
            .map(std::io::BufReader::new)
            .map(std::io::BufReader::lines)
            .map(|lines| {
                let mut completions = lines
                    .filter_map(std::result::Result::ok)
                    .filter_map(|line| FuzzyMatch::new(line, query, &fuzzy))
                    .collect::<Vec<_>>();
                completions.sort_unstable();
                completions
                    .into_iter()
                    .take(count)
                    .map(FuzzyMatch::matched)
                    .collect()
            })
            .or_else(|_| Ok(vec![]))
    }

    /// Queries the suggestion API for this executor for suggestions
    ///
    /// # Arguments
    ///
    /// * `query` - Query string to to get suggestions for
    ///
    /// # Errors
    ///
    /// * If response is not 200 OK, then [`Error(Fetch)`](../error/struct.Error.html)
    /// * If response cannot be parsed, then [`Error(Parse)`](../error/struct.Error.html)
    pub fn suggest(&self, query: &str) -> Result<Vec<String>> {
        if query.len() < 3 || self.suggestion.is_empty() || self.parser == parser::Parser::None {
            return Ok(vec![]);
        }

        let result = {
            let response = ureq::get(format!("{}{}", self.suggestion, query).as_str()).call()?;
            response
                .into_string()
                .map_err(|e| error::Error::Parse(Box::new(e)))?
        };

        parser::parse(&self.parser, &result)
    }
}

/// Contains all [targets](struct.Executor.html) known
///
/// This is the representation of the configuration that gets serialized and deserialized
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Executors(Vec<Executor>);

/// Loads a instance of [`Executors`](struct.Executors.html) based on `path`
///
/// # Arguments
///
/// * `path` - Path from where to load [`Executors`](struct.Executors.html)
///
/// # Errors
///
/// * If the path for the configuration cannot be created, then [`Error(Path)`](../error/struct.Error.html)
/// * If default path cannot be read, then [`Error(Read)`](../error/struct.Error.html)
/// * If the configuration cannot be deserialized, then [`Error(Deserialize)`](../error/struct.Error.html)
///
/// # See also
/// [`load_default()`](fn.load_default.html)
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
            None => Err(error::Error::Path),
        })
}

/// Loads a instance of [`Executors`](struct.Executors.html) based on `path`
///
/// # Arguments
///
/// * `path` - Path from where to load [`Executors`](struct.Executors.html)
///
/// # Errors
///
/// * If `path` cannot be read, then [`Error(Read)`](../error/struct.Error.html)
/// * If the configuration cannot be deserialized, then [`Error(Deserialize)`](../error/struct.Error.html)
///
/// # See also
/// [`load_default()`](fn.load_default.html)
pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Executors> {
    bincode::deserialize(
        std::fs::read(&path)
            .map_err(|e| error::Error::Read(path.as_ref().into(), e))?
            .as_slice(),
    )
    .map(Executors)
    .map_err(|e| error::Error::Deserialize(Box::new(e)))
}

/// Creates a new [`Executors`](struct.Executors.html) based on the json provided through `std::io::stdin`
///
/// This is useful for generating new configuration by json by calling `load_from_stdin()` followed
/// by [`Executors::save_default()`](struct.Executors.html#method.save_default)
///
/// # Errors
///
/// If the json provided cannot be deserialized, then [`Error(Deserialize)`](../error/struct.Error.html)
pub fn load_from_stdin() -> Result<Executors> {
    let executors: Vec<Executor> = serde_json::from_reader(std::io::stdin())
        .map_err(|e| error::Error::Deserialize(Box::new(e)))?;
    Ok(Executors(
        executors.into_iter().map(Executor::clean_up_name).collect(),
    ))
}

impl Executors {
    #[inline]
    fn executors(&self) -> &Vec<Executor> {
        &self.0
    }

    /// Returns all the [`targets`](struct.Executor.html) for querying
    #[must_use]
    pub fn list_targets(&self) -> Vec<&String> {
        self.executors()
            .iter()
            .map(|executor| &executor.name)
            .collect()
    }

    /// Saves this `Executor` to disk in the default path
    ///
    /// # Errors
    ///
    /// * If the path for the configuration cannot be created, then [`Error(Path)`](../error/struct.Error.html)
    /// * If default path cannot be written, then [`Error(Write)`](../error/struct.Error.html)
    /// * If the configuration cannot be serialized, then [`Error(Serialize)`](../error/struct.Error.html)
    ///
    /// # See also
    /// [`save(path)`](#method.save)
    pub fn save_default(&self) -> Result {
        default_path()
            .map(|path| path.join(CONFIG_FILE))
            .and_then(|path| self.save(path))
    }

    /// Saves this `Executor` to disk in the default path
    ///
    /// # Arguments
    ///
    /// * `path` - Path where to save [`Executors`](struct.Executors.html)
    ///
    /// # Errors
    ///
    /// * If the path for the configuration cannot be created, then [`Error(Path)`](../error/struct.Error.html)
    /// * If default path cannot be written, then [`Error(Write)`](../error/struct.Error.html)
    /// * If the configuration cannot be serialized, then [`Error(Serialize)`](../error/struct.Error.html)
    ///
    /// # See also
    /// [`save(path)`](#method.save_default)
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(&parent).map_err(|e| error::Error::Write(parent.into(), e))?;
        }
        let bytes = bincode::serialize(self.executors())
            .map_err(|e| error::Error::Serialize(Box::new(e)))?;
        std::fs::write(&path, bytes).map_err(|e| error::Error::Write(path.as_ref().into(), e))
    }

    /// Output this `Executor` as a json representation
    ///
    /// # Errors
    ///
    /// * If the configuration cannot be serialized, then [`Error(Serialize)`](../error/struct.Error.html)
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self).map_err(|e| error::Error::Serialize(Box::new(e)))
    }

    /// Get the target that matches the provided `name`
    ///
    /// # Arguments
    ///
    /// `name` - Name of the [`target`](struct.Executor.html) to be returned
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&Executor> {
        let lower_case_name = name.to_lowercase();

        // Try full names first
        for executor in self.executors() {
            if executor.name == lower_case_name {
                return Some(executor);
            }
        }

        // Then try aliases
        for executor in self.executors() {
            if executor.alias == lower_case_name {
                return Some(executor);
            }
        }

        None
    }
}
