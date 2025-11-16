use chumsky::prelude::*;
use crate::ast::{Program, Stmt, Expr, Argument, Type};

mod string;
use string::string_parser;

pub fn parser<'a>() -> impl Parser<'a, &'a str, Program, extra::Err<Rich<'a, char>>> {
    let ident = text::ident().padded();
    let type_parser = ident.clone().map(|s: &str| Type::TypeIdent(s.to_string())).padded();
    let number = text::int(10).padded().map(|s: &str| s.parse::<f64>().unwrap());

    // Recursive expression placeholder
    let mut expr_ref = Recursive::declare();

    // Argument parsing
    let argument = ident
        .then_ignore(just(':').padded())
        .then(type_parser.clone())
        .map(|(name, t): (&str, Type)| Argument { name: name.to_string(), r#type: t, default: None });

    let arg_list = argument
        .separated_by(just(',').padded())
        .allow_trailing()
        .collect::<Vec<Argument>>()
        .delimited_by(just('('), just(')'))
        .padded();

    // Variable declaration (uses expr_ref)
    let var_decl_token = choice((just("let").to(false), just("var").to(true)))
        .padded()
        .then(ident.map(|s: &str| s.to_string()))
        .then( just(':').padded().ignore_then(type_parser.clone()).or_not() )
        .then_ignore(just('=').padded());

    let var_decl = var_decl_token
        .then(expr_ref.clone())
        .map(|(((mutable, name), opt_type), value)| Stmt::VarDecl { mutable, name, r#type: opt_type, value });

    // Function body: statements + optional trailing expression as Return
    let body_block = var_decl.clone().repeated().collect::<Vec<Stmt>>()
        .then(expr_ref.clone().or_not())
        .then_ignore(just("end").padded())
        .map(|(mut stmts, ret)| { if let Some(expr) = ret { stmts.push(Stmt::Return(expr)); } stmts });

    let function = just("fn")
        .ignore_then(arg_list.clone())
        .then(just(':').ignore_then(type_parser.clone()).or_not())
        .then_ignore(just("do").padded())
        .then(body_block)
        .map(|((arguments, return_type), body): ((Vec<Argument>, Option<Type>), Vec<Stmt>)| Expr::Function { arguments, return_type, body })
        .padded();

    // Expression forms (boxed to ensure Clone for recursion)
    let number_p = number.map(Expr::Number).boxed();
    let string_p = string_parser().boxed();
    let function_p = function.boxed();
    let ident_p = ident.map(|s: &str| Expr::Identifier(s.to_string())).boxed();

    let expr_all = choice((number_p, string_p, function_p, ident_p));

    expr_ref.define(expr_all.clone());

    // Program: statements with optional trailing expression -> Return
    var_decl
        .repeated()
        .collect::<Vec<Stmt>>()
        .then(expr_ref.clone().or_not())
        .then_ignore(end())
        .map(|(mut statements, ret)| { if let Some(expr) = ret { statements.push(Stmt::Return(expr)); } Program { statements } })
}

pub fn parse(source: &str) -> Result<Program, Vec<Rich<char>>> {
    parser().parse(source).into_result()
}
