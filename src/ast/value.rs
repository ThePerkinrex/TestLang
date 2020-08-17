use crate::scope::Scope;
use crate::span::Span;
use crate::error::Error;

use super::types::{HasType, Type, TypeError};
use super::expr::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Num(f64),
	Str(String),
	Fn(Type, Box<Expr>),
	Never,
	Void,
}

impl<T: Clone> HasType<T> for Value {
	fn get_type_with_call_cb<F: FnMut(&Span<super::expr::Expr>) -> Result<(), Error>>(&self, _: &Scope<T>, _: &mut F) -> Result<Type, Span<TypeError>> {
		Ok(match self {
			Self::Num(_) => Type::Number,
			Self::Str(_) => Type::String,
			Self::Fn(t, _) => t.clone(),
			Self::Never => Type::Never,
			Self::Void => Type::Void,
		})
	}
}

impl std::fmt::Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Num(n) => write!(f, "{}", n),
			Self::Str(s) => write!(f, "\"{}\"", s),
			Self::Fn(t, b) => write!(f, "{} {}", t, b),
			Self::Void => write!(f, "void"),
			Self::Never => write!(f, "!"),
		}
	}
}
