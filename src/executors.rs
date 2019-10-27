extern crate bincode;
extern crate dirs;
extern crate serde_json;

use super::error;
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

pub fn load_default() -> Result<Executors, error::Error> {
    load(
        default_path()
            .map_err(|e| error::new("executors::load_default::default_path", e))?
            .join(CONFIG_FILE),
    )
}

pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Executors, error::Error> {
    let config = std::fs::read(&path).map_err(|e| error::new("executors::load::read", e))?;
    let executors: Executors = bincode::deserialize(&config[..])
        .map_err(|e| error::new("executors::load::deserialize", e))?;
    Ok(executors)
}

pub fn load_from_stdin() -> Result<Executors, error::Error> {
    let executors: Vec<Executor> = serde_json::from_reader(std::io::stdin())
        .map_err(|e| error::new("executors::load_from_stdin", e))?;
    Ok(Executors { executors })
}

impl Executor {
    pub fn execute(&self, query: String) -> Result<(), error::Error> {
        Ok(())
    }

    pub fn suggest(&self, query: String) -> Result<(), error::Error> {
        Ok(())
    }
}

impl Executors {
    pub fn list_targets(&self) -> Result<(), error::Error> {
        if self.executors.len() < 1 {
            Err(error::new("executors::list_targets", "No targets found"))
        } else {
            for executor in &self.executors {
                println!("{}", &executor.name);
            }
            Ok(())
        }
    }

    pub fn save_default(&self) -> Result<(), error::Error> {
        self.save(
            default_path()
                .map_err(|e| error::new("executors::save_default::default_path", e))?
                .join(CONFIG_FILE),
        )
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), error::Error> {
        let bytes = bincode::serialize(&self.executors)
            .map_err(|e| error::new("executors::save::serialize", e))?;
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(&parent)
                .map_err(|e| error::new("executors::save::create_dir_all", e))?;
        }
        std::fs::write(&path, bytes).map_err(|e| error::new("executors::save::write", e))
    }

    pub fn to_json(&self) -> Result<(), error::Error> {
        println!(
            "{}",
            serde_json::to_string_pretty(&self.executors)
                .map_err(|e| error::new("executors::to_json", e))?
        );
        Ok(())
    }

    pub fn find(&self, name: String) -> Result<&Executor, error::Error> {
        for executor in &self.executors {
            if executor.name == name {
                return Ok(executor);
            }
        }
        Err(error::new(
            "executors::find",
            format!("target not found: {}", name),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    //    #[should_panic(expected = "No such file or directory (os error 2)")]
    fn test_json_roundtrip() {
        let executors = Executors {
            executors: vec![
                Executor {
                    name: "na".to_owned(),
                    command: "ca".to_owned(),
                    suggestion: "sa".to_owned(),
                },
                Executor {
                    name: "nb".to_owned(),
                    command: "cb".to_owned(),
                    suggestion: "sb".to_owned(),
                },
            ],
        };

        let json = serde_json::to_string_pretty(&executors).unwrap();
        let parsed: Executors = serde_json::from_str(json.as_ref()).unwrap();
        assert_eq!(executors, parsed);
    }
}
