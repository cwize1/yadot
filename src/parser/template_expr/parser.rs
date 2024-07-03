#[cfg(test)]
mod tests;

use std::ops::Range;

use anyhow::{anyhow, Error};
use chumsky::{prelude::*, Stream};

use crate::ast::{Expr, ExprIdent, ExprObjectIndex, ExprQuery, ExprString, Statement, StatementIf};

use super::lexer::{gen_lexer, Token};

pub struct TemplateExprParser {
    lexer: Box<dyn Parser<char, Vec<(Token, Range<usize>)>, Error = Simple<char>>>,
    parser: Box<dyn Parser<Token, (Statement, Range<usize>), Error = Simple<Token>>>,
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

    pub fn parse(&self, expr_str: &str) -> Result<(Statement, usize), Error> {
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

        let res = self.parser.parse(Stream::from_iter(eoi, tokens.into_iter()));
        if let Err(errs) = res {
            for err in &errs {
                println!("Parse error: {}", err)
            }
            return Err(anyhow!("expression parse errors (count={})", errs.len()));
        }
        let (statement, span) = res.unwrap();
        Ok((statement, span.end()))
    }
}

fn gen_template_expression_parser() -> impl Parser<Token, (Statement, Range<usize>), Error = Simple<Token>> {
    let expr = recursive(|expr| {
        let value = select! {
            Token::String(value) => Expr::String(ExprString{value}),
            Token::Ident(ident) if ident == "inline" => Expr::Inline,
            Token::Ident(ident) if ident == "drop" => Expr::Drop,
        }
        .labelled("value");

        let ident = select! { Token::Ident(name) => ExprIdent{name}}.labelled("identifier");

        let query_root = just(Token::Dot).to(ExprQuery::Root);

        let query = query_root
            .clone()
            .then(ident)
            .map(|(object, index)| {
                ExprQuery::ObjectIndex(ExprObjectIndex {
                    object: Box::new(object),
                    index,
                })
            })
            .then(just(Token::Dot).ignore_then(ident).repeated())
            .foldl(|object, index| {
                ExprQuery::ObjectIndex(ExprObjectIndex {
                    object: Box::new(object),
                    index,
                })
            });

        let query = query.or(query_root).map(|query| Expr::Query(query));

        let atom = value.or(query);

        atom
    });

    let if_statment = just(Token::Ident("if".to_string()))
        .ignore_then(expr.clone())
        .map(|condition| Statement::If(StatementIf { condition }));

    let expr_statement = expr.map(|expr| Statement::Expr(expr));

    let statement = if_statment.or(expr_statement);

    let templ_expr = just(Token::Start)
        .ignore_then(statement)
        .then_ignore(just(Token::End))
        .map_with_span(|statement, span| (statement, span));

    templ_expr
}
