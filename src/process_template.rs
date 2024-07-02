#[cfg(test)]
mod tests;

use anyhow::{anyhow, Error};
use yaml_rust::{Yaml, YamlLoader};

use crate::{interpreter::interpret, parser::Parser, yaml_utils::yaml_emit_to_string};

pub fn process_yaml_template_str(template_string: &str, config_string: &str) -> Result<String, Error> {
    let docs = process_yaml_template(template_string, config_string)?;
    let out_str = yaml_emit_to_string(&docs)?;
    Ok(out_str)
}

pub fn process_yaml_template(template_string: &str, config_string: &str) -> Result<Vec<Yaml>, Error> {
    let parser = Parser::new();
    let template = parser.parse(template_string)?;

    let config = YamlLoader::load_from_str(config_string)?;
    let config = match &config[..] {
        [] => &Yaml::Null,
        [config] => config,
        _ => return Err(anyhow!("config yaml must only have a single document")),
    };

    let file = interpret(&template, config)?;
    Ok(file)
}
