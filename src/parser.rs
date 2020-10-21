use super::error;
use super::Result;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub enum Parser {
    Google,
    Duck,
    None,
}

#[derive(PartialEq, Debug)]
struct Google(Vec<String>);
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
struct Duck(Vec<DuckPhrase>);
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
struct DuckPhrase {
    phrase: String,
}

impl<'a> serde::Deserialize<'a> for Google {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        struct Visitor;

        impl<'a> serde::de::Visitor<'a> for Visitor {
            type Value = Google;

            fn expecting(
                &self,
                fmt: &mut serde::export::Formatter<'_>,
            ) -> std::result::Result<(), serde::export::fmt::Error> {
                write!(fmt, "an array with: the query and an array of suggestions")
            }

            fn visit_seq<V>(self, mut visitor: V) -> std::result::Result<Self::Value, V::Error>
            where
                V: serde::de::SeqAccess<'a>,
            {
                // Ignored the first element (query)
                visitor.next_element::<String>()?;

                let phrases = visitor
                    .next_element::<Vec<String>>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;

                // Ignore the trailing objects, if any
                while let Some(serde::de::IgnoredAny) = visitor.next_element()? {}

                Ok(Google(phrases))
            }
        }
        deserializer.deserialize_seq(Visitor)
    }
}

impl Duck {
    #[inline]
    fn phrases(self) -> Vec<DuckPhrase> {
        self.0
    }
}

impl DuckPhrase {
    #[inline]
    fn phrase(self) -> String {
        self.phrase
    }
}

pub fn parse(parser: &Parser, result: &str) -> Result<Vec<String>> {
    match parser {
        Parser::Google => Ok(serde_json::from_str::<Google>(result)
            .map_err(error::parse)?
            .0),
        Parser::Duck => Ok(serde_json::from_str::<Duck>(result)
            .map(Duck::phrases)
            .map_err(error::parse)?
            .into_iter()
            .map(DuckPhrase::phrase)
            .collect()),
        Parser::None => Ok(vec![]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_parsing() {
        let result = r#"["bla",["bladet","blake shelton","black","black panther","blake lively","black mirror","blank","bladkongen","blade runner","blacklist"]]"#;
        let suggestions = parse(&Parser::Google, result).unwrap();
        assert_eq!(suggestions.len(), 10);
        assert_eq!(suggestions[0], "bladet");
        assert_eq!(suggestions[9], "blacklist");
    }

    #[test]
    fn test_google_parsing_extended() {
        let result = r#"["bla",["black","black widow","blake lively","bladet","blackpink","blaafarvev�rket","blacklist","black panther","black box teater","blazer"],[],{"google:suggestsubtypes":[[433],[433],[433],[433,131],[433,131],[],[433],[433],[],[]]}]"#;
        let suggestions = parse(&Parser::Google, result).unwrap();
        assert_eq!(suggestions.len(), 10);
        assert_eq!(suggestions[0], "black");
        assert_eq!(suggestions[9], "blazer");
    }

    #[test]
    fn test_duck_parsing() {
        let result = r#"[{"phrase":"gopher football"},{"phrase":"gopher"},{"phrase":"gophersports.com"},{"phrase":"gopher football schedule"},{"phrase":"gopher sports"},{"phrase":"gopher 5 winning numbers"},{"phrase":"gopher football score"},{"phrase":"gopher snake"},{"phrase":"gopher hockey"},{"phrase":"gopher volleyball"}]"#;
        let suggestions = parse(&Parser::Duck, result).unwrap();
        assert_eq!(suggestions.len(), 10);
        assert_eq!(suggestions[0], "gopher football");
        assert_eq!(suggestions[9], "gopher volleyball");
    }
}
