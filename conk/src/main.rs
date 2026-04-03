use std::fs;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "../grammar/conk.pest"]
struct ConkParser;

fn main() {
    let input = fs::read_to_string("example.conk").expect("Failed to read file");

    let pairs = ConkParser::parse(Rule::file, &input).expect("Parse failed");

    println!("{:#?}", pairs);
}
