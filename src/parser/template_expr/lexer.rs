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
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Start => write!(f, "${{"),
            Token::End => write!(f, "$}}"),
            Token::String(value) => write!(f, "{:?}", value),
            Token::Ident(name) => write!(f, "{}", name),
            Token::Dot => write!(f, "."),
            Token::Eq => write!(f, "=="),
            Token::Ne => write!(f, "!="),
        }
    }
}

pub fn gen_lexer() -> impl Parser<char, Vec<(Token, Range<usize>)>, Error = Simple<char>> {
    let start = just("${{").map(|_| Token::Start);
    let end = just("}}").map(|_| Token::End);

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

    let op = one_of("!=")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .try_map(|s, span| match s.as_str() {
            "==" => Ok(Token::Eq),
            "!=" => Ok(Token::Ne),
            _ => Err(Simple::custom(span, format!("unknown operator {}", s))),
        });

    let ctrl = one_of(".").map(|c| match c {
        '.' => Token::Dot,
        _ => unreachable!(),
    });

    let token = start.or(end).or(string).or(ident).or(ctrl).or(op);

    let token = token.map_with_span(|tok, span| (tok, span)).padded().repeated();
    token
}
