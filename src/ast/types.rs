use crate::span::Span;
use crate::scope::Scope;
use super::traits::Trait;
use super::Ident;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
	Number,
	String,
	Array(Box<Span<Type>>),
	Fn(Vec<Span<Type>>, Box<Span<Type>>),
	Void,
	Err,
	NoReturn,
}

impl Type {
	pub fn get_traits(&self) -> Vec<Trait> {
		match self {
			Self::Number => vec![
				Trait::Add(Self::Number, Self::Number),
				Trait::Sub(Self::Number, Self::Number),
				Trait::Mul(Self::Number, Self::Number),
				Trait::Div(Self::Number, Self::Number),
				Trait::Exp(Self::Number, Self::Number),
			],
			Self::String => vec![Trait::Add(Self::String, Self::String)],
			Self::Array(t) => vec![Trait::Add(t.val(), self.clone())],
			Self::Fn(args, ret) => vec![Trait::Call(
				args.iter().map(|x| x.val()).collect(),
				ret.val(),
			)],
			Self::Void => vec![],
			Self::Err => vec![],
			Self::NoReturn => vec![],
		}
	}
}

impl std::fmt::Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		match self {
			Self::Number => write!(f, "number"),
			Self::String => write!(f, "string"),
			Self::Array(typ) => write!(f, "[{}]", typ),
			Self::Fn(args, ret) => write!(
				f,
				"fn({}) -> {}",
				args.iter()
					.map(|x| format!("{}", x))
					.collect::<Vec<String>>()
					.join(", "),
				ret
			),
			Self::Void => write!(f, "void"),
			Self::Err => write!(f, "ERROR (Type not known)"),
			Self::NoReturn => write!(f, "!"),
		}
	}
}

#[derive(Debug, Clone)]
pub enum TypeError {
	TraitNotImplemented(Trait),
	BranchesDontMatch,
	IdentNotFound(Ident),
}

pub trait HasType<T: Clone> {
	fn get_type(&self, scope: &Scope<T>) -> Result<Type, Span<TypeError>>;
}
