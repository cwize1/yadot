// Copyright (c) Chris Gunn.
// Licensed under the MIT license.

#[cfg(test)]
mod tests;

use std::{ops::Range, rc::Rc};

use anyhow::{anyhow, Error};
use chumsky::{prelude::*, Stream};

use crate::ast::{Expr, ExprIndex, ExprInteger, ExprOpBinary, ExprQuery, ExprReal, ExprString, Statement, StatementIf};

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
            Token::String(value) => Expr::String(ExprString{value: Rc::new(value)}),
            Token::Integer(value) => Expr::Integer(ExprInteger{value}),
            Token::Real(value) => Expr::Real(ExprReal{value: Rc::new(value)}),
            Token::Ident(ident) if ident == "inline" => Expr::Inline,
            Token::Ident(ident) if ident == "drop" => Expr::Drop,
            Token::Ident(ident) if ident == "true" => Expr::True,
            Token::Ident(ident) if ident == "false" => Expr::False,
        }
        .labelled("value");

        let ident = select! { Token::Ident(name) => name}.labelled("identifier");

        let query_root = just(Token::Dot).to(ExprQuery::Root);

        enum SubQuery {
            Index(Expr),
        }

        let subquery_ident = ident.map(|index| SubQuery::Index(Expr::String(ExprString { value: Rc::new(index) })));

        let subquery_index = expr
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map(|index| SubQuery::Index(index));

        let subquery = subquery_ident.or(subquery_index);

        let subquery_fold = move |object, subquery| match subquery {
            SubQuery::Index(index) => ExprQuery::Index(ExprIndex {
                object: Box::new(object),
                index: Box::new(index),
            }),
        };

        let query = query_root
            .clone()
            .then(subquery.clone())
            .map(move |(object, index)| subquery_fold(object, index))
            .then(just(Token::Dot).ignore_then(subquery).repeated())
            .foldl(subquery_fold);

        let query = query.or(query_root).map(|query| Expr::Query(query));

        let atom = value.or(query);

        let compare_op = just(Token::Eq).or(just(Token::Ne));

        let compare = atom
            .clone()
            .then(compare_op.then(atom).repeated())
            .foldl(|left, (token, right)| match token {
                Token::Eq => Expr::Eq(ExprOpBinary {
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                Token::Ne => Expr::Ne(ExprOpBinary {
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                _ => unreachable!(),
            });

        compare
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
