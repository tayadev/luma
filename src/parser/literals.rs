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

/// Creates a parser for array literals [expr, expr, ...]
pub fn array<'a, WS, E>(ws: WS, expr: E) -> Boxed<'a, 'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    expr.separated_by(just(',').padded_by(ws.clone()))
        .allow_trailing()
        .collect::<Vec<Expr>>()
        .delimited_by(just('[').padded_by(ws.clone()), just(']').padded_by(ws.clone()))
        .map(Expr::Array)
        .boxed()
}

/// Creates a parser for table literals {key = value, ...}
pub fn table<'a, WS, I, E>(ws: WS, ident: I, expr: E) -> Boxed<'a, 'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    I: Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    let table_entry = ident
        .then_ignore(just('=').padded_by(ws.clone()))
        .then(expr)
        .map(|(k, v): (&str, Expr)| (k.to_string(), v));
    
    table_entry
        .separated_by(just(',').padded_by(ws.clone()))
        .allow_trailing()
        .collect::<Vec<(String, Expr)>>()
        .delimited_by(just('{').padded_by(ws.clone()), just('}').padded_by(ws.clone()))
        .map(Expr::Table)
        .boxed()
}
