use std::collections::HashMap;

use crate::ast::value::Value;

#[derive(Debug, Clone)]
pub struct Config {
    pub entries: HashMap<String, Value>,
}
