use pest::Parser;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::ast::{
    ArgumentList, BlockAttribute, Config, ConkAST, Declaration, Entity, Enum, Field,
    FieldAttribute, NamedArgument, Template, TypeExpr, Value,
};
use crate::parser::error::{Error, SemanticError};
use crate::parser::pest::{ConkParser, Rule};

pub fn parse_ast_from_file(path: impl AsRef<Path>) -> Result<ConkAST, Error> {
    let path = path.as_ref();
    let input = std::fs::read_to_string(path)?;

    let conk_file = parse_ast_from_string(&input)?;
    Ok(conk_file)
}

pub fn parse_ast_from_string(input: &str) -> Result<ConkAST, Error> {
    let mut pairs = ConkParser::parse(Rule::file, input)?;
    let file_pair = pairs.next().unwrap();

    let mut config = None;
    let mut declarations = Vec::new();
    let mut seen_declarations = HashSet::new();

    for pair in file_pair.into_inner() {
        match pair.as_rule() {
            Rule::config => {
                config = Some(parse_config(pair)?);
            }

            Rule::declaration => {
                let decl = parse_declaration(pair)?;
                let name = match &decl {
                    Declaration::Enum(e) => &e.name,
                    Declaration::Template(t) => &t.name,
                    Declaration::Entity(e) => &e.name,
                };
                if !seen_declarations.insert(name.clone()) {
                    return Err(SemanticError::DuplicateDeclarationName(name.clone()).into());
                }
                declarations.push(decl);
            }

            Rule::EOI => break,

            _ => unreachable!("Unexpected rule in file: {:?}", pair.as_rule()),
        }
    }

    Ok(ConkAST {
        config,
        declarations,
    })
}

fn parse_config(pair: pest::iterators::Pair<Rule>) -> Result<Config, Error> {
    let mut entries = HashMap::new();
    for entry_pair in pair.into_inner() {
        if entry_pair.as_rule() == Rule::config_entry {
            let mut inner = entry_pair.into_inner();
            let id = parse_identifier(inner.next().unwrap());
            let val = parse_value(inner.next().unwrap())?;
            if entries.insert(id.clone(), val).is_some() {
                return Err(SemanticError::DuplicateConfigKey(id).into());
            }
        }
    }
    Ok(Config { entries })
}

fn parse_declaration(pair: pest::iterators::Pair<Rule>) -> Result<Declaration, Error> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::enum_declaration => Ok(Declaration::Enum(parse_enum_declaration(inner)?)),

        Rule::template_declaration => Ok(Declaration::Template(parse_template_declaration(inner)?)),

        Rule::entity_declaration => Ok(Declaration::Entity(parse_entity_declaration(inner)?)),

        _ => unreachable!("Unexpected declaration: {:?}", inner.as_rule()),
    }
}

fn parse_enum_declaration(pair: pest::iterators::Pair<Rule>) -> Result<Enum, Error> {
    let mut schema = None;
    let mut name = String::new();
    let mut values = Vec::new();
    let mut attributes = Vec::new();

    let mut seen_values = HashSet::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::schema_prefix => schema = Some(parse_schema_prefix(inner)),

            Rule::identifier => name = parse_identifier(inner),

            Rule::enum_value => {
                let val_name = parse_identifier(inner.into_inner().next().unwrap());
                if !seen_values.insert(val_name.clone()) {
                    return Err(SemanticError::DuplicateEnumValue(val_name).into());
                }
                values.push(val_name);
            }

            Rule::block_attribute => attributes.push(parse_block_attribute(inner)?),

            _ => unreachable!("Unexpected in enum_declaration: {:?}", inner.as_rule()),
        }
    }

    Ok(Enum {
        schema,
        name,
        values,
        attributes,
    })
}

fn parse_template_declaration(pair: pest::iterators::Pair<Rule>) -> Result<Template, Error> {
    let mut name = String::new();
    let mut fields = Vec::new();
    let mut attributes = Vec::new();

    let mut seen_fields = HashSet::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => name = parse_identifier(inner),

            Rule::field => {
                let field = parse_field(inner)?;
                if !seen_fields.insert(field.name.clone()) {
                    return Err(SemanticError::DuplicateFieldName(field.name).into());
                }
                fields.push(field);
            }

            Rule::block_attribute => attributes.push(parse_block_attribute(inner)?),

            _ => unreachable!("Unexpected in template_declaration: {:?}", inner.as_rule()),
        }
    }

    Ok(Template {
        name,
        fields,
        attributes,
    })
}

fn parse_entity_declaration(pair: pest::iterators::Pair<Rule>) -> Result<Entity, Error> {
    let mut schema = None;
    let mut name = String::new();
    let mut templates = Vec::new();
    let mut inherits = Vec::new();
    let mut fields = Vec::new();
    let mut attributes = Vec::new();

    let mut seen_fields = HashSet::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::schema_prefix => schema = Some(parse_schema_prefix(inner)),

            Rule::identifier => name = parse_identifier(inner),

            Rule::template_usage => {
                let list = inner.into_inner().next().unwrap();
                templates = parse_identifier_list(list);
            }

            Rule::inheritance_usage => {
                let list = inner.into_inner().next().unwrap();
                inherits = parse_identifier_list(list);
            }

            Rule::field => {
                let field = parse_field(inner)?;
                if !seen_fields.insert(field.name.clone()) {
                    return Err(SemanticError::DuplicateFieldName(field.name).into());
                }
                fields.push(field);
            }

            Rule::block_attribute => attributes.push(parse_block_attribute(inner)?),

            _ => unreachable!("Unexpected in entity_declaration: {:?}", inner.as_rule()),
        }
    }

    Ok(Entity {
        schema,
        name,
        templates,
        inherits,
        fields,
        attributes,
    })
}

fn parse_field(pair: pest::iterators::Pair<Rule>) -> Result<Field, Error> {
    let mut inner = pair.into_inner();
    let name = parse_identifier(inner.next().unwrap());
    let type_expr = parse_type_expr(inner.next().unwrap())?;

    let mut attributes = Vec::new();
    let mut seen_attrs = HashSet::new();
    for attr_pair in inner {
        let attr = parse_field_attribute(attr_pair)?;
        if !seen_attrs.insert(attr.name.clone()) {
            return Err(SemanticError::DuplicateFieldAttribute {
                field: name.clone(),
                attribute: attr.name,
            }
            .into());
        }
        attributes.push(attr);
    }

    Ok(Field {
        name,
        type_: type_expr,
        attributes,
    })
}

fn parse_type_expr(pair: pest::iterators::Pair<Rule>) -> Result<TypeExpr, Error> {
    let mut inner = pair.into_inner();
    let name = parse_identifier(inner.next().unwrap());
    let mut params = Vec::new();

    if let Some(params_pair) = inner.next() {
        if let Some(list_pair) = params_pair.into_inner().next() {
            for val_pair in list_pair.into_inner() {
                params.push(parse_value(val_pair)?);
            }
        }
    }

    Ok(TypeExpr { name, params })
}

fn parse_field_attribute(pair: pest::iterators::Pair<Rule>) -> Result<FieldAttribute, Error> {
    let mut inner = pair.into_inner();
    let name = parse_identifier(inner.next().unwrap());
    let mut args = ArgumentList::default();

    if let Some(args_pair) = inner.next() {
        if let Some(list_pair) = args_pair.into_inner().next() {
            args = parse_argument_list(list_pair)?;
        }
    }

    Ok(FieldAttribute { name, args })
}

fn parse_block_attribute(pair: pest::iterators::Pair<Rule>) -> Result<BlockAttribute, Error> {
    let mut inner = pair.into_inner();
    let name = parse_identifier(inner.next().unwrap());
    let mut args = ArgumentList::default();

    let args_pair = inner.next().unwrap();
    if let Some(list_pair) = args_pair.into_inner().next() {
        args = parse_argument_list(list_pair)?;
    }

    Ok(BlockAttribute { name, args })
}

fn parse_argument_list(pair: pest::iterators::Pair<Rule>) -> Result<ArgumentList, Error> {
    let mut args = ArgumentList::default();
    let mut seen_names = HashSet::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::named_argument => {
                let mut kv = inner.into_inner();
                let name = parse_identifier(kv.next().unwrap());
                if !seen_names.insert(name.clone()) {
                    return Err(SemanticError::DuplicateArgumentName(name).into());
                }
                let value = parse_value(kv.next().unwrap())?;
                args.named.push(NamedArgument { name, value });
            }

            _ => {
                // Must be a value
                let value = parse_value(inner)?;
                args.positional.push(value);
            }
        }
    }
    Ok(args)
}

fn parse_value(pair: pest::iterators::Pair<Rule>) -> Result<Value, Error> {
    match pair.as_rule() {
        Rule::string_literal => Ok(Value::String(parse_string_literal(pair))),

        Rule::integer_literal => Ok(Value::Integer(pair.as_str().parse()?)),

        Rule::float_literal => Ok(Value::Float(pair.as_str().parse()?)),

        Rule::list_literal => {
            let mut items = Vec::new();
            if let Some(list) = pair.into_inner().next() {
                for item in list.into_inner() {
                    items.push(parse_value(item)?);
                }
            }
            Ok(Value::List(items))
        }

        Rule::function_call => {
            let mut inner = pair.into_inner();
            let name = parse_identifier(inner.next().unwrap());

            let mut args = ArgumentList::default();

            if let Some(list) = inner.next() {
                args = parse_argument_list(list)?;
            }

            Ok(Value::FunctionCall { name, args })
        }

        Rule::qualified_identifier => {
            let mut parts = Vec::new();
            for part in pair.into_inner() {
                if part.as_rule() == Rule::schema_prefix {
                    parts.push(parse_schema_prefix(part));
                } else {
                    parts.push(parse_identifier(part));
                }
            }
            Ok(Value::QualifiedIdentifier(parts))
        }

        Rule::identifier => Ok(Value::Identifier(parse_identifier(pair))),

        _ => unreachable!("Unexpected value type: {:?}", pair.as_rule()),
    }
}

fn parse_schema_prefix(pair: pest::iterators::Pair<Rule>) -> String {
    parse_string_literal(pair.into_inner().next().unwrap())
}

fn parse_identifier(pair: pest::iterators::Pair<Rule>) -> String {
    pair.as_str().to_string()
}

fn parse_identifier_list(pair: pest::iterators::Pair<Rule>) -> Vec<String> {
    pair.into_inner().map(|p| p.as_str().to_string()).collect()
}

fn parse_string_literal(pair: pest::iterators::Pair<Rule>) -> String {
    let s = pair.as_str();
    // Remove wrapping quotes and replace \" with "
    s[1..s.len() - 1].replace("\\\"", "\"")
}
