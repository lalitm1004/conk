use pest::iterators::Pair;

use crate::ast::{
    Error,
    data_structures::{attribute::parse_attribute, value::parse_value},
    pest::Rule,
};

use super::{FieldAttribute, Value};

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub field_attributes: Vec<FieldAttribute>,
}

#[derive(Debug, Clone)]
pub struct FieldType {
    pub schema: Option<String>,
    pub name: String,
    pub parameters: Vec<Value>,
}

pub fn parse_field(pair: Pair<Rule>) -> Result<Field, Error> {
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let type_pair = inner.next().unwrap();
    let field_type = parse_field_type(type_pair)?;

    let field_attributes = inner
        .map(|p| parse_attribute(p))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Field {
        name,
        field_type,
        field_attributes,
    })
}

pub fn parse_field_type(pair: Pair<Rule>) -> Result<FieldType, Error> {
    let mut inner = pair.into_inner();

    let name_pair = inner.next().unwrap();
    let (schema, name) = match name_pair.as_rule() {
        Rule::schema_identifier => {
            let mut parts = name_pair.into_inner();
            let raw_schema = parts.next().unwrap().as_str();
            let schema = raw_schema
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .unwrap()
                .replace("\\\"", "\"");
            let name = parts.next().unwrap().as_str().to_string();
            (Some(schema), name)
        }
        Rule::identifier => (None, name_pair.as_str().to_string()),
        _ => unreachable!(),
    };

    let parameters = match inner.next() {
        Some(params_pair) => {
            if let Some(seq) = params_pair.into_inner().next() {
                seq.into_inner()
                    .map(parse_value)
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                vec![]
            }
        }
        None => vec![],
    };

    Ok(FieldType {
        schema,
        name,
        parameters,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::ConkParser;
    use pest::Parser;

    fn parse_field_str(input: &str) -> Result<Field, Error> {
        let mut pairs = ConkParser::parse(Rule::field, input)?;
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

        parse_field(pair)
    }

    fn parse_field_type_str(input: &str) -> Result<FieldType, Error> {
        let mut pairs = ConkParser::parse(Rule::field_type, input)?;
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

        parse_field_type(pair)
    }

    #[test]
    fn field_type_cases() {
        let ok = [
            "String",
            "\"schema\".String",
            "String()",
            "String(10)",
            "String(10, 20)",
            "String(1,)",
        ];

        let err = [
            "",
            "123",
            "\"schema\".",
            "String(",
            "String)",
            "String(1 2)",
        ];

        for input in ok {
            let res = parse_field_type_str(input);
            assert!(
                res.is_ok(),
                "Expected Ok for {:?}, got Err: {:?}",
                input,
                res
            );
        }

        for input in err {
            let res = parse_field_type_str(input);
            assert!(
                res.is_err(),
                "Expected Err for {:?}, got Ok: {:?}",
                input,
                res
            );
        }
    }

    #[test]
    fn field_cases() {
        let ok = [
            "id String",
            "id String @unique",
            "id String(10) @unique @default(1)",
        ];

        let err = [
            "",
            "id",
            "id String @",
            "1id String",
            "id \"schema\".",
            "id String(1",
            "id String() @",
        ];

        for input in ok {
            let res = parse_field_str(input);
            assert!(
                res.is_ok(),
                "Expected Ok for {:?}, got Err: {:?}",
                input,
                res
            );
        }

        for input in err {
            let res = parse_field_str(input);
            assert!(
                res.is_err(),
                "Expected Err for {:?}, got Ok: {:?}",
                input,
                res
            );
        }
    }

    #[test]
    fn field_structure() {
        let f = parse_field_str("id String(10) @unique").unwrap();
        assert_eq!(f.name, "id");
        assert_eq!(f.field_type.name, "String");
        assert_eq!(f.field_type.parameters.len(), 1);
        assert_eq!(f.field_attributes.len(), 1);
    }
}
