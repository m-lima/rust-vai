pub enum Parser {
    GOOGLE,
    DUCK,
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

fn parse_google(_result: &String) -> Vec<String> {
    //    serde_json::from_str::<google::Json>(result.as_str())
    //        .map(|parsed| println!("{:#?}", parsed.0.first().unwrap()))
    //        .unwrap_or(());
    vec![]
}

fn parse_duck(_result: &String) -> Vec<String> {
    vec![]
}

pub fn complete(query: &String, url: &String, parser_name: &String) -> Vec<String> {
    if query.len() < 3 || url.is_empty() || parser_name.is_empty() {
        return vec![];
    }

    let result = {
        let response = ureq::get(format!("{}{}", url, query).as_str()).call();

        if !response.ok() {
            return vec![];
        }

        response.into_string().map(Some).unwrap_or(None)
    };

    if let Some(result) = result {
        match parser_name.as_str() {
            "GOOGLE" => parse_google(&result),
            "DUCK" => parse_duck(&result),
            _ => vec![],
        }
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_parsing() {
        let response =  String::from(r#"["bla",["bladet","blake shelton","black","black panther","blake lively","black mirror","blank","bladkongen","blade runner","blacklist"]]"#);
        parse_google(&response);
    }

    #[test]
    fn test_duck_parsing() {
        let response =  String::from(r#"[{"phrase":"gopher football"},{"phrase":"gopher"},{"phrase":"gophersports.com"},{"phrase":"gopher football schedule"},{"phrase":"gopher sports"},{"phrase":"gopher 5 winning numbers"},{"phrase":"gopher football score"},{"phrase":"gopher snake"},{"phrase":"gopher hockey"},{"phrase":"gopher volleyball"}]"#);
        parse_duck(&response);
    }
}
