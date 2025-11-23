use crate::ast::{Expr, Span, TableKey};
use chumsky::prelude::*;

/// Creates a parser for number literals (integers, floats, hex, binary, scientific)
pub fn number<'a, WS>(ws: WS) -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    let hex = just("0x")
        .or(just("0X"))
        .ignore_then(
            one_of("0123456789abcdefABCDEF")
                .repeated()
                .at_least(1)
                .collect::<String>(),
        )
        .try_map(|s: String, span| {
            let val = i64::from_str_radix(&s, 16).unwrap_or(0);
            Ok(Expr::Number {
                value: val as f64,
                span: Some(Span::from_chumsky(span)),
            })
        });

    let binary = just("0b")
        .or(just("0B"))
        .ignore_then(one_of("01").repeated().at_least(1).collect::<String>())
        .try_map(|s: String, span| {
            let val = i64::from_str_radix(&s, 2).unwrap_or(0);
            Ok(Expr::Number {
                value: val as f64,
                span: Some(Span::from_chumsky(span)),
            })
        });

    let decimal = text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .then(
            one_of("eE")
                .then(one_of("+-").or_not())
                .then(text::digits(10))
                .or_not(),
        )
        .to_slice()
        .try_map(|s: &str, span| {
            let val = s.parse::<f64>().unwrap_or(0.0);
            Ok(Expr::Number {
                value: val,
                span: Some(Span::from_chumsky(span)),
            })
        });

    choice((hex, binary, decimal)).padded_by(ws).boxed()
}

/// Creates a parser for boolean literals (true/false)
pub fn boolean<'a, WS>(ws: WS) -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    choice((
        just("true").try_map(|_, span| {
            Ok(Expr::Boolean {
                value: true,
                span: Some(Span::from_chumsky(span)),
            })
        }),
        just("false").try_map(|_, span| {
            Ok(Expr::Boolean {
                value: false,
                span: Some(Span::from_chumsky(span)),
            })
        }),
    ))
    .padded_by(ws)
    .boxed()
}

/// Creates a parser for null literal
pub fn null<'a, WS>(ws: WS) -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    just("null")
        .try_map(|_, span| {
            Ok(Expr::Null {
                span: Some(Span::from_chumsky(span)),
            })
        })
        .padded_by(ws)
        .boxed()
}

/// Creates a parser for list literals [expr, expr, ...]
pub fn list<'a, WS, E>(ws: WS, expr: E) -> Boxed<'a, 'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    expr.separated_by(just(',').padded_by(ws.clone()))
        .allow_trailing()
        .collect::<Vec<Expr>>()
        .delimited_by(
            just('[').padded_by(ws.clone()),
            just(']').padded_by(ws.clone()),
        )
        .try_map(|elements, span| {
            Ok(Expr::List {
                elements,
                span: Some(Span::from_chumsky(span)),
            })
        })
        .boxed()
}

/// Creates a parser for table literals {key = value, ...}
/// Supports:
/// - Identifier keys: key = value
/// - String literal keys: "key with spaces" = value  
/// - Computed keys: [expression] = value
pub fn table<'a, WS, I, E, S>(
    ws: WS,
    ident: I,
    expr: E,
    string_lit: S,
) -> Boxed<'a, 'a, &'a str, Expr, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    I: Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone + 'a,
    E: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
    S: Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    // Identifier key: key = value
    let identifier_key = ident
        .clone()
        .map(|s: &str| TableKey::Identifier(s.to_string()));

    // String literal key: "key with spaces" = value
    let string_key = string_lit.try_map(|expr, span| match expr {
        Expr::String { value, .. } => Ok(TableKey::StringLiteral(value)),
        _ => Err(Rich::custom(
            span,
            "String literal key cannot contain interpolation",
        )),
    });

    // Computed key: [expression] = value
    let computed_key = expr
        .clone()
        .delimited_by(
            just('[').padded_by(ws.clone()),
            just(']').padded_by(ws.clone()),
        )
        .map(|e| TableKey::Computed(Box::new(e)));

    // Any of the three key types followed by = and value
    let kv_entry = choice((
        computed_key.clone(),
        string_key.clone(),
        identifier_key.clone(),
    ))
    .then_ignore(just('=').padded_by(ws.clone()))
    .then(expr.clone())
    .map(|(k, v)| (k, v));

    // Shorthand entry: identifier alone expands to key=name, value=Identifier(name)
    let shorthand_entry = ident.try_map(|s: &str, span| {
        let name = s.to_string();
        Ok((
            TableKey::Identifier(name.clone()),
            Expr::Identifier {
                name,
                span: Some(Span::from_chumsky(span)),
            },
        ))
    });

    let table_entry = choice((kv_entry, shorthand_entry));

    table_entry
        .separated_by(just(',').padded_by(ws.clone()))
        .allow_trailing()
        .collect::<Vec<(TableKey, Expr)>>()
        .delimited_by(
            just('{').padded_by(ws.clone()),
            just('}').padded_by(ws.clone()),
        )
        .try_map(|fields, span| {
            Ok(Expr::Table {
                fields,
                span: Some(Span::from_chumsky(span)),
            })
        })
        .boxed()
}
