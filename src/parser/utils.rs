use crate::ast::{Expr, Stmt};

/// Applies implicit return semantics to a block of statements.
///
/// Rules:
/// 1. If there's an explicit trailing expression (ret parameter), convert it to a Return statement
/// 2. Otherwise, if the last statement is an ExprStmt, convert it to a Return
/// 3. Otherwise, leave the statements as-is
///
/// This implements the language's "everything is a value" semantics where blocks,
/// functions, if expressions, and match arms all implicitly return their last expression.
pub fn apply_implicit_return(mut stmts: Vec<Stmt>, ret: Option<Expr>) -> Vec<Stmt> {
    if let Some(expr) = ret {
        // Explicit trailing expression captured separately
        stmts.push(Stmt::Return {
            value: expr,
            span: None,
        });
    } else if let Some(last) = stmts.pop() {
        // No separate trailing expression; convert last ExprStmt into implicit return
        match last {
            Stmt::ExprStmt(e) => stmts.push(Stmt::Return {
                value: e,
                span: None,
            }),
            other => stmts.push(other),
        }
    }
    stmts
}

/// Applies implicit return semantics to a statement-only block (no trailing expression).
///
/// If the last statement is an ExprStmt, converts it to a Return.
/// This is used in contexts where we only parse statements (like else blocks).
///
/// This is a convenience wrapper around `apply_implicit_return` with `None` for the trailing expression.
pub fn apply_implicit_return_stmts(stmts: Vec<Stmt>) -> Vec<Stmt> {
    apply_implicit_return(stmts, None)
}
