use anyhow::Error;

mod template_expr_parser;
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
    todo!();
}
