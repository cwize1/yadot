mod interpreter_run;

use anyhow::Error;
use yaml_rust::Yaml;

use crate::ast::FileTemplate;

use interpreter_run::InterpreterRun;

pub fn interpret(file_templ: &FileTemplate, config: &Yaml) -> Result<Vec<Yaml>, Error> {
    let mut interpreter_run = InterpreterRun::new(config);
    let file = interpreter_run.interpret_file(&file_templ)?;
    Ok(file)
}
