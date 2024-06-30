use anyhow::Error;

use parser::Parser;

mod ast;
mod parser;

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
    println!("{:#?}", template);
    Ok(())
}
