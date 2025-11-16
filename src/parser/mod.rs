use chumsky::prelude::*;
use crate::ast::{Program, Stmt, Expr, Argument, Type, BinaryOp, Pattern};

mod string;
use string::string_parser;

pub fn parser<'a>() -> impl Parser<'a, &'a str, Program, extra::Err<Rich<'a, char>>> {
    // Comments and whitespace
    let line_comment = just("--")
        .then(none_of("\n").repeated())
        .ignored();
    
    let block_comment = just("--[[")
        .then(any().and_is(just("]]").not()).repeated())
        .then(just("]]"))
        .ignored();
    
    let comment = choice((block_comment, line_comment));
    let ws_item = comment.or(one_of(" \t\r\n").ignored());
    let ws = ws_item.repeated();
    
    let keywords = ["let", "var", "fn", "do", "end", "return", "true", "false", "null"];
    let ident = text::ident()
        .try_map(move |s: &str, span| {
            if keywords.contains(&s) {
                Err(Rich::custom(span, format!("'{}' is a keyword and cannot be used as an identifier", s)))
            } else {
                Ok(s)
            }
        })
        .padded_by(ws.clone());
    let type_parser = ident.clone().map(|s: &str| Type::TypeIdent(s.to_string()));
    let number = text::int(10).padded_by(ws.clone()).map(|s: &str| s.parse::<f64>().unwrap());

    // Recursive expression placeholder
    let mut expr_ref = Recursive::declare();
    let mut stmt_ref = Recursive::declare();

    // Boolean and Null literals
    let boolean = choice((
        just("true").to(Expr::Boolean(true)),
        just("false").to(Expr::Boolean(false)),
    )).padded_by(ws.clone()).boxed();
    
    let null = just("null").to(Expr::Null).padded_by(ws.clone()).boxed();

    // Array literal
    let array = expr_ref.clone()
        .separated_by(just(',').padded_by(ws.clone()))
        .allow_trailing()
        .collect::<Vec<Expr>>()
        .delimited_by(just('[').padded_by(ws.clone()), just(']').padded_by(ws.clone()))
        .map(Expr::Array)
        .boxed();

    // Table literal
    let table_entry = ident.clone()
        .then_ignore(just('=').padded_by(ws.clone()))
        .then(expr_ref.clone())
        .map(|(k, v): (&str, Expr)| (k.to_string(), v));
    
    let table = table_entry
        .separated_by(just(',').padded_by(ws.clone()))
        .allow_trailing()
        .collect::<Vec<(String, Expr)>>()
        .delimited_by(just('{').padded_by(ws.clone()), just('}').padded_by(ws.clone()))
        .map(Expr::Table)
        .boxed();

    // Argument parsing with default values
    let argument = ident.clone()
        .then_ignore(just(':').padded_by(ws.clone()))
        .then(type_parser.clone())
        .then(just('=').padded_by(ws.clone()).ignore_then(expr_ref.clone()).or_not())
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

    // Pattern parsing for destructuring
    let pattern_ident = ident.clone().map(|s: &str| Pattern::Ident(s.to_string()));
    
    let array_pattern = pattern_ident.clone()
        .separated_by(just(',').padded_by(ws.clone()))
        .at_least(1)
        .collect::<Vec<Pattern>>()
        .then(
            just(',').padded_by(ws.clone())
                .ignore_then(just("..."))
                .ignore_then(ident.clone().map(|s: &str| s.to_string()))
                .or_not()
        )
        .delimited_by(just('[').padded_by(ws.clone()), just(']').padded_by(ws.clone()))
        .map(|(elements, rest)| Pattern::ArrayPattern { elements, rest })
        .boxed();
    
    let table_pattern = ident.clone()
        .separated_by(just(',').padded_by(ws.clone()))
        .at_least(1)
        .collect::<Vec<&str>>()
        .delimited_by(just('{').padded_by(ws.clone()), just('}').padded_by(ws.clone()))
        .map(|fields: Vec<&str>| Pattern::TablePattern(fields.into_iter().map(|s| s.to_string()).collect()))
        .boxed();
    
    let pattern = choice((array_pattern, table_pattern, pattern_ident));

    // Variable declaration (uses expr_ref)
    let var_decl_token = choice((just("let").to(false), just("var").to(true)))
        .padded_by(ws.clone())
        .then(choice((
            pattern.clone().map(|p| match p {
                Pattern::Ident(name) => (None, Some(name)),
                _ => (Some(p), None),
            }),
        )))
        .then(just(':').padded_by(ws.clone()).ignore_then(type_parser.clone()).or_not())
        .then_ignore(just('=').padded_by(ws.clone()));

    let var_decl = var_decl_token
        .then(expr_ref.clone())
        .map(|(((mutable, (pattern, name)), opt_type), value)| {
            if let Some(pattern) = pattern {
                Stmt::DestructuringVarDecl { mutable, pattern, value }
            } else {
                Stmt::VarDecl { mutable, name: name.unwrap(), r#type: opt_type, value }
            }
        });

    // Return statement
    let return_stmt = just("return")
        .padded_by(ws.clone())
        .ignore_then(expr_ref.clone())
        .map(Stmt::Return)
        .boxed();

    // Statement definition
    let stmt = choice((return_stmt, var_decl.clone()));
    stmt_ref.define(stmt.clone());

    // Block expression
    let block_expr = just("do")
        .padded_by(ws.clone())
        .ignore_then(stmt_ref.clone().repeated().collect::<Vec<Stmt>>())
        .then(expr_ref.clone().or_not())
        .then_ignore(just("end").padded_by(ws.clone()))
        .map(|(mut stmts, ret)| { 
            if let Some(expr) = ret { 
                stmts.push(Stmt::Return(expr)); 
            } 
            Expr::Block(stmts)
        })
        .boxed();

    // Function body: statements + optional trailing expression as Return
    let body_block = stmt_ref.clone().repeated().collect::<Vec<Stmt>>()
        .then(expr_ref.clone().or_not())
        .then_ignore(just("end").padded_by(ws.clone()))
        .map(|(mut stmts, ret)| { if let Some(expr) = ret { stmts.push(Stmt::Return(expr)); } stmts });

    let function = just("fn")
        .padded_by(ws.clone())
        .ignore_then(arg_list.clone())
        .then(just(':').padded_by(ws.clone()).ignore_then(type_parser.clone()).or_not())
        .then_ignore(just("do").padded_by(ws.clone()))
        .then(body_block)
        .map(|((arguments, return_type), body): ((Vec<Argument>, Option<Type>), Vec<Stmt>)| Expr::Function { arguments, return_type, body })
        .boxed();

    // Primary expressions (atoms)
    let primary = choice((
        number.map(Expr::Number).boxed(),
        string_parser(ws.clone()).boxed(),
        boolean,
        null,
        array,
        table,
        block_expr,
        function,
        ident.map(|s: &str| Expr::Identifier(s.to_string())).boxed(),
    ));

    // Binary operators with precedence
    let op = |c| just(c).padded_by(ws.clone());
    
    let mul_op = choice((
        op('*').to(BinaryOp::Mul),
        op('/').to(BinaryOp::Div),
        op('%').to(BinaryOp::Mod),
    ));
    
    let add_op = choice((
        op('+').to(BinaryOp::Add),
        op('-').to(BinaryOp::Sub),
    ));
    
    let cmp_op = choice((
        just("==").padded_by(ws.clone()).to(BinaryOp::Eq),
        just("!=").padded_by(ws.clone()).to(BinaryOp::Ne),
        just("<=").padded_by(ws.clone()).to(BinaryOp::Le),
        just(">=").padded_by(ws.clone()).to(BinaryOp::Ge),
        op('<').to(BinaryOp::Lt),
        op('>').to(BinaryOp::Gt),
    ));

    // Build expression with precedence: comparison > addition > multiplication
    let mul_expr = primary.clone()
        .foldl(mul_op.then(primary.clone()).repeated(), |left, (op, right)| {
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

    expr_ref.define(cmp_expr);

    // Program: statements with optional trailing expression -> Return
    ws.clone().ignore_then(
        stmt.clone()
            .repeated()
            .collect::<Vec<Stmt>>()
            .then(expr_ref.clone().or_not())
            .then_ignore(ws.clone())
            .then_ignore(end())
            .map(|(mut statements, ret)| { if let Some(expr) = ret { statements.push(Stmt::Return(expr)); } Program { statements } })
    )
}

pub fn parse(source: &str) -> Result<Program, Vec<Rich<'_, char>>> {
    parser().parse(source).into_result()
}
