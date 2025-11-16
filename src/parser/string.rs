use chumsky::prelude::*;
use crate::ast::Expr;

// Build an Expr from a raw string with ${} interpolations.
pub fn build_string_expr(raw: String) -> Expr {
    let mut segments: Vec<Expr> = Vec::new();
    let mut cursor = 0;
    while let Some(start) = raw[cursor..].find("${") {
        let abs_start = cursor + start;
        // add preceding literal
        if abs_start > cursor {
            let lit = &raw[cursor..abs_start];
            if !lit.is_empty() { segments.push(Expr::String(lit.to_string())); }
        }
        // find closing }
        let expr_start = abs_start + 2;
        if let Some(close_rel) = raw[expr_start..].find('}') {
            let abs_close = expr_start + close_rel;
            let inner = raw[expr_start..abs_close].trim();
            let expr = if let Ok(num) = inner.parse::<f64>() { Expr::Number(num) } else { Expr::Identifier(inner.to_string()) };
            segments.push(expr);
            cursor = abs_close + 1;
        } else {
            // malformed, treat the rest as literal and stop
            let lit = &raw[abs_start..];
            if !lit.is_empty() { segments.push(Expr::String(lit.to_string())); }
            cursor = raw.len();
        }
    }
    // trailing literal
    if cursor < raw.len() {
        let tail = &raw[cursor..];
        if !tail.is_empty() { segments.push(Expr::String(tail.to_string())); }
    }
    match segments.len() {
        0 => Expr::String(raw),
        1 => segments.remove(0),
        _ => Expr::Concat(segments),
    }
}

pub fn string_parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> {
    let content = any()
        .filter(|c: &char| *c != '"')
        .repeated()
        .collect::<String>();
    just('"')
        .ignore_then(content)
        .then_ignore(just('"'))
        .map(build_string_expr)
        .padded()
}
