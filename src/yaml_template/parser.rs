use anyhow::Error;
use yaml_rust::parser::Parser;

use super::ast::FileTemplate;

pub fn parse_yaml_template(input: &str) -> Result<FileTemplate, Error> {
    let yaml_parser = Parser::new(input.chars());
    todo!();
}
