use chumsky::prelude::*;
use crate::ast::{Stmt, Expr, Pattern, Type};

/// Creates a parser for return statements
pub fn return_stmt<'a, WS, E>(ws: WS, expr: E) -> Boxed<'a, 'a, &'a str, Stmt, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("return")
        .padded_by(ws)
        .ignore_then(expr)
        .map(Stmt::Return)
        .boxed()
}

/// Creates a parser for variable declarations (let/var)
pub fn var_decl<'a, WS, P, T, E>(
    ws: WS,
    pattern: P,
    type_parser: T,
    expr: E,
) -> Boxed<'a, 'a, &'a str, Stmt, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    P: Parser<'a, &'a str, Pattern, extra::Err<Rich<'a, char>>> + Clone + 'a,
    T: Parser<'a, &'a str, Type, extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    let var_decl_token = choice((just("let").to(false), just("var").to(true)))
        .padded_by(ws.clone())
        .then(choice((
            pattern.map(|p| match p {
                Pattern::Ident(name) => (None, Some(name)),
                _ => (Some(p), None),
            }),
        )))
        .then(just(':').padded_by(ws.clone()).ignore_then(type_parser).or_not())
        .then_ignore(just('=').padded_by(ws));

    var_decl_token
        .then(expr)
        .map(|(((mutable, (pattern, name)), opt_type), value)| {
            if let Some(pattern) = pattern {
                Stmt::DestructuringVarDecl { mutable, pattern, value }
            } else {
                Stmt::VarDecl { mutable, name: name.unwrap(), r#type: opt_type, value }
            }
        })
        .boxed()
}
