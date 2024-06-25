use anyhow::{Error, anyhow};
use lrlex::lrlex_mod;
use lrpar::lrpar_mod;

lrlex_mod!("exprlang/exprlang.l");
lrpar_mod!("exprlang/exprlang.y");

fn main() {
    let err = process_yaml_template("hello: ${{ \"world\" }}");
    if let Err(err) = err {
        eprintln!("yadot failed: {err:?}");
    }
}

fn process_yaml_template(input: &str) -> Result<(), Error> {



    Ok(())
}

fn parse_template_expression(expr_str: &str) -> Result<&str, Error> {
    let lexerdef = exprlang_l::lexerdef();
    let lexer = lexerdef.lexer(expr_str);
    let (res, errs) = exprlang_y::parse(&lexer);
    if errs.len() > 0 {
        for e in &errs {
            eprintln!("{}", e.pp(&lexer, &exprlang_y::token_epp));
        }
        return Err(anyhow!("expression parse errors (count={})", errs.len()))
    }
    let res = res.unwrap();
    let res = res.unwrap();
    Ok(res)
}
