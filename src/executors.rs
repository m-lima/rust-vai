use super::completer;
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

fn clean_up_names(executor: Executor) -> Executor {
    Executor {
        name: executor.name.to_lowercase(),
        command: executor.command,
        suggestion: executor.suggestion,
        completer: executor.completer,
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Executor {
    name: String,
    command: String,
    suggestion: String,
    completer: String,
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
    let executors: Vec<Executor> = bincode::deserialize(&config[..])
        .map_err(|e| error::new("executors::load::deserialize", e))?;
    Ok(Executors { executors })
}

pub fn load_from_stdin() -> Result<Executors, error::Error> {
    let executors: Vec<Executor> =
        serde_json::from_reader::<std::io::Stdin, Vec<Executor>>(std::io::stdin())
            .map_err(|e| error::new("executors::load_from_stdin", e))?
            .into_iter()
            .map(clean_up_names)
            .collect();
    Ok(Executors { executors })
}

impl Executor {
    pub fn execute(&self, query: String) -> Result<(), error::Error> {
        match webbrowser::open(format!("{}{}", self.command, query).as_str()) {
            Ok(_) => self.save_history(query),
            Err(e) => Err(error::new("executors::executor::execute", e)),
        }
    }

    pub fn suggest(&self, query: String) -> Result<(), error::Error> {
        completer::complete(&query, &self.suggestion, &self.completer)?;

        println!("--");

        use std::io::BufRead;
        std::io::BufReader::new(
            std::fs::OpenOptions::new()
                .write(false)
                .read(true)
                .open(
                    default_path()
                        .map_err(|e| error::new("executors::executor::suggest::default_path", e))?
                        .join(format!("{}{}", HISTORY_PREFIX, self.name)),
                )
                .map_err(|e| error::new("executors::executor::suggest::open", e))?,
        )
        .lines()
        .filter(|line| match line {
            Ok(line) => line.starts_with(&query),
            Err(_) => false,
        })
        .for_each(|line| match line {
            Ok(line) => println!("{}", line),
            Err(_) => (),
        });
        Ok(())
    }

    fn load_history(&self) -> Result<String, error::Error> {
        std::fs::read_to_string(
            default_path()
                .map_err(|e| error::new("executors::executor::load_history::default_path", e))?
                .join(format!("{}{}", HISTORY_PREFIX, self.name)),
        )
        .map_err(|e| error::new("executors::executor::load_history::read_to_string", e))
    }

    fn save_history(&self, query: String) -> Result<(), error::Error> {
        std::fs::write(
            default_path()
                .map_err(|e| error::new("executors::executor::save_history::default_path", e))?
                .join(format!("{}{}", HISTORY_PREFIX, self.name)),
            match self.load_history() {
                Ok(history) => format!("{}\n{}", query, history),
                Err(_) => query + "\n",
            },
        )
        .map_err(|e| error::new("executors::executor::save_history::write", e))
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
        let lower_case_name = name.to_lowercase();
        for executor in &self.executors {
            if executor.name == lower_case_name {
                return Ok(executor);
            }
        }
        Err(error::new(
            "executors::find",
            format!("target not found: {}", name),
        ))
    }
}
