mod attribute;
mod config;
mod declaration;
mod field;
mod value;

use std::path::Path;

pub use attribute::{BlockAttribute, FieldAttribute};
pub use config::Config;
pub use declaration::{EntityDeclaration, EnumDeclaration, TemplateDeclaration};
pub use field::{Field, FieldType};
pub use value::{Argument, Value};

use crate::ast::{ConkParser, Error, pest::Rule};

use pest::Parser;

#[derive(Debug, Clone)]
pub struct ConkAST {
    pub config: Option<Config>,
    pub enum_declarations: Vec<EnumDeclaration>,
    pub template_declarations: Vec<TemplateDeclaration>,
    pub entity_declarations: Vec<EntityDeclaration>,
}

pub fn parse_ast_from_file(file_path: impl AsRef<Path>) -> Result<ConkAST, Error> {
    let input = std::fs::read_to_string(file_path)?;
    let conk_ast = parse_ast_from_str(&input)?;
    Ok(conk_ast)
}

pub fn parse_ast_from_str(input: &str) -> Result<ConkAST, Error> {
    let mut pairs = ConkParser::parse(Rule::file, input)?;
    let file_pair = pairs.next().unwrap();

    let mut config = None;
    let mut enum_declarations = Vec::new();
    let mut template_declarations = Vec::new();
    let mut entity_declarations = Vec::new();

    for pair in file_pair.into_inner() {
        match pair.as_rule() {
            Rule::config => {
                config = Some(config::parse_config(pair)?);
            }

            Rule::declaration => {
                let decl_pair = pair.into_inner().next().unwrap();
                match decl_pair.as_rule() {
                    Rule::enum_declaration => {
                        enum_declarations.push(declaration::parse_enum_declaration(decl_pair)?);
                    }

                    Rule::template_declaration => {
                        template_declarations
                            .push(declaration::parse_template_declaration(decl_pair)?);
                    }

                    Rule::entity_declaration => {
                        entity_declarations.push(declaration::parse_entity_declaration(decl_pair)?);
                    }

                    _ => unreachable!("Unexpected rule in file: {:?}", decl_pair.as_rule()),
                }
            }

            Rule::EOI => {}

            _ => unreachable!("Unexpected rule in file: {:?}", pair.as_rule()),
        }
    }

    Ok(ConkAST {
        config,
        enum_declarations,
        template_declarations,
        entity_declarations,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::ConkParser;
    use pest::Parser;

    #[test]
    fn file_structure() {
        let input = r#"
            config {
                a: 1
            }
            enum Status { ACTIVE }
            template T { id String }
            entity E { id String }
        "#;

        let ast = parse_ast_from_str(input).unwrap();

        assert!(ast.config.is_some());
        assert_eq!(ast.enum_declarations.len(), 1);
        assert_eq!(ast.template_declarations.len(), 1);
        assert_eq!(ast.entity_declarations.len(), 1);
    }

    #[test]
    fn file_cases() {
        let ok = [
            r#"
            config {
                a: 1
            }
            enum Status { ACTIVE }
            template T { id String }
            entity E { id String }
        "#,
            "entity Empty {}",
        ];

        let err = [
            "config { a: 1 } invalid_decl {}",
            "config { a: 1",
            "config { a: }",
            "enum A { @@ }",
            "entity E : { }",
        ];

        for input in ok {
            let parse_res = ConkParser::parse(Rule::file, input);
            assert!(parse_res.is_ok(), "Expected Ok for {:?}", input);

            let ast_res = parse_ast_from_str(input);
            assert!(ast_res.is_ok(), "Expected Ok for {:?}", input);
        }

        for input in err {
            let parse_res = ConkParser::parse(Rule::file, input);

            if parse_res.is_ok() {
                let ast_res = parse_ast_from_str(input);
                assert!(ast_res.is_err(), "Expected Err for {:?}", input);
            } else {
                assert!(parse_res.is_err(), "Expected Err for {:?}", input);
            }
        }
    }
}
