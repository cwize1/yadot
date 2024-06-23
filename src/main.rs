use std::{error::Error, io::{self, BufRead, Write}};

use lrlex::lrlex_mod;
use lrpar::lrpar_mod;

lrlex_mod!("exprlang/exprlang.l");
lrpar_mod!("exprlang/exprlang.y");

fn main() {
    
    let stdin = io::stdin();
    loop {
        print!(">>> ");
        io::stdout().flush().ok();
        match stdin.lock().lines().next() {
            Some(Ok(ref l)) => {
                if l.trim().is_empty() {
                    continue;
                }
                // Now we create a lexer with the `lexer` method with which
                // we can lex an input.
                
                // Pass the lexer to the parser and lex and parse the input.
                
                for e in errs {
                    println!("{}", e.pp(&lexer, &exprlang_y::token_epp));
                }
                match res {
                    Some(Ok(r)) => println!("Result: {:?}", r),
                    _ => eprintln!("Unable to evaluate expression.")
                }
            }
            _ => break
        }
    }
}

fn process_yaml_template(input: &str) -> Result<String, Box<dyn Error>> {
    let lexerdef = exprlang_l::lexerdef();
    let lexer = lexerdef.lexer(input);
    let (res, errs) = exprlang_y::parse(&lexer);
}
