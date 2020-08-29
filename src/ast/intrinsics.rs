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
	MulNum,
	DivNum,
	ExpNum,

	EqStr,
	EqNum,
	EqBool,
}

impl Intrinsic {
	pub fn from_str(s: &str) -> Option<Self> {
		match s {
			"INTRINSIC_PRINT" => Some(Self::Print),
			"INTRINSIC_ADD_STR" => Some(Self::AddStr),
			"INTRINSIC_ADD_NUM" => Some(Self::AddNum),
			"INTRINSIC_SUB_NUM" => Some(Self::SubNum),
			"INTRINSIC_MUL_NUM" => Some(Self::MulNum),
			"INTRINSIC_DIV_NUM" => Some(Self::DivNum),
			"INTRINSIC_EXP_NUM" => Some(Self::ExpNum),

			"INTRINSIC_EQ_NUM" => Some(Self::EqNum),
			"INTRINSIC_EQ_STR" => Some(Self::EqStr),
			"INTRINSIC_EQ_BOOL" => Some(Self::EqBool),
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
			Self::MulNum => Ok(TypeData::Number),
			Self::DivNum => Ok(TypeData::Number),
			Self::ExpNum => Ok(TypeData::Number),

			Self::EqBool => Ok(TypeData::Bool),
			Self::EqNum => Ok(TypeData::Bool),
			Self::EqStr => Ok(TypeData::Bool),
		}
	}
}