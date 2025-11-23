use crate::ast::TablePatternField;
use crate::ast::{Literal, Pattern, Span};
use chumsky::prelude::*;

/// Creates a parser for literal patterns
pub fn literal_pattern<'a, WS>(
    ws: WS,
) -> Boxed<'a, 'a, &'a str, Pattern, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    // Number literal
    let number = text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .try_map(|s: &str, span| {
            Ok(Pattern::Literal {
                value: Literal::Number(s.parse::<f64>().unwrap()),
                span: Some(Span::from_chumsky(span)),
            })
        })
        .padded_by(ws.clone());

    // String literal (simple, no interpolation needed for patterns)
    let string_literal = just('"')
        .ignore_then(none_of('"').repeated().collect::<String>())
        .then_ignore(just('"'))
        .try_map(|s, span| {
            Ok(Pattern::Literal {
                value: Literal::String(s),
                span: Some(Span::from_chumsky(span)),
            })
        })
        .padded_by(ws.clone());

    // Boolean literals
    let bool_true = text::keyword("true")
        .try_map(|_, span| {
            Ok(Pattern::Literal {
                value: Literal::Boolean(true),
                span: Some(Span::from_chumsky(span)),
            })
        })
        .padded_by(ws.clone());
    let bool_false = text::keyword("false")
        .try_map(|_, span| {
            Ok(Pattern::Literal {
                value: Literal::Boolean(false),
                span: Some(Span::from_chumsky(span)),
            })
        })
        .padded_by(ws.clone());

    // Null literal
    let null = text::keyword("null")
        .try_map(|_, span| {
            Ok(Pattern::Literal {
                value: Literal::Null,
                span: Some(Span::from_chumsky(span)),
            })
        })
        .padded_by(ws.clone());

    choice((number, string_literal, bool_true, bool_false, null)).boxed()
}

/// Creates a parser for all pattern types (ident, list, table, wildcard, literal)
/// Note: Tag patterns are semantically the same as Ident patterns in parsing,
/// but are distinguished during type checking in match contexts
pub fn pattern<'a, WS, I>(
    ws: WS,
    ident: I,
) -> Boxed<'a, 'a, &'a str, Pattern, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    I: Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    recursive(|pattern_ref| {
        let wildcard = just('_')
            .padded_by(ws.clone())
            .try_map(|_, span| {
                Ok(Pattern::Wildcard {
                    span: Some(Span::from_chumsky(span)),
                })
            })
            .boxed();

        // List patterns support nested patterns
        let list_pattern = pattern_ref
            .clone()
            .separated_by(just(',').padded_by(ws.clone()))
            .at_least(1)
            .collect::<Vec<Pattern>>()
            .then(
                just(',')
                    .padded_by(ws.clone())
                    .ignore_then(just("..."))
                    .ignore_then(ident.clone().map(|s: &str| s.to_string()))
                    .or_not(),
            )
            .delimited_by(
                just('[').padded_by(ws.clone()),
                just(']').padded_by(ws.clone()),
            )
            .try_map(|(elements, rest), span| {
                Ok(Pattern::ListPattern {
                    elements,
                    rest,
                    span: Some(Span::from_chumsky(span)),
                })
            })
            .boxed();

        // Table patterns with field renames: {key}, {key: binding}
        let table_field = ident
            .clone()
            .then(
                just(':')
                    .padded_by(ws.clone())
                    .ignore_then(ident.clone())
                    .or_not(),
            )
            .map(|(key, binding): (&str, Option<&str>)| TablePatternField {
                key: key.to_string(),
                binding: binding.map(|s| s.to_string()),
            });

        let table_pattern = table_field
            .separated_by(just(',').padded_by(ws.clone()))
            .at_least(1)
            .collect::<Vec<TablePatternField>>()
            .delimited_by(
                just('{').padded_by(ws.clone()),
                just('}').padded_by(ws.clone()),
            )
            .try_map(|fields, span| {
                Ok(Pattern::TablePattern {
                    fields,
                    span: Some(Span::from_chumsky(span)),
                })
            })
            .boxed();

        // Identifier pattern (default)
        let ident_pattern = ident
            .clone()
            .try_map(|s: &str, span| {
                Ok(Pattern::Ident {
                    name: s.to_string(),
                    span: Some(Span::from_chumsky(span)),
                })
            })
            .boxed();

        let literal = literal_pattern(ws.clone());

        // Try structural patterns first (they have delimiters), then literal, wildcard, then ident
        choice((
            list_pattern,
            table_pattern,
            literal,
            wildcard,
            ident_pattern, // Identifiers become Ident patterns (can be treated as Tag in match)
        ))
    })
    .boxed()
}
