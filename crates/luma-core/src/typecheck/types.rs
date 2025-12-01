//! Type representation for the type checker.

/// Internal type representation for type checking.
#[derive(Debug, Clone, PartialEq)]
pub enum TcType {
    Any,
    Unknown,
    Number,
    String,
    Boolean,
    Null,
    List(Box<TcType>),
    Table,
    /// Table with a known (best-effort) set of fields; used for structural presence checks.
    TableWithFields(Vec<String>),
    Function {
        params: Vec<TcType>,
        ret: Box<TcType>,
        /// If Some(idx), the parameter at index idx is variadic (captures remaining args as a list)
        variadic_index: Option<usize>,
    },
}

impl TcType {
    /// Check if two types are compatible for assignment or comparison.
    pub fn is_compatible(&self, other: &TcType) -> bool {
        match (self, other) {
            (TcType::Any, _) | (_, TcType::Any) => true,
            (TcType::Unknown, _) | (_, TcType::Unknown) => true,
            (TcType::Number, TcType::Number) => true,
            (TcType::String, TcType::String) => true,
            (TcType::Boolean, TcType::Boolean) => true,
            (TcType::Null, TcType::Null) => true,
            (TcType::List(a), TcType::List(b)) => a.is_compatible(b),
            (TcType::Table, TcType::Table) => true,
            (TcType::TableWithFields(_), TcType::Table) => true,
            (TcType::Table, TcType::TableWithFields(_)) => true,
            (TcType::TableWithFields(_), TcType::TableWithFields(_)) => true,
            (
                TcType::Function {
                    params: p1,
                    ret: r1,
                    variadic_index: v1,
                },
                TcType::Function {
                    params: p2,
                    ret: r2,
                    variadic_index: v2,
                },
            ) => {
                p1.len() == p2.len()
                    && p1.iter().zip(p2.iter()).all(|(a, b)| a.is_compatible(b))
                    && r1.is_compatible(r2)
                    && v1 == v2
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for TcType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TcType::Any => write!(f, "Any"),
            TcType::Unknown => write!(f, "Unknown"),
            TcType::Number => write!(f, "Number"),
            TcType::String => write!(f, "String"),
            TcType::Boolean => write!(f, "Boolean"),
            TcType::Null => write!(f, "Null"),
            TcType::List(inner) => write!(f, "List({inner})"),
            TcType::Table => write!(f, "Table"),
            TcType::TableWithFields(fields) => {
                write!(f, "Table({})", fields.join(", "))
            }
            TcType::Function {
                params,
                ret,
                variadic_index,
            } => {
                write!(f, "Function(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if Some(i) == *variadic_index {
                        write!(f, "...{param}")?;
                    } else {
                        write!(f, "{param}")?;
                    }
                }
                write!(f, ") -> {ret}")
            }
        }
    }
}

/// Information about a declared variable.
#[derive(Debug, Clone)]
pub struct VarInfo {
    pub ty: TcType,
    pub mutable: bool,
    #[allow(dead_code)]
    pub annotated: bool,
}
