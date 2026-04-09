use crate::ast::pest::Rule;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(#[from] pest::error::Error<Rule>),

    #[error("parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("parse float error: {0}")]
    ParseFloat(#[from] std::num::ParseFloatError),
}
