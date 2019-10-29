use super::Result;

#[derive(Debug, Clone)]
struct FetchError(u16);
impl std::error::Error for FetchError {}
impl std::fmt::Display for FetchError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "Got status code {} from suggest query", self.0)
    }
}

#[derive(Debug, Clone)]
struct UnkownParser(String);
impl std::error::Error for UnkownParser {}
impl std::fmt::Display for UnkownParser {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "Unknown parser: {}", self.0)
    }
}

mod google {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub enum Item {
        Query(String),
        Suggestions(Vec<String>),
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct Json(pub Vec<Item>);
}

fn parse_google(result: String) -> Result<Vec<String>> {
    Ok(vec![result])
}

fn parse_duck(result: String) -> Result<Vec<String>> {
    Ok(vec![result])
}

pub fn complete(query: &String, target: &super::executors::Executor) -> Result<Vec<String>> {
    if query.len() < 3
        || target.suggestion.is_empty()
        || target.completer == super::parser::Parser::NONE
    {
        return Ok(vec![]);
    }

    let result = {
        let response = ureq::get(format!("{}{}", target.suggestion, query).as_str()).call();

        if !response.ok() {
            return Err(FetchError(response.status()).into());
        }

        response.into_string()?
    };

    match &target.completer {
        super::parser::Parser::GOOGLE => parse_google(result),
        super::parser::Parser::DUCK => parse_duck(result),
        super::parser::Parser::NONE => Ok(vec![]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_parsing() {
        let response =  String::from(r#"["bla",["bladet","blake shelton","black","black panther","blake lively","black mirror","blank","bladkongen","blade runner","blacklist"]]"#);
        let _ = parse_google(response);
    }

    #[test]
    fn test_duck_parsing() {
        let response =  String::from(r#"[{"phrase":"gopher football"},{"phrase":"gopher"},{"phrase":"gophersports.com"},{"phrase":"gopher football schedule"},{"phrase":"gopher sports"},{"phrase":"gopher 5 winning numbers"},{"phrase":"gopher football score"},{"phrase":"gopher snake"},{"phrase":"gopher hockey"},{"phrase":"gopher volleyball"}]"#);
        let _ = parse_duck(response);
    }
}
