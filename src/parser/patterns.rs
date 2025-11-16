use chumsky::prelude::*;
use crate::ast::Pattern;

/// Creates a parser for a single identifier pattern
pub fn ident_pattern<'a, I>(ident: I) -> Boxed<'a, 'a, &'a str, Pattern, extra::Err<Rich<'a, char>>>
where
    I: Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    ident.map(|s: &str| Pattern::Ident(s.to_string())).boxed()
}

/// Creates a parser for all pattern types (ident, array, table)
pub fn pattern<'a, WS, I>(
    ws: WS,
    ident: I,
) -> Boxed<'a, 'a, &'a str, Pattern, extra::Err<Rich<'a, char>>>
where
    WS: Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
    I: Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    let pattern_ident = ident_pattern(ident.clone());
    
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
    
    choice((array_pattern, table_pattern, pattern_ident)).boxed()
}
