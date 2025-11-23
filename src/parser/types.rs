//! Type parsing for Luma language
//!
//! Handles parsing of type annotations including:
//! - Simple type identifiers (Number, String, Boolean, etc.)
//! - Generic types (List(String), Result(Number, String))
//! - Function types (fn(Number, String): Boolean)
//! - The Any type for dynamic typing

use crate::ast::{Span, Type};
use crate::parser::lexer;
use chumsky::prelude::*;

/// Parser for type annotations
///
/// Supports:
/// - `Any` - dynamic type
/// - `TypeIdent` - simple type names
/// - `GenericType` - parameterized types like List(String)
/// - `FunctionType` - function signatures like fn(Number): String
pub fn type_parser<'a>(
    ws: impl Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> + Clone + 'a,
) -> impl Parser<'a, &'a str, Type, extra::Err<Rich<'a, char>>> + Clone {
    recursive(|type_ref| {
        // Parse "Any" keyword as a type
        let any_type = just("Any").padded_by(ws.clone()).try_map(|_, span| {
            Ok(Type::Any {
                span: Some(Span::from_chumsky(span)),
            })
        });

        // Parse simple type identifier (not "Any")
        let type_ident = text::ident()
            .try_map(move |s: &str, span| {
                if s == "Any" {
                    // "Any" should be parsed by any_type parser
                    Err(Rich::custom(span, ""))
                } else if lexer::KEYWORDS.contains(&s) {
                    Err(Rich::custom(
                        span,
                        format!("'{}' is a keyword and cannot be used as a type", s),
                    ))
                } else {
                    Ok(Type::TypeIdent {
                        name: s.to_string(),
                        span: Some(Span::from_chumsky(span)),
                    })
                }
            })
            .padded_by(ws.clone());

        // Parse function type: fn(Type, Type, ...): Type
        let function_type = just("fn")
            .padded_by(ws.clone())
            .ignore_then(
                type_ref
                    .clone()
                    .separated_by(just(',').padded_by(ws.clone()))
                    .allow_trailing()
                    .collect::<Vec<Type>>()
                    .delimited_by(
                        just('(').padded_by(ws.clone()),
                        just(')').padded_by(ws.clone()),
                    ),
            )
            .then_ignore(just(':').padded_by(ws.clone()))
            .then(type_ref.clone())
            .try_map(|(param_types, return_type), span| {
                Ok(Type::FunctionType {
                    param_types,
                    return_type: Box::new(return_type),
                    span: Some(Span::from_chumsky(span)),
                })
            });

        // Parse generic type: TypeIdent(Type, Type, ...)
        let generic_type = text::ident()
            .try_map(move |s: &str, span| {
                if lexer::KEYWORDS.contains(&s) {
                    Err(Rich::custom(
                        span,
                        format!("'{}' is a keyword and cannot be used as a type", s),
                    ))
                } else {
                    Ok(s.to_string())
                }
            })
            .padded_by(ws.clone())
            .then(
                type_ref
                    .clone()
                    .separated_by(just(',').padded_by(ws.clone()))
                    .at_least(1)
                    .allow_trailing()
                    .collect::<Vec<Type>>()
                    .delimited_by(
                        just('(').padded_by(ws.clone()),
                        just(')').padded_by(ws.clone()),
                    ),
            )
            .try_map(|(name, type_args), span| {
                Ok(Type::GenericType {
                    name,
                    type_args,
                    span: Some(Span::from_chumsky(span)),
                })
            });

        // Combine all type parsers, with priority: function > generic > any > ident
        choice((
            function_type.boxed(),
            generic_type.boxed(),
            any_type.boxed(),
            type_ident.boxed(),
        ))
    })
}
