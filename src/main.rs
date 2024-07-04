// Copyright (c) Chris Gunn.
// Licensed under the MIT license.

mod ast;
mod cow_yaml;
mod interpreter;
mod parser;
mod process_template;
mod yaml_utils;

use std::fs;

use anyhow::{Context, Error};
use clap::{Arg, ArgAction, Command, ValueHint};
use process_template::process_yaml_template_str;

fn main() -> Result<(), Error> {
    let command = Command::new("yadot")
        .arg(
            Arg::new("path")
                .required(true)
                .action(ArgAction::Set)
                .help("YAML template file"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .required(false)
                .action(ArgAction::Set)
                .value_hint(ValueHint::FilePath)
                .help("YAML file containing values that can be used in the template"),
        )
        .arg(
            Arg::new("out")
                .long("out")
                .short('o')
                .required(false)
                .action(ArgAction::Set)
                .value_hint(ValueHint::FilePath)
                .help("Path to output result to"),
        );

    let matches = command.get_matches();

    let template_path = matches.get_one::<String>("path").unwrap();
    let config_path = matches.get_one::<String>("config");
    let out_path = matches.get_one::<String>("out");

    let template = &fs::read_to_string(template_path).context(format!("reading template file ({})", template_path))?;

    let config = match config_path {
        Some(config_path) => {
            &fs::read_to_string(config_path).context(format!("reading config file ({})", config_path))?
        }
        None => "",
    };

    let result = process_yaml_template_str(template_path, template, config)?;

    match out_path {
        Some(out_path) => fs::write(out_path, result).context(format!("writing to output file ({})", out_path))?,
        None => println!("{}", result),
    }

    Ok(())
}
