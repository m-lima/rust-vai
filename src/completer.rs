use super::error;

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

fn parse_google(result: &String) {
    serde_json::from_str::<google::Json>(result.as_str())
        .map(|parsed| println!("{:#?}", parsed.0.first().unwrap()))
        .unwrap_or(());
}

fn parse_duck(_result: &String) {}

impl std::convert::From<curl::Error> for error::Error {
    fn from(e: curl::Error) -> Self {
        error::new("curl", e)
    }
}

fn fetch(url: &String, query: &String) -> Result<String, error::Error> {
    let mut buffer = Vec::new();
    let mut curler = curl::easy::Easy::new();

    {
        let encoded_url = format!("{}{}", &url, &curler.url_encode(query.as_bytes()));
        curler.url(encoded_url.as_str())?;
        curler.get(true)?;
        let mut transfer = curler.transfer();
        transfer.write_function(|data| {
            &buffer.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    match curler.response_code()? {
        200 => Ok(()),
        code => Err(error::new(
            "completer::complete::fetch::curl",
            format!("got response code {}", code),
        )),
    }?;

    String::from_utf8(buffer).map_err(|e| error::new("completer::complete::fetch::parse", e))
}

pub fn complete(query: &String, url: &String, parser_name: &String) -> Result<(), error::Error> {
    if query.len() < 3 || url.is_empty() || parser_name.is_empty() {
        return Err(error::new("completer::complete", "Nothing to parse"));
    }

    let result = fetch(&url, &query)?;

    match parser_name.as_str() {
        "GOOGLE" => parse_google(&result),
        "DUCK" => parse_duck(&result),
        _ => (),
    };
    Ok(())
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
