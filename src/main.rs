mod ast;
mod interpreter;
mod parser;

use anyhow::Error;

use interpreter::interpret;
use parser::Parser;
use yaml_rust::YamlEmitter;

fn main() {
    let out_res = process_yaml_template("hello: ${{ \"world\" }}");
    if let Err(err) = out_res {
        eprintln!("yadot failed: {err:?}");
        return;
    }
}

fn process_yaml_template(input: &str) -> Result<(), Error> {
    let parser = Parser::new();
    let template = parser.parse(input)?;
    let file = interpret(&template)?;

    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    for doc in &file {
        emitter.dump(doc)?;
    }

    println!("{}", out_str);
    Ok(())
}
