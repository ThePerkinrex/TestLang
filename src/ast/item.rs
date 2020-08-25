use crate::span::Span;

use super::expr::Expr;
use super::types::{FnSignature, ImplTrait, Trait, TypeData};
use super::Ident;

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
	Fn(
		Span<Ident>,
		Vec<(Span<Ident>, Span<TypeData>)>,
		Span<TypeData>,
		Span<Expr>,
	),
	TraitDef(Span<Ident>, Trait),
	ImplTrait(Span<TypeData>, ImplTrait),
}

impl Item {
	pub fn get_type(&self) -> Option<TypeData> {
		match self {
			Self::Fn(_, args, ret, _) => {
				Some(TypeData::Fn(FnSignature(args.clone(), Box::new(ret.clone()))))
			},
			_ => None
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
			Self::TraitDef(_, t) => write!(f, "{}", t),
			Self::ImplTrait(t, impl_trait) => write!(f, "impl {} for {}", impl_trait, t)
		}
	}
}
