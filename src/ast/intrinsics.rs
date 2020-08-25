use super::types::{HasType, TypeData, TypeError};
use crate::scope::{Scope, TypeDB};
use crate::span::Span;
use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intrinsic {
	Print
}

impl Intrinsic {
	pub fn from_str(s: &str) -> Option<Self> {
		match s {
			"INTRINSIC_PRINT" => Some(Self::Print),
			_ => None
		}
	}
}

impl<T: Clone> HasType<T> for Intrinsic {
	fn get_type_with_call_cb<F: FnMut(&Span<super::expr::Expr>) -> Result<(), Error>>(&self, _: &Scope<T>, _: &mut TypeDB, _: &mut F) -> std::result::Result<TypeData, Span<TypeError>> {
		match self {
			Self::Print => Ok(TypeData::Void)
		}
	}
}