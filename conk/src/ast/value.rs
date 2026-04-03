use crate::ast::ArgumentList;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),

    Identifier(String),

    QualifiedIdentifier(Vec<String>),

    FunctionCall { name: String, args: ArgumentList },

    List(Vec<Value>),
}
