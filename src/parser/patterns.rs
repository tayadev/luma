use chumsky::prelude::*;
use crate::ast::Pattern;

/// Creates a parser for a single identifier pattern
pub fn ident_pattern<'a, I>(ident: I) -> Boxed<'a, 'a, &'a str, Pattern, extra::Err<Rich<'a, char>>>
where
    I: Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> + Clone + 'a,
{
    ident.map(|s: &str| Pattern::Ident(s.to_string())).boxed()
}
