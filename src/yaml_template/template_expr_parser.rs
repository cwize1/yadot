use anyhow::{Error, anyhow};
use chumsky::prelude::*;

pub fn parse_template_expression(expr_str: &str) -> Result<String, Error> {
    let parser = gen_template_expression_parser();
    let expr_res = parser.parse(expr_str);
    if let Err(errs) = expr_res {
        for err in &errs {
            println!("Parse error: {}", err)
        }
        return Err(anyhow!("expression parse errors (count={})", errs.len()))
    }
    let expr = expr_res.unwrap();
    Ok(expr)
}

fn gen_template_expression_parser() -> impl Parser<char, String, Error = Simple<char>> {
    let escape = just('\\').ignore_then(
        just('\\')
            .or(just('/'))
            .or(just('"'))
            .or(just('b').to('\x08'))
            .or(just('f').to('\x0C'))
            .or(just('n').to('\n'))
            .or(just('r').to('\r'))
            .or(just('t').to('\t'))
            .or(just('u').ignore_then(
                filter(|c: &char| c.is_digit(16))
                    .repeated()
                    .exactly(4)
                    .collect::<String>()
                    .validate(|digits, span, emit| {
                        char::from_u32(u32::from_str_radix(&digits, 16).unwrap())
                            .unwrap_or_else(|| {
                                emit(Simple::custom(span, "invalid unicode character"));
                                '\u{FFFD}' // unicode replacement character
                            })
                    }),
            )),
    );

    let string = just('"')
        .ignore_then(filter(|c| *c != '\\' && *c != '"').or(escape).repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .labelled("string");

    let expr = string;

    let templ_expr = just("${{")
        .ignore_then(expr)
        .then_ignore(just("}}"));

    templ_expr
}
