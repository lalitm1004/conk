/// NOTE: The syntax used in the tests are not indicative
/// of the actual syntax that will be used for the DSL
use super::convert::{parse_file_from_path, parse_file_from_str};
use crate::ast::{Argument, Declaration, Value};
use crate::parser::error::{Error, SemanticError};
use std::io::Write;

// ==============================================
// HAPPY PATH TESTS - STRUCTURAL EXHAUSTIVENESS
// ==============================================

#[test]
fn test_empty_input() {
    let input = "";
    let file = parse_file_from_str(input).expect("Should parse empty file");
    assert!(file.config.is_none());
    assert!(file.declarations.is_empty());
}

#[test]
fn test_config_exhaustive_values() {
    let input = r#"
        config {
            str_val: "test"
            int_val: +42
            float_val: -3.1415
            list_val: ["a", 1, 2.5]
            func_val: env("DATABASE_URL")
            qual_val: "schema".ref.id
            id_val: true
        }
    "#;
    let file = parse_file_from_str(input).expect("Config should parse");
    let config = file.config.unwrap();
    assert_eq!(config.entries.len(), 7);

    // Asserting value extraction mapped from Pest tree properly
    assert!(matches!(
        config.entries.get("str_val").unwrap(),
        Value::String(s) if s == "test"
    ));
    assert!(matches!(
        config.entries.get("int_val").unwrap(),
        Value::Integer(42)
    ));
    assert!(matches!(
        config.entries.get("float_val").unwrap(),
        Value::Float(f) if *f == -3.1415
    ));
    // Check list
    if let Value::List(l) = config.entries.get("list_val").unwrap() {
        assert_eq!(l.len(), 3);
        assert!(matches!(&l[0], Value::String(s) if s == "a"));
        assert!(matches!(&l[1], Value::Integer(1)));
        assert!(matches!(&l[2], Value::Float(f) if *f == 2.5));
    } else {
        panic!("Expected list");
    }

    // Check function call
    if let Value::FunctionCall { name, args } = config.entries.get("func_val").unwrap() {
        assert_eq!(name, "env");
        assert_eq!(args.len(), 1);
        assert!(matches!(&args[0], Value::String(s) if s == "DATABASE_URL"));
    } else {
        panic!("Expected function call");
    }

    // Check qualified id
    if let Value::QualifiedIdentifier(parts) = config.entries.get("qual_val").unwrap() {
        assert_eq!(
            parts,
            &vec!["schema".to_string(), "ref".to_string(), "id".to_string()]
        );
    } else {
        panic!("Expected qualified identifier");
    }

    // Check raw id boolean placeholder
    assert!(matches!(
        config.entries.get("id_val").unwrap(),
        Value::Identifier(i) if i == "true"
    ));
}

#[test]
fn test_enum_all_variants() {
    // 1. Without schema prefix
    let req1 = parse_file_from_str("enum Simple { A B }").unwrap();
    let Declaration::Enum(e1) = &req1.declarations[0] else {
        panic!()
    };
    assert_eq!(e1.schema, None);
    assert_eq!(e1.values, vec!["A", "B"]);

    // 2. With schema prefix and attributes
    let input = r#"
        "auth" enum Role {
            @@map("roles")
            ADMIN
            USER
            @@comment("used")
        }
    "#;
    let req2 = parse_file_from_str(input).unwrap();
    let Declaration::Enum(e2) = &req2.declarations[0] else {
        panic!()
    };
    assert_eq!(e2.schema, Some("auth".to_string()));
    assert_eq!(e2.values, vec!["ADMIN", "USER"]);
    assert_eq!(e2.attributes.len(), 2);
    assert_eq!(e2.attributes[0].name, "map");
    assert_eq!(e2.attributes[1].name, "comment");
}

#[test]
fn test_template_exhaustive() {
    let input = r#"
        template Auditable {
            @@track(true)
            created_at TIMESTAMP @default(now())
            updated_at TIMESTAMP
        }
    "#;
    let file = parse_file_from_str(input).unwrap();
    let Declaration::Template(t) = &file.declarations[0] else {
        panic!()
    };

    assert_eq!(t.name, "Auditable");
    assert_eq!(t.attributes.len(), 1);
    assert_eq!(t.attributes[0].name, "track");

    assert_eq!(t.fields.len(), 2);
    assert_eq!(t.fields[0].name, "created_at");
    assert_eq!(t.fields[0].type_.name, "TIMESTAMP");
    assert_eq!(t.fields[0].attributes.len(), 1);

    assert_eq!(t.fields[1].name, "updated_at");
    assert_eq!(t.fields[1].type_.name, "TIMESTAMP");
    assert_eq!(t.fields[1].attributes.len(), 0);
}

#[test]
fn test_entity_kitchen_sink() {
    let input = r#"
        "public" entity User : Auditable, HasRoles inherits (BaseRecord) {
            @@table("users")
            @@index([email], unique: true, name: "idx_email")

            id UUID @id @default(gen_random_uuid())
            email VARCHAR(255) @unique @db("email_addr")
            score NUMERIC(10, 2)
            tags String @array(true)
            preferences JSON @default("{}")
        }
    "#;
    let file = parse_file_from_str(input).unwrap();
    let Declaration::Entity(e) = &file.declarations[0] else {
        panic!()
    };

    assert_eq!(e.schema, Some("public".to_string()));
    assert_eq!(e.name, "User");
    assert_eq!(e.templates, vec!["Auditable", "HasRoles"]);
    assert_eq!(e.inherits, vec!["BaseRecord"]);
    assert_eq!(e.attributes.len(), 2);

    // index attribute check
    let idx_attr = &e.attributes[1];
    assert_eq!(idx_attr.name, "index");
    assert_eq!(idx_attr.args.len(), 3);

    // args: [email] (positional list)
    assert!(matches!(
        &idx_attr.args[0],
        Argument::Positional(Value::List(_))
    ));
    // args: unique: true
    assert!(
        matches!(&idx_attr.args[1], Argument::Named { name, value: Value::Identifier(i) } if name == "unique" && i == "true")
    );
    // args: name: "idx_email"
    assert!(
        matches!(&idx_attr.args[2], Argument::Named { name, value: Value::String(s) } if name == "name" && s == "idx_email")
    );

    assert_eq!(e.fields.len(), 5);

    // VARCHAR params check
    let email_field = &e.fields[1];
    assert_eq!(email_field.type_.name, "VARCHAR");
    assert_eq!(email_field.type_.params.len(), 1);
    assert!(matches!(email_field.type_.params[0], Value::Integer(255)));
    assert_eq!(email_field.attributes.len(), 2);
    assert_eq!(email_field.attributes[0].name, "unique");
    assert_eq!(email_field.attributes[1].name, "db");

    // NUMERIC params check
    let score_field = &e.fields[2];
    assert_eq!(score_field.type_.name, "NUMERIC");
    assert_eq!(score_field.type_.params.len(), 2);
    assert!(matches!(score_field.type_.params[0], Value::Integer(10)));
    assert!(matches!(score_field.type_.params[1], Value::Integer(2)));
}

#[test]
fn test_string_literal_edge_cases() {
    let input = r#"
        config {
            escaped_quotes: "this has \"escaped\" quotes"
            empty: ""
            spaces: "   "
        }
    "#;
    let file = parse_file_from_str(input).unwrap();
    let config = file.config.unwrap();

    assert!(
        matches!(config.entries.get("escaped_quotes").unwrap(), Value::String(s) if s == "this has \"escaped\" quotes")
    );
    assert!(matches!(config.entries.get("empty").unwrap(), Value::String(s) if s == ""));
    assert!(matches!(config.entries.get("spaces").unwrap(), Value::String(s) if s == "   "));
}

// ==============================================
// ERROR PATH TESTS - NEGATIVE EXHAUSTIVENESS
// ==============================================

#[test]
fn test_io_error_unreadable_file() {
    let result = parse_file_from_path("/invalid/path/that/does/not/exist.conk");
    match result {
        Err(Error::Io(_)) => {} // Expected outcome
        _ => panic!("Expected IO Error for nonexistent file, got {:?}", result),
    }
}

#[test]
fn test_parse_integer_overflow() {
    // Value strictly larger than i64::MAX (9223372036854775807) resulting in ParseIntError
    let input = r#"
        config {
            huge: 999999999999999999999999999999
        }
    "#;
    let result = parse_file_from_str(input);
    match result {
        Err(Error::ParseInt(_)) => {}
        _ => panic!("Expected ParseInt error on overflow, got {:?}", result),
    }
}

// Floating point overflow via std::f64 won't normally error out on just parse (yields inf),
// but we just test valid f64 parsing to ensure float values execute at all.
#[test]
fn test_float_valid_no_panic() {
    let input = "config { val: 1.0000001 }";
    assert!(parse_file_from_str(input).is_ok());
}

#[test]
fn test_pest_grammar_error_missing_tokens() {
    // Testing multiple granular syntax failures that yield Pest Rule::Error outputs
    let cases = vec![
        ("config {", "Missing closing brace in config"),
        ("config { foo: }", "Missing value for config entry"),
        ("enum Role { @@map }", "Missing parens for block attribute"),
        ("entity User { id }", "Missing type for field"),
        (
            "template Auditable { created_at TIMESTAMP @ }",
            "Hanging field attribute",
        ),
        (
            "entity User inherits BaseModel { }",
            "Missing parens around inherits",
        ),
        ("enum {}", "Missing name in enum"),
        ("\"auth\" entity { }", "Missing entity name"),
        ("config { t: [1,2,,] }", "Trailing double commas"),
        ("config { t: func(1,,) }", "Function nested broken commas"),
    ];

    for (input, description) in cases {
        let result = parse_file_from_str(input);
        match result {
            Err(Error::Parse(_)) => {} // Expected
            _ => panic!(
                "Expected Parse error for {}: {}, got {:?}",
                description, input, result
            ),
        }
    }
}

#[test]
fn test_pest_grammar_invalid_characters() {
    let inputs = [
        "config { foo: 12.3.4 }", // Invalid float format
        "config { %$# }",         // Unrecognized tokens
        "entity 123User { }",     // Identifier starting with number
    ];

    for input in inputs {
        let result = parse_file_from_str(input);
        match result {
            Err(Error::Parse(_)) => {}
            _ => panic!("Expected parse error on invalid characters: {}", input),
        }
    }
}

#[test]
fn test_file_from_path_happy_path() {
    // We can simulate an actual file behavior utilizing tempfile or writing out briefly.
    let dir = std::env::temp_dir();
    let filepath = dir.join("test_file_happy.conk");

    let contents = "config { val: 42 }";
    let mut file = std::fs::File::create(&filepath).unwrap();
    file.write_all(contents.as_bytes()).unwrap();

    let result = parse_file_from_path(&filepath).unwrap();
    let config = result.config.unwrap();
    assert!(matches!(
        config.entries.get("val").unwrap(),
        Value::Integer(42)
    ));

    let _ = std::fs::remove_file(filepath); // cleanup
}

// ==============================================
// SEMANTIC DUPLICATION TESTS
// ==============================================

#[test]
fn test_semantic_duplicate_config_key() {
    let input = r#"
        config {
            key: 1
            key: 2
        }
    "#;
    let result = parse_file_from_str(input);
    match result {
        Err(Error::Semantic(SemanticError::DuplicateConfigKey(key))) => assert_eq!(key, "key"),
        _ => panic!(
            "Expected Semantic error for duplicate config key, got {:?}",
            result
        ),
    }
}

#[test]
fn test_semantic_duplicate_declaration() {
    let input = r#"
        entity User { }
        template User { }
    "#;
    let result = parse_file_from_str(input);
    match result {
        Err(Error::Semantic(SemanticError::DuplicateDeclarationName(name))) => {
            assert_eq!(name, "User")
        }
        _ => panic!(
            "Expected Semantic error for duplicate declaration, got {:?}",
            result
        ),
    }
}

#[test]
fn test_semantic_duplicate_enum_value() {
    let input = r#"
        enum Role {
            ADMIN
            USER
            ADMIN
        }
    "#;
    let result = parse_file_from_str(input);
    match result {
        Err(Error::Semantic(SemanticError::DuplicateEnumValue(val))) => assert_eq!(val, "ADMIN"),
        _ => panic!(
            "Expected Semantic error for duplicate enum value, got {:?}",
            result
        ),
    }
}

#[test]
fn test_semantic_duplicate_field_entity() {
    let input = r#"
        entity Box {
            size INT
            size FLOAT
        }
    "#;
    let result = parse_file_from_str(input);
    match result {
        Err(Error::Semantic(SemanticError::DuplicateFieldName(name))) => assert_eq!(name, "size"),
        _ => panic!(
            "Expected Semantic error for duplicate entity field, got {:?}",
            result
        ),
    }
}

#[test]
fn test_semantic_duplicate_field_template() {
    let input = r#"
        template Box {
            size INT
            size FLOAT
        }
    "#;
    let result = parse_file_from_str(input);
    match result {
        Err(Error::Semantic(SemanticError::DuplicateFieldName(name))) => assert_eq!(name, "size"),
        _ => panic!(
            "Expected Semantic error for duplicate template field, got {:?}",
            result
        ),
    }
}

#[test]
fn test_semantic_duplicate_field_attribute() {
    let input = r#"
        entity Box {
            id INT @id @id
        }
    "#;
    let result = parse_file_from_str(input);
    match result {
        Err(Error::Semantic(SemanticError::DuplicateFieldAttribute { field, attribute })) => {
            assert_eq!(field, "id");
            assert_eq!(attribute, "id");
        }
        _ => panic!(
            "Expected Semantic error for duplicate field attribute, got {:?}",
            result
        ),
    }
}

// ==============================================
// EXHAUSTIVE EDGE CASE TESTS
// ==============================================

#[test]
fn test_edge_case_empty_config() {
    let input = "config { }";
    let file = parse_file_from_str(input).unwrap();
    assert!(file.config.unwrap().entries.is_empty());
}

#[test]
fn test_edge_case_entity_empty_body() {
    let input = "entity User { }";
    let file = parse_file_from_str(input).unwrap();
    let Declaration::Entity(e) = &file.declarations[0] else {
        panic!()
    };
    assert!(e.fields.is_empty());
    assert!(e.attributes.is_empty());
    assert!(e.templates.is_empty());
    assert!(e.inherits.is_empty());
}

#[test]
fn test_edge_case_weird_string_escapes() {
    let input = r#"
        config {
            test: "line\nbreak\ttab\\slash\"quote"
        }
    "#;
    let file = parse_file_from_str(input).unwrap();
    let config = file.config.unwrap();
    let Value::String(s) = config.entries.get("test").unwrap() else {
        panic!()
    };
    // Our parse handles raw escapes literally per convert.rs, only escaping \"
    assert!(s.contains("line\\nbreak\\ttab\\\\slash\"quote"));
}

#[test]
fn test_edge_case_mixed_attributes_everywhere() {
    let input = r#"
        "my_schema" entity Everything : TemplateA, TemplateB inherits (Base1, Base2) {
            @@map("this")
            @@index([a, b], c: "d")
            
            f1 String @id @mapped("f_1")
            
            @@comment("block attributes interleaved")
            
            f2 UUID @default(uuid())
        }
    "#;
    let file = parse_file_from_str(input).unwrap();
    let Declaration::Entity(e) = &file.declarations[0] else {
        panic!()
    };
    // Should parse without complaint
    assert_eq!(e.attributes.len(), 3);
    assert_eq!(e.fields.len(), 2);
    assert_eq!(e.templates.len(), 2);
    assert_eq!(e.inherits.len(), 2);
}

#[test]
fn test_edge_case_field_type_params_empty() {
    let input = r#"
        template Basic {
            f1 String()
            f2 Int([])
            f3 Float(1,)
        }
    "#;
    let file = parse_file_from_str(input).unwrap();
    let Declaration::Template(t) = &file.declarations[0] else {
        panic!()
    };
    assert_eq!(t.fields[0].type_.params.len(), 0);
    assert!(matches!(&t.fields[1].type_.params[0], Value::List(l) if l.is_empty()));
    assert_eq!(t.fields[2].type_.params.len(), 1);
}

#[test]
fn test_edge_case_semantic_identical_name_diff_type() {
    let input = r#"
        entity Role { }
        enum Role { A }
    "#;
    let result = parse_file_from_str(input);
    match result {
        Err(Error::Semantic(SemanticError::DuplicateDeclarationName(name))) => {
            assert_eq!(name, "Role")
        }
        _ => panic!("Expected duplicate declaration across different types"),
    }
}
