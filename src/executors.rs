extern crate bincode;
extern crate dirs;
extern crate serde_json;

use serde::{Deserialize, Serialize};

const HISTORY_PREFIX: &'static str = "history_";
const CONFIG_FILE: &'static str = "config";

fn default_path() -> Result<std::path::PathBuf, &'static str> {
    std::env::var("VAI_CONFIG")
        .map(|var| std::path::PathBuf::from(var))
        .or_else(|_| match dirs::config_dir() {
            Some(path) => Ok(path.join("vai")),
            None => Err("Could not infer HOME directory"),
        })
}

fn map_error<E: std::string::ToString, T>(err: E) -> Result<T, String> {
    Err(err.to_string())
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Executor {
    name: String,
    command: String,
    suggestion: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Executors {
    executors: Vec<Executor>,
}

pub fn load_default() -> Result<Executors, String> {
    load(default_path()?.join(CONFIG_FILE))
}

pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Executors, String> {
    let config = std::fs::read(&path).or_else(map_error)?;
    let executors: Executors = bincode::deserialize(&config[..]).or_else(map_error)?;
    Ok(executors)
}

pub fn load_from_stdin() -> Result<Executors, String> {
    let executors: Executors = serde_json::from_reader(std::io::stdin()).or_else(map_error)?;
    Ok(executors)
}

impl Executor {
    pub fn execute(query: String) {}
    pub fn suggest(query: String) {}
}

impl Executors {
    pub fn save_default(&self) -> Result<(), String> {
        self.save(default_path()?.join(CONFIG_FILE))
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), String> {
        let bytes = bincode::serialize(&self.executors).or_else(map_error)?;
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(&parent).or_else(map_error)?;
        }
        std::fs::write(&path, bytes).or_else(map_error)
    }

    pub fn to_json(&self) -> Result<String, String> {
        let json = serde_json::to_string_pretty(&self.executors).or_else(map_error)?;
        println!("{}", json);
        Ok(json)
    }

    pub fn find(&self, name: String) -> Option<&Executor> {
        for executor in &self.executors {
            if executor.name == name {
                return Some(executor);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "No such file or directory (os error 2)")]
    fn test_no_env_no_file() {
        load_default().unwrap();
    }
}
