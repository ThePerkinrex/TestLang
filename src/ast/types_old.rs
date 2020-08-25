use super::expr::Expr;
use super::traits::Trait;
use super::Ident;
use crate::error::Error;
use crate::scope::Scope;
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Type {
	Number,
	String,
	Array(Box<Span<Type>>),
	Fn(Vec<(Span<Ident>, Span<Type>)>, Box<Span<Type>>),
	Void,
	Err,
	Never,
	// Generic(Generic),
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
				Trait::Neg(Self::Number)
			],
			Self::String => vec![Trait::Add(Self::String, Self::String)],
			Self::Array(t) => vec![Trait::Add(t.val(), self.clone())],
			Self::Fn(args, ret) => vec![Trait::Call(
				args.iter().map(|(_, x)| x.val()).collect(),
				ret.val(),
			)],
			Self::Void => vec![],
			Self::Err => vec![],
			Self::Never => vec![],
			// Self::Generic(g) => g.constraints.clone(),
		}
	}
}

impl PartialEq for Type {
	fn eq(&self, rhs: &Self) -> bool {
		match (self, rhs) {
			(Self::Number, Self::Number) => true,
			(Self::String, Self::String) => true,
			(Self::Never, Self::Never) => true,
			(Self::Err, Self::Err) => true,
			(Self::Void, Self::Void) => true,
			(Self::Array(a), Self::Array(b)) => a.as_ref() == b.as_ref(),
			(Self::Fn(args_a, ret_a), Self::Fn(args_b, ret_b)) => {
				let mut ret = ret_a.as_ref().as_ref() == ret_b.as_ref().as_ref();
				let mut i = 0;
				ret = ret && args_a.len() == args_b.len();
				while let (Some((_, arg_a)), Some((_, arg_b)), true) = (args_a.get(i), args_b.get(i), ret){
					ret = ret && arg_a.as_ref() == arg_b.as_ref();
					i += 1;
				}
				ret
			}
			_ => false
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
					.map(|(name, x)| format!("{}: {}", name, x))
					.collect::<Vec<String>>()
					.join(", "),
				ret
			),
			Self::Void => write!(f, "void"),
			Self::Err => write!(f, "ERROR (Type not known)"),
			Self::Never => write!(f, "!"),
			// Self::Generic(g) => write!(f, "{}", g),
		}
	}
}

#[derive(Debug, Clone)]
pub enum TypeError {
	TraitNotImplemented(Trait),
	BranchesDontMatch,
	IdentNotFound(Ident),
	Err(Error)
}

pub trait HasType<T: Clone> {
	fn get_type_with_call_cb<F: FnMut(&Span<Expr>) -> Result<(), Error>>(
		&self,
		scope: &Scope<T>,
		f: &mut F,
	) -> Result<Type, Span<TypeError>>;

	fn get_type(&self, scope: &Scope<T>) -> std::result::Result<Type, Span<TypeError>> {
		self.get_type_with_call_cb(scope, &mut |_| Ok(()))
	}
}
