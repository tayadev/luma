use chumsky::prelude::*;
use crate::ast::{Program, Stmt, Expr, Type, CallArgument};

mod string;
mod lexer;
mod operators;
mod literals;
mod patterns;
mod statements;
mod expressions;
mod utils;

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
    
    // Type parser - handles TypeIdent, GenericType, FunctionType, and Any
    let mut type_ref = Recursive::declare();
    
    // Parse "Any" keyword as a type
    let any_type = just("Any")
        .padded_by(ws.clone())
        .to(Type::Any);
    
    // Parse simple type identifier (not "Any")
    let type_ident = text::ident()
        .try_map(move |s: &str, span| {
            if s == "Any" {
                // "Any" should be parsed by any_type parser
                Err(Rich::custom(span, ""))
            } else if lexer::KEYWORDS.contains(&s) {
                Err(Rich::custom(span, format!("'{}' is a keyword and cannot be used as a type", s)))
            } else {
                Ok(Type::TypeIdent(s.to_string()))
            }
        })
        .padded_by(ws.clone());
    
    // Parse function type: fn(Type, Type, ...): Type
    let function_type = just("fn")
        .padded_by(ws.clone())
        .ignore_then(
            type_ref.clone()
                .separated_by(just(',').padded_by(ws.clone()))
                .allow_trailing()
                .collect::<Vec<Type>>()
                .delimited_by(
                    just('(').padded_by(ws.clone()),
                    just(')').padded_by(ws.clone())
                )
        )
        .then_ignore(just(':').padded_by(ws.clone()))
        .then(type_ref.clone())
        .map(|(param_types, return_type)| Type::FunctionType {
            param_types,
            return_type: Box::new(return_type),
        });
    
    // Parse generic type: TypeIdent(Type, Type, ...)
    let generic_type = text::ident()
        .try_map(move |s: &str, span| {
            if lexer::KEYWORDS.contains(&s) {
                Err(Rich::custom(span, format!("'{}' is a keyword and cannot be used as a type", s)))
            } else {
                Ok(s.to_string())
            }
        })
        .padded_by(ws.clone())
        .then(
            type_ref.clone()
                .separated_by(just(',').padded_by(ws.clone()))
                .at_least(1)
                .allow_trailing()
                .collect::<Vec<Type>>()
                .delimited_by(
                    just('(').padded_by(ws.clone()),
                    just(')').padded_by(ws.clone())
                )
        )
        .map(|(name, type_args)| Type::GenericType { name, type_args });
    
    // Combine all type parsers, with priority: function > generic > any > ident
    let type_parser = choice((
        function_type.boxed(),
        generic_type.boxed(),
        any_type.boxed(),
        type_ident.boxed(),
    ));
    
    type_ref.define(type_parser.clone());
    
    let number = literals::number(ws.clone());

    // Recursive expression and statement placeholders
    let mut expr_ref = Recursive::declare();
    let mut stmt_ref = Recursive::declare();

    // Boolean and Null literals
    let boolean = literals::boolean(ws.clone());
    let null = literals::null(ws.clone());

    // Array and Table literals
    let array = literals::array(ws.clone(), expr_ref.clone());
    let table = literals::table(
        ws.clone(), 
        ident.clone(), 
        expr_ref.clone(), 
        string_parser(ws.clone(), expr_ref.clone()).boxed()
    );

    // Pattern parsing for destructuring
    let pattern = patterns::pattern(ws.clone(), ident.clone());

    // Expression parsers (blocks, functions, and if expressions)
    let block_expr = expressions::block(ws.clone(), stmt_ref.clone(), expr_ref.clone());
    let function = expressions::function(ws.clone(), ident.clone(), type_parser.clone(), stmt_ref.clone(), expr_ref.clone());
    let if_expr = expressions::if_expr(ws.clone(), stmt_ref.clone(), expr_ref.clone());
    let import_expr = expressions::import(ws.clone(), expr_ref.clone());
    let match_expression = expressions::match_expr(ws.clone(), expr_ref.clone(), stmt_ref.clone(), pattern.clone());

    // Parenthesized expressions - allows precedence override
    let paren_expr = expr_ref.clone()
        .delimited_by(
            just('(').padded_by(ws.clone()),
            just(')').padded_by(ws.clone())
        )
        .boxed();

    // Primary expressions (atoms)
    let primary = choice((
        number.boxed(),
        string_parser(ws.clone(), expr_ref.clone()).boxed(),
        boolean,
        null,
        array,
        table,
        if_expr,
        block_expr,
        match_expression,
        function,
        import_expr,
        paren_expr,
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
    // Parse call arguments - can be positional (expr) or named (name = expr)
    let call_arg = {
        let named = ident.clone()
            .then_ignore(just('=').padded_by(ws.clone()))
            .then(expr_ref.clone())
            .map(|(name, value): (&str, Expr)| CallArgument::Named { 
                name: name.to_string(), 
                value 
            });
        
        let positional = expr_ref.clone()
            .map(CallArgument::Positional);
        
        choice((named, positional))
    };
    
    let call_args = call_arg
        .separated_by(just(',').padded_by(ws.clone()))
        .allow_trailing()
        .collect::<Vec<CallArgument>>()
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
        Call(Vec<CallArgument>),
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
    let eq_op = operators::eq_op(ws.clone());
    let and_op = operators::and_op(ws.clone());
    let or_op = operators::or_op(ws.clone());

    // Build expression with precedence: || > && > == != > < <= > >= > + - > * / % > postfix > unary
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

    let eq_expr = cmp_expr.clone()
        .foldl(eq_op.then(cmp_expr.clone()).repeated(), |left, (op, right)| {
            Expr::Binary { left: Box::new(left), op, right: Box::new(right) }
        })
        .boxed();

    let and_expr = eq_expr.clone()
        .foldl(and_op.then(eq_expr.clone()).repeated(), |left, (op, right)| {
            Expr::Logical { left: Box::new(left), op, right: Box::new(right) }
        })
        .boxed();

    let or_expr = and_expr.clone()
        .foldl(or_op.then(and_expr.clone()).repeated(), |left, (op, right)| {
            Expr::Logical { left: Box::new(left), op, right: Box::new(right) }
        })
        .boxed();

    expr_ref.define(or_expr);

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
            .map(|(statements, ret)| {
                Program { 
                    statements: utils::apply_implicit_return(statements, ret) 
                }
            })
    )
}

pub fn parse(source: &str) -> Result<Program, Vec<Rich<'_, char>>> {
    parser().parse(source).into_result()
}
