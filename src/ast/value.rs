use crate::scope::{Scope, TypeDB};
use crate::span::Span;
use crate::error::Error;

use super::types::{HasType, TypeData, TypeError};
use super::expr::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Num(f64),
	Str(String),
	Fn(TypeData, Box<Expr>),
	Never,
	Void,
}

impl<T: Clone> HasType<T> for Value {
	fn get_type_with_call_cb<F: FnMut(&Span<super::expr::Expr>) -> Result<(), Error>>(&self, _: &Scope<T>, _:&mut TypeDB, _: &mut F) -> Result<TypeData, Span<TypeError>> {
		Ok(match self {
			Self::Num(_) => TypeData::Number,
			Self::Str(_) => TypeData::String,
			Self::Fn(t, _) => t.clone(),
			Self::Never => TypeData::Never,
			Self::Void => TypeData::Void,
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
