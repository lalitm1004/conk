use super::Value;

#[derive(Debug, Clone)]
pub struct Config {
    pub entries: Vec<(String, Value)>,
}
