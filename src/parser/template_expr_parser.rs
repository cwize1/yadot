#[cfg(test)]
mod tests;

use std::ops::Range;

use anyhow::{anyhow, Error};
use chumsky::{prelude::*, Stream};

use crate::ast::{Expr, ExprString};

use super::template_expr_lexer::{gen_lexer, Token};

pub struct TemplateExprParser {
    lexer: Box<dyn Parser<char, Vec<(Token, Range<usize>)>, Error = Simple<char>>>,
    parser: Box<dyn Parser<Token, (Expr, Range<usize>), Error = Simple<Token>>>,
}

impl TemplateExprParser {
    pub fn new() -> TemplateExprParser {
        let lexer = gen_lexer();
        let parser = gen_template_expression_parser();
        TemplateExprParser {
            lexer: Box::new(lexer),
            parser: Box::new(parser),
        }
    }

    pub fn parse(&self, expr_str: &str) -> Result<(Expr, usize), Error> {
        let tokens_res = self.lexer.parse(expr_str);
        if let Err(errs) = tokens_res {
            for err in &errs {
                println!("Parse error: {}", err)
            }
            return Err(anyhow!("expression parse errors (count={})", errs.len()));
        }

        let expr_str_len = expr_str.chars().count();
        let tokens = tokens_res.unwrap();
        let eoi = expr_str_len..expr_str_len + 1;

        let expr_res = self.parser.parse(Stream::from_iter(eoi, tokens.into_iter()));
        if let Err(errs) = expr_res {
            for err in &errs {
                println!("Parse error: {}", err)
            }
            return Err(anyhow!("expression parse errors (count={})", errs.len()));
        }
        let (expr, span) = expr_res.unwrap();
        Ok((expr, span.end()))
    }
}

fn gen_template_expression_parser() -> impl Parser<Token, (Expr, Range<usize>), Error = Simple<Token>> {
    let value = select! {
        Token::String(value) => Expr::String(ExprString{value}),
        Token::Ident(ident) if ident == "inline" => Expr::Inline,
        Token::Ident(ident) if ident == "drop" => Expr::Drop,
    };

    let expr = value;

    let templ_expr = just(Token::Start)
        .ignore_then(expr)
        .then_ignore(just(Token::End))
        .map_with_span(|expr, span| (expr, span));

    templ_expr
}
