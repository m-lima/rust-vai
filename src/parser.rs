use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Parser {
    GOOGLE,
    DUCK,
    NONE,
}
