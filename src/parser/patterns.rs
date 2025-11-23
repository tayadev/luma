use chumsky::prelude::*;
use crate::ast::{Pattern, TablePatternField, Literal};

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
        .map(|s: &str| {
            Pattern::Literal(Literal::Number(s.parse::<f64>().unwrap()))
        })
        .padded_by(ws.clone());

    // String literal (simple, no interpolation needed for patterns)
    let string_literal = just('"')
        .ignore_then(none_of('"').repeated().collect::<String>())
        .then_ignore(just('"'))
        .map(|s| Pattern::Literal(Literal::String(s)))
        .padded_by(ws.clone());

    // Boolean literals
    let bool_true = text::keyword("true")
        .to(Pattern::Literal(Literal::Boolean(true)))
        .padded_by(ws.clone());
    let bool_false = text::keyword("false")
        .to(Pattern::Literal(Literal::Boolean(false)))
        .padded_by(ws.clone());

    // Null literal
    let null = text::keyword("null")
        .to(Pattern::Literal(Literal::Null))
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
        let wildcard = just('_').padded_by(ws.clone()).to(Pattern::Wildcard).boxed();
        
        // List patterns support nested patterns
        let list_pattern = pattern_ref.clone()
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
            .map(|(elements, rest)| Pattern::ListPattern { elements, rest, span: None })
            .boxed();
        
        // Table patterns with field renames: {key}, {key: binding}
        let table_field = ident.clone()
            .then(
                just(':')
                    .padded_by(ws.clone())
                    .ignore_then(ident.clone())
                    .or_not()
            )
            .map(|(key, binding): (&str, Option<&str>)| TablePatternField {
                key: key.to_string(),
                binding: binding.map(|s| s.to_string()),
            });
        
        let table_pattern = table_field
            .separated_by(just(',').padded_by(ws.clone()))
            .at_least(1)
            .collect::<Vec<TablePatternField>>()
            .delimited_by(just('{').padded_by(ws.clone()), just('}').padded_by(ws.clone()))
            .map(|fields| Pattern::TablePattern { fields, span: None })
            .boxed();
        
        // Identifier pattern (default)
        let ident_pattern = ident.clone()
            .map(|s: &str| Pattern::Ident(s.to_string()))
            .boxed();
        
        let literal = literal_pattern(ws.clone());
        
        // Try structural patterns first (they have delimiters), then literal, wildcard, then ident
        choice((
            list_pattern,
            table_pattern,
            literal,
            wildcard,
            ident_pattern,  // Identifiers become Ident patterns (can be treated as Tag in match)
        ))
    }).boxed()
}
