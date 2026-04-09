use super::{BlockAttribute, Field};

#[derive(Debug, Clone)]
pub struct EnumDeclaration {
    pub schema: Option<String>,
    pub name: String,
    pub values: Vec<String>,
    pub block_attributes: Vec<BlockAttribute>,
}

#[derive(Debug, Clone)]
pub struct TemplateDeclaration {
    pub name: String,
    pub fields: Vec<Field>,
    pub block_attributes: Vec<BlockAttribute>,
}

#[derive(Debug, Clone)]
pub struct EntityDeclaration {
    pub schema: Option<String>,
    pub name: String,

    pub templates: Vec<String>,
    pub inherits: Vec<String>,

    pub fields: Vec<Field>,
    pub block_attributes: Vec<BlockAttribute>,
}
