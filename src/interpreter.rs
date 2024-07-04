// Copyright (c) Chris Gunn.
// Licensed under the MIT license.

mod interpreter_run;

use anyhow::Error;

use crate::{ast::FileTemplate, cow_yaml::Yaml};

use interpreter_run::InterpreterRun;

pub fn interpret(file_templ: &FileTemplate, config: Yaml) -> Result<Vec<Yaml>, Error> {
    let mut interpreter_run = InterpreterRun::new(config);
    let file = interpreter_run.interpret_file(&file_templ)?;
    Ok(file)
}
