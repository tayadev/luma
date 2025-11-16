use chumsky::prelude::*;
use crate::ast::Expr;

/// Creates a parser for boolean literals (true/false)
pub fn boolean<'a, WS>(ws: WS) -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    choice((
        just("true").to(Expr::Boolean(true)),
        just("false").to(Expr::Boolean(false)),
    )).padded_by(ws).boxed()
}

/// Creates a parser for null literal
pub fn null<'a, WS>(ws: WS) -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("null").to(Expr::Null).padded_by(ws).boxed()
}
