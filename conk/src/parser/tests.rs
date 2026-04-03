/// NOTE: The syntax used in the tests are not indicative
/// of the actual syntax that will be used for the DSL
use super::convert::{parse_file_from_path, parse_file_from_str};
use crate::ast::{Declaration, NamedArgument, Value};
use crate::parser::error::{Error, SemanticError};
use std::io::Write;

// ============================================================
// HELPERS
// ============================================================

fn parse_ok(input: &str) -> crate::ast::ConkFile {
    parse_file_from_str(input).unwrap_or_else(|e| panic!("Expected Ok, got: {e}"))
}

fn parse_err(input: &str) -> Error {
    parse_file_from_str(input).unwrap_err()
}

fn assert_parse_error(input: &str, desc: &str) {
    match parse_err(input) {
        Error::Parse(_) => {}
        e => panic!("Expected Parse error for '{desc}', got: {e:?}"),
    }
}

fn entity(file: &crate::ast::ConkFile, idx: usize) -> &crate::ast::Entity {
    match &file.declarations[idx] {
        Declaration::Entity(e) => e,
        d => panic!("Expected Entity at index {idx}, got {:?}", d),
    }
}

fn template(file: &crate::ast::ConkFile, idx: usize) -> &crate::ast::Template {
    match &file.declarations[idx] {
        Declaration::Template(t) => t,
        d => panic!("Expected Template at index {idx}, got {:?}", d),
    }
}

fn enum_decl(file: &crate::ast::ConkFile, idx: usize) -> &crate::ast::Enum {
    match &file.declarations[idx] {
        Declaration::Enum(e) => e,
        d => panic!("Expected Enum at index {idx}, got {:?}", d),
    }
}

// ============================================================
// SECTION 1: EMPTY / WHITESPACE-ONLY INPUTS
// ============================================================

#[test]
fn empty_string() {
    let f = parse_ok("");
    assert!(f.config.is_none());
    assert!(f.declarations.is_empty());
}

#[test]
fn only_whitespace() {
    let f = parse_ok("   \n\t\r\n  ");
    assert!(f.config.is_none());
    assert!(f.declarations.is_empty());
}

#[test]
fn only_line_comments() {
    let f = parse_ok("// this is a comment\n// another comment");
    assert!(f.config.is_none());
    assert!(f.declarations.is_empty());
}

#[test]
fn comment_between_declarations() {
    let input = r#"
        // Before entity
        entity Foo { }
        // Between
        entity Bar { }
        // After
    "#;
    let f = parse_ok(input);
    assert_eq!(f.declarations.len(), 2);
}

#[test]
fn comment_inside_entity_body() {
    let input = r#"
        entity Foo {
            // comment before field
            id UUID @id
            // comment between fields
            name String
            // comment after fields
        }
    "#;
    let f = parse_ok(input);
    let e = entity(&f, 0);
    assert_eq!(e.fields.len(), 2);
}

#[test]
fn comment_inside_config() {
    let input = r#"
        config {
            // comment inside config
            key: "value"
        }
    "#;
    let f = parse_ok(input);
    assert_eq!(f.config.unwrap().entries.len(), 1);
}

#[test]
fn comment_at_end_of_line_after_value() {
    let input = r#"
        config {
            key: "value" // inline comment
        }
    "#;
    let f = parse_ok(input);
    assert_eq!(f.config.unwrap().entries.len(), 1);
}

#[test]
fn comment_after_field_attribute() {
    let input = r#"
        entity Foo {
            id UUID @id // primary key
        }
    "#;
    let f = parse_ok(input);
    assert_eq!(entity(&f, 0).fields.len(), 1);
}

// ============================================================
// SECTION 2: CONFIG BLOCK EXHAUSTIVE
// ============================================================

#[test]
fn config_empty_block() {
    let f = parse_ok("config { }");
    assert!(f.config.unwrap().entries.is_empty());
}

#[test]
fn config_single_string_entry() {
    let f = parse_ok(r#"config { db: "postgres" }"#);
    let c = f.config.unwrap();
    assert!(matches!(c.entries.get("db").unwrap(), Value::String(s) if s == "postgres"));
}

#[test]
fn config_single_integer_positive() {
    let f = parse_ok("config { port: 5432 }");
    assert!(matches!(
        f.config.unwrap().entries.get("port").unwrap(),
        Value::Integer(5432)
    ));
}

#[test]
fn config_single_integer_with_plus_sign() {
    let f = parse_ok("config { port: +5432 }");
    assert!(matches!(
        f.config.unwrap().entries.get("port").unwrap(),
        Value::Integer(5432)
    ));
}

#[test]
fn config_single_integer_negative() {
    let f = parse_ok("config { offset: -100 }");
    assert!(matches!(
        f.config.unwrap().entries.get("offset").unwrap(),
        Value::Integer(-100)
    ));
}

#[test]
fn config_integer_zero() {
    let f = parse_ok("config { val: 0 }");
    assert!(matches!(
        f.config.unwrap().entries.get("val").unwrap(),
        Value::Integer(0)
    ));
}

#[test]
fn config_integer_max_i64() {
    let f = parse_ok("config { val: 9223372036854775807 }");
    assert!(matches!(
        f.config.unwrap().entries.get("val").unwrap(),
        Value::Integer(9223372036854775807)
    ));
}

#[test]
fn config_integer_min_i64() {
    let f = parse_ok("config { val: -9223372036854775808 }");
    assert!(matches!(
        f.config.unwrap().entries.get("val").unwrap(),
        Value::Integer(i64::MIN)
    ));
}

#[test]
fn config_float_positive() {
    let f = parse_ok("config { ratio: 3.14 }");
    if let Value::Float(v) = f.config.unwrap().entries.get("ratio").unwrap() {
        assert!((v - 3.14).abs() < f64::EPSILON);
    } else {
        panic!("Expected float");
    }
}

#[test]
fn config_float_negative() {
    let f = parse_ok("config { ratio: -0.5 }");
    if let Value::Float(v) = f.config.unwrap().entries.get("ratio").unwrap() {
        assert!((*v - -0.5_f64).abs() < f64::EPSILON);
    } else {
        panic!("Expected float");
    }
}

#[test]
fn config_float_zero() {
    let f = parse_ok("config { val: 0.0 }");
    assert!(matches!(f.config.unwrap().entries.get("val").unwrap(), Value::Float(v) if *v == 0.0));
}

#[test]
fn config_float_with_plus_sign() {
    let f = parse_ok("config { val: +1.5 }");
    assert!(
        matches!(f.config.unwrap().entries.get("val").unwrap(), Value::Float(v) if (*v - 1.5).abs() < f64::EPSILON)
    );
}

#[test]
fn config_identifier_value_true() {
    let f = parse_ok("config { debug: true }");
    assert!(
        matches!(f.config.unwrap().entries.get("debug").unwrap(), Value::Identifier(s) if s == "true")
    );
}

#[test]
fn config_identifier_value_false() {
    let f = parse_ok("config { debug: false }");
    assert!(
        matches!(f.config.unwrap().entries.get("debug").unwrap(), Value::Identifier(s) if s == "false")
    );
}

#[test]
fn config_identifier_value_arbitrary() {
    let f = parse_ok("config { mode: production }");
    assert!(
        matches!(f.config.unwrap().entries.get("mode").unwrap(), Value::Identifier(s) if s == "production")
    );
}

#[test]
fn config_empty_list() {
    let f = parse_ok("config { items: [] }");
    assert!(
        matches!(f.config.unwrap().entries.get("items").unwrap(), Value::List(l) if l.is_empty())
    );
}

#[test]
fn config_list_single_element() {
    let f = parse_ok(r#"config { items: ["a"] }"#);
    if let Value::List(l) = f.config.unwrap().entries.get("items").unwrap() {
        assert_eq!(l.len(), 1);
        assert!(matches!(&l[0], Value::String(s) if s == "a"));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn config_list_multiple_types() {
    let f = parse_ok(r#"config { items: ["str", 42, 3.14, true] }"#);
    if let Value::List(l) = f.config.unwrap().entries.get("items").unwrap() {
        assert_eq!(l.len(), 4);
        assert!(matches!(&l[0], Value::String(_)));
        assert!(matches!(&l[1], Value::Integer(42)));
        assert!(matches!(&l[2], Value::Float(_)));
        assert!(matches!(&l[3], Value::Identifier(s) if s == "true"));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn config_list_trailing_comma() {
    let f = parse_ok(r#"config { items: [1, 2, 3,] }"#);
    if let Value::List(l) = f.config.unwrap().entries.get("items").unwrap() {
        assert_eq!(l.len(), 3);
    } else {
        panic!("Expected list");
    }
}

#[test]
fn config_nested_list() {
    let f = parse_ok("config { items: [[1, 2], [3, 4]] }");
    if let Value::List(outer) = f.config.unwrap().entries.get("items").unwrap() {
        assert_eq!(outer.len(), 2);
        assert!(matches!(&outer[0], Value::List(inner) if inner.len() == 2));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn config_function_call_no_args() {
    let f = parse_ok("config { ts: now() }");
    if let Value::FunctionCall { name, args } = f.config.unwrap().entries.get("ts").unwrap() {
        assert_eq!(name, "now");
        assert!(args.is_empty());
    } else {
        panic!("Expected function call");
    }
}

#[test]
fn config_function_call_one_arg() {
    let f = parse_ok(r#"config { url: env("DATABASE_URL") }"#);
    if let Value::FunctionCall { name, args } = f.config.unwrap().entries.get("url").unwrap() {
        assert_eq!(name, "env");
        assert_eq!((args.positional.len() + args.named.len()), 1);
        assert!(matches!(&args.positional[0], Value::String(s) if s == "DATABASE_URL"));
    } else {
        panic!("Expected function call");
    }
}

#[test]
fn config_function_call_multiple_args() {
    let f = parse_ok("config { v: coalesce(1, 2, 3) }");
    if let Value::FunctionCall { name, args } = f.config.unwrap().entries.get("v").unwrap() {
        assert_eq!(name, "coalesce");
        assert_eq!((args.positional.len() + args.named.len()), 3);
    } else {
        panic!("Expected function call");
    }
}

#[test]
fn config_function_call_trailing_comma() {
    let f = parse_ok("config { v: func(1,) }");
    if let Value::FunctionCall { args, .. } = f.config.unwrap().entries.get("v").unwrap() {
        assert_eq!((args.positional.len() + args.named.len()), 1);
    } else {
        panic!("Expected function call");
    }
}

#[test]
fn config_qualified_identifier_two_parts() {
    let f = parse_ok("config { ref: User.id }");
    if let Value::QualifiedIdentifier(parts) = f.config.unwrap().entries.get("ref").unwrap() {
        assert_eq!(parts, &["User", "id"]);
    } else {
        panic!("Expected qualified identifier");
    }
}

#[test]
fn config_qualified_identifier_three_parts() {
    let f = parse_ok("config { ref: schema.User.id }");
    if let Value::QualifiedIdentifier(parts) = f.config.unwrap().entries.get("ref").unwrap() {
        assert_eq!(parts, &["schema", "User", "id"]);
    } else {
        panic!("Expected qualified identifier");
    }
}

#[test]
fn config_qualified_identifier_with_schema_prefix() {
    let f = parse_ok(r#"config { ref: "public".User.id }"#);
    if let Value::QualifiedIdentifier(parts) = f.config.unwrap().entries.get("ref").unwrap() {
        assert_eq!(parts, &["public", "User", "id"]);
    } else {
        panic!("Expected qualified identifier");
    }
}

#[test]
fn config_many_entries() {
    let input = r#"
        config {
            a: 1
            b: 2
            c: 3
            d: 4
            e: 5
            f: 6
            g: 7
            h: 8
            i: 9
            j: 10
        }
    "#;
    let f = parse_ok(input);
    assert_eq!(f.config.unwrap().entries.len(), 10);
}

#[test]
fn config_underscore_identifier_key() {
    let f = parse_ok("config { _private: 1 }");
    assert!(f.config.unwrap().entries.contains_key("_private"));
}

#[test]
fn config_alphanumeric_identifier_key() {
    let f = parse_ok("config { key1: 1 }");
    assert!(f.config.unwrap().entries.contains_key("key1"));
}

// ============================================================
// SECTION 3: STRING LITERAL EDGE CASES
// ============================================================

#[test]
fn string_empty() {
    let f = parse_ok(r#"config { v: "" }"#);
    assert!(
        matches!(f.config.unwrap().entries.get("v").unwrap(), Value::String(s) if s.is_empty())
    );
}

#[test]
fn string_spaces_only() {
    let f = parse_ok(r#"config { v: "   " }"#);
    assert!(matches!(f.config.unwrap().entries.get("v").unwrap(), Value::String(s) if s == "   "));
}

#[test]
fn string_escaped_quote() {
    let f = parse_ok(r#"config { v: "say \"hello\"" }"#);
    assert!(
        matches!(f.config.unwrap().entries.get("v").unwrap(), Value::String(s) if s == r#"say "hello""#)
    );
}

#[test]
fn string_backslash_not_escaped() {
    // Backslashes other than \" are stored literally
    let f = parse_ok(r#"config { v: "path\\to\\file" }"#);
    if let Value::String(s) = f.config.unwrap().entries.get("v").unwrap() {
        assert!(s.contains("\\\\"));
    }
}

#[test]
fn string_unicode_content() {
    let f = parse_ok(r#"config { v: "héllo wörld" }"#);
    assert!(
        matches!(f.config.unwrap().entries.get("v").unwrap(), Value::String(s) if s == "héllo wörld")
    );
}

#[test]
fn string_with_numbers() {
    let f = parse_ok(r#"config { v: "abc123" }"#);
    assert!(
        matches!(f.config.unwrap().entries.get("v").unwrap(), Value::String(s) if s == "abc123")
    );
}

#[test]
fn string_with_special_characters() {
    let f = parse_ok(r#"config { v: "!@#$%^&*()" }"#);
    assert!(matches!(
        f.config.unwrap().entries.get("v").unwrap(),
        Value::String(_)
    ));
}

#[test]
fn string_with_newline_literal() {
    // The parser allows any char except `"` in strings (raw)
    let f = parse_ok("config { v: \"line1\nline2\" }");
    assert!(matches!(
        f.config.unwrap().entries.get("v").unwrap(),
        Value::String(_)
    ));
}

// ============================================================
// SECTION 4: ENUM DECLARATIONS
// ============================================================

#[test]
fn enum_minimal_one_value() {
    let f = parse_ok("enum Status { Active }");
    let e = enum_decl(&f, 0);
    assert_eq!(e.name, "Status");
    assert_eq!(e.values, vec!["Active"]);
    assert!(e.schema.is_none());
    assert!(e.attributes.is_empty());
}

#[test]
fn enum_many_values() {
    let f = parse_ok("enum Day { Mon Tue Wed Thu Fri Sat Sun }");
    let e = enum_decl(&f, 0);
    assert_eq!(e.values.len(), 7);
    assert_eq!(e.values[0], "Mon");
    assert_eq!(e.values[6], "Sun");
}

#[test]
fn enum_with_schema_prefix() {
    let f = parse_ok(r#""auth" enum Role { Admin User }"#);
    let e = enum_decl(&f, 0);
    assert_eq!(e.schema, Some("auth".to_string()));
    assert_eq!(e.name, "Role");
}

#[test]
fn enum_with_schema_prefix_empty_schema() {
    let f = parse_ok(r#""" enum Role { Admin }"#);
    let e = enum_decl(&f, 0);
    assert_eq!(e.schema, Some("".to_string()));
}

#[test]
fn enum_with_block_attribute_before_values() {
    let input = r#"
        enum Status {
            @@map("statuses")
            Active
            Inactive
        }
    "#;
    let f = parse_ok(input);
    let e = enum_decl(&f, 0);
    assert_eq!(e.attributes.len(), 1);
    assert_eq!(e.attributes[0].name, "map");
    assert_eq!(e.values, vec!["Active", "Inactive"]);
}

#[test]
fn enum_with_block_attribute_after_values() {
    let input = r#"
        enum Status {
            Active
            Inactive
            @@comment("status options")
        }
    "#;
    let f = parse_ok(input);
    let e = enum_decl(&f, 0);
    assert_eq!(e.attributes.len(), 1);
    assert_eq!(e.values.len(), 2);
}

#[test]
fn enum_with_block_attributes_both_sides() {
    let input = r#"
        enum Status {
            @@map("statuses")
            Active
            Inactive
            @@comment("note")
        }
    "#;
    let f = parse_ok(input);
    let e = enum_decl(&f, 0);
    assert_eq!(e.attributes.len(), 2);
    assert_eq!(e.values.len(), 2);
}

#[test]
fn enum_block_attribute_no_args_fails() {
    // block_attribute requires attribute_args (parens) in grammar
    assert_parse_error("enum Foo { @@map A }", "block attribute without parens");
}

#[test]
fn enum_block_attribute_empty_parens() {
    let f = parse_ok("enum Foo { @@track() A }");
    let e = enum_decl(&f, 0);
    assert_eq!(
        (e.attributes[0].args.positional.len() + e.attributes[0].args.named.len()),
        0
    );
}

#[test]
fn enum_block_attribute_positional_string_arg() {
    let f = parse_ok(r#"enum Foo { @@map("foo_enum") A }"#);
    let e = enum_decl(&f, 0);
    assert!(matches!(&e.attributes[0].args.positional[0], Value::String(s) if s == "foo_enum"));
}

#[test]
fn enum_block_attribute_named_arg() {
    let f = parse_ok(r#"enum Foo { @@schema(name: "public") A }"#);
    let e = enum_decl(&f, 0);
    assert!(
        matches!(&e.attributes[0].args.named[0], NamedArgument { name, value: Value::String(_) } if name == "name")
    );
}

#[test]
fn enum_values_with_underscores() {
    let f = parse_ok("enum Foo { PENDING_REVIEW UNDER_REVIEW FULLY_REVIEWED }");
    let e = enum_decl(&f, 0);
    assert_eq!(e.values.len(), 3);
    assert_eq!(e.values[1], "UNDER_REVIEW");
}

#[test]
fn enum_values_mixed_case() {
    let f = parse_ok("enum Foo { PascalCase camelCase SCREAMING_SNAKE _leading_underscore }");
    let e = enum_decl(&f, 0);
    assert_eq!(e.values.len(), 4);
}

// ============================================================
// SECTION 5: TEMPLATE DECLARATIONS
// ============================================================

#[test]
fn template_empty_body() {
    let f = parse_ok("template Empty { }");
    let t = template(&f, 0);
    assert_eq!(t.name, "Empty");
    assert!(t.fields.is_empty());
    assert!(t.attributes.is_empty());
}

#[test]
fn template_single_field_no_attrs() {
    let f = parse_ok("template T { id UUID }");
    let t = template(&f, 0);
    assert_eq!(t.fields.len(), 1);
    assert_eq!(t.fields[0].name, "id");
    assert_eq!(t.fields[0].type_.name, "UUID");
}

#[test]
fn template_multiple_fields() {
    let input = "template T { a String b Int c Float d Bool }";
    let f = parse_ok(input);
    let t = template(&f, 0);
    assert_eq!(t.fields.len(), 4);
    assert_eq!(t.fields[0].name, "a");
    assert_eq!(t.fields[3].name, "d");
}

#[test]
fn template_field_with_single_attr() {
    let f = parse_ok("template T { id UUID @id }");
    let t = template(&f, 0);
    assert_eq!(t.fields[0].attributes.len(), 1);
    assert_eq!(t.fields[0].attributes[0].name, "id");
}

#[test]
fn template_field_with_multiple_attrs() {
    let f = parse_ok("template T { id UUID @id @default(gen_random_uuid()) @unique }");
    let t = template(&f, 0);
    assert_eq!(t.fields[0].attributes.len(), 3);
}

#[test]
fn template_with_block_attribute() {
    let f = parse_ok(r#"template T { @@track() id UUID }"#);
    let t = template(&f, 0);
    assert_eq!(t.attributes.len(), 1);
    assert_eq!(t.fields.len(), 1);
}

#[test]
fn template_with_type_params() {
    let f = parse_ok("template T { name VARCHAR(100) }");
    let t = template(&f, 0);
    assert_eq!(t.fields[0].type_.name, "VARCHAR");
    assert_eq!(t.fields[0].type_.params.len(), 1);
    assert!(matches!(t.fields[0].type_.params[0], Value::Integer(100)));
}

#[test]
fn template_field_attr_no_parens() {
    // @nullable with no args is valid
    let f = parse_ok("template T { val String @nullable }");
    let t = template(&f, 0);
    assert_eq!(t.fields[0].attributes[0].name, "nullable");
    assert!(t.fields[0].attributes[0].args.is_empty());
}

#[test]
fn template_field_attr_empty_parens() {
    let f = parse_ok("template T { val String @nullable() }");
    let t = template(&f, 0);
    assert_eq!(
        (t.fields[0].attributes[0].args.positional.len()
            + t.fields[0].attributes[0].args.named.len()),
        0
    );
}

#[test]
fn template_underscore_field_name() {
    let f = parse_ok("template T { _internal UUID }");
    let t = template(&f, 0);
    assert_eq!(t.fields[0].name, "_internal");
}

// ============================================================
// SECTION 6: ENTITY DECLARATIONS
// ============================================================

#[test]
fn entity_empty_body() {
    let f = parse_ok("entity Empty { }");
    let e = entity(&f, 0);
    assert_eq!(e.name, "Empty");
    assert!(e.fields.is_empty());
    assert!(e.attributes.is_empty());
    assert!(e.templates.is_empty());
    assert!(e.inherits.is_empty());
    assert!(e.schema.is_none());
}

#[test]
fn entity_with_schema_prefix() {
    let f = parse_ok(r#""public" entity Users { }"#);
    let e = entity(&f, 0);
    assert_eq!(e.schema, Some("public".to_string()));
}

#[test]
fn entity_with_empty_schema_prefix() {
    let f = parse_ok(r#""" entity Users { }"#);
    let e = entity(&f, 0);
    assert_eq!(e.schema, Some("".to_string()));
}

#[test]
fn entity_single_template() {
    let f = parse_ok("entity User : Auditable { }");
    let e = entity(&f, 0);
    assert_eq!(e.templates, vec!["Auditable"]);
}

#[test]
fn entity_multiple_templates() {
    let f = parse_ok("entity User : Auditable, HasRoles, SoftDeletable { }");
    let e = entity(&f, 0);
    assert_eq!(e.templates, vec!["Auditable", "HasRoles", "SoftDeletable"]);
}

#[test]
fn entity_template_trailing_comma() {
    let f = parse_ok("entity User : Auditable, { }");
    let e = entity(&f, 0);
    assert_eq!(e.templates, vec!["Auditable"]);
}

#[test]
fn entity_single_inherits() {
    let f = parse_ok("entity Admin inherits (User) { }");
    let e = entity(&f, 0);
    assert_eq!(e.inherits, vec!["User"]);
}

#[test]
fn entity_multiple_inherits() {
    let f = parse_ok("entity Admin inherits (User, BaseEntity) { }");
    let e = entity(&f, 0);
    assert_eq!(e.inherits, vec!["User", "BaseEntity"]);
}

#[test]
fn entity_inherits_trailing_comma() {
    let f = parse_ok("entity Admin inherits (User,) { }");
    let e = entity(&f, 0);
    assert_eq!(e.inherits, vec!["User"]);
}

#[test]
fn entity_templates_and_inherits() {
    let f = parse_ok("entity Admin : Auditable inherits (User, Base) { }");
    let e = entity(&f, 0);
    assert_eq!(e.templates, vec!["Auditable"]);
    assert_eq!(e.inherits, vec!["User", "Base"]);
}

#[test]
fn entity_schema_templates_inherits_fields_attrs() {
    let input = r#"
        "public" entity Admin : Auditable, Tracked inherits (User, Base) {
            @@table("admins")
            extra_field String
        }
    "#;
    let f = parse_ok(input);
    let e = entity(&f, 0);
    assert_eq!(e.schema, Some("public".to_string()));
    assert_eq!(e.templates, vec!["Auditable", "Tracked"]);
    assert_eq!(e.inherits, vec!["User", "Base"]);
    assert_eq!(e.attributes.len(), 1);
    assert_eq!(e.fields.len(), 1);
}

#[test]
fn entity_field_with_qualified_identifier_default() {
    let input = r#"
        entity Post {
            created_by UUID @ref("public".User.id, on_delete: Cascade)
        }
    "#;
    let f = parse_ok(input);
    let e = entity(&f, 0);
    let attr = &e.fields[0].attributes[0];
    assert_eq!(attr.name, "ref");
    assert_eq!((attr.args.positional.len() + attr.args.named.len()), 2);
    // First arg is positional qualified identifier
    match &attr.args.positional[0] {
        Value::QualifiedIdentifier(parts) => {
            assert_eq!(parts, &["public", "User", "id"]);
        }
        other => panic!("Expected qualified identifier, got {:?}", other),
    }
    // Second arg is named
    assert!(matches!(&attr.args.named[0], NamedArgument { name, .. } if name == "on_delete"));
}

#[test]
fn entity_multiple_block_attributes() {
    let input = r#"
        entity Foo {
            @@table("foos")
            @@index([a, b])
            @@unique([a])
            id UUID @id
        }
    "#;
    let f = parse_ok(input);
    let e = entity(&f, 0);
    assert_eq!(e.attributes.len(), 3);
}

#[test]
fn entity_block_attributes_interleaved_with_fields() {
    let input = r#"
        entity Foo {
            @@before()
            a String
            @@between()
            b Int
            @@after()
        }
    "#;
    let f = parse_ok(input);
    let e = entity(&f, 0);
    assert_eq!(e.attributes.len(), 3);
    assert_eq!(e.fields.len(), 2);
}

// ============================================================
// SECTION 7: FIELD TYPES
// ============================================================

#[test]
fn field_type_no_params() {
    let f = parse_ok("entity E { col String }");
    assert_eq!(entity(&f, 0).fields[0].type_.params.len(), 0);
}

#[test]
fn field_type_empty_parens() {
    let f = parse_ok("entity E { col String() }");
    assert_eq!(entity(&f, 0).fields[0].type_.params.len(), 0);
}

#[test]
fn field_type_one_integer_param() {
    let f = parse_ok("entity E { col VARCHAR(255) }");
    let t = &entity(&f, 0).fields[0].type_;
    assert_eq!(t.name, "VARCHAR");
    assert!(matches!(t.params[0], Value::Integer(255)));
}

#[test]
fn field_type_two_integer_params() {
    let f = parse_ok("entity E { col NUMERIC(10, 2) }");
    let t = &entity(&f, 0).fields[0].type_;
    assert_eq!(t.params.len(), 2);
    assert!(matches!(t.params[0], Value::Integer(10)));
    assert!(matches!(t.params[1], Value::Integer(2)));
}

#[test]
fn field_type_string_param() {
    let f = parse_ok(r#"entity E { col CUSTOM("format") }"#);
    let t = &entity(&f, 0).fields[0].type_;
    assert!(matches!(&t.params[0], Value::String(s) if s == "format"));
}

#[test]
fn field_type_list_param() {
    let f = parse_ok("entity E { col ARRAY([1, 2, 3]) }");
    let t = &entity(&f, 0).fields[0].type_;
    assert!(matches!(&t.params[0], Value::List(l) if l.len() == 3));
}

#[test]
fn field_type_trailing_comma_in_params() {
    let f = parse_ok("entity E { col NUMERIC(10,) }");
    let t = &entity(&f, 0).fields[0].type_;
    assert_eq!(t.params.len(), 1);
}

#[test]
fn field_type_underscore_name() {
    let f = parse_ok("entity E { col _custom_type }");
    assert_eq!(entity(&f, 0).fields[0].type_.name, "_custom_type");
}

// ============================================================
// SECTION 8: FIELD ATTRIBUTES
// ============================================================

#[test]
fn field_attr_no_args_no_parens() {
    let f = parse_ok("entity E { id UUID @id }");
    let attr = &entity(&f, 0).fields[0].attributes[0];
    assert_eq!(attr.name, "id");
    assert!(attr.args.is_empty());
}

#[test]
fn field_attr_empty_parens() {
    let f = parse_ok("entity E { id UUID @id() }");
    let attr = &entity(&f, 0).fields[0].attributes[0];
    assert_eq!((attr.args.positional.len() + attr.args.named.len()), 0);
}

#[test]
fn field_attr_positional_string() {
    let f = parse_ok(r#"entity E { col String @db("column_name") }"#);
    let attr = &entity(&f, 0).fields[0].attributes[0];
    assert!(matches!(&attr.args.positional[0], Value::String(s) if s == "column_name"));
}

#[test]
fn field_attr_positional_integer() {
    let f = parse_ok("entity E { col Int @max(100) }");
    let attr = &entity(&f, 0).fields[0].attributes[0];
    assert!(matches!(&attr.args.positional[0], Value::Integer(100)));
}

#[test]
fn field_attr_positional_float() {
    let f = parse_ok("entity E { score Float @range(0.0, 1.0) }");
    let attr = &entity(&f, 0).fields[0].attributes[0];
    assert_eq!((attr.args.positional.len() + attr.args.named.len()), 2);
}

#[test]
fn field_attr_positional_function_call() {
    let f = parse_ok("entity E { ts TIMESTAMP @default(now()) }");
    let attr = &entity(&f, 0).fields[0].attributes[0];
    assert!(matches!(&attr.args.positional[0], Value::FunctionCall { name, .. } if name == "now"));
}

#[test]
fn field_attr_positional_identifier() {
    let f = parse_ok("entity E { ts TIMESTAMP @default(NULL) }");
    let attr = &entity(&f, 0).fields[0].attributes[0];
    assert!(matches!(&attr.args.positional[0], Value::Identifier(s) if s == "NULL"));
}

#[test]
fn field_attr_named_arg() {
    let f = parse_ok("entity E { ts TIMESTAMP @default(value: now()) }");
    let attr = &entity(&f, 0).fields[0].attributes[0];
    assert!(matches!(&attr.args.named[0], NamedArgument { name, .. } if name == "value"));
}

#[test]
fn field_attr_mixed_positional_and_named() {
    let f = parse_ok(r#"entity E { fk UUID @ref(User.id, on_delete: Cascade) }"#);
    let attr = &entity(&f, 0).fields[0].attributes[0];
    assert_eq!((attr.args.positional.len() + attr.args.named.len()), 2);
    assert!(matches!(&attr.args.positional[0], _));
    assert!(matches!(&attr.args.named[0], NamedArgument { .. }));
}

#[test]
fn field_attr_trailing_comma_in_args() {
    let f = parse_ok(r#"entity E { fk UUID @ref(User.id,) }"#);
    let attr = &entity(&f, 0).fields[0].attributes[0];
    assert_eq!((attr.args.positional.len() + attr.args.named.len()), 1);
}

#[test]
fn field_attr_list_arg() {
    let f = parse_ok("entity E { id UUID @check([1, 2, 3]) }");
    let attr = &entity(&f, 0).fields[0].attributes[0];
    assert!(matches!(&attr.args.positional[0], Value::List(_)));
}

#[test]
fn field_multiple_attrs_order_preserved() {
    let f = parse_ok("entity E { id UUID @a @b @c }");
    let attrs = &entity(&f, 0).fields[0].attributes;
    assert_eq!(attrs[0].name, "a");
    assert_eq!(attrs[1].name, "b");
    assert_eq!(attrs[2].name, "c");
}

// ============================================================
// SECTION 9: BLOCK ATTRIBUTES
// ============================================================

#[test]
fn block_attr_empty_parens() {
    let f = parse_ok("entity E { @@track() }");
    let e = entity(&f, 0);
    assert_eq!(
        (e.attributes[0].args.positional.len() + e.attributes[0].args.named.len()),
        0
    );
}

#[test]
fn block_attr_positional_string() {
    let f = parse_ok(r#"entity E { @@map("table_name") }"#);
    let e = entity(&f, 0);
    assert!(matches!(&e.attributes[0].args.positional[0], Value::String(s) if s == "table_name"));
}

#[test]
fn block_attr_list_of_identifiers() {
    let f = parse_ok("entity E { @@index([col1, col2]) }");
    let e = entity(&f, 0);
    assert!(matches!(&e.attributes[0].args.positional[0], Value::List(l) if l.len() == 2));
}

#[test]
fn block_attr_multiple_named_args() {
    let f = parse_ok(r#"entity E { @@index([a], unique: true, name: "idx") }"#);
    let e = entity(&f, 0);
    assert_eq!(
        (e.attributes[0].args.positional.len() + e.attributes[0].args.named.len()),
        3
    );
}

#[test]
fn block_attr_trailing_comma() {
    let f = parse_ok(r#"entity E { @@map("t",) }"#);
    let e = entity(&f, 0);
    assert_eq!(
        (e.attributes[0].args.positional.len() + e.attributes[0].args.named.len()),
        1
    );
}

// ============================================================
// SECTION 10: VALUES IN ARGUMENTS
// ============================================================

#[test]
fn argument_value_nested_function_call() {
    let f = parse_ok("entity E { v UUID @default(coalesce(gen_random_uuid())) }");
    let attr = &entity(&f, 0).fields[0].attributes[0];
    if let Value::FunctionCall { name, args } = &attr.args.positional[0] {
        assert_eq!(name, "coalesce");
        assert_eq!((args.positional.len() + args.named.len()), 1);
        assert!(
            matches!(&args.positional[0], Value::FunctionCall { name, .. } if name == "gen_random_uuid")
        );
    } else {
        panic!("Expected nested function call");
    }
}

#[test]
fn argument_value_qualified_identifier_in_attr() {
    let f = parse_ok(r#"entity E { fk UUID @ref("schema".Table.col) }"#);
    let attr = &entity(&f, 0).fields[0].attributes[0];
    if let Value::QualifiedIdentifier(parts) = &attr.args.positional[0] {
        assert_eq!(parts, &["schema", "Table", "col"]);
    } else {
        panic!("Expected qualified identifier");
    }
}

// ============================================================
// SECTION 11: MULTIPLE DECLARATIONS
// ============================================================

#[test]
fn multiple_declarations_order_preserved() {
    let input = r#"
        enum A { X }
        template B { f String }
        entity C { }
        enum D { Y }
        entity E { }
    "#;
    let f = parse_ok(input);
    assert_eq!(f.declarations.len(), 5);
    assert!(matches!(&f.declarations[0], Declaration::Enum(_)));
    assert!(matches!(&f.declarations[1], Declaration::Template(_)));
    assert!(matches!(&f.declarations[2], Declaration::Entity(_)));
    assert!(matches!(&f.declarations[3], Declaration::Enum(_)));
    assert!(matches!(&f.declarations[4], Declaration::Entity(_)));
}

#[test]
fn config_with_multiple_declarations() {
    let input = r#"
        config { env: "prod" }
        entity User { }
        enum Role { Admin }
    "#;
    let f = parse_ok(input);
    assert!(f.config.is_some());
    assert_eq!(f.declarations.len(), 2);
}

#[test]
fn many_entities() {
    let mut input = String::new();
    for i in 0..200 {
        input.push_str(&format!("entity Entity{i} {{ }}\n"));
    }
    let f = parse_ok(&input);
    assert_eq!(f.declarations.len(), 200);
}

// ============================================================
// SECTION 12: IDENTIFIERS
// ============================================================

#[test]
fn identifier_starts_with_letter() {
    let f = parse_ok("entity Abc { }");
    assert_eq!(entity(&f, 0).name, "Abc");
}

#[test]
fn identifier_starts_with_underscore() {
    let f = parse_ok("entity _Hidden { }");
    assert_eq!(entity(&f, 0).name, "_Hidden");
}

#[test]
fn identifier_all_underscores() {
    let f = parse_ok("entity ___ { }");
    assert_eq!(entity(&f, 0).name, "___");
}

#[test]
fn identifier_with_numbers_in_middle() {
    let f = parse_ok("entity Entity123 { }");
    assert_eq!(entity(&f, 0).name, "Entity123");
}

#[test]
fn identifier_with_numbers_at_end() {
    let f = parse_ok("entity Entity1 { }");
    assert_eq!(entity(&f, 0).name, "Entity1");
}

#[test]
fn identifier_all_caps() {
    let f = parse_ok("entity ENTITY { }");
    assert_eq!(entity(&f, 0).name, "ENTITY");
}

#[test]
fn identifier_single_char() {
    let f = parse_ok("entity A { }");
    assert_eq!(entity(&f, 0).name, "A");
}

// ============================================================
// SECTION 13: SYNTAX / PARSE ERROR CASES
// ============================================================

#[test]
fn error_config_missing_closing_brace() {
    assert_parse_error("config {", "missing closing brace");
}

#[test]
fn error_config_missing_value() {
    assert_parse_error("config { key: }", "config entry missing value");
}

#[test]
fn error_config_missing_colon() {
    assert_parse_error("config { key 1 }", "config entry missing colon");
}

#[test]
fn error_enum_missing_name() {
    assert_parse_error("enum { A }", "enum missing name");
}

#[test]
fn error_enum_missing_opening_brace() {
    assert_parse_error("enum Role A B }", "enum missing opening brace");
}

#[test]
fn error_enum_missing_closing_brace() {
    assert_parse_error("enum Role { A B", "enum missing closing brace");
}

#[test]
fn error_enum_no_values() {
    // Grammar requires enum_value+ so empty enum body fails
    assert_parse_error("enum Role { }", "enum with no values");
}

#[test]
fn error_entity_missing_name() {
    assert_parse_error(r#""public" entity { }"#, "entity missing name");
}

#[test]
fn error_entity_missing_opening_brace() {
    assert_parse_error("entity User id UUID }", "entity missing opening brace");
}

#[test]
fn error_entity_missing_closing_brace() {
    assert_parse_error("entity User {", "entity missing closing brace");
}

#[test]
fn error_entity_field_missing_type() {
    assert_parse_error("entity User { id }", "field missing type");
}

#[test]
fn error_entity_inherits_missing_parens() {
    assert_parse_error("entity User inherits Base { }", "inherits missing parens");
}

#[test]
fn error_entity_inherits_missing_closing_paren() {
    assert_parse_error(
        "entity User inherits (Base { }",
        "inherits missing closing paren",
    );
}

#[test]
fn error_template_missing_name() {
    assert_parse_error("template { id UUID }", "template missing name");
}

#[test]
fn error_template_missing_body() {
    assert_parse_error("template T", "template missing body");
}

#[test]
fn error_field_attribute_missing_name() {
    assert_parse_error("entity E { id UUID @ }", "field attribute missing name");
}

#[test]
fn error_block_attribute_missing_parens() {
    assert_parse_error("entity E { @@map }", "block attribute missing parens");
}

#[test]
fn error_identifier_starts_with_number() {
    assert_parse_error("entity 1Bad { }", "identifier starts with number");
}

#[test]
fn error_identifier_starts_with_hyphen() {
    assert_parse_error("entity bad-name { }", "identifier with hyphen");
}

#[test]
fn error_double_trailing_comma_in_list() {
    assert_parse_error("config { v: [1,,] }", "double trailing comma in list");
}

#[test]
fn error_double_comma_in_function_args() {
    assert_parse_error("config { v: func(1,,2) }", "double comma in function args");
}

#[test]
fn error_unrecognized_token() {
    assert_parse_error("config { %$# }", "unrecognized token");
}

#[test]
fn error_float_double_dot() {
    assert_parse_error("config { v: 1.2.3 }", "invalid float with two dots");
}

#[test]
fn error_unclosed_string() {
    assert_parse_error(r#"config { v: "unclosed }"#, "unclosed string");
}

#[test]
fn error_entity_colon_without_template() {
    // A colon with nothing after it should fail
    assert_parse_error("entity User : { }", "colon without template name");
}

// ============================================================
// SECTION 14: SEMANTIC ERROR CASES
// ============================================================

#[test]
fn semantic_duplicate_config_key_same_type() {
    let input = "config { key: 1\n key: 2 }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateConfigKey(k)) => assert_eq!(k, "key"),
        e => panic!("Expected DuplicateConfigKey, got {e:?}"),
    }
}

#[test]
fn semantic_duplicate_config_key_different_type() {
    let input = r#"config { key: 1 key: "two" }"#;
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateConfigKey(k)) => assert_eq!(k, "key"),
        e => panic!("Expected DuplicateConfigKey, got {e:?}"),
    }
}

#[test]
fn semantic_duplicate_declaration_both_entity() {
    let input = "entity User { }\nentity User { }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateDeclarationName(n)) => assert_eq!(n, "User"),
        e => panic!("Expected DuplicateDeclarationName, got {e:?}"),
    }
}

#[test]
fn semantic_duplicate_declaration_entity_and_template() {
    let input = "entity Foo { }\ntemplate Foo { }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateDeclarationName(n)) => assert_eq!(n, "Foo"),
        e => panic!("Expected DuplicateDeclarationName, got {e:?}"),
    }
}

#[test]
fn semantic_duplicate_declaration_entity_and_enum() {
    let input = "entity Role { }\nenum Role { A }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateDeclarationName(n)) => assert_eq!(n, "Role"),
        e => panic!("Expected DuplicateDeclarationName, got {e:?}"),
    }
}

#[test]
fn semantic_duplicate_declaration_template_and_enum() {
    let input = "template Role { }\nenum Role { A }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateDeclarationName(n)) => assert_eq!(n, "Role"),
        e => panic!("Expected DuplicateDeclarationName, got {e:?}"),
    }
}

#[test]
fn semantic_duplicate_declaration_three_way() {
    let input = "entity A { }\ntemplate B { }\nentity B { }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateDeclarationName(n)) => assert_eq!(n, "B"),
        e => panic!("Expected DuplicateDeclarationName, got {e:?}"),
    }
}

#[test]
fn semantic_no_duplicate_when_different_names() {
    let input = "entity User { }\nentity Post { }\nentity Comment { }";
    let f = parse_ok(input);
    assert_eq!(f.declarations.len(), 3);
}

#[test]
fn semantic_duplicate_enum_value() {
    let input = "enum Role { ADMIN USER ADMIN }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateEnumValue(v)) => assert_eq!(v, "ADMIN"),
        e => panic!("Expected DuplicateEnumValue, got {e:?}"),
    }
}

#[test]
fn semantic_no_duplicate_enum_value_when_different() {
    let f = parse_ok("enum Role { ADMIN USER GUEST }");
    assert_eq!(enum_decl(&f, 0).values.len(), 3);
}

#[test]
fn semantic_duplicate_field_in_entity() {
    let input = "entity E { col String\n col Int }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateFieldName(n)) => assert_eq!(n, "col"),
        e => panic!("Expected DuplicateFieldName, got {e:?}"),
    }
}

#[test]
fn semantic_duplicate_field_in_template() {
    let input = "template T { col String\n col Int }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateFieldName(n)) => assert_eq!(n, "col"),
        e => panic!("Expected DuplicateFieldName, got {e:?}"),
    }
}

#[test]
fn semantic_no_duplicate_fields_across_entities() {
    // Same field name in two different entities is OK
    let input = "entity A { id UUID }\nentity B { id UUID }";
    let f = parse_ok(input);
    assert_eq!(f.declarations.len(), 2);
}

#[test]
fn semantic_duplicate_field_attribute_same_field() {
    let input = "entity E { id UUID @id @id }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateFieldAttribute { field, attribute }) => {
            assert_eq!(field, "id");
            assert_eq!(attribute, "id");
        }
        e => panic!("Expected DuplicateFieldAttribute, got {e:?}"),
    }
}

#[test]
fn semantic_duplicate_field_attribute_default() {
    let input = "entity E { ts TIMESTAMP @default(now()) @default(now()) }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateFieldAttribute { attribute, .. }) => {
            assert_eq!(attribute, "default");
        }
        e => panic!("Expected DuplicateFieldAttribute, got {e:?}"),
    }
}

#[test]
fn semantic_different_attrs_on_same_field_ok() {
    let f = parse_ok("entity E { id UUID @id @default(gen_random_uuid()) @unique }");
    assert_eq!(entity(&f, 0).fields[0].attributes.len(), 3);
}

#[test]
fn semantic_duplicate_field_attr_error_carries_correct_field_name() {
    let input = "entity E { good_field String @unique\n bad_field Int @check(1) @check(2) }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateFieldAttribute { field, attribute }) => {
            assert_eq!(field, "bad_field");
            assert_eq!(attribute, "check");
        }
        e => panic!("Expected DuplicateFieldAttribute, got {e:?}"),
    }
}

// ============================================================
// SECTION 15: INTEGER PARSING ERRORS
// ============================================================

#[test]
fn error_integer_overflow_positive() {
    let input = "config { v: 99999999999999999999999999999 }";
    assert!(matches!(parse_err(input), Error::ParseInt(_)));
}

#[test]
fn error_integer_overflow_negative() {
    let input = "config { v: -99999999999999999999999999999 }";
    assert!(matches!(parse_err(input), Error::ParseInt(_)));
}

#[test]
fn integer_boundary_max_i64_ok() {
    let f = parse_ok("config { v: 9223372036854775807 }");
    assert!(matches!(
        f.config.unwrap().entries.get("v").unwrap(),
        Value::Integer(i64::MAX)
    ));
}

#[test]
fn integer_boundary_exceeds_max_i64() {
    // 9223372036854775808 = i64::MAX + 1
    let input = "config { v: 9223372036854775808 }";
    assert!(matches!(parse_err(input), Error::ParseInt(_)));
}

// ============================================================
// SECTION 16: FILE I/O ERRORS
// ============================================================

#[test]
fn io_error_nonexistent_file() {
    match parse_file_from_path("/no/such/file.conk") {
        Err(Error::Io(_)) => {}
        other => panic!("Expected Io error, got {:?}", other),
    }
}

#[test]
fn io_error_directory_path() {
    match parse_file_from_path("/tmp") {
        Err(Error::Io(_)) => {}
        other => panic!("Expected Io error for directory path, got {:?}", other),
    }
}

#[test]
fn file_path_valid_file_parses() {
    let dir = std::env::temp_dir();
    let path = dir.join("conk_test_valid.conk");
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(b"entity A { }").unwrap();
    drop(f);

    let result = parse_file_from_path(&path).unwrap();
    assert_eq!(result.declarations.len(), 1);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn file_path_empty_file_parses() {
    let dir = std::env::temp_dir();
    let path = dir.join("conk_test_empty.conk");
    std::fs::File::create(&path).unwrap();

    let result = parse_file_from_path(&path).unwrap();
    assert!(result.config.is_none());
    assert!(result.declarations.is_empty());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn file_path_with_all_features_parses() {
    let dir = std::env::temp_dir();
    let path = dir.join("conk_test_full.conk");
    let contents = r#"
        config { env: "test" }
        enum Role { Admin User }
        template Base { id UUID @id }
        entity User : Base { name String }
    "#;
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(contents.as_bytes()).unwrap();
    drop(f);

    let result = parse_file_from_path(&path).unwrap();
    assert!(result.config.is_some());
    assert_eq!(result.declarations.len(), 3);
    let _ = std::fs::remove_file(&path);
}

// ============================================================
// SECTION 17: SCHEMA PREFIX EXHAUSTIVE
// ============================================================

#[test]
fn schema_prefix_on_enum() {
    let f = parse_ok(r#""myschema" enum E { A }"#);
    assert_eq!(enum_decl(&f, 0).schema, Some("myschema".to_string()));
}

#[test]
fn schema_prefix_on_entity() {
    let f = parse_ok(r#""myschema" entity E { }"#);
    assert_eq!(entity(&f, 0).schema, Some("myschema".to_string()));
}

#[test]
fn schema_prefix_with_hyphen_in_name() {
    let f = parse_ok(r#""my-schema" entity E { }"#);
    assert_eq!(entity(&f, 0).schema, Some("my-schema".to_string()));
}

#[test]
fn schema_prefix_with_numbers() {
    let f = parse_ok(r#""schema123" entity E { }"#);
    assert_eq!(entity(&f, 0).schema, Some("schema123".to_string()));
}

#[test]
fn no_schema_prefix_is_none() {
    let f = parse_ok("entity E { }");
    assert_eq!(entity(&f, 0).schema, None);
}

#[test]
fn template_has_no_schema_prefix() {
    // Templates don't support schema prefix per grammar
    let f = parse_ok("template T { }");
    let t = template(&f, 0);
    assert_eq!(t.name, "T");
    // Template struct has no schema field — just confirm it parsed
}

// ============================================================
// SECTION 18: QUALIFIED IDENTIFIER EDGE CASES
// ============================================================

#[test]
fn qualified_identifier_two_parts_no_schema() {
    let f = parse_ok("config { ref: Foo.bar }");
    if let Value::QualifiedIdentifier(parts) = f.config.unwrap().entries.get("ref").unwrap() {
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "Foo");
        assert_eq!(parts[1], "bar");
    } else {
        panic!("Expected QualifiedIdentifier");
    }
}

#[test]
fn qualified_identifier_three_parts_no_schema() {
    let f = parse_ok("config { ref: a.b.c }");
    if let Value::QualifiedIdentifier(parts) = f.config.unwrap().entries.get("ref").unwrap() {
        assert_eq!(parts, &["a", "b", "c"]);
    } else {
        panic!("Expected QualifiedIdentifier");
    }
}

#[test]
fn qualified_identifier_with_string_schema() {
    let f = parse_ok(r#"config { ref: "myschema".Table.col }"#);
    if let Value::QualifiedIdentifier(parts) = f.config.unwrap().entries.get("ref").unwrap() {
        assert_eq!(parts, &["myschema", "Table", "col"]);
    } else {
        panic!("Expected QualifiedIdentifier");
    }
}

// ============================================================
// SECTION 19: COMPLEX / INTEGRATION TESTS
// ============================================================

#[test]
fn integration_full_example_like_file() {
    let input = r#"
        config {
            default_schema: "public"
        }

        "auth" enum UserRole {
            Admin
            Editor
            Viewer
            @@map("UserRoleEnum")
        }

        template CreatedAt {
            created_at DateTimeTz @default(now())
        }

        template RandomTemplate {
            random String @nullable
        }

        entity User: CreatedAt {
            id UUID @id @default(gen_random_uuid())
            name String
            email String @unique
            role UserRole
            @@index([name, email])
        }

        entity City: CreatedAt {
            id UUID @id @default(gen_random_uuid())
            name String
            description String @nullable
        }

        entity CapitalCity: RandomTemplate
            inherits (City, EntityThatDoesNotExist) {
            state String
            @@map("capital_city")
        }

        "public" entity Post {
            id UUID @id @default(gen_random_uuid())
            created_by UUID @ref("public".User.id, on_delete: Cascade, on_update: Restrict)
            @@map("post")
        }
    "#;

    let f = parse_ok(input);
    assert!(f.config.is_some());
    assert_eq!(f.declarations.len(), 7);

    // Config
    let config = f.config.as_ref().unwrap();
    assert!(
        matches!(config.entries.get("default_schema").unwrap(), Value::String(s) if s == "public")
    );

    // Enum
    let role = enum_decl(&f, 0);
    assert_eq!(role.schema, Some("auth".to_string()));
    assert_eq!(role.name, "UserRole");
    assert_eq!(role.values, vec!["Admin", "Editor", "Viewer"]);

    // Templates
    let created_at = template(&f, 1);
    assert_eq!(created_at.name, "CreatedAt");
    assert_eq!(created_at.fields.len(), 1);

    // User entity
    let user = entity(&f, 3);
    assert_eq!(user.name, "User");
    assert_eq!(user.templates, vec!["CreatedAt"]);
    assert_eq!(user.fields.len(), 4);
    assert_eq!(user.attributes.len(), 1);

    // Post entity (schema-prefixed)
    let post = entity(&f, 6);
    println!("{:#?}", post);
    assert_eq!(post.schema, Some("public".to_string()));
    assert_eq!(post.fields.len(), 2);
    let fk_attrs = &post.fields[1].attributes;
    assert_eq!(fk_attrs[0].name, "ref");
    assert_eq!(
        (fk_attrs[0].args.positional.len() + fk_attrs[0].args.named.len()),
        3
    );
}

#[test]
fn integration_deeply_nested_function_args() {
    let input = r#"
        entity E {
            v String @check(validate(cast(value(), String), minLen: 1))
        }
    "#;
    let f = parse_ok(input);
    let attr = &entity(&f, 0).fields[0].attributes[0];
    assert_eq!(attr.name, "check");
    assert_eq!((attr.args.positional.len() + attr.args.named.len()), 1);
}

#[test]
fn integration_entity_all_value_types_in_attrs() {
    let input = r#"
        entity E {
            f1 String @a("str")
            f2 Int @b(42)
            f3 Float @c(3.14)
            f4 Bool @d(true)
            f5 UUID @e(gen_random_uuid())
            f6 UUID @f(Table.col)
            f7 UUID @g([1, 2, 3])
        }
    "#;
    let f = parse_ok(input);
    let e = entity(&f, 0);
    assert_eq!(e.fields.len(), 7);
    assert!(matches!(
        &e.fields[0].attributes[0].args.positional[0],
        Value::String(_)
    ));
    assert!(matches!(
        &e.fields[1].attributes[0].args.positional[0],
        Value::Integer(42)
    ));
    assert!(matches!(
        &e.fields[2].attributes[0].args.positional[0],
        Value::Float(_)
    ));
    assert!(matches!(
        &e.fields[3].attributes[0].args.positional[0],
        Value::Identifier(_)
    ));
    assert!(matches!(
        &e.fields[4].attributes[0].args.positional[0],
        Value::FunctionCall { .. }
    ));
    assert!(matches!(
        &e.fields[5].attributes[0].args.positional[0],
        Value::QualifiedIdentifier(_)
    ));
    assert!(matches!(
        &e.fields[6].attributes[0].args.positional[0],
        Value::List(_)
    ));
}

#[test]
fn integration_whitespace_insensitive() {
    // Compact single-line vs. expanded multi-line
    let compact = r#"config{env:"prod"}entity User{id UUID @id}"#;
    let expanded = r#"
        config {
            env: "prod"
        }

        entity User {
            id UUID @id
        }
    "#;
    let f_compact = parse_ok(compact);
    let f_expanded = parse_ok(expanded);

    let c1 = f_compact.config.unwrap();
    let c2 = f_expanded.config.unwrap();
    assert!(matches!(c1.entries.get("env").unwrap(), Value::String(s) if s == "prod"));
    assert!(matches!(c2.entries.get("env").unwrap(), Value::String(s) if s == "prod"));
}

#[test]
fn integration_large_entity_many_fields() {
    let mut input = String::from("entity Big {\n");
    for i in 0..50 {
        input.push_str(&format!("    field{i} String\n"));
    }
    input.push('}');
    let f = parse_ok(&input);
    assert_eq!(entity(&f, 0).fields.len(), 50);
}

#[test]
fn integration_enum_with_many_values_and_attrs() {
    let mut input = String::from("enum Big {\n    @@map(\"big\")\n");
    for i in 0..30 {
        input.push_str(&format!("    Value{i}\n"));
    }
    input.push('}');
    let f = parse_ok(&input);
    let e = enum_decl(&f, 0);
    assert_eq!(e.values.len(), 30);
    assert_eq!(e.attributes.len(), 1);
}

// ============================================================
// SECTION 20: FLOAT EDGE CASES
// ============================================================

#[test]
fn float_very_small() {
    let f = parse_ok("config { v: 0.000000001 }");
    assert!(matches!(
        f.config.unwrap().entries.get("v").unwrap(),
        Value::Float(_)
    ));
}

#[test]
fn float_very_large() {
    let f = parse_ok("config { v: 999999999999.999999 }");
    assert!(matches!(
        f.config.unwrap().entries.get("v").unwrap(),
        Value::Float(_)
    ));
}

#[test]
fn float_and_integer_precedence() {
    // Grammar says float comes before integer, so 1.0 should be Float not Integer
    let f = parse_ok("config { a: 1.0 b: 1 }");
    let c = f.config.unwrap();
    assert!(matches!(c.entries.get("a").unwrap(), Value::Float(_)));
    assert!(matches!(c.entries.get("b").unwrap(), Value::Integer(1)));
}

// ============================================================
// SECTION 21: COMMENTS EDGE CASES
// ============================================================

#[test]
fn comment_with_special_characters() {
    let f = parse_ok("// !@#$%^&*(){}[]|<>?/\\~`\nentity E { }");
    assert_eq!(f.declarations.len(), 1);
}

#[test]
fn comment_that_looks_like_code() {
    let f = parse_ok("// entity Fake { id UUID @id }\nentity Real { }");
    assert_eq!(f.declarations.len(), 1);
    assert_eq!(entity(&f, 0).name, "Real");
}

#[test]
fn two_forward_slashes_in_string_not_comment() {
    let f = parse_ok(r#"config { url: "http://localhost" }"#);
    assert!(
        matches!(f.config.unwrap().entries.get("url").unwrap(), Value::String(s) if s == "http://localhost")
    );
}

// ============================================================
// SECTION 22: ARGUMENT ORDER TESTS
// ============================================================

#[test]
fn argument_list_order_preserved() {
    let f = parse_ok(r#"entity E { f UUID @ref(A.b, x: 1, y: "two", z: true) }"#);
    let args = &entity(&f, 0).fields[0].attributes[0].args;
    assert_eq!((args.positional.len() + args.named.len()), 4);
    assert!(matches!(&args.positional[0], _));
    assert!(matches!(&args.named[0], NamedArgument { name, .. } if name == "x"));
    assert!(matches!(&args.named[1], NamedArgument { name, .. } if name == "y"));
    assert!(matches!(&args.named[2], NamedArgument { name, .. } if name == "z"));
}

#[test]
fn block_attribute_argument_order_preserved() {
    let f = parse_ok(r#"entity E { @@idx([a, b], unique: true, name: "idx_ab") }"#);
    let args = &entity(&f, 0).attributes[0].args;
    assert_eq!((args.positional.len() + args.named.len()), 3);
    assert!(matches!(&args.positional[0], Value::List(_)));
    assert!(matches!(&args.named[0], NamedArgument { name, .. } if name == "unique"));
    assert!(matches!(&args.named[1], NamedArgument { name, .. } if name == "name"));
}

// ============================================================
// SECTION 23: REGRESSION / PREVIOUSLY TRICKY CASES
// ============================================================

#[test]
fn regression_function_call_not_confused_with_type_params() {
    // now() is a function call value, not a type param
    let f = parse_ok("entity E { ts TIMESTAMP @default(now()) }");
    let attr = &entity(&f, 0).fields[0].attributes[0];
    assert!(matches!(&attr.args.positional[0], Value::FunctionCall { name, .. } if name == "now"));
}

#[test]
fn regression_function_call_before_qualified_id_in_value_position() {
    // f() should match function_call before qualified_identifier since both start with identifier
    let f = parse_ok("config { a: f() b: a.b }");
    let c = f.config.unwrap();
    assert!(matches!(
        c.entries.get("a").unwrap(),
        Value::FunctionCall { .. }
    ));
    assert!(matches!(
        c.entries.get("b").unwrap(),
        Value::QualifiedIdentifier(_)
    ));
}

#[test]
fn regression_float_before_integer_disambiguation() {
    // "1.5" should parse as float, not integer 1 followed by something
    let f = parse_ok("config { v: 1.5 }");
    assert!(matches!(
        f.config.unwrap().entries.get("v").unwrap(),
        Value::Float(_)
    ));
}

#[test]
fn regression_schema_prefix_on_qualified_id_value() {
    // "schema".Table.col should result in 3 parts: ["schema", "Table", "col"]
    let f = parse_ok(r#"entity E { fk UUID @ref("public".Users.id) }"#);
    let attr = &entity(&f, 0).fields[0].attributes[0];
    match &attr.args.positional[0] {
        Value::QualifiedIdentifier(parts) => {
            assert_eq!(parts, &["public", "Users", "id"]);
        }
        other => panic!("Expected QualifiedIdentifier, got {:?}", other),
    }
}

#[test]
fn regression_template_with_trailing_comma_in_entity() {
    let f = parse_ok("entity E : A, B, { }");
    assert_eq!(entity(&f, 0).templates, vec!["A", "B"]);
}

#[test]
fn regression_config_does_not_need_to_appear() {
    let f = parse_ok("entity E { }");
    assert!(f.config.is_none());
    assert_eq!(f.declarations.len(), 1);
}

#[test]
fn regression_multiple_enums_same_schema_different_names() {
    let input = r#"
        "auth" enum Role { A }
        "auth" enum Status { X }
    "#;
    let f = parse_ok(input);
    assert_eq!(f.declarations.len(), 2);
    assert_eq!(enum_decl(&f, 0).name, "Role");
    assert_eq!(enum_decl(&f, 1).name, "Status");
}

// ============================================================
// SECTION 24: NAMED ARGUMENT EDGE CASES
// ============================================================

#[test]
fn named_argument_value_is_function_call() {
    let f = parse_ok("entity E { v UUID @default(value: gen_random_uuid()) }");
    let arg = &entity(&f, 0).fields[0].attributes[0].args.named[0];
    assert!(
        matches!(arg, NamedArgument { name, value: Value::FunctionCall { .. } } if name == "value")
    );
}

#[test]
fn named_argument_value_is_list() {
    let f = parse_ok("entity E { v String @check(allowed: [1, 2, 3]) }");
    let arg = &entity(&f, 0).fields[0].attributes[0].args.named[0];
    assert!(matches!(arg, NamedArgument { name, value: Value::List(_) } if name == "allowed"));
}

#[test]
fn named_argument_value_is_qualified_identifier() {
    let f = parse_ok("entity E { v UUID @ref(target: Table.col) }");
    let arg = &entity(&f, 0).fields[0].attributes[0].args.named[0];
    assert!(
        matches!(arg, NamedArgument { name, value: Value::QualifiedIdentifier(_) } if name == "target")
    );
}

#[test]
fn named_argument_value_is_integer() {
    let f = parse_ok("entity E { v Int @range(min: 0, max: 100) }");
    let args = &entity(&f, 0).fields[0].attributes[0].args;
    assert!(
        matches!(&args.named[0], NamedArgument { name, value: Value::Integer(0) } if name == "min")
    );
    assert!(
        matches!(&args.named[1], NamedArgument { name, value: Value::Integer(100) } if name == "max")
    );
}

#[test]
fn named_argument_value_is_float() {
    let f = parse_ok("entity E { v Float @precision(digits: 10.5) }");
    let arg = &entity(&f, 0).fields[0].attributes[0].args.named[0];
    assert!(matches!(arg, NamedArgument { name, value: Value::Float(_) } if name == "digits"));
}

// ============================================================
// SECTION 25: PARSE THEN ROUND-TRIP STRUCTURAL CONSISTENCY
// ============================================================

#[test]
fn field_count_matches_input() {
    let input = r#"
        entity User {
            id UUID @id
            first_name String
            last_name String
            email String @unique
            age Int
            score Float
            active Bool
            created_at TIMESTAMP @default(now())
        }
    "#;
    let f = parse_ok(input);
    assert_eq!(entity(&f, 0).fields.len(), 8);
}

#[test]
fn block_attribute_count_matches_input() {
    let input = r#"
        entity User {
            @@map("users")
            @@index([email])
            @@index([first_name, last_name])
            @@unique([email])
            id UUID
        }
    "#;
    let f = parse_ok(input);
    assert_eq!(entity(&f, 0).attributes.len(), 4);
}

#[test]
fn config_entries_count_matches_input() {
    let input = r#"
        config {
            a: 1
            b: 2
            c: 3
        }
    "#;
    let f = parse_ok(input);
    assert_eq!(f.config.unwrap().entries.len(), 3);
}

#[test]
fn enum_value_count_matches_input() {
    let f = parse_ok("enum Foo { A B C D E }");
    assert_eq!(enum_decl(&f, 0).values.len(), 5);
}

// ============================================================
// SECTION 26: AST VALUE STRUCTURAL TESTS
// ============================================================

#[test]
fn value_list_empty_vs_none() {
    // [] should be List with zero items, not None
    let f = parse_ok("config { v: [] }");
    assert!(matches!(f.config.unwrap().entries.get("v").unwrap(), Value::List(l) if l.is_empty()));
}

#[test]
fn value_function_call_args_empty_vs_none() {
    let f = parse_ok("config { v: now() }");
    if let Value::FunctionCall { args, .. } = f.config.unwrap().entries.get("v").unwrap() {
        assert!(args.is_empty());
    } else {
        panic!("Expected FunctionCall");
    }
}

#[test]
fn value_list_preserves_order() {
    let f = parse_ok(r#"config { v: ["z", "a", "m"] }"#);
    if let Value::List(items) = f.config.unwrap().entries.get("v").unwrap() {
        assert!(matches!(&items[0], Value::String(s) if s == "z"));
        assert!(matches!(&items[1], Value::String(s) if s == "a"));
        assert!(matches!(&items[2], Value::String(s) if s == "m"));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn value_function_call_args_order_preserved() {
    let f = parse_ok(r#"config { v: f("a", "b", "c") }"#);
    if let Value::FunctionCall { args, .. } = f.config.unwrap().entries.get("v").unwrap() {
        assert!(matches!(&args.positional[0], Value::String(s) if s == "a"));
        assert!(matches!(&args.positional[2], Value::String(s) if s == "c"));
    } else {
        panic!("Expected FunctionCall");
    }
}

// ============================================================
// SECTION 27: ENTITY FIELD POSITION TRACKING
// ============================================================

#[test]
fn fields_order_preserved_in_entity() {
    let input = r#"
        entity E {
            z_field String
            a_field Int
            m_field Float
        }
    "#;
    let f = parse_ok(input);
    let fields = &entity(&f, 0).fields;
    assert_eq!(fields[0].name, "z_field");
    assert_eq!(fields[1].name, "a_field");
    assert_eq!(fields[2].name, "m_field");
}

#[test]
fn fields_order_preserved_in_template() {
    let input = "template T { z String a Int m Float }";
    let f = parse_ok(input);
    let fields = &template(&f, 0).fields;
    assert_eq!(fields[0].name, "z");
    assert_eq!(fields[1].name, "a");
    assert_eq!(fields[2].name, "m");
}

// ============================================================
// SECTION 28: ARGUMENT SEMANTIC DUPLICATIONS
// ============================================================

#[test]
fn semantic_duplicate_argument_name_in_function() {
    let input = "config { val: f(a: 1, a: 2) }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateArgumentName(n)) => assert_eq!(n, "a"),
        e => panic!("Expected DuplicateArgumentName, got {e:?}"),
    }
}

#[test]
fn semantic_duplicate_argument_name_in_attribute() {
    let input = "entity E { val Int @range(min: 0, min: 10) }";
    match parse_err(input) {
        Error::Semantic(SemanticError::DuplicateArgumentName(n)) => assert_eq!(n, "min"),
        e => panic!("Expected DuplicateArgumentName, got {e:?}"),
    }
}
