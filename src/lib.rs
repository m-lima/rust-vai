#![deny(warnings)]
#![deny(clippy::pedantic)]
#![warn(rust_2018_idioms)]

pub mod executors;
mod parser;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
