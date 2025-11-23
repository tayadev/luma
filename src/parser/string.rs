use crate::ast::{BinaryOp, Expr, Span};
use chumsky::prelude::*;

// Parser-based string interpolation with full expression support inside ${}
pub fn string_parser<'a, WS, E>(
    ws: WS,
    expr: E,
) -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    // Escape sequences: \n, \t, \", \\ and \$ (treated literally)
    let escape = just('\\').ignore_then(any()).map(|c| match c {
        'n' => '\n',
        't' => '\t',
        '"' => '"',
        '\\' => '\\',
        '$' => '$',
        other => other,
    });

    // Interpolation: ${ <expr> }
    let interpolation = just("${")
        .ignore_then(expr.clone())
        .then_ignore(just('}'))
        .boxed();

    // Plain character (any char except quote and backslash, handled separately)
    let plain_char = any().filter(|c| *c != '"' && *c != '\\');

    // Segment enum (local) to accumulate pieces
    #[derive(Clone)]
    enum Segment {
        Text(char),
        Expr(Expr),
    }

    let segment = choice((
        interpolation.map(Segment::Expr).boxed(),
        escape.map(Segment::Text).boxed(),
        plain_char.map(Segment::Text).boxed(),
    ))
    .boxed();

    let body = segment.repeated().collect::<Vec<Segment>>();
    just('"')
        .ignore_then(body)
        .then_ignore(just('"'))
        .try_map(|segments, span| {
            let mut parts: Vec<Expr> = Vec::new();
            let mut buf = String::new();
            for seg in segments {
                match seg {
                    Segment::Text(c) => buf.push(c),
                    Segment::Expr(e) => {
                        if !buf.is_empty() {
                            parts.push(Expr::String {
                                value: buf.clone(),
                                span: None, // Inner string fragments don't need spans
                            });
                            buf.clear();
                        }
                        parts.push(e);
                    }
                }
            }
            if !buf.is_empty() {
                parts.push(Expr::String {
                    value: buf,
                    span: None,
                });
            }
            Ok(match parts.len() {
                0 => Expr::String {
                    value: String::new(),
                    span: Some(Span::from_chumsky(span)),
                },
                1 => {
                    // Single part - return it but update span
                    let mut expr = parts.remove(0);
                    if let Expr::String {
                        span: ref mut s, ..
                    } = expr
                    {
                        *s = Some(Span::from_chumsky(span));
                    }
                    expr
                }
                _ => parts
                    .into_iter()
                    .reduce(|left, right| Expr::Binary {
                        left: Box::new(left),
                        op: BinaryOp::Add,
                        right: Box::new(right),
                        span: None,
                    })
                    .unwrap(),
            })
        })
        .padded_by(ws)
}
