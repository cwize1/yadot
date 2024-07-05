// Copyright (c) Chris Gunn.
// Licensed under the MIT license.

#[cfg(test)]
mod tests;

use std::{collections::HashMap, rc::Rc};

use anyhow::{anyhow, Context, Error};

use crate::{
    cow_yaml::{parse_yaml_str, Yaml},
    interpreter::interpret,
    parser::Parser,
    yaml_utils::yaml_emit_to_string,
};

pub enum VariableValue {
    String(String),
    Yaml(String),
}

pub fn process_yaml_template_str(
    filename: &str,
    template_string: &str,
    config_string: &str,
    varargs: HashMap<String, VariableValue>,
) -> Result<String, Error> {
    let variables = varargs_to_variables(varargs)?;
    let docs = process_yaml_template(filename, template_string, config_string, variables)?;
    let out_str = yaml_emit_to_string(&docs)?;
    Ok(out_str)
}

fn process_yaml_template(
    filename: &str,
    template_string: &str,
    config_string: &str,
    variables: HashMap<String, Yaml>,
) -> Result<Vec<Yaml>, Error> {
    let parser = Parser::new();
    let template = parser.parse(filename, template_string)?;

    let config = parse_yaml_str(config_string).context("failed to parse config")?;
    let config = match &config[..] {
        [] => Yaml::Null,
        [config] => config.clone(),
        _ => return Err(anyhow!("config yaml must only have a single document")),
    };

    let file = interpret(&template, config, variables)?;
    Ok(file)
}

fn varargs_to_variables(varargs: HashMap<String, VariableValue>) -> Result<HashMap<String, Yaml>, Error> {
    let mut variables = HashMap::new();
    for (name, value) in varargs {
        let value = match value {
            VariableValue::String(value) => Yaml::String(Rc::new(value)),
            VariableValue::Yaml(value) => {
                let value = parse_yaml_str(&value).context(format!("failed to parse yamlarg '{}'", name))?;
                let value = match &value[..] {
                    [] => Yaml::Null,
                    [value] => value.clone(),
                    _ => return Err(anyhow!("yamlarg '{}' has more than one document", name)),
                };
                value
            }
        };

        variables.insert(name, value);
    }
    Ok(variables)
}
