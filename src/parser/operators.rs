use chumsky::prelude::*;
use crate::ast::{BinaryOp, UnaryOp, LogicalOp, AssignOp};

/// Creates a parser for unary operators
pub fn unary_op<'a, WS>(ws: WS) -> impl Parser<'a, &'a str, UnaryOp, extra::Err<Rich<'a, char>>> + Clone
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    choice((
        just("not").padded_by(ws.clone()).to(UnaryOp::Not),
        just('-').padded_by(ws.clone()).to(UnaryOp::Neg),
    ))
}

/// Creates a parser for logical operators
pub fn logical_op<'a, WS>(ws: WS) -> impl Parser<'a, &'a str, LogicalOp, extra::Err<Rich<'a, char>>> + Clone
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    choice((
        just("and").padded_by(ws.clone()).to(LogicalOp::And),
        just("or").padded_by(ws.clone()).to(LogicalOp::Or),
    ))
}

/// Creates a parser for assignment operators
pub fn assign_op<'a, WS>(ws: WS) -> impl Parser<'a, &'a str, AssignOp, extra::Err<Rich<'a, char>>> + Clone
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just('=').padded_by(ws).to(AssignOp::Assign)
}

/// Creates a parser for multiplication/division/modulo operators
pub fn mul_op<'a, WS>(ws: WS) -> impl Parser<'a, &'a str, BinaryOp, extra::Err<Rich<'a, char>>> + Clone
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    let op = |c| just(c).padded_by(ws.clone());
    choice((
        op('*').to(BinaryOp::Mul),
        op('/').to(BinaryOp::Div),
        op('%').to(BinaryOp::Mod),
    ))
}

/// Creates a parser for addition/subtraction operators
pub fn add_op<'a, WS>(ws: WS) -> impl Parser<'a, &'a str, BinaryOp, extra::Err<Rich<'a, char>>> + Clone
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    let op = |c| just(c).padded_by(ws.clone());
    choice((
        op('+').to(BinaryOp::Add),
        op('-').to(BinaryOp::Sub),
    ))
}

/// Creates a parser for comparison operators
pub fn cmp_op<'a, WS>(ws: WS) -> impl Parser<'a, &'a str, BinaryOp, extra::Err<Rich<'a, char>>> + Clone
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    let op = |c| just(c).padded_by(ws.clone());
    choice((
        just("==").padded_by(ws.clone()).to(BinaryOp::Eq),
        just("!=").padded_by(ws.clone()).to(BinaryOp::Ne),
        just("<=").padded_by(ws.clone()).to(BinaryOp::Le),
        just(">=").padded_by(ws.clone()).to(BinaryOp::Ge),
        op('<').to(BinaryOp::Lt),
        op('>').to(BinaryOp::Gt),
    ))
}
