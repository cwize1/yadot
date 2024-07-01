#[cfg(test)]
mod tests;

use anyhow::{anyhow, Error};
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use crate::{interpreter::interpret, parser::Parser};

pub fn process_yaml_template(template_string: &str, config_string: &str) -> Result<String, Error> {
    let parser = Parser::new();
    let template = parser.parse(template_string)?;

    let config = YamlLoader::load_from_str(config_string)?;
    let config = match &config[..] {
        [] => &Yaml::Null,
        [config] => config,
        _ => return Err(anyhow!("config yaml must only have a single document")),
    };

    let file = interpret(&template, config)?;

    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    for doc in &file {
        emitter.dump(doc)?;
    }

    Ok(out_str)
}
