mod interpreterrun;

use anyhow::Error;
use yaml_rust::Yaml;

use crate::ast::FileTemplate;

use interpreterrun::InterpreterRun;

pub fn interpret(file_templ: &FileTemplate) -> Result<Vec<Yaml>, Error> {
    let mut interpreter_run = InterpreterRun::new();
    let file = interpreter_run.interpret_file(&file_templ)?;
    Ok(file)
}
