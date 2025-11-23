use chumsky::prelude::*;

/// Parser for line comments (-- to end of line)
pub fn line_comment<'a>() -> impl Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone {
    just("--")
        .then(none_of("\n").repeated())
        .ignored()
}

/// Parser for block comments (--[[ ... ]])
pub fn block_comment<'a>() -> impl Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone {
    just("--[[")
        .then(any().and_is(just("]]").not()).repeated())
        .then(just("]]"))
        .ignored()
}

/// Parser for all whitespace and comments
pub fn ws<'a>() -> impl Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone {
    let comment = choice((block_comment(), line_comment()));
    let ws_item = comment.or(one_of(" \t\r\n").ignored());
    ws_item.repeated()
}

/// List of reserved keywords
pub const KEYWORDS: &[&str] = &[
    "let", "var", "fn", "do", "end", "return", 
    "true", "false", "null", "if", "else", 
    "while", "for", "in", "break", "continue",
    "match", "await",
];
