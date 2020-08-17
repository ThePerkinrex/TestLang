use crate::operators::Operator;
use crate::scope::Scope;
use crate::span::Span;
use crate::error::Error;

use super::intrinsics::Intrinsic;
use super::traits::Trait;
use super::types::{HasType, Type, TypeError};
use super::value::Value;
use super::{Block, Ident};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
	Value(Value),
	Ident(Ident),
	Call(Box<Span<Expr>>, Vec<Span<Expr>>),
	Neg(Box<Span<Expr>>),
	Add(Box<Span<Expr>>, Box<Span<Expr>>),
	Sub(Box<Span<Expr>>, Box<Span<Expr>>),
	Mul(Box<Span<Expr>>, Box<Span<Expr>>),
	Div(Box<Span<Expr>>, Box<Span<Expr>>),
	Exp(Box<Span<Expr>>, Box<Span<Expr>>),
	Block(Block),
	// If(
	// 	Box<Span<Expr>>,
	// 	Box<Span<Expr>>,
	// 	Vec<(Span<Expr>, Span<Expr>)>,
	// 	Option<Box<Span<Expr>>>,
	// ),
	Return(Box<Span<Expr>>),
	Define(Ident, Box<Span<Expr>>),
	DefineMut(Ident, Box<Span<Expr>>),
	CompilerIntrinsic(Intrinsic),
	None,
}

impl Expr {
	pub fn from_op(op: Operator, rhs: Span<Expr>, lhs: Span<Expr>) -> Self {
		match op {
			Operator::Add => Self::Add(Box::new(rhs), Box::new(lhs)),
			Operator::Sub => Self::Sub(Box::new(rhs), Box::new(lhs)),
			Operator::Mul => Self::Mul(Box::new(rhs), Box::new(lhs)),
			Operator::Div => Self::Div(Box::new(rhs), Box::new(lhs)),
			Operator::Exp => Self::Exp(Box::new(rhs), Box::new(lhs)),
		}
	}
}

impl<T: Clone> HasType<T> for Span<Expr> {
	fn get_type_with_call_cb<F: FnMut(&Span<Expr>) -> Result<(), Error>>(&self, scope: &Scope<T>, f: &mut F) -> Result<Type, Span<TypeError>> {
		Ok(match self.as_ref() {
			Expr::Define(_, _) => Type::Void,
			Expr::DefineMut(_, _) => Type::Void,
			Expr::Return(_) => Type::Never,
			Expr::CompilerIntrinsic(i) => match i.get_type_with_call_cb(scope, f) {
				Ok(v) => v,
				Err(e) => return Err(e),
			},
			Expr::Value(v) => match v.get_type_with_call_cb(scope, f) {
				Ok(v) => v,
				Err(e) => return Err(e),
			}
			Expr::Ident(id) => {
				if let Ok(t) = scope.get_type(id) {
					t.clone().unwrap()
				} else {
					return Err(self.clone().map(TypeError::IdentNotFound(id.clone())));
				}
			}
			Expr::Neg(expr) => {
				let expr_type = match expr.get_type_with_call_cb(scope, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				for t in expr_type.get_traits() {
					if let Trait::Neg(t) = t {
						return Ok(t);
					}
				}
				return Err(self
					.clone()
					.map(TypeError::TraitNotImplemented(Trait::Neg(Type::Err))));
			}
			Expr::Add(rhs, lhs) => {
				let rhs_type = match rhs.get_type_with_call_cb(scope, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				let lhs_type = match lhs.get_type_with_call_cb(scope, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				for t in rhs_type.get_traits() {
					if let Trait::Add(lhs_required_type, ret_type) = t {
						if lhs_required_type == lhs_type {
							return Ok(ret_type);
						}
					}
				}
				return Err(rhs.clone().map(TypeError::TraitNotImplemented(Trait::Add(
					lhs_type,
					Type::Err,
				))));
			}
			Expr::Sub(rhs, lhs) => {
				let rhs_type = match rhs.get_type_with_call_cb(scope, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				let lhs_type = match lhs.get_type_with_call_cb(scope, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				for t in rhs_type.get_traits() {
					if let Trait::Sub(lhs_required_type, ret_type) = t {
						if lhs_required_type == lhs_type {
							return Ok(ret_type);
						}
					}
				}
				return Err(rhs.clone().map(TypeError::TraitNotImplemented(Trait::Sub(
					lhs_type,
					Type::Err,
				))));
			}
			Expr::Mul(rhs, lhs) => {
				let rhs_type = match rhs.get_type_with_call_cb(scope, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				let lhs_type = match lhs.get_type_with_call_cb(scope, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				for t in rhs_type.get_traits() {
					if let Trait::Mul(lhs_required_type, ret_type) = t {
						if lhs_required_type == lhs_type {
							return Ok(ret_type);
						}
					}
				}
				return Err(rhs.clone().map(TypeError::TraitNotImplemented(Trait::Mul(
					lhs_type,
					Type::Err,
				))));
			}
			Expr::Div(rhs, lhs) => {
				let rhs_type = match rhs.get_type_with_call_cb(scope, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				let lhs_type = match lhs.get_type_with_call_cb(scope, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				for t in rhs_type.get_traits() {
					if let Trait::Div(lhs_required_type, ret_type) = t {
						if lhs_required_type == lhs_type {
							return Ok(ret_type);
						}
					}
				}
				return Err(rhs.clone().map(TypeError::TraitNotImplemented(Trait::Div(
					lhs_type,
					Type::Err,
				))));
			}
			Expr::Exp(rhs, lhs) => {
				let rhs_type = match rhs.get_type_with_call_cb(scope, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				let lhs_type = match lhs.get_type_with_call_cb(scope, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				for t in rhs_type.get_traits() {
					if let Trait::Exp(lhs_required_type, ret_type) = t {
						if lhs_required_type == lhs_type {
							return Ok(ret_type);
						}
					}
				}
				return Err(rhs.clone().map(TypeError::TraitNotImplemented(Trait::Exp(
					lhs_type,
					Type::Err,
				))));
			}
			Expr::Call(callee, args) => {
				if let Err(e) = f(&callee) {
					return Err(e.clone().span(TypeError::Err(e)))
				}
				let callee_type = match callee.get_type_with_call_cb(scope, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				let mut args_types = Vec::new();
				for x in args {
					args_types.push(match x.get_type_with_call_cb(scope, f) {
						Ok(v) => v,
						Err(e) => return Err(e),
					})
				}
				for t in callee_type.get_traits() {
					if let Trait::Call(a, t) = t {
						if a == args_types {
							return Ok(t);
						}
					}
				}
				return Err(callee
					.clone()
					.map(TypeError::TraitNotImplemented(Trait::Call(
						args_types,
						Type::Err,
					))));
			}
			Expr::Block(e) => {
				for expr in e {
					if let Expr::Return(expr) = expr.val() {
						return expr.get_type_with_call_cb(scope, f);
					}
				}
				Type::Void
			}
			// Expr::If(_, _, _, _) => todo!(),
			Expr::None => Type::Void,
		})
	}
}

impl std::fmt::Display for Expr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		match self {
			Self::Add(rhs, lhs) => write!(f, "({} + {})", rhs, lhs),
			Self::Sub(rhs, lhs) => write!(f, "({} - {})", rhs, lhs),
			Self::Mul(rhs, lhs) => write!(f, "({} * {})", rhs, lhs),
			Self::Div(rhs, lhs) => write!(f, "({} / {})", rhs, lhs),
			Self::Exp(rhs, lhs) => write!(f, "({} ** {})", rhs, lhs),
			Self::Value(v) => write!(f, "{}", v),
			Self::Ident(name) => write!(f, "{}", name),
			Self::Call(body, args) => write!(
				f,
				"{}({})",
				body,
				args.iter()
					.map(|x| x.to_string())
					.collect::<Vec<String>>()
					.join(", ")
			),
			Self::Neg(body) => write!(f, "-{}", body),
			Self::Block(b) => write!(
				f,
				"{{\n{}}}",
				b.iter()
					.map(|x| format!("{};\n", x))
					.fold(String::new(), |x, s| s + &x)
			),
			// Self::If(condition, val, elifs, el) => {
			// 	write!(f, "if {} {}", condition, val)?;
			// 	for (cond, v) in elifs {
			// 		write!(f, " else if {} {}", cond, v)?;
			// 	}
			// 	if let Some(v) = el {
			// 		write!(f, " else {}", v)?;
			// 	}
			// 	write!(f, "")
			// }
			Self::Define(id, expr) => write!(f, "let {} = {};", id, expr),
			Self::DefineMut(id, expr) => write!(f, "let mut {} = {};", id, expr),
			Self::Return(e) => write!(f, "return {}", e),
			Self::CompilerIntrinsic(i) => write!(f, "INTRINSIC#{:?}#", i),
			Self::None => write!(f, ""),
		}
	}
}
