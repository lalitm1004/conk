use pest::iterators::Pair;

use crate::ast::{Error, data_structures::value::parse_argument, pest::Rule};

use super::Argument;

#[derive(Debug, Clone)]
pub struct FieldAttribute {
    pub name: String,
    pub arguments: Vec<Argument>,
}

#[derive(Debug, Clone)]
pub struct BlockAttribute {
    pub name: String,
    pub arguments: Vec<Argument>,
}

pub trait AttributeLike {
    fn new(name: String, arguments: Vec<Argument>) -> Self;
}

impl AttributeLike for FieldAttribute {
    fn new(name: String, arguments: Vec<Argument>) -> Self {
        Self { name, arguments }
    }
}

impl AttributeLike for BlockAttribute {
    fn new(name: String, arguments: Vec<Argument>) -> Self {
        Self { name, arguments }
    }
}

pub fn parse_attribute<T: AttributeLike>(pair: Pair<Rule>) -> Result<T, Error> {
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();

    let arguments = match inner.next() {
        Some(seq_pair) => seq_pair
            .into_inner()
            .filter(|p| p.as_rule() == Rule::argument)
            .map(parse_argument)
            .collect::<Result<Vec<_>, _>>()?,
        None => vec![],
    };

    Ok(T::new(name, arguments))
}

#[cfg(test)]
mod test {
    use pest::Parser;

    use crate::ast::ConkParser;

    use super::*;

    // -------------------------------------------------------------------------
    // Attribute parsing
    // -------------------------------------------------------------------------

    fn parse_attr_str<T: AttributeLike>(rule: Rule, input: &str) -> Result<T, Error> {
        let mut pairs = ConkParser::parse(rule, input)?;
        let pair = pairs.next().unwrap();
        if pair.as_span().end() != input.len() {
            return Err(pest::error::Error::<Rule>::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: "trailing input".into(),
                },
                pair.as_span(),
            )
            .into());
        }
        parse_attribute::<T>(pair)
    }

    #[test]
    fn field_attribute_cases() {
        let ok = ["@a", "@a()", "@a(1)", "@a(1,2)", "@a(x:1)", "@a(1,x:2)"];
        let err = ["@@a", "a()", "@a((2)", "@a(1,2))"];

        for input in ok {
            let res = parse_attr_str::<FieldAttribute>(Rule::field_attribute, input);
            assert!(res.is_ok(), "Expected Ok got Err on {:?}", input);
        }

        for input in err {
            let res = parse_attr_str::<FieldAttribute>(Rule::field_attribute, input);
            assert!(res.is_err(), "Expected Err got Ok on {:?}", input)
        }
    }

    #[test]
    fn block_attribute_cases() {
        let ok = ["@@a", "@@a()", "@@a(1)", "@@a(x:1)", "@@a(1,x:2,)"];
        let err = ["@@", "@@a(", "@@1", "@a())"];

        for input in ok {
            let res = parse_attr_str::<BlockAttribute>(Rule::block_attribute, input);
            assert!(res.is_ok(), "Expected Ok got Err on {:?}", input);
        }

        for input in err {
            let res = parse_attr_str::<BlockAttribute>(Rule::block_attribute, input);
            assert!(res.is_err(), "Expected Err got Ok on {:?}", input)
        }
    }

    #[test]
    fn attribute_structure() {
        let attr =
            parse_attr_str::<FieldAttribute>(Rule::field_attribute, "@test(1, x:2)").unwrap();

        assert_eq!(attr.name, "test");
        assert_eq!(attr.arguments.len(), 2);
    }
}
