use crate::error::Error;
use crate::operators::Operator;
use crate::scope::{Scope, TypeDB};
use crate::span::Span;

use super::intrinsics::Intrinsic;
use super::types::{HasType, TypeData, TypeError};
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
	Eq(Box<Span<Expr>>, Box<Span<Expr>>),
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
			Operator::Eq => Self::Eq(Box::new(rhs), Box::new(lhs)),
		}
	}
}

impl<T: Clone> HasType<T> for Span<Expr> {
	fn get_type_with_call_cb<F: FnMut(&Span<Expr>) -> Result<(), Error>>(
		&self,
		scope: &Scope<T>,
		type_db: &mut TypeDB,
		f: &mut F,
	) -> Result<TypeData, Span<TypeError>> {
		Ok(match self.as_ref() {
			Expr::Define(_, _) => TypeData::Void,
			Expr::DefineMut(_, _) => TypeData::Void,
			Expr::Return(_) => TypeData::Never,
			Expr::CompilerIntrinsic(i) => match i.get_type_with_call_cb(scope, type_db, f) {
				Ok(v) => v,
				Err(e) => return Err(e),
			},
			Expr::Value(v) => match v.get_type_with_call_cb(scope, type_db, f) {
				Ok(v) => v,
				Err(e) => return Err(e),
			},
			Expr::Ident(id) => {
				if let Ok(t) = scope.get_type(id) {
					t.clone().unwrap()
				} else {
					return Err(self.clone().map(TypeError::IdentNotFound(id.clone())));
				}
			}
			Expr::Neg(expr) => {
				let expr_type = match expr.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				if let Some(trait_impl) = type_db.get(&expr_type).get_impl_trait("Neg", &[]) {
					let t = trait_impl.get_typedef("Output").unwrap();
					return Ok(t.type_data().clone());
				}
				return Err(self.clone().map(TypeError::TraitNotImplemented("Neg".into(), vec![], expr_type)));
			}
			Expr::Add(rhs, lhs) => {
				let rhs_type = match rhs.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				let lhs_type = match lhs.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				if let Some(trait_impl) = type_db.get(&rhs_type).get_impl_trait("Add", &[&type_db.get(&lhs_type)]) {
					let t = trait_impl.get_typedef("Output").unwrap();
					return Ok(t.type_data().clone());
				}
				return Err(rhs
					.clone()
					.map(TypeError::TraitNotImplemented("Add".into(), vec![type_db.get(&lhs_type)], rhs_type)));
			}
			Expr::Sub(rhs, lhs) => {
				let rhs_type = match rhs.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				let lhs_type = match lhs.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				if let Some(trait_impl) = type_db.get(&rhs_type).get_impl_trait("Sub", &[&type_db.get(&lhs_type)]) {
					let t = trait_impl.get_typedef("Output").unwrap();
					return Ok(t.type_data().clone());
				}
				return Err(rhs
					.clone()
					.map(TypeError::TraitNotImplemented("Sub".into(), vec![type_db.get(&lhs_type)], rhs_type)));
			}
			Expr::Mul(rhs, lhs) => {
				let rhs_type = match rhs.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				let lhs_type = match lhs.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				if let Some(trait_impl) = type_db.get(&rhs_type).get_impl_trait("Mul", &[&type_db.get(&lhs_type)]) {
					let t = trait_impl.get_typedef("Output").unwrap();
					return Ok(t.type_data().clone());
				}
				return Err(rhs
					.clone()
					.map(TypeError::TraitNotImplemented("Mul".into(), vec![type_db.get(&lhs_type)], rhs_type)));
			}
			Expr::Div(rhs, lhs) => {
				let rhs_type = match rhs.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				let lhs_type = match lhs.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				if let Some(trait_impl) = type_db.get(&rhs_type).get_impl_trait("Div", &[&type_db.get(&lhs_type)]) {
					let t = trait_impl.get_typedef("Output").unwrap();
					return Ok(t.type_data().clone());
				}
				return Err(rhs
					.clone()
					.map(TypeError::TraitNotImplemented("Div".into(), vec![type_db.get(&lhs_type)], rhs_type)));
			}
			Expr::Exp(rhs, lhs) => {
				let rhs_type = match rhs.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				let lhs_type = match lhs.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				if let Some(trait_impl) = type_db.get(&rhs_type).get_impl_trait("Exp", &[&type_db.get(&lhs_type)]) {
					let t = trait_impl.get_typedef("Output").unwrap();
					return Ok(t.type_data().clone());
				}
				return Err(rhs
					.clone()
					.map(TypeError::TraitNotImplemented("Exp".into(), vec![type_db.get(&lhs_type)], rhs_type)));
			}
			Expr::Eq(rhs, lhs) => {
				let rhs_type = match rhs.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				let lhs_type = match lhs.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				if type_db.get(&rhs_type).get_impl_trait("Eq", &[&type_db.get(&lhs_type)]).is_some() {
					
					return Ok(TypeData::Bool);
				}
				return Err(rhs
					.clone()
					.map(TypeError::TraitNotImplemented("Eq".into(), vec![type_db.get(&lhs_type)], rhs_type)));
			}
			Expr::Call(callee, args) => {
				if let Err(e) = f(&callee) {
					return Err(e.clone().span(TypeError::Err(e)));
				}
				let callee_type = match callee.get_type_with_call_cb(scope, type_db, f) {
					Ok(v) => v,
					Err(e) => return Err(e),
				};
				let mut args_types = Vec::new();
				for x in args {
					args_types.push(match x.get_type_with_call_cb(scope, type_db, f) {
						Ok(v) => v,
						Err(e) => return Err(e),
					})
				}
				//println!("Checking if {} is fn", callee_type);
				if let TypeData::Fn(fn_sign) = callee_type.clone() {
					if fn_sign.matches_args(&args_types) {
						//println!("{} is fn that matches argtypes", callee_type);
						return Ok(fn_sign.1.val());
					} else {
						return Err(callee.clone().map(TypeError::TraitNotImplemented(
							"Call".into(),
							vec![fn_sign.args_types_tuple()], callee_type
						)));
					}
				}
				let args_tuple = TypeData::Tuple(args_types).default_type();
				if let Some(trait_impl) = type_db.get(&callee_type)
					.get_impl_trait("Call", &[&args_tuple])
				{
					let t = trait_impl.get_typedef("Output").unwrap();
					return Ok(t.type_data().clone());
				}
				return Err(callee.clone().map(TypeError::TraitNotImplemented(
					"Call".into(),
					vec![args_tuple],
					callee_type
				)));
			}
			Expr::Block(e) => {
				for expr in e {
					if let Expr::Return(expr) = expr.val() {
						return expr.get_type_with_call_cb(scope, type_db, f);
					}
				}
				TypeData::Void
			}
			// Expr::If(_, _, _, _) => todo!(),
			Expr::None => TypeData::Void,
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
			Self::Eq(rhs, lhs) => write!(f, "({} == {})", rhs, lhs),
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
