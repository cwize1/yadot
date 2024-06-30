use std::ops::Range;

use anyhow::{anyhow, Error};
use chumsky::prelude::*;

use crate::ast::{Expr, ExprString};

pub struct TemplateExprParser {
    parser: Box<dyn Parser<char, (String, Range<usize>), Error = Simple<char>>>,
}

impl TemplateExprParser {
    pub fn new() -> TemplateExprParser {
        let parser = gen_template_expression_parser();
        TemplateExprParser {
            parser: Box::new(parser),
        }
    }

    pub fn parse(&self, expr_str: &str) -> Result<(Expr, usize), Error> {
        let expr_res = self.parser.parse(expr_str);
        if let Err(errs) = expr_res {
            for err in &errs {
                println!("Parse error: {}", err)
            }
            return Err(anyhow!("expression parse errors (count={})", errs.len()));
        }
        let (expr, span) = expr_res.unwrap();
        let expr = Expr::String(ExprString { value: expr });
        Ok((expr, span.end()))
    }
}

fn gen_template_expression_parser(
) -> impl Parser<char, (String, Range<usize>), Error = Simple<char>> {
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
                        char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or_else(
                            || {
                                emit(Simple::custom(span, "invalid unicode character"));
                                '\u{FFFD}' // unicode replacement character
                            },
                        )
                    }),
            )),
    );

    let string = just('"')
        .ignore_then(filter(|c| *c != '\\' && *c != '"').or(escape).repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .labelled("string");

    let expr = string.padded();

    let templ_expr = just("${{")
        .ignore_then(expr)
        .then_ignore(just("}}"))
        .map_with_span(|expr, span| (expr, span));

    templ_expr
}
