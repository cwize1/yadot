// Copyright (c) Chris Gunn.
// Licensed under the MIT license.

#[cfg(test)]
mod tests;

use anyhow::{anyhow, Error};

use crate::{
    cow_yaml::{parse_yaml_str, Yaml},
    interpreter::interpret,
    parser::Parser,
    yaml_utils::yaml_emit_to_string,
};

pub fn process_yaml_template_str(filename: &str, template_string: &str, config_string: &str) -> Result<String, Error> {
    let docs = process_yaml_template(filename, template_string, config_string)?;
    let out_str = yaml_emit_to_string(&docs)?;
    Ok(out_str)
}

pub fn process_yaml_template(filename: &str, template_string: &str, config_string: &str) -> Result<Vec<Yaml>, Error> {
    let parser = Parser::new();
    let template = parser.parse(filename, template_string)?;

    let config = parse_yaml_str(config_string)?;
    let config = match &config[..] {
        [] => Yaml::Null,
        [config] => config.clone(),
        _ => return Err(anyhow!("config yaml must only have a single document")),
    };

    let file = interpret(&template, config)?;
    Ok(file)
}
