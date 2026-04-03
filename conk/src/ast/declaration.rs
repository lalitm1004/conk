use crate::ast::{attribute::BlockAttribute, field::Field};

#[derive(Debug, Clone)]
pub enum Declaration {
    Enum(Enum),
    Template(Template),
    Entity(Entity),
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub schema: Option<String>,
    pub name: String,
    pub values: Vec<String>,
    pub attributes: Vec<BlockAttribute>,
}

#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub fields: Vec<Field>,
    pub attributes: Vec<BlockAttribute>,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub schema: Option<String>,
    pub name: String,

    pub templates: Vec<String>,
    pub inherits: Vec<String>,

    pub fields: Vec<Field>,
    pub attributes: Vec<BlockAttribute>,
}
