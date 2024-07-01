#[cfg(test)]
mod tests;

use anyhow::Error;
use yaml_rust::YamlEmitter;

use crate::{interpreter::interpret, parser::Parser};

pub fn process_yaml_template(template_string: &str) -> Result<String, Error> {
    let parser = Parser::new();
    let template = parser.parse(template_string)?;
    let file = interpret(&template)?;

    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    for doc in &file {
        emitter.dump(doc)?;
    }

    Ok(out_str)
}
