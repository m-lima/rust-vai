use super::Result;

//pub enum Parser {
//    GOOGLE,
//    DUCK,
//}

#[derive(Debug, Clone)]
struct FetchError(u16);
impl std::error::Error for FetchError {}
impl std::fmt::Display for FetchError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "Got status code {} from suggest query", self.0)
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

fn _parse_google(_result: &String) -> Result<Vec<String>> {
    //    serde_json::from_str::<google::Json>(result.as_str())
    //        .map(|parsed| println!("{:#?}", parsed.0.first().unwrap()))
    //        .unwrap_or(());
    Ok(vec![])
}

fn _parse_duck(_result: &String) -> Result<Vec<String>> {
    Ok(vec![])
}

pub fn _complete(query: &String, url: &String, parser_name: &String) -> Result<Vec<String>> {
    if query.len() < 3 || url.is_empty() || parser_name.is_empty() {
        return Ok(vec![]);
    }

    let result = {
        let response = ureq::get(format!("{}{}", url, query).as_str()).call();

        if !response.ok() {
            return Ok(vec![]);
        }

        response.into_string().map(Some).unwrap_or(None)
    };

    if let Some(result) = result {
        match parser_name.as_str() {
            "GOOGLE" => _parse_google(&result),
            "DUCK" => _parse_duck(&result),
            _ => Ok(vec![]),
        }
    } else {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_parsing() {
        let response =  String::from(r#"["bla",["bladet","blake shelton","black","black panther","blake lively","black mirror","blank","bladkongen","blade runner","blacklist"]]"#);
        let _ = _parse_google(&response);
    }

    #[test]
    fn test_duck_parsing() {
        let response =  String::from(r#"[{"phrase":"gopher football"},{"phrase":"gopher"},{"phrase":"gophersports.com"},{"phrase":"gopher football schedule"},{"phrase":"gopher sports"},{"phrase":"gopher 5 winning numbers"},{"phrase":"gopher football score"},{"phrase":"gopher snake"},{"phrase":"gopher hockey"},{"phrase":"gopher volleyball"}]"#);
        let _ = _parse_duck(&response);
    }
}
