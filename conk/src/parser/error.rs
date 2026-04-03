use crate::parser::pest::Rule;
use pest::error::Error as PestError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(#[from] PestError<Rule>),

    #[error("parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("parse float error: {0}")]
    ParseFloat(#[from] std::num::ParseFloatError),

    #[error(transparent)]
    Semantic(#[from] SemanticError),
}

#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("Duplicate config key: {0}")]
    DuplicateConfigKey(String),

    #[error("Duplicate declaration name: {0}")]
    DuplicateDeclarationName(String),

    #[error("Duplicate enum value: {0}")]
    DuplicateEnumValue(String),

    #[error("Duplicate field name: {0}")]
    DuplicateFieldName(String),

    #[error("Duplicate field attribute on '{field}': @{attribute}")]
    DuplicateFieldAttribute { field: String, attribute: String },
}
