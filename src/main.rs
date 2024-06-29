use anyhow::Error;

use yaml_template::parser::parse_yaml_template;

mod yaml_template;

fn main() {
    let out_res = process_yaml_template("hello: ${{ \"world\" }}");
    if let Err(err) = out_res {
        eprintln!("yadot failed: {err:?}");
        return;
    }
    let out = out_res.unwrap();
    println!("{out}");
}

fn process_yaml_template(input: &str) -> Result<String, Error> {
    let template = parse_yaml_template(input)?;
    todo!();
}
