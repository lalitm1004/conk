use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "../grammar/conk.pest"]
pub struct ConkParser;
