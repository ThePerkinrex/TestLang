use super::types::{HasType, Type, TypeError};
use crate::scope::Scope;
use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intrinsic {
	Print
}

impl Intrinsic {
	pub fn from_str(s: &str) -> Option<Self> {
		match s {
			"INSTRINSIC_PRINT" => Some(Self::Print),
			_ => None
		}
	}
}

impl<T: Clone> HasType<T> for Intrinsic {
	fn get_type(&self, scope: &Scope<T>) -> std::result::Result<Type, Span<TypeError>> {
		match self {
			Self::Print => Ok(Type::Void)
		}
	}
}