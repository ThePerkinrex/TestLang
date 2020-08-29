use super::types::{HasType, TypeData, TypeError};
use crate::scope::{Scope, TypeDB};
use crate::span::Span;
use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intrinsic {
	Print,
	AddStr,
	AddNum,
	SubNum,
}

impl Intrinsic {
	pub fn from_str(s: &str) -> Option<Self> {
		match s {
			"INTRINSIC_PRINT" => Some(Self::Print),
			"INTRINSIC_ADD_STR" => Some(Self::AddStr),
			"INTRINSIC_ADD_NUM" => Some(Self::AddNum),
			"INTRINSIC_SUB_NUM" => Some(Self::SubNum),
			_ => None
		}
	}
}

impl<T: Clone> HasType<T> for Intrinsic {
	fn get_type_with_call_cb<F: FnMut(&Span<super::expr::Expr>) -> Result<(), Error>>(&self, _: &Scope<T>, _: &mut TypeDB, _: &mut F) -> std::result::Result<TypeData, Span<TypeError>> {
		match self {
			Self::Print => Ok(TypeData::Void),
			Self::AddStr => Ok(TypeData::String),
			Self::AddNum => Ok(TypeData::Number),
			Self::SubNum => Ok(TypeData::Number),
		}
	}
}