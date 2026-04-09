use pest::iterators::Pair;

use crate::ast::{Error, pest::Rule};

#[derive(Debug, Clone)]
pub enum Value {
    Float(f64),

    Integer(i64),

    List(Vec<Value>),

    FunctionCall {
        schema: Option<String>,
        name: String,
        arguments: Vec<Argument>,
    },

    QualifiedIdentifier {
        schema: Option<String>,
        identifier: String,
        qualifications: Vec<String>,
    },

    Identifier {
        schema: Option<String>,
        identifier: String,
    },

    String(String),
}

#[derive(Debug, Clone)]
pub enum Argument {
    Positional { value: Value },
    Named { name: String, value: Value },
}

pub fn parse_value(pair: Pair<Rule>) -> Result<Value, Error> {
    match pair.as_rule() {
        Rule::float_literal => Ok(Value::Float(pair.as_str().parse()?)),

        Rule::integer_literal => Ok(Value::Integer(pair.as_str().parse()?)),

        Rule::list_literal => {
            let mut items = vec![];

            if let Some(seq) = pair.into_inner().next() {
                for item in seq.into_inner() {
                    items.push(parse_value(item)?);
                }
            }

            Ok(Value::List(items))
        }

        Rule::function_call => {
            let mut inner = pair.into_inner();

            let first = inner.next().unwrap();

            let (schema, name) = match first.as_rule() {
                Rule::identifier => (None, first.as_str().to_string()),

                Rule::schema_identifier => {
                    let mut parts = first.into_inner();
                    let schema = parts.next().unwrap().as_str().to_string();
                    let name = parts.next().unwrap().as_str().to_string();
                    (Some(schema), name)
                }

                _ => unreachable!(),
            };

            let arguments = match inner.next() {
                Some(seq_pair) => seq_pair
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::argument)
                    .map(parse_argument)
                    .collect::<Result<Vec<_>, _>>()?,
                None => vec![],
            };

            Ok(Value::FunctionCall {
                schema,
                name,
                arguments,
            })
        }

        Rule::schema_qualified_identifier => {
            let mut parts = pair.into_inner();

            let raw_schema = parts.next().unwrap().as_str();

            let schema = raw_schema
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .unwrap()
                .replace("\\\"", "\"");

            let mut q = parts.next().unwrap().into_inner();

            let base = q.next().unwrap().as_str().to_string();
            let qualifications = q.map(|p| p.as_str().to_string()).collect();

            Ok(Value::QualifiedIdentifier {
                schema: Some(schema),
                identifier: base,
                qualifications,
            })
        }

        Rule::qualified_identifier => {
            let mut parts = pair.into_inner();

            let base = parts.next().unwrap().as_str().to_string();

            let qualifications = parts.map(|p| p.as_str().to_string()).collect();

            Ok(Value::QualifiedIdentifier {
                schema: None,
                identifier: base,
                qualifications,
            })
        }

        Rule::schema_identifier => {
            let mut parts = pair.into_inner();

            let raw_schema = parts.next().unwrap().as_str();

            let schema = raw_schema
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .unwrap()
                .replace("\\\"", "\"");

            let identifier = parts.next().unwrap().as_str().to_string();

            Ok(Value::Identifier {
                schema: Some(schema),
                identifier,
            })
        }

        Rule::identifier => Ok(Value::Identifier {
            schema: None,
            identifier: pair.as_str().to_string(),
        }),

        Rule::string_literal => {
            let raw = pair.as_str();

            // Remove surrounding quotes
            let inner = &raw[1..raw.len() - 1];

            // Handle escaped quotes
            let value = inner.replace("\\\"", "\"");

            Ok(Value::String(value))
        }

        _ => unreachable!(),
    }
}

pub fn parse_argument(pair: Pair<Rule>) -> Result<Argument, Error> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::named_argument => {
            let mut parts = inner.into_inner();
            let name = parts.next().unwrap().as_str().to_string();
            let value = parse_value(parts.next().unwrap())?;
            Ok(Argument::Named { name, value })
        }

        Rule::positional_argument => {
            let value_pair = inner.into_inner().next().unwrap();
            let value = parse_value(value_pair)?;
            Ok(Argument::Positional { value })
        }

        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::ConkParser;
    use pest::{
        Parser,
        error::{Error as PestError, ErrorVariant},
    };

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    fn parse_value_str(rule: Rule, input: &str) -> Result<Value, Error> {
        let mut pairs = ConkParser::parse(rule, input)?;
        let pair = pairs.next().unwrap();

        if pair.as_span().end() != input.len() {
            return Err(PestError::new_from_span(
                ErrorVariant::CustomError {
                    message: "unexpected trailing input".into(),
                },
                pair.as_span(),
            )
            .into());
        }

        parse_value(pair)
    }

    fn assert_value_ok(rule: Rule, input: &str) {
        let res = parse_value_str(rule, input);
        assert!(
            res.is_ok(),
            "Expected Ok for {:?}, got Err: {:?}",
            input,
            res
        );
    }

    fn assert_value_err(rule: Rule, input: &str) {
        let res = parse_value_str(rule, input);
        assert!(
            res.is_err(),
            "Expected Err for {:?}, got Ok: {:?}",
            input,
            res
        );
    }

    fn parse_argument_str(input: &str) -> Result<Argument, Error> {
        let mut pairs = ConkParser::parse(Rule::argument, input)?;
        let pair = pairs.next().unwrap();

        if pair.as_span().end() != input.len() {
            return Err(PestError::new_from_span(
                ErrorVariant::CustomError {
                    message: "unexpected trailing input".into(),
                },
                pair.as_span(),
            )
            .into());
        }

        parse_argument(pair)
    }

    fn assert_argument_ok(input: &str) {
        let res = parse_argument_str(input);
        assert!(
            res.is_ok(),
            "Expected Ok for {:?}, got Err: {:?}",
            input,
            res
        );
    }

    fn assert_argument_err(input: &str) {
        let res = parse_argument_str(input);
        assert!(
            res.is_err(),
            "Expected Err for {:?}, got Ok: {:?}",
            input,
            res
        );
    }

    // -------------------------------------------------------------------------
    // Integer
    // -------------------------------------------------------------------------

    #[test]
    fn integer_cases() {
        let ok = ["0", "42", "+7", "-15"];
        let err = ["", "+", "-", "abc", "12.3"];

        for input in ok {
            assert_value_ok(Rule::integer_literal, input);
        }

        for input in err {
            assert_value_err(Rule::integer_literal, input);
        }
    }

    #[test]
    fn integer_overflow() {
        let input = "999999999999999999999999999999";
        let err = parse_value_str(Rule::integer_literal, input).unwrap_err();

        match err {
            Error::ParseInt(_) => {}
            other => panic!("Expected ParseInt error, got {:?}", other),
        }
    }

    // -------------------------------------------------------------------------
    // Float
    // -------------------------------------------------------------------------

    #[test]
    fn float_cases() {
        let ok = ["0.0", "1.23", "+3.14", "-0.01"];
        let err = ["", "123", ".", "1.", ".5", "abc", "1.2.3"];

        for input in ok {
            assert_value_ok(Rule::float_literal, input);
        }

        for input in err {
            assert_value_err(Rule::float_literal, input);
        }
    }

    // -------------------------------------------------------------------------
    // List
    // -------------------------------------------------------------------------

    #[test]
    fn list_cases() {
        let ok = [
            "[]",
            "[1]",
            "[1,2,3]",
            "[1,2,3,]",
            "[0,-1,+2]",
            "[1.0,2.5]",
            "[[1,2],[3,4]]",
            "[[[]]]",
        ];

        let err = [
            "",
            "[",
            "]",
            "[1",
            "1]",
            "[,]",
            "[1,,2]",
            "[1 2]",
            "[[1], [2],",
            "[[3], [4],]]",
        ];

        for input in ok {
            assert_value_ok(Rule::list_literal, input);
        }

        for input in err {
            assert_value_err(Rule::list_literal, input);
        }
    }

    // -------------------------------------------------------------------------
    // Argument
    // -------------------------------------------------------------------------

    #[test]
    fn argument_cases() {
        let ok = [
            "42",
            "-7",
            "3.14",
            "[]",
            "[1,2]",
            "foo()",
            "a:1",
            "x:3.14",
            "items:[1,2]",
            "fn:foo(1)",
            "a : 1",
        ];

        let err = ["", ":1", "a:"];

        for input in ok {
            assert_argument_ok(input);
        }

        for input in err {
            assert_argument_err(input);
        }
    }

    #[test]
    fn argument_structure_named() {
        let arg = parse_argument_str("a:1").unwrap();

        match arg {
            Argument::Named { name, value } => {
                assert_eq!(name, "a");
                assert!(matches!(value, Value::Integer(1)));
            }
            _ => panic!("expected named argument"),
        }
    }

    #[test]
    fn argument_structure_positional() {
        let arg = parse_argument_str("3.14").unwrap();

        match arg {
            Argument::Positional {
                value: Value::Float(f),
            } => {
                assert!((f - 3.14).abs() < f64::EPSILON);
            }
            _ => panic!("expected positional float"),
        }
    }

    // -------------------------------------------------------------------------
    // Function calls
    // -------------------------------------------------------------------------

    #[test]
    fn function_call_valid_cases() {
        let cases = [
            "foo()",
            "foo(1)",
            "foo(1,2,3)",
            "foo(a:1)",
            "foo(1,a:2)",
            "foo(bar())",
            "foo(bar(1),baz(2))",
            "\"s\".foo(1)",
        ];

        for input in cases {
            assert_value_ok(Rule::value, input);
        }
    }

    #[test]
    fn function_call_invalid_cases() {
        let cases = ["foo(", "foo)", "foo(,)", "foo(1,,2)", "foo(1 2)"];

        for input in cases {
            assert_value_err(Rule::value, input);
        }
    }

    #[test]
    fn function_call_structure() {
        let v = parse_value_str(Rule::value, "foo(1, a:2)").unwrap();

        match v {
            Value::FunctionCall {
                name, arguments, ..
            } => {
                assert_eq!(name, "foo");
                assert_eq!(arguments.len(), 2);

                assert!(matches!(
                    arguments[0],
                    Argument::Positional {
                        value: Value::Integer(1)
                    }
                ));

                assert!(matches!(
                    arguments[1],
                    Argument::Named { ref name, value: Value::Integer(2) }
                    if name == "a"
                ));
            }
            _ => panic!("expected FunctionCall"),
        }
    }

    #[test]
    fn function_call_nested() {
        let v = parse_value_str(Rule::value, "a(b(c(1)))").unwrap();

        match v {
            Value::FunctionCall {
                name, arguments, ..
            } => {
                assert_eq!(name, "a");
                assert_eq!(arguments.len(), 1);
            }
            _ => panic!("expected nested FunctionCall"),
        }
    }

    #[test]
    fn function_call_trailing_comma() {
        let v = parse_value_str(Rule::value, "foo(1,2,3,)").unwrap();

        match v {
            Value::FunctionCall { arguments, .. } => {
                assert_eq!(arguments.len(), 3);
            }
            _ => panic!("expected FunctionCall"),
        }
    }

    // -------------------------------------------------------------------------
    // Schema Qualified Identifier
    // -------------------------------------------------------------------------

    #[test]
    fn schema_qualified_identifier_cases() {
        let ok = ["\"s\".a.b", "\"schema\".x.y.z"];

        let err = ["a.b.c", "\"s\".a", "\"s\".", "\"s\".a."];

        for input in ok {
            assert_value_ok(Rule::schema_qualified_identifier, input);
        }

        for input in err {
            assert_value_err(Rule::schema_qualified_identifier, input);
        }
    }

    #[test]
    fn schema_qualified_identifier_structure() {
        let v = parse_value_str(Rule::schema_qualified_identifier, "\"s\".a.b").unwrap();

        match v {
            Value::QualifiedIdentifier {
                schema,
                identifier,
                qualifications,
            } => {
                assert_eq!(schema.unwrap(), "s");
                assert_eq!(identifier, "a");
                assert_eq!(qualifications, vec!["b"]);
            }
            _ => panic!("expected schema qualified identifier"),
        }
    }

    // -------------------------------------------------------------------------
    // Qualified Identifier
    // -------------------------------------------------------------------------

    #[test]
    fn qualified_identifier_cases() {
        let ok = ["a.b", "a.b.c", "root.child.leaf"];

        let err = ["a", ".b", "a.", "a..b"];

        for input in ok {
            assert_value_ok(Rule::qualified_identifier, input);
        }

        for input in err {
            assert_value_err(Rule::qualified_identifier, input);
        }
    }

    #[test]
    fn qualified_identifier_structure() {
        let v = parse_value_str(Rule::qualified_identifier, "a.b.c").unwrap();

        match v {
            Value::QualifiedIdentifier {
                schema,
                identifier,
                qualifications,
            } => {
                assert!(schema.is_none());
                assert_eq!(identifier, "a");
                assert_eq!(qualifications, vec!["b", "c"]);
            }
            _ => panic!("expected qualified identifier"),
        }
    }

    // -------------------------------------------------------------------------
    // Schema Identifier
    // -------------------------------------------------------------------------

    #[test]
    fn schema_identifier_cases() {
        let ok = ["\"s\".a", "\"schema\".name"];

        let err = ["a.b", "\"s\".", ".a"];

        for input in ok {
            assert_value_ok(Rule::schema_identifier, input);
        }

        for input in err {
            assert_value_err(Rule::schema_identifier, input);
        }
    }

    #[test]
    fn schema_identifier_structure() {
        let v = parse_value_str(Rule::schema_identifier, "\"s\".foo").unwrap();

        match v {
            Value::Identifier { schema, identifier } => {
                assert_eq!(schema.unwrap(), "s");
                assert_eq!(identifier, "foo");
            }
            _ => panic!("expected schema identifier"),
        }
    }

    // -------------------------------------------------------------------------
    // Identifier
    // -------------------------------------------------------------------------

    #[test]
    fn identifier_cases() {
        let ok = ["a", "abc", "_x", "a1", "_123"];

        let err = ["", "1a", "-", "a-b"];

        for input in ok {
            assert_value_ok(Rule::identifier, input);
        }

        for input in err {
            assert_value_err(Rule::identifier, input);
        }
    }

    #[test]
    fn identifier_structure() {
        let v = parse_value_str(Rule::identifier, "foo").unwrap();

        match v {
            Value::Identifier { schema, identifier } => {
                assert!(schema.is_none());
                assert_eq!(identifier, "foo");
            }
            _ => panic!("expected identifier"),
        }
    }

    // -------------------------------------------------------------------------
    // String
    // -------------------------------------------------------------------------

    #[test]
    fn string_cases() {
        let ok = [
            "\"\"",
            "\"hello\"",
            "\"with spaces\"",
            "\"escaped \\\" quote\"",
        ];

        let err = ["", "\"", "hello", "\"unterminated"];

        for input in ok {
            assert_value_ok(Rule::string_literal, input);
        }

        for input in err {
            assert_value_err(Rule::string_literal, input);
        }
    }

    #[test]
    fn string_structure() {
        let v = parse_value_str(Rule::string_literal, "\"abc\"").unwrap();

        match v {
            Value::String(s) => assert_eq!(s, "abc"),
            _ => panic!("expected string"),
        }
    }
}
