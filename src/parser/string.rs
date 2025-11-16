use chumsky::prelude::*;
use crate::ast::Expr;

// Process escape sequences in a string
fn process_escapes(raw: &str) -> String {
    let mut result = String::new();
    let mut chars = raw.chars().peekable();
    
    // Use a placeholder for escaped $
    const ESCAPED_DOLLAR: char = '\x00';
    
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                chars.next(); // consume the next char
                match next {
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    '$' => {
                        // \$ becomes a placeholder to prevent interpolation
                        result.push(ESCAPED_DOLLAR);
                    }
                    _ => {
                        // Unknown escape, keep as-is
                        result.push('\\');
                        result.push(next);
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(c);
        }
    }
    result
}

// Build an Expr from a raw string with ${} interpolations.
pub fn build_string_expr(raw: String) -> Expr {
    let processed = process_escapes(&raw);
    let mut segments: Vec<Expr> = Vec::new();
    let mut cursor = 0;
    
    // Placeholder for escaped $
    const ESCAPED_DOLLAR: char = '\x00';
    
    while let Some(start) = processed[cursor..].find("${") {
        let abs_start = cursor + start;
        // add preceding literal
        if abs_start > cursor {
            let lit = &processed[cursor..abs_start];
            if !lit.is_empty() { 
                // Replace escaped dollar placeholders with actual dollar signs
                let lit = lit.replace(ESCAPED_DOLLAR, "$");
                segments.push(Expr::String(lit)); 
            }
        }
        // find closing }
        let expr_start = abs_start + 2;
        if let Some(close_rel) = processed[expr_start..].find('}') {
            let abs_close = expr_start + close_rel;
            let inner = processed[expr_start..abs_close].trim();
            let expr = if let Ok(num) = inner.parse::<f64>() { Expr::Number(num) } else { Expr::Identifier(inner.to_string()) };
            segments.push(expr);
            cursor = abs_close + 1;
        } else {
            // malformed, treat the rest as literal and stop
            let lit = &processed[abs_start..];
            if !lit.is_empty() { 
                let lit = lit.replace(ESCAPED_DOLLAR, "$");
                segments.push(Expr::String(lit)); 
            }
            cursor = processed.len();
        }
    }
    // trailing literal
    if cursor < processed.len() {
        let tail = &processed[cursor..];
        if !tail.is_empty() { 
            let tail = tail.replace(ESCAPED_DOLLAR, "$");
            segments.push(Expr::String(tail)); 
        }
    }
    match segments.len() {
        0 => {
            let s = processed.replace(ESCAPED_DOLLAR, "$");
            Expr::String(s)
        },
        1 => segments.remove(0),
        _ => Expr::Concat(segments),
    }
}

pub fn string_parser<'a, WS>(ws: WS) -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    // String content that handles escape sequences
    let escape = just('\\').ignore_then(any());
    let string_char = escape.or(none_of("\""));
    let content = string_char.repeated().collect::<String>();
    
    just('"')
        .ignore_then(content)
        .then_ignore(just('"'))
        .map(build_string_expr)
        .padded_by(ws)
}
