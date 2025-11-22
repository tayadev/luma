use chumsky::prelude::*;
use crate::ast::{Expr, Stmt, Argument, Type};

/// Creates a parser for block expressions (do...end)
pub fn block<'a, WS, S, E>(
    ws: WS,
    stmt: S,
    expr: E,
) -> Boxed<'a, 'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    S: Parser<'a, &'a str, Stmt, extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("do")
        .padded_by(ws.clone())
        .ignore_then(stmt.repeated().collect::<Vec<Stmt>>())
        .then(expr.or_not())
        .then_ignore(just("end").padded_by(ws))
        .map(|(mut stmts, ret)| {
            if let Some(expr) = ret {
                // Explicit trailing expression captured separately
                stmts.push(Stmt::Return(expr));
            } else if let Some(last) = stmts.pop() {
                // No separate trailing expression; convert last ExprStmt into implicit return
                match last {
                    Stmt::ExprStmt(e) => stmts.push(Stmt::Return(e)),
                    other => {
                        stmts.push(other);
                    }
                }
            }
            Expr::Block(stmts)
        })
        .boxed()
}

/// Creates a parser for if expressions (if...do...else...end)
pub fn if_expr<'a, WS, S, E>(
    ws: WS,
    stmt: S,
    expr: E,
) -> Boxed<'a, 'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    S: Parser<'a, &'a str, Stmt, extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    let else_block = just("else")
        .padded_by(ws.clone())
        .then_ignore(just("do").padded_by(ws.clone()))
        .ignore_then(stmt.clone().repeated().collect::<Vec<Stmt>>())
        .map(|mut body| {
            if let Some(last) = body.pop() {
                match last {
                    Stmt::ExprStmt(e) => body.push(Stmt::Return(e)),
                    other => body.push(other),
                }
            }
            body
        });

    just("if")
        .padded_by(ws.clone())
        .ignore_then(expr)
        .then_ignore(just("do").padded_by(ws.clone()))
        .then(stmt.repeated().collect::<Vec<Stmt>>())
        .then(else_block.or_not())
        .then_ignore(just("end").padded_by(ws))
        .map(|((condition, mut then_block), else_block)| {
            if let Some(last) = then_block.pop() {
                match last {
                    Stmt::ExprStmt(e) => then_block.push(Stmt::Return(e)),
                    other => then_block.push(other),
                }
            }
            Expr::If { 
                condition: Box::new(condition), 
                then_block, 
                else_block 
            }
        })
        .boxed()
}

/// Creates a parser for function expressions
pub fn function<'a, WS, I, T, S, E>(
    ws: WS,
    ident: I,
    type_parser: T,
    stmt: S,
    expr: E,
) -> Boxed<'a, 'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    I: Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone + 'a,
    T: Parser<'a, &'a str, Type, extra::Err<Rich<'a, char>>> + Clone + 'a,
    S: Parser<'a, &'a str, Stmt, extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    // Argument parsing with default values
    let argument = ident.clone()
        .then_ignore(just(':').padded_by(ws.clone()))
        .then(type_parser.clone())
        .then(just('=').padded_by(ws.clone()).ignore_then(expr.clone()).or_not())
        .map(|((name, t), default): ((&str, Type), Option<Expr>)| Argument { 
            name: name.to_string(), 
            r#type: t, 
            default 
        });

    let arg_list = argument
        .separated_by(just(',').padded_by(ws.clone()))
        .allow_trailing()
        .collect::<Vec<Argument>>()
        .delimited_by(just('(').padded_by(ws.clone()), just(')').padded_by(ws.clone()));

    // Function body: statements + optional trailing expression as Return
    let body_block = stmt.repeated().collect::<Vec<Stmt>>()
        .then(expr.or_not())
        .then_ignore(just("end").padded_by(ws.clone()))
        .map(|(mut stmts, ret)| {
            if let Some(expr) = ret {
                stmts.push(Stmt::Return(expr));
            } else if let Some(last) = stmts.pop() {
                match last {
                    Stmt::ExprStmt(e) => stmts.push(Stmt::Return(e)),
                    other => stmts.push(other),
                }
            }
            stmts
        });

    just("fn")
        .padded_by(ws.clone())
        .ignore_then(arg_list)
        .then(just(':').padded_by(ws.clone()).ignore_then(type_parser).or_not())
        .then_ignore(just("do").padded_by(ws))
        .then(body_block)
        .map(|((arguments, return_type), body): ((Vec<Argument>, Option<Type>), Vec<Stmt>)| {
            Expr::Function { arguments, return_type, body }
        })
        .boxed()
}

/// Creates a parser for import expressions (import("path"))
pub fn import<'a, WS, E>(
    ws: WS,
    expr: E,
) -> Boxed<'a, 'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("import")
        .padded_by(ws.clone())
        .ignore_then(
            expr
                .delimited_by(
                    just('(').padded_by(ws.clone()),
                    just(')').padded_by(ws)
                )
        )
        .map(|path| Expr::Import { path: Box::new(path) })
        .boxed()
}
