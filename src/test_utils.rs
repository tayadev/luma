//! Test utilities for stripping spans from AST nodes for fixture comparison

use crate::ast::{Expr, Pattern, Program, Stmt};

/// Strip all spans from a Program for fixture comparison
pub fn strip_all_spans(program: Program) -> Program {
    Program {
        statements: program
            .statements
            .into_iter()
            .map(strip_spans_stmt)
            .collect(),
    }
}

fn strip_spans_stmt(stmt: Stmt) -> Stmt {
    match stmt {
        Stmt::VarDecl {
            mutable,
            name,
            r#type,
            value,
            ..
        } => Stmt::VarDecl {
            mutable,
            name,
            r#type,
            value: strip_spans_expr(value),
            span: None,
        },
        Stmt::DestructuringVarDecl {
            mutable,
            pattern,
            value,
            ..
        } => Stmt::DestructuringVarDecl {
            mutable,
            pattern: strip_spans_pattern(pattern),
            value: strip_spans_expr(value),
            span: None,
        },
        Stmt::Assignment {
            target, op, value, ..
        } => Stmt::Assignment {
            target: strip_spans_expr(target),
            op,
            value: strip_spans_expr(value),
            span: None,
        },
        Stmt::If {
            condition,
            then_block,
            elif_blocks,
            else_block,
            ..
        } => Stmt::If {
            condition: strip_spans_expr(condition),
            then_block: then_block.into_iter().map(strip_spans_stmt).collect(),
            elif_blocks: elif_blocks
                .into_iter()
                .map(|(c, b)| {
                    (
                        strip_spans_expr(c),
                        b.into_iter().map(strip_spans_stmt).collect(),
                    )
                })
                .collect(),
            else_block: else_block.map(|b| b.into_iter().map(strip_spans_stmt).collect()),
            span: None,
        },
        Stmt::While {
            condition, body, ..
        } => Stmt::While {
            condition: strip_spans_expr(condition),
            body: body.into_iter().map(strip_spans_stmt).collect(),
            span: None,
        },
        Stmt::DoWhile {
            body, condition, ..
        } => Stmt::DoWhile {
            body: body.into_iter().map(strip_spans_stmt).collect(),
            condition: strip_spans_expr(condition),
            span: None,
        },
        Stmt::For {
            pattern,
            iterator,
            body,
            ..
        } => Stmt::For {
            pattern: strip_spans_pattern(pattern),
            iterator: strip_spans_expr(iterator),
            body: body.into_iter().map(strip_spans_stmt).collect(),
            span: None,
        },
        Stmt::Match { expr, arms, .. } => Stmt::Match {
            expr: strip_spans_expr(expr),
            arms: arms
                .into_iter()
                .map(|(p, b)| {
                    (
                        strip_spans_pattern(p),
                        b.into_iter().map(strip_spans_stmt).collect(),
                    )
                })
                .collect(),
            span: None,
        },
        Stmt::Return { value, .. } => Stmt::Return {
            value: strip_spans_expr(value),
            span: None,
        },
        Stmt::Break { level, .. } => Stmt::Break { level, span: None },
        Stmt::Continue { level, .. } => Stmt::Continue { level, span: None },
        Stmt::ExprStmt { expr, .. } => Stmt::ExprStmt {
            expr: strip_spans_expr(expr),
            span: None,
        },
    }
}

fn strip_spans_expr(expr: Expr) -> Expr {
    match expr {
        Expr::Binary {
            left, op, right, ..
        } => Expr::Binary {
            left: Box::new(strip_spans_expr(*left)),
            op,
            right: Box::new(strip_spans_expr(*right)),
            span: None,
        },
        Expr::Unary { op, operand, .. } => Expr::Unary {
            op,
            operand: Box::new(strip_spans_expr(*operand)),
            span: None,
        },
        Expr::Logical {
            left, op, right, ..
        } => Expr::Logical {
            left: Box::new(strip_spans_expr(*left)),
            op,
            right: Box::new(strip_spans_expr(*right)),
            span: None,
        },
        Expr::Call {
            callee, arguments, ..
        } => Expr::Call {
            callee: Box::new(strip_spans_expr(*callee)),
            arguments,
            span: None,
        },
        Expr::MemberAccess { object, member, .. } => Expr::MemberAccess {
            object: Box::new(strip_spans_expr(*object)),
            member,
            span: None,
        },
        Expr::Index { object, index, .. } => Expr::Index {
            object: Box::new(strip_spans_expr(*object)),
            index: Box::new(strip_spans_expr(*index)),
            span: None,
        },
        Expr::Function {
            arguments,
            return_type,
            body,
            ..
        } => Expr::Function {
            arguments,
            return_type,
            body: body.into_iter().map(strip_spans_stmt).collect(),
            span: None,
        },
        Expr::If {
            condition,
            then_block,
            else_block,
            ..
        } => Expr::If {
            condition: Box::new(strip_spans_expr(*condition)),
            then_block: then_block.into_iter().map(strip_spans_stmt).collect(),
            else_block: else_block.map(|b| b.into_iter().map(strip_spans_stmt).collect()),
            span: None,
        },
        Expr::Match { expr, arms, .. } => Expr::Match {
            expr: Box::new(strip_spans_expr(*expr)),
            arms: arms
                .into_iter()
                .map(|(p, b)| {
                    (
                        strip_spans_pattern(p),
                        b.into_iter().map(strip_spans_stmt).collect(),
                    )
                })
                .collect(),
            span: None,
        },
        Expr::Block { statements, .. } => Expr::Block {
            statements: statements.into_iter().map(strip_spans_stmt).collect(),
            span: None,
        },
        Expr::List { elements, .. } => Expr::List {
            elements: elements.into_iter().map(strip_spans_expr).collect(),
            span: None,
        },
        Expr::Table { fields, .. } => Expr::Table {
            fields: fields
                .into_iter()
                .map(|(k, v)| (k, strip_spans_expr(v)))
                .collect(),
            span: None,
        },
        Expr::Import { path, .. } => Expr::Import {
            path,
            span: None,
        },
        other => other,
    }
}

fn strip_spans_pattern(pat: Pattern) -> Pattern {
    match pat {
        Pattern::ListPattern { elements, rest, .. } => Pattern::ListPattern {
            elements: elements.into_iter().map(strip_spans_pattern).collect(),
            rest,
            span: None,
        },
        Pattern::TablePattern { fields, .. } => Pattern::TablePattern { fields, span: None },
        other => other,
    }
}
