use chumsky::prelude::*;
use crate::ast::{Program, Stmt, Expr, Type};

mod string;
mod lexer;
mod operators;
mod literals;
mod patterns;
mod statements;
mod expressions;

use string::string_parser;

pub fn parser<'a>() -> impl Parser<'a, &'a str, Program, extra::Err<Rich<'a, char>>> {
    // Comments and whitespace
    let ws = lexer::ws();
    
    let ident = text::ident()
        .try_map(move |s: &str, span| {
            if lexer::KEYWORDS.contains(&s) {
                Err(Rich::custom(span, format!("'{}' is a keyword and cannot be used as an identifier", s)))
            } else {
                Ok(s)
            }
        })
        .padded_by(ws.clone());
    let type_parser = ident.clone().map(|s: &str| Type::TypeIdent(s.to_string()));
    let number = literals::number(ws.clone());

    // Recursive expression and statement placeholders
    let mut expr_ref = Recursive::declare();
    let mut stmt_ref = Recursive::declare();

    // Boolean and Null literals
    let boolean = literals::boolean(ws.clone());
    let null = literals::null(ws.clone());

    // Array and Table literals
    let array = literals::array(ws.clone(), expr_ref.clone());
    let table = literals::table(ws.clone(), ident.clone(), expr_ref.clone());

    // Pattern parsing for destructuring
    let pattern = patterns::pattern(ws.clone(), ident.clone());

    // Expression parsers (blocks and functions)
    let block_expr = expressions::block(ws.clone(), stmt_ref.clone(), expr_ref.clone());
    let function = expressions::function(ws.clone(), ident.clone(), type_parser.clone(), stmt_ref.clone(), expr_ref.clone());

    // Primary expressions (atoms)
    let primary = choice((
        number.boxed(),
        string_parser(ws.clone()).boxed(),
        boolean,
        null,
        array,
        table,
        block_expr,
        function,
        ident.clone().map(|s: &str| Expr::Identifier(s.to_string())).boxed(),
    )).boxed();

    // Unary operators (not, -)
    let unary_op = operators::unary_op(ws.clone());
    let unary_expr = unary_op
        .repeated()
        .foldr(primary.clone(), |op, operand| {
            Expr::Unary { op, operand: Box::new(operand) }
        })
        .boxed();

    // Postfix operators: function calls, member access, and indexing
    let call_args = expr_ref.clone()
        .separated_by(just(',').padded_by(ws.clone()))
        .allow_trailing()
        .collect::<Vec<Expr>>()
        .delimited_by(
            just('(').padded_by(ws.clone()),
            just(')').padded_by(ws.clone())
        );

    let member_access = just('.')
        .padded_by(ws.clone())
        .ignore_then(ident.clone());

    let index = expr_ref.clone()
        .delimited_by(
            just('[').padded_by(ws.clone()),
            just(']').padded_by(ws.clone())
        );

    #[derive(Clone)]
    enum PostfixOp {
        Call(Vec<Expr>),
        Member(String),
        Index(Box<Expr>),
    }

    let postfix_op = choice((
        call_args.map(PostfixOp::Call),
        member_access.map(|m: &str| PostfixOp::Member(m.to_string())),
        index.map(|e| PostfixOp::Index(Box::new(e))),
    ));

    let postfix = unary_expr.clone().foldl(
        postfix_op.repeated(),
        |expr, op| match op {
            PostfixOp::Call(arguments) => Expr::Call { 
                callee: Box::new(expr), 
                arguments 
            },
            PostfixOp::Member(member) => Expr::MemberAccess { 
                object: Box::new(expr), 
                member 
            },
            PostfixOp::Index(index) => Expr::Index { 
                object: Box::new(expr), 
                index 
            },
        }
    ).boxed();

    // Binary operators with precedence
    let mul_op = operators::mul_op(ws.clone());
    let add_op = operators::add_op(ws.clone());
    let cmp_op = operators::cmp_op(ws.clone());
    let logical_op = operators::logical_op(ws.clone());

    // Build expression with precedence: logical > comparison > addition > multiplication > postfix > unary
    let mul_expr = postfix.clone()
        .foldl(mul_op.then(postfix.clone()).repeated(), |left, (op, right)| {
            Expr::Binary { left: Box::new(left), op, right: Box::new(right) }
        })
        .boxed();
    
    let add_expr = mul_expr.clone()
        .foldl(add_op.then(mul_expr.clone()).repeated(), |left, (op, right)| {
            Expr::Binary { left: Box::new(left), op, right: Box::new(right) }
        })
        .boxed();
    
    let cmp_expr = add_expr.clone()
        .foldl(cmp_op.then(add_expr.clone()).repeated(), |left, (op, right)| {
            Expr::Binary { left: Box::new(left), op, right: Box::new(right) }
        })
        .boxed();

    let logical_expr = cmp_expr.clone()
        .foldl(logical_op.then(cmp_expr.clone()).repeated(), |left, (op, right)| {
            Expr::Logical { left: Box::new(left), op, right: Box::new(right) }
        })
        .boxed();

    expr_ref.define(logical_expr);

    // Statement parsers



    let var_decl = statements::var_decl(ws.clone(), pattern.clone(), type_parser.clone(), expr_ref.clone());
    let return_stmt = statements::return_stmt(ws.clone(), expr_ref.clone());
    let if_stmt = statements::if_stmt(ws.clone(), expr_ref.clone(), stmt_ref.clone());
    let while_stmt = statements::while_stmt(ws.clone(), expr_ref.clone(), stmt_ref.clone());
    let do_while_stmt = statements::do_while_stmt(ws.clone(), expr_ref.clone(), stmt_ref.clone());
    let for_stmt = statements::for_stmt(ws.clone(), pattern.clone(), expr_ref.clone(), stmt_ref.clone());
    let break_stmt = statements::break_stmt(ws.clone());
    let continue_stmt = statements::continue_stmt(ws.clone());
    let assignment = statements::assignment(ws.clone(), expr_ref.clone());
    let expr_stmt = statements::expr_stmt(expr_ref.clone());

    let match_stmt = statements::match_stmt(ws.clone(), expr_ref.clone(), stmt_ref.clone(), pattern.clone());

    let stmt = choice((
        match_stmt,
        return_stmt,
        break_stmt,
        continue_stmt,
        var_decl,
        if_stmt,
        do_while_stmt,  // Must come before while_stmt to avoid ambiguity with "do"
        while_stmt,
        for_stmt,
        assignment,
        expr_stmt,
    )).boxed();
    stmt_ref.define(stmt.clone());

    // Program: statements with optional trailing expression -> Return
    ws.clone().ignore_then(
        stmt.clone()
            .repeated()
            .collect::<Vec<Stmt>>()
            .then(expr_ref.clone().or_not())
            .then_ignore(ws.clone())
            .then_ignore(end())
            .map(|(mut statements, ret)| {
                if let Some(expr) = ret {
                    statements.push(Stmt::Return(expr));
                } else if let Some(last) = statements.pop() {
                    match last {
                        Stmt::ExprStmt(e) => statements.push(Stmt::Return(e)),
                        other => statements.push(other),
                    }
                }
                Program { statements }
            })
    )
}

pub fn parse(source: &str) -> Result<Program, Vec<Rich<'_, char>>> {
    parser().parse(source).into_result()
}
