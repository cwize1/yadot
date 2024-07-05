// Copyright (c) Chris Gunn.
// Licensed under the MIT license.

mod ast;
mod cow_yaml;
mod interpreter;
mod parser;
mod process_template;
mod variable_arg;
mod yaml_utils;

use std::{
    collections::{BinaryHeap, HashMap},
    fs,
};

use anyhow::{Context, Error};
use clap::{Arg, ArgAction, Command, ValueHint};
use process_template::{process_yaml_template_str, VariableValue};
use variable_arg::VariableArg;

fn main() -> Result<(), Error> {
    let command = Command::new("yadot")
        .arg(
            Arg::new("path")
                .required(true)
                .action(ArgAction::Set)
                .value_hint(ValueHint::FilePath)
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
        )
        .arg(
            Arg::new("arg")
                .long("arg")
                .required(false)
                .action(ArgAction::Append)
                .num_args(2)
                .value_names(["name", "value"])
                .help("Assigns a string value to a variable"),
        )
        .arg(
            Arg::new("argyaml")
                .long("argyaml")
                .required(false)
                .action(ArgAction::Append)
                .num_args(2)
                .value_names(["name", "value"])
                .help("Assigns a YAML/JSON value to a variable"),
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

    // We want to process the --arg and --argyaml args in a single combined ordering.
    let mut ordered_varargs = BinaryHeap::new();

    if let Some(args) = matches.get_occurrences::<String>("arg") {
        let args_indices: Vec<usize> = matches.indices_of("arg").unwrap().collect();

        for (i, mut arg) in args.enumerate() {
            let name = arg.next().unwrap();
            let value = arg.next().unwrap();
            let index = args_indices[i * 2];

            eprintln!("{}: {}={}", index, name, value);

            ordered_varargs.push(VariableArg {
                index,
                name: name.clone(),
                value: VariableValue::String(value.clone()),
            });
        }
    }

    if let Some(argyamls) = matches.get_occurrences::<String>("argyaml") {
        let args_indices: Vec<usize> = matches.indices_of("argyaml").unwrap().collect();

        for (i, mut arg) in argyamls.enumerate() {
            let name = arg.next().unwrap();
            let value = arg.next().unwrap();
            let index = args_indices[i * 2];

            eprintln!("{}: {}={}", index, name, value);

            ordered_varargs.push(VariableArg {
                index,
                name: name.clone(),
                value: VariableValue::Yaml(value.clone()),
            });
        }
    }

    let mut varargs = HashMap::new();
    while let Some(vararg) = ordered_varargs.pop() {
        varargs.insert(vararg.name, vararg.value);
    }

    let result = process_yaml_template_str(template_path, template, config, varargs)?;

    match out_path {
        Some(out_path) => fs::write(out_path, result).context(format!("writing to output file ({})", out_path))?,
        None => println!("{}", result),
    }

    Ok(())
}
