use std::ops::Range;

use chumsky::{error::Simple, primitive::{filter, just}, text::{self, TextParser}, Parser};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Start,
    End,
    String(String),
    Ident(String),
    Period,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Start => write!(f, "${{"),
            Token::End => write!(f, "$}}"),
            Token::String(value) => write!(f, "{:?}", value),
            Token::Ident(name) => write!(f, "{}", name),
            Token::Period => write!(f, "."),
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

    let period = just(".").map(|_| Token::Period);

    let token = start.or(end).or(string).or(ident).or(period);

    let token = token.map_with_span(|tok, span| (tok, span)).padded().repeated();
    token
}
