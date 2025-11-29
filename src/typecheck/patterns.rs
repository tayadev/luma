//! Pattern type checking and match exhaustiveness.

use std::collections::HashSet;

use crate::ast::*;

use super::environment::TypeEnv;
use super::types::{TcType, VarInfo};

/// Known tag patterns for Result/Option types that should not be treated as catch-all bindings.
pub const KNOWN_TAG_PATTERNS: &[&str] = &["ok", "err", "some", "none"];

impl TypeEnv {
    /// Type check a pattern and bind its variables.
    pub fn check_pattern(&mut self, pattern: &Pattern, ty: &TcType, mutable: bool, in_match: bool) {
        match pattern {
            Pattern::Ident { name, .. } => {
                if in_match && KNOWN_TAG_PATTERNS.contains(&name.as_str()) {
                    // Tag pattern in match: don't bind a variable
                } else {
                    self.declare(
                        name.clone(),
                        VarInfo {
                            ty: ty.clone(),
                            mutable,
                            annotated: false,
                        },
                    );
                }
            }
            Pattern::ListPattern { elements, rest, .. } => match ty {
                TcType::List(elem_ty) => {
                    for elem in elements {
                        self.check_pattern(elem, elem_ty, mutable, in_match);
                    }
                    if let Some(rest_name) = rest {
                        self.declare(
                            rest_name.clone(),
                            VarInfo {
                                ty: TcType::List(elem_ty.clone()),
                                mutable,
                                annotated: false,
                            },
                        );
                    }
                }
                TcType::Unknown | TcType::Any => {
                    for elem in elements {
                        self.check_pattern(elem, &TcType::Unknown, mutable, in_match);
                    }
                    if let Some(rest_name) = rest {
                        self.declare(
                            rest_name.clone(),
                            VarInfo {
                                ty: TcType::List(Box::new(TcType::Unknown)),
                                mutable,
                                annotated: false,
                            },
                        );
                    }
                }
                _ => {
                    self.error(
                        format!("List pattern requires List type, got {ty}"),
                        pattern.span(),
                    );
                }
            },
            Pattern::TablePattern { fields, .. } => {
                match ty {
                    TcType::TableWithFields(present) => {
                        // Validate required fields exist by name
                        for f in fields {
                            if !present.contains(&f.key) {
                                self.error(
                                    format!(
                                        "Table pattern requires field '{}' not present on value",
                                        f.key
                                    ),
                                    pattern.span(),
                                );
                            }
                        }
                        // Bind variables with Unknown type (no per-field typing yet)
                        for field in fields {
                            let binding_name = field.binding.as_ref().unwrap_or(&field.key);
                            self.declare(
                                binding_name.clone(),
                                VarInfo {
                                    ty: TcType::Unknown,
                                    mutable,
                                    annotated: false,
                                },
                            );
                        }
                    }
                    TcType::Table => {
                        for field in fields {
                            let binding_name = field.binding.as_ref().unwrap_or(&field.key);
                            self.declare(
                                binding_name.clone(),
                                VarInfo {
                                    ty: TcType::Unknown,
                                    mutable,
                                    annotated: false,
                                },
                            );
                        }
                    }
                    TcType::Unknown | TcType::Any => {
                        for field in fields {
                            let binding_name = field.binding.as_ref().unwrap_or(&field.key);
                            self.declare(
                                binding_name.clone(),
                                VarInfo {
                                    ty: TcType::Unknown,
                                    mutable,
                                    annotated: false,
                                },
                            );
                        }
                    }
                    _ => {
                        self.error(
                            format!("Table pattern requires Table type, got {ty}"),
                            pattern.span(),
                        );
                    }
                }
            }
            Pattern::Wildcard { .. } => {
                // Wildcard pattern doesn't bind any variables, just accepts any type
            }
            Pattern::Literal { value: _, .. } => {
                // Literal patterns don't bind variables, just match values
            }
        }
    }

    /// Check for unreachable patterns in a match expression.
    /// A pattern is unreachable if a previous pattern already catches all cases.
    pub fn check_unreachable_patterns(&mut self, arms: &[(Pattern, Vec<Stmt>)]) {
        let mut seen_catch_all = false;

        for (i, (pattern, _)) in arms.iter().enumerate() {
            if seen_catch_all {
                self.error(format!(
                    "Unreachable pattern: pattern #{} is unreachable because a previous pattern already covers all cases",
                    i + 1
                ), pattern.span());
            }

            // Check if this pattern is a catch-all
            match pattern {
                Pattern::Wildcard { .. } => {
                    seen_catch_all = true;
                }
                Pattern::Ident { name, .. } => {
                    // Identifier patterns that are not known tags are catch-all bindings
                    if !KNOWN_TAG_PATTERNS.contains(&name.as_str()) {
                        seen_catch_all = true;
                    }
                }
                _ => {
                    // Other patterns are not catch-all
                }
            }
        }
    }

    /// Check if a match expression is exhaustive.
    /// A match is exhaustive if:
    /// 1. It has a wildcard pattern (_), OR
    /// 2. It covers all known variants (like ok/err for Result, some/none for Option), OR
    /// 3. It covers all literal values (not practical, so we require wildcard for literals)
    pub fn check_match_exhaustiveness(
        &mut self,
        arms: &[(Pattern, Vec<Stmt>)],
        matched_ty: Option<&TcType>,
        match_span: Option<Span>,
    ) {
        let mut has_wildcard = false;
        let mut has_literal = false;
        let mut tags = HashSet::new();

        for (pattern, _) in arms {
            match pattern {
                Pattern::Wildcard { .. } => {
                    has_wildcard = true;
                }
                Pattern::Ident { name, .. } => {
                    // Identifier pattern in match can be:
                    // 1. A catch-all binding (acts like wildcard)
                    // 2. A tag pattern for Result/Option (ok/err/some/none)
                    // We check if it's a known tag; otherwise treat as catch-all
                    if KNOWN_TAG_PATTERNS.contains(&name.as_str()) {
                        tags.insert(name.as_str());
                    } else {
                        // Unknown identifier - treat as catch-all binding
                        has_wildcard = true;
                    }
                }
                Pattern::Literal { value: _, .. } => {
                    has_literal = true;
                }
                Pattern::ListPattern { .. } | Pattern::TablePattern { .. } => {
                    // Structural patterns are specific, not catch-all
                }
            }
        }

        // If we used tag patterns and the matched type has known fields, check presence first
        if !tags.is_empty()
            && !has_wildcard
            && let Some(ty) = matched_ty
            && let TcType::TableWithFields(fields) = ty
        {
            for &tag in &tags {
                if !fields.contains(&tag.to_string()) {
                    self.error(
                        format!("Match tag '{tag}' not present on matched table type"),
                        match_span,
                    );
                }
            }
        }

        // If we have a wildcard or catch-all identifier, we're exhaustive
        if has_wildcard {
            return;
        }

        // Check if we have all known tag variants
        let has_result_tags = tags.contains("ok") && tags.contains("err");
        let has_option_tags = tags.contains("some") && tags.contains("none");

        if has_result_tags || has_option_tags {
            // Exhaustive for Result or Option types
            return;
        }

        // If we have literal patterns without wildcard, not exhaustive
        if has_literal {
            self.error("Match expression is not exhaustive: literal patterns require a wildcard (_) or catch-all case".to_string(), match_span);
            return;
        }

        // If we have tags but not all variants, not exhaustive
        if !tags.is_empty() {
            self.error(format!(
                "Match expression is not exhaustive: found tags {tags:?} but missing wildcard or all variants (e.g., ok/err or some/none)"
            ), match_span);
            return;
        }

        // Otherwise, we need a wildcard
        self.error(
            "Match expression is not exhaustive: add a wildcard (_) pattern or cover all cases"
                .to_string(),
            match_span,
        );
    }
}
