extern crate bincode;
extern crate dirs;
extern crate serde_json;

use serde::{Deserialize, Serialize};

const HISTORY_PREFIX: &'static str = "history_";

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

pub fn load_mock() -> Result<Executors, String> {
    Ok(Executors {
        executors: vec![
            Executor {
                name: String::from("N One"),
                command: String::from("C One"),
                suggestion: String::from("S One"),
            },
            Executor {
                name: String::from("N Two"),
                command: String::from("C Two"),
                suggestion: String::from("S Two"),
            },
        ],
    })
}

pub fn load_default() -> Result<Executors, String> {
    load(
        std::env::var("VAI_CONFIG")
            .map(|var| std::path::PathBuf::from(var))
            .or_else(|_| match dirs::config_dir() {
                Some(path) => Ok(path.join("vai").join("config")),
                None => Err("Could not infer HOME directory"),
            })?,
    )
}

pub fn load<P: AsRef<std::path::Path>>(config_path: P) -> Result<Executors, String> {
    let config = std::fs::read(config_path).or_else(|err| Err(err.to_string()))?;
    let executors: Executors =
        bincode::deserialize(&config[..]).or_else(|err| Err(err.to_string()))?;
    Ok(executors)
}

impl Executor {
    pub fn execute(query: String) {}
    pub fn suggest(query: String) {}
}

impl Executors {
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), String> {
        let bytes = bincode::serialize(&self.executors).or_else(|err| Err(err.to_string()))?;
        std::fs::write(path, bytes).or_else(|err| Err(err.to_string()))
    }

    pub fn to_json(&self) -> Result<String, String> {
        let json =
            serde_json::to_string_pretty(&self.executors).or_else(|err| Err(err.to_string()))?;
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
