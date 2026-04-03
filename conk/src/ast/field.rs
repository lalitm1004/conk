use crate::ast::{attribute::FieldAttribute, value::Value};

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub type_: TypeExpr,
    pub attributes: Vec<FieldAttribute>,
}

#[derive(Debug, Clone)]
pub struct TypeExpr {
    pub name: String,
    pub params: Vec<Value>,
}
