use pest::iterators::Pair;

use crate::ast::{Error, data_structures::value::parse_value, pest::Rule};

use super::Value;

#[derive(Debug, Clone)]
pub struct Config {
    pub entries: Vec<(String, Value)>,
}

pub fn parse_config(pair: Pair<Rule>) -> Result<Config, Error> {
    let mut entries = Vec::new();
    for entry_pair in pair.into_inner() {
        if entry_pair.as_rule() == Rule::config_entry {
            let mut inner = entry_pair.into_inner();
            let identifier = inner.next().unwrap().as_str().to_string();
            let value = parse_value(inner.next().unwrap())?;
            entries.push((identifier, value));
        }
    }
    Ok(Config { entries })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::ConkParser;
    use pest::Parser;

    fn parse_config_str(input: &str) -> Result<Config, Error> {
        let mut pairs = ConkParser::parse(Rule::config, input)?;
        let pair = pairs.next().unwrap();

        if pair.as_span().end() != input.len() {
            return Err(pest::error::Error::<Rule>::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: "unexpected trailing input".into(),
                },
                pair.as_span(),
            )
            .into());
        }

        parse_config(pair)
    }

    #[test]
    fn config_cases() {
        let ok = [
            "config {}",
            "config { a: 1 }",
            "config { a: 1 b: 2 }",
            "config { a: 1 \n b: 2 }",
        ];

        let err = [
            "config",
            "config {",
            "config }",
            "config { a }",
            "config { a: }",
            "config { : 1 }",
            "config ( a: 1 )",
            "config { a: 1, }",
            "config { a: 1 b: 2 c }",
            "config { a: 1 }}",
            "config {{ a: 1 }",
            "{ a: 1 }",
        ];

        for input in ok {
            let res = parse_config_str(input);
            assert!(
                res.is_ok(),
                "Expected Ok for {:?}, got Err: {:?}",
                input,
                res
            );
        }

        for input in err {
            let res = parse_config_str(input);
            assert!(
                res.is_err(),
                "Expected Err for {:?}, got Ok: {:?}",
                input,
                res
            );
        }
    }

    #[test]
    fn config_structure() {
        let cfg = parse_config_str("config { a: 1 b: 2 }").unwrap();
        assert_eq!(cfg.entries.len(), 2);
        assert_eq!(cfg.entries[0].0, "a");
        assert!(matches!(cfg.entries[0].1, Value::Integer(1)));
        assert_eq!(cfg.entries[1].0, "b");
        assert!(matches!(cfg.entries[1].1, Value::Integer(2)));
    }
}
