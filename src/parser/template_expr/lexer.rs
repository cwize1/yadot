// Copyright (c) Chris Gunn.
// Licensed under the MIT license.

// This file borrows heavily from chumsky's examples, particularly in relation to JSON string and number parsing.
//
// Chumsky's license:
//
//   The MIT License (MIT)
//
//   Copyright (c) 2021 Joshua Barretto
//
//   Permission is hereby granted, free of charge, to any person obtaining a copy
//   of this software and associated documentation files (the "Software"), to deal
//   in the Software without restriction, including without limitation the rights
//   to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//   copies of the Software, and to permit persons to whom the Software is
//   furnished to do so, subject to the following conditions:
//
//   The above copyright notice and this permission notice shall be included in all
//   copies or substantial portions of the Software.

//   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//   IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//   FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//   AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//   LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//   OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//   SOFTWARE.

use std::ops::Range;

use chumsky::{
    error::Simple,
    primitive::{filter, just, one_of},
    text::{self, TextParser},
    Parser,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Start,
    End,
    String(String),
    Ident(String),
    Dot,
    Eq,
    Ne,
    LBracket,
    RBracket,
    Integer(i64),
    Real(String),
    Variable(String),
    Comma,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Start => f.write_str("${{"),
            Token::End => f.write_str("}}"),
            Token::String(value) => write!(f, "{:?}", value),
            Token::Ident(name) => f.write_str(name),
            Token::Dot => f.write_str("."),
            Token::Eq => f.write_str("=="),
            Token::Ne => f.write_str("!="),
            Token::LBracket => f.write_str("["),
            Token::RBracket => f.write_str("]"),
            Token::Integer(i) => write!(f, "{}", i),
            Token::Real(string) => f.write_str(string),
            Token::Variable(name) => write!(f, "${}", name),
            Token::Comma => f.write_str(","),
        }
    }
}

pub fn gen_lexer() -> impl Parser<char, Vec<(Token, Range<usize>)>, Error = Simple<char>> {
    let start = just("${{").map(|_| Token::Start);
    let end = just("}}").map(|_| Token::End);

    let frac = just('.').chain(text::digits(10));

    let exp = just('e')
        .or(just('E'))
        .chain(just('+').or(just('-')).or_not())
        .chain::<char, _, _>(text::digits(10));

    let number = just('-')
        .or_not()
        .chain::<char, _, _>(text::int(10))
        .chain::<char, _, _>(frac.or_not().flatten())
        .chain::<char, _, _>(exp.or_not().flatten())
        .collect::<String>()
        .map(|string| match string.parse::<i64>() {
            Ok(i) => Token::Integer(i),
            Err(_) => Token::Real(string),
        })
        .labelled("number");

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
                        char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or_else(|| {
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
        .map(|value| Token::String(value))
        .labelled("string");

    let ident = text::ident().map(|ident| Token::Ident(ident));

    let variable = just("$").ignore_then(text::ident()).map(|name| Token::Variable(name));

    let op = one_of("!=")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .try_map(|s, span| match s.as_str() {
            "==" => Ok(Token::Eq),
            "!=" => Ok(Token::Ne),
            _ => Err(Simple::custom(span, format!("unknown operator {}", s))),
        });

    let ctrl = one_of(".,[]").map(|c| match c {
        '.' => Token::Dot,
        ',' => Token::Comma,
        '[' => Token::LBracket,
        ']' => Token::RBracket,
        _ => unreachable!(),
    });

    let token = start
        .or(end)
        .or(string)
        .or(number)
        .or(variable)
        .or(ident)
        .or(ctrl)
        .or(op);

    let token = token.map_with_span(|tok, span| (tok, span)).padded().repeated();
    token
}
