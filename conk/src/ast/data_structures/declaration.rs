use pest::iterators::Pair;

use crate::ast::{
    Error,
    data_structures::{attribute::parse_attribute, field::parse_field},
    pest::Rule,
};

use super::{BlockAttribute, Field};

#[derive(Debug, Clone)]
pub struct EnumDeclaration {
    pub schema: Option<String>,
    pub name: String,
    pub values: Vec<String>,
    pub block_attributes: Vec<BlockAttribute>,
}

#[derive(Debug, Clone)]
pub struct TemplateDeclaration {
    pub name: String,
    pub fields: Vec<Field>,
    pub block_attributes: Vec<BlockAttribute>,
}

#[derive(Debug, Clone)]
pub struct EntityDeclaration {
    pub schema: Option<String>,
    pub name: String,

    pub templates: Vec<String>,
    pub inherits: Vec<String>,

    pub fields: Vec<Field>,
    pub block_attributes: Vec<BlockAttribute>,
}

fn parse_schema(pair: Pair<Rule>) -> String {
    let raw = pair.as_str();
    raw.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap()
        .replace("\\\"", "\"")
}

pub fn parse_enum_declaration(pair: Pair<Rule>) -> Result<EnumDeclaration, Error> {
    let mut schema = None;
    let mut name = String::new();
    let mut values = Vec::new();
    let mut block_attributes = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::schema => schema = Some(parse_schema(inner_pair)),
            Rule::identifier => name = inner_pair.as_str().to_string(),
            Rule::enum_value => {
                values.push(inner_pair.into_inner().next().unwrap().as_str().to_string())
            }
            Rule::block_attribute => block_attributes.push(parse_attribute(inner_pair)?),
            _ => unreachable!(),
        }
    }

    Ok(EnumDeclaration {
        schema,
        name,
        values,
        block_attributes,
    })
}

pub fn parse_template_declaration(pair: Pair<Rule>) -> Result<TemplateDeclaration, Error> {
    let mut name = String::new();
    let mut fields = Vec::new();
    let mut block_attributes = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::identifier => name = inner_pair.as_str().to_string(),
            Rule::field => fields.push(parse_field(inner_pair)?),
            Rule::block_attribute => block_attributes.push(parse_attribute(inner_pair)?),
            _ => unreachable!(),
        }
    }

    Ok(TemplateDeclaration {
        name,
        fields,
        block_attributes,
    })
}

pub fn parse_entity_declaration(pair: Pair<Rule>) -> Result<EntityDeclaration, Error> {
    let mut schema = None;
    let mut name = String::new();
    let mut templates = Vec::new();
    let mut inherits = Vec::new();
    let mut fields = Vec::new();
    let mut block_attributes = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::schema => schema = Some(parse_schema(inner_pair)),
            Rule::identifier => name = inner_pair.as_str().to_string(),
            Rule::template_usage => {
                for ident_pair in inner_pair.into_inner().next().unwrap().into_inner() {
                    templates.push(ident_pair.as_str().to_string());
                }
            }
            Rule::inheritance_usage => {
                for ident_pair in inner_pair.into_inner().next().unwrap().into_inner() {
                    let ident_str = match ident_pair.as_rule() {
                        Rule::schema_identifier => {
                            let mut parts = ident_pair.into_inner();
                            let s = parse_schema(parts.next().unwrap());
                            let n = parts.next().unwrap().as_str();
                            format!("\"{}\".{}", s, n)
                        }
                        Rule::identifier => ident_pair.as_str().to_string(),
                        _ => unreachable!(),
                    };
                    inherits.push(ident_str);
                }
            }
            Rule::field => fields.push(parse_field(inner_pair)?),
            Rule::block_attribute => block_attributes.push(parse_attribute(inner_pair)?),
            _ => unreachable!(),
        }
    }

    Ok(EntityDeclaration {
        schema,
        name,
        templates,
        inherits,
        fields,
        block_attributes,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::ConkParser;
    use pest::Parser;

    fn parse_enum_str(input: &str) -> Result<EnumDeclaration, Error> {
        let mut pairs = ConkParser::parse(Rule::enum_declaration, input)?;
        let pair = pairs.next().unwrap();
        if pair.as_span().end() != input.len() {
            return Err(pest::error::Error::<Rule>::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: "trailing".into(),
                },
                pair.as_span(),
            )
            .into());
        }
        parse_enum_declaration(pair)
    }

    fn parse_template_str(input: &str) -> Result<TemplateDeclaration, Error> {
        let mut pairs = ConkParser::parse(Rule::template_declaration, input)?;
        let pair = pairs.next().unwrap();
        if pair.as_span().end() != input.len() {
            return Err(pest::error::Error::<Rule>::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: "trailing".into(),
                },
                pair.as_span(),
            )
            .into());
        }
        parse_template_declaration(pair)
    }

    fn parse_entity_str(input: &str) -> Result<EntityDeclaration, Error> {
        let mut pairs = ConkParser::parse(Rule::entity_declaration, input)?;
        let pair = pairs.next().unwrap();
        if pair.as_span().end() != input.len() {
            return Err(pest::error::Error::<Rule>::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: "trailing".into(),
                },
                pair.as_span(),
            )
            .into());
        }
        parse_entity_declaration(pair)
    }

    #[test]
    fn enum_cases() {
        let ok = [
            "enum Color { RED GREEN BLUE }",
            "\"db\" enum Status { @@comment(\"x\") ACTIVE INACTIVE }",
        ];

        let err = [
            "enum Color",
            "enum Color {}",
            "enum { RED }",
            "\"db\".enum Color { RED }",
            "enum Color { RED GREEN, }",
            "enum Color { RED, GREEN }",
            "enum Color { @@doc RED @invalid }",
        ];

        for input in ok {
            let res = parse_enum_str(input);
            assert!(
                res.is_ok(),
                "Expected Ok for {:?}, got Err: {:?}",
                input,
                res
            );
        }

        for input in err {
            let res = parse_enum_str(input);
            assert!(
                res.is_err(),
                "Expected Err for {:?}, got Ok: {:?}",
                input,
                res
            );
        }
    }

    #[test]
    fn enum_structure() {
        let e = parse_enum_str("\"db\" enum Color { @@doc(\"hi\") RED GREEN }").unwrap();
        assert_eq!(e.schema.unwrap(), "db");
        assert_eq!(e.name, "Color");
        assert_eq!(e.values, vec!["RED", "GREEN"]);
        assert_eq!(e.block_attributes.len(), 1);
    }

    #[test]
    fn template_cases() {
        let ok = [
            "template Timestampable { created_at DateTime @@index(created_at) }",
            "template TestingMoreTemplates { @@something }",
        ];

        let err = [
            "template T",
            "template { id String }",
            "template T ( id String )",
            "template T { id String, }",
            "template T { @@doc(\"hi\") @invalid }",
        ];

        for input in ok {
            let res = parse_template_str(input);
            assert!(
                res.is_ok(),
                "Expected Ok for {:?}, got Err: {:?}",
                input,
                res
            );
        }

        for input in err {
            let res = parse_template_str(input);
            assert!(
                res.is_err(),
                "Expected Err for {:?}, got Ok: {:?}",
                input,
                res
            );
        }
    }

    #[test]
    fn template_structure() {
        let t = parse_template_str("template Timestampable { created_at DateTime }").unwrap();
        assert_eq!(t.name, "Timestampable");
        assert_eq!(t.fields.len(), 1);
        assert_eq!(t.fields[0].name, "created_at");
    }

    #[test]
    fn entity_cases() {
        let ok = [
            "entity User { id String }",
            "\"public\" entity User { id String }",
            "entity User : Timestampable {}",
            "entity User : T1, T2 {}",
            "entity User inherits (Base) {}",
            "entity User inherits (\"schema\".Base) {}",
            "entity Admin : Timestampable inherits (User) { role String }",
            "entity E : T1, { id String }",
        ];

        let err = [
            "entity E",
            "\"db\".entity E { id String }",
            "entity { id String }",
            "entity E : { id String }",
            "entity E inherits { id String }",
        ];

        for input in ok {
            let res = parse_entity_str(input);
            assert!(
                res.is_ok(),
                "Expected Ok for {:?}, got Err: {:?}",
                input,
                res
            );
        }

        for input in err {
            let res = parse_entity_str(input);
            assert!(
                res.is_err(),
                "Expected Err for {:?}, got Ok: {:?}",
                input,
                res
            );
        }
    }

    #[test]
    fn entity_structure() {
        let e = parse_entity_str("entity User : Timestampable inherits (Person) { id String }")
            .unwrap();
        assert_eq!(e.name, "User");
        assert_eq!(e.templates, vec!["Timestampable"]);
        assert_eq!(e.inherits, vec!["Person"]);
        assert_eq!(e.fields.len(), 1);
        assert_eq!(e.fields[0].name, "id");
    }
}
