use crate::ast::value::Value;

#[derive(Debug, Clone)]
pub struct FieldAttribute {
    pub name: String,
    pub args: Vec<Argument>,
}

#[derive(Debug, Clone)]
pub struct BlockAttribute {
    pub name: String,
    pub args: Vec<Argument>,
}

#[derive(Debug, Clone)]
pub enum Argument {
    Positional(Value),
    Named { name: String, value: Value },
}
