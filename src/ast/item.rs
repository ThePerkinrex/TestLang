use crate::span::Span;

use super::types::Type;
use super::expr::Expr;
use super::Ident;

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
	Fn(
		Span<Ident>,
		Vec<(Span<Ident>, Span<Type>)>,
		Span<Type>,
		Span<Expr>,
	),
}

impl Item {
	pub fn get_type(&self) -> Type {
		match self {
			Self::Fn(_, args, ret, _) => Type::Fn(
				args.iter().map(|(_, x)| x.clone()).collect(),
				Box::new(ret.clone()),
			),
		}
	}
}

impl std::fmt::Display for Item {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		match self {
			Self::Fn(name, args, return_type, body) => write!(
				f,
				"fn {}({}) -> {} {}",
				name,
				args.iter()
					.map(|(id, typ)| format!("{}: {}", id, typ))
					.collect::<Vec<String>>()
					.join(", "),
				return_type,
				body
			),
		}
	}
}