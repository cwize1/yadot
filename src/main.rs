mod ast;
mod interpreter;
mod parser;
mod process_template;
mod yaml_utils;

use process_template::process_yaml_template_str;

fn main() {
    let out_res = process_yaml_template_str("", "hello: ${{ \"world\" }}", "");
    if let Err(err) = out_res {
        eprintln!("yadot failed: {err:?}");
        return;
    }

    let out_str = out_res.unwrap();
    println!("{}", out_str);
}
