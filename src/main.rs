use anyhow::Error;

use parser::parse_yaml_template;

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
    let template = parse_yaml_template(input)?;
    println!("{:#?}", template);
    Ok(())
}
