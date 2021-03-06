use crate::ast::{intrinsics::Intrinsic, Block, Expr, FnSignature, HasType, Item, TypeData, Value};
use crate::error::Error;
use crate::file_provider::fs::FileProvider;
use crate::scope::{self, Scope, TypeDB};
use crate::span::Span;

use std::path::Path;

pub fn interpret_items(items: &[Span<Item>], type_db: &mut TypeDB) {
	run_fn(items, &String::from("main"), type_db).expect("Main function not found");
}

pub fn load_scope() -> Scope<Value> {
	let mut scope = Scope::root();
	load_std(&mut scope, "src/std").unwrap();
	scope
}

fn run_fn(items: &[Span<Item>], fn_name: &String, type_db: &mut TypeDB) -> Result<Value, ()> {
	let mut scope = load_scope();
	load_items_into_scope(&mut scope, type_db, &items);
	if let Ok(Value::Fn(_, boxed_body)) = scope.clone().get_value(fn_name) {
		if let Expr::Block(body) = boxed_body.as_ref() {
			return Ok(run_block(&mut scope, type_db, body));
		}
	}
	Err(())
}

fn load_std<P: AsRef<Path>>(scope: &mut Scope<Value>, std_path: P) -> Result<(), Error> {
	let fp = FileProvider::new(&std_path);
	{
		let tokens = match crate::tokens::tokenize("print.lang", &fp) {
			Ok(v) => v,
			Err(e) => return Err(e),
		};
		let items = match crate::parser::parse_lines(tokens, true) {
			Ok(v) => v,
			Err(e) => return Err(e),
		};
		for item in items {
			match item.as_ref() {
				Item::Fn(name, _, _, body) => {
					let fn_type = item.as_ref().get_type().unwrap();
					scope.add_variable(
						name.val(),
						scope::Type::NoMut(fn_type.clone()),
						Value::Fn(fn_type, Box::new(body.val())),
					)
				}
				Item::TraitDef(_, _) => Ok(()),
				Item::ImplTrait(_, _) => Ok(()),
			}
			.expect("Error adding std fn");
		}
	}
	Ok(())
}

fn load_items_into_scope(scope: &mut Scope<Value>, _type_db: &TypeDB, items: &[Span<Item>]) {
	for item in items {
		match item.as_ref() {
			Item::Fn(name, _, _, body) => {
				let fn_type = item.as_ref().get_type().unwrap();
				scope.add_variable(
					name.val(),
					scope::Type::NoMut(fn_type.clone()),
					Value::Fn(fn_type, Box::new(body.val())),
				)
			}
			Item::TraitDef(_, _) => Ok(()),
			Item::ImplTrait(_, _) => Ok(()),
		}
		.expect("Error adding item");
	}
}

fn run_block(scope: &mut Scope<Value>, type_db: &mut TypeDB, block: &Block) -> Value {
	*scope = scope.clone().push();
	for e in block {
		match run_expr(scope, type_db, e.as_ref()) {
			RetVal::Return(r) => return r,
			_ => (),
		}
	}
	*scope = scope.clone().pop();
	Value::Void
}

pub enum RetVal {
	Value(Value),
	Return(Value),
}

pub fn run_expr(scope: &mut Scope<Value>, type_db: &mut TypeDB, expr: &Expr) -> RetVal {
	RetVal::Value(match expr {
		Expr::None => Value::Void,
		Expr::Value(v) => v.clone(),
		Expr::Define(id, expr) => {
			let res = match run_expr(scope, type_db, expr.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let typ = expr.get_type(scope, type_db).unwrap();
			scope
				.add_variable(id.clone(), scope::Type::NoMut(typ), res)
				.unwrap();
			Value::Void
		}
		Expr::DefineMut(id, expr) => {
			let res = match run_expr(scope, type_db, expr.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let typ = expr.get_type(scope, type_db).unwrap();
			scope
				.add_variable(id.clone(), scope::Type::Mut(typ), res)
				.unwrap();
			Value::Void
		}
		Expr::Return(e) => {
			let res = match run_expr(scope, type_db, e.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			return RetVal::Return(res);
		}
		Expr::Ident(id) => {
			// println!("IDENT: {}", id);
			// println!("Scopeval: {:?}", scope.get_value(id));
			scope.get_value(id).unwrap().clone()
		}
		Expr::Block(b) => run_block(scope, type_db, b),
		Expr::CompilerIntrinsic(c) => return run_intrinsic(scope, c),
		Expr::Add(rhs, lhs) => {
			let rhs_value = match run_expr(scope, type_db, rhs.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let rhs_type_data = rhs_value.get_type(scope, type_db).unwrap();
			let rhs_type = type_db.get(&rhs_type_data);
			let lhs_value = match run_expr(scope, type_db, rhs.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let lhs_type_data = lhs_value.get_type(scope, type_db).unwrap();
			let lhs_type = type_db.get(&lhs_type_data);
			let impl_trait = rhs_type.get_impl_trait("Add", &[&lhs_type]).unwrap();
			let (method_sig, method_body) = impl_trait.get_method(&"add".into()).unwrap();
			match inner_run_fn(scope, type_db, method_sig, method_body.as_ref(), &[rhs, lhs]) {
				RetVal::Value(v) => v,
				x => return x
			}
		}
		Expr::Sub(rhs, lhs) => {
			let rhs_value = match run_expr(scope, type_db, rhs.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let rhs_type_data = rhs_value.get_type(scope, type_db).unwrap();
			let rhs_type = type_db.get(&rhs_type_data);
			let lhs_value = match run_expr(scope, type_db, rhs.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let lhs_type_data = lhs_value.get_type(scope, type_db).unwrap();
			let lhs_type = type_db.get(&lhs_type_data);
			let impl_trait = rhs_type.get_impl_trait("Sub", &[&lhs_type]).unwrap();
			let (method_sig, method_body) = impl_trait.get_method(&"sub".into()).unwrap();
			match inner_run_fn(scope, type_db, method_sig, method_body.as_ref(), &[rhs, lhs]) {
				RetVal::Value(v) => v,
				x => return x
			}
		}
		Expr::Mul(rhs, lhs) => {
			let rhs_value = match run_expr(scope, type_db, rhs.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let rhs_type_data = rhs_value.get_type(scope, type_db).unwrap();
			let rhs_type = type_db.get(&rhs_type_data);
			let lhs_value = match run_expr(scope, type_db, rhs.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let lhs_type_data = lhs_value.get_type(scope, type_db).unwrap();
			let lhs_type = type_db.get(&lhs_type_data);
			let impl_trait = rhs_type.get_impl_trait("Mul", &[&lhs_type]).unwrap();
			let (method_sig, method_body) = impl_trait.get_method(&"mul".into()).unwrap();
			match inner_run_fn(scope, type_db, method_sig, method_body.as_ref(), &[rhs, lhs]) {
				RetVal::Value(v) => v,
				x => return x
			}
		}
		Expr::Div(rhs, lhs) => {
			let rhs_value = match run_expr(scope, type_db, rhs.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let rhs_type_data = rhs_value.get_type(scope, type_db).unwrap();
			let rhs_type = type_db.get(&rhs_type_data);
			let lhs_value = match run_expr(scope, type_db, rhs.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let lhs_type_data = lhs_value.get_type(scope, type_db).unwrap();
			let lhs_type = type_db.get(&lhs_type_data);
			let impl_trait = rhs_type.get_impl_trait("Div", &[&lhs_type]).unwrap();
			let (method_sig, method_body) = impl_trait.get_method(&"div".into()).unwrap();
			match inner_run_fn(scope, type_db, method_sig, method_body.as_ref(), &[rhs, lhs]) {
				RetVal::Value(v) => v,
				x => return x
			}
		}
		Expr::Exp(rhs, lhs) => {
			let rhs_value = match run_expr(scope, type_db, rhs.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let rhs_type_data = rhs_value.get_type(scope, type_db).unwrap();
			let rhs_type = type_db.get(&rhs_type_data);
			let lhs_value = match run_expr(scope, type_db, rhs.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let lhs_type_data = lhs_value.get_type(scope, type_db).unwrap();
			let lhs_type = type_db.get(&lhs_type_data);
			let impl_trait = rhs_type.get_impl_trait("Exp", &[&lhs_type]).unwrap();
			let (method_sig, method_body) = impl_trait.get_method(&"exp".into()).unwrap();
			match inner_run_fn(scope, type_db, method_sig, method_body.as_ref(), &[rhs, lhs]) {
				RetVal::Value(v) => v,
				x => return x
			}
		}
		Expr::Eq(rhs, lhs) => {
			let rhs_value = match run_expr(scope, type_db, rhs.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let rhs_type_data = rhs_value.get_type(scope, type_db).unwrap();
			let rhs_type = type_db.get(&rhs_type_data);
			let lhs_value = match run_expr(scope, type_db, rhs.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let lhs_type_data = lhs_value.get_type(scope, type_db).unwrap();
			let lhs_type = type_db.get(&lhs_type_data);
			let impl_trait = rhs_type.get_impl_trait("Eq", &[&lhs_type]).unwrap();
			let (method_sig, method_body) = impl_trait.get_method(&"eq".into()).unwrap();
			match inner_run_fn(scope, type_db, method_sig, method_body.as_ref(), &[rhs, lhs]) {
				RetVal::Value(v) => v,
				x => return x
			}
		}
		Expr::Neg(e) => {
			let rhs_value = match run_expr(scope, type_db, e.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let rhs_type_data = rhs_value.get_type(scope, type_db).unwrap();
			let rhs_type = type_db.get(&rhs_type_data);
			
			let impl_trait = rhs_type.get_impl_trait("Neg", &[]).unwrap();
			let (method_sig, method_body) = impl_trait.get_method(&"neg".into()).unwrap();
			match inner_run_fn(scope, type_db, method_sig, method_body.as_ref(), &[e]) {
				RetVal::Value(v) => v,
				x => return x
			}
		}
		Expr::Call(callee, args) => {
			let v = match run_expr(scope, type_db, callee.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			match v {
				Value::Fn(typ, block) => {
					let mut new_scope = scope.clone().push();
					if let TypeData::Fn(FnSignature(a, _)) = typ {
						for (arg_expr, (arg_name, arg_type)) in args.iter().zip(a.iter()) {
							new_scope
								.add_variable(
									arg_name.val(),
									scope::Type::NoMut(arg_type.val()),
									match run_expr(scope, type_db, arg_expr.as_ref()) {
										RetVal::Value(v) => v,
										x => return x,
									},
								)
								.unwrap();
						}
					} else {
						unreachable!()
					}
					let ret = match run_expr(&mut new_scope, type_db, block.as_ref()) {
						RetVal::Value(v) => v,
						x => return x,
					};
					*scope = new_scope.pop();
					ret
				}
				_ => unreachable!(),
			}
		}
	})
}

fn inner_run_fn(
	scope: &mut Scope<Value>,
	type_db: &mut TypeDB,
	sig: &FnSignature,
	block: &Expr,
	args: &[&Span<Expr>],
) -> RetVal {
	let mut new_scope = scope.clone().push();
	let FnSignature(a, _) = sig;
	for (arg_expr, (arg_name, arg_type)) in args.iter().zip(a.iter()) {
		new_scope
			.add_variable(
				arg_name.val(),
				scope::Type::NoMut(arg_type.val()),
				match run_expr(scope, type_db, arg_expr.as_ref()) {
					RetVal::Value(v) => v,
					x => return x,
				},
			)
			.unwrap();
	}

	let ret = match run_expr(&mut new_scope, type_db, block) {
		RetVal::Value(v) => v,
		x => return x,
	};
	*scope = new_scope.pop();
	RetVal::Value(ret)
}

fn run_intrinsic(scope: &Scope<Value>, intrinsic: &Intrinsic) -> RetVal {
	match intrinsic {
		Intrinsic::Print => {
			if let Value::Str(s) = scope.get_value(&"s".into()).unwrap() {
				println!("{}", s)
			} else {
				unreachable!()
			}
			RetVal::Value(Value::Void)
		}
		Intrinsic::AddStr => {
			if let Value::Str(a) = scope.get_value(&"self".into()).unwrap() {
				if let Value::Str(b) = scope.get_value(&"other".into()).unwrap() {
					RetVal::Value(Value::Str(format!("{}{}", a, b)))
				} else {
					unreachable!()
				}
			} else {
				unreachable!()
			}
		}
		Intrinsic::AddNum => {
			if let Value::Num(a) = scope.get_value(&"self".into()).unwrap() {
				if let Value::Num(b) = scope.get_value(&"other".into()).unwrap() {
					RetVal::Value(Value::Num(a + b))
				} else {
					unreachable!()
				}
			} else {
				unreachable!()
			}
		}
		Intrinsic::SubNum => {
			if let Value::Num(a) = scope.get_value(&"self".into()).unwrap() {
				if let Value::Num(b) = scope.get_value(&"other".into()).unwrap() {
					RetVal::Value(Value::Num(a - b))
				} else {
					unreachable!()
				}
			} else {
				unreachable!()
			}
		}
		Intrinsic::MulNum => {
			if let Value::Num(a) = scope.get_value(&"self".into()).unwrap() {
				if let Value::Num(b) = scope.get_value(&"other".into()).unwrap() {
					RetVal::Value(Value::Num(a * b))
				} else {
					unreachable!()
				}
			} else {
				unreachable!()
			}
		}
		Intrinsic::DivNum => {
			if let Value::Num(a) = scope.get_value(&"self".into()).unwrap() {
				if let Value::Num(b) = scope.get_value(&"other".into()).unwrap() {
					RetVal::Value(Value::Num(a / b))
				} else {
					unreachable!()
				}
			} else {
				unreachable!()
			}
		}
		Intrinsic::ExpNum => {
			if let Value::Num(a) = scope.get_value(&"self".into()).unwrap() {
				if let Value::Num(b) = scope.get_value(&"other".into()).unwrap() {
					
					RetVal::Value(Value::Num(a.powf(*b)))
				} else {
					unreachable!()
				}
			} else {
				unreachable!()
			}
		}

		Intrinsic::EqNum => {
			if let Value::Num(a) = scope.get_value(&"self".into()).unwrap() {
				if let Value::Num(b) = scope.get_value(&"other".into()).unwrap() {
					RetVal::Value(if a == b {Value::True} else {Value::False})
				} else {
					unreachable!()
				}
			} else {
				unreachable!()
			}
		}
		Intrinsic::EqStr => {
			if let Value::Str(a) = scope.get_value(&"self".into()).unwrap() {
				if let Value::Str(b) = scope.get_value(&"other".into()).unwrap() {
					RetVal::Value(if a == b {Value::True} else {Value::False})
				} else {
					unreachable!()
				}
			} else {
				unreachable!()
			}
		}
		Intrinsic::EqBool => {
			let a = scope.get_value(&"self".into()).unwrap();
			let b = scope.get_value(&"other".into()).unwrap();
			RetVal::Value(match (a, b) {
				(Value::True, Value::True) => Value::True,
				(Value::False, Value::False) => Value::True,
				(Value::True, Value::False) => Value::False,
				(Value::False, Value::True) => Value::False,
				_ => unreachable!()
			})
		}
	}
}
