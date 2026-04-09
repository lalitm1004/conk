use super::{FieldAttribute, Value};

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub field_attributes: Vec<FieldAttribute>,
}

#[derive(Debug, Clone)]
pub struct FieldType {
    pub schema: Option<String>,
    pub name: String,
    pub parameters: Vec<Value>,
}
