use crate::ast::value::Value;

#[derive(Debug, Clone)]
pub struct FieldAttribute {
    pub name: String,
    pub args: ArgumentList,
}

#[derive(Debug, Clone)]
pub struct BlockAttribute {
    pub name: String,
    pub args: ArgumentList,
}

#[derive(Debug, Clone, Default)]
pub struct ArgumentList {
    pub positional: Vec<Value>,
    pub named: Vec<NamedArgument>,
}

impl ArgumentList {
    pub fn len(&self) -> usize {
        self.positional.len() + self.named.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.positional.is_empty() && self.named.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct NamedArgument {
    pub name: String,
    pub value: Value,
}
