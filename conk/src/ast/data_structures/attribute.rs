use super::Argument;

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
