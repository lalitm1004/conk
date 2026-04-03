use crate::ast::{config::Config, declaration::Declaration};

#[derive(Debug, Clone)]
pub struct ConkFile {
    pub config: Option<Config>,
    pub declarations: Vec<Declaration>,
}
