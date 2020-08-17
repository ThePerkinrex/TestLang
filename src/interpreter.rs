use crate::ast::{intrinsics::Intrinsic, Block, Expr, HasType, Item, Type, Value};
use crate::error::Error;
use crate::file_provider::fs::FileProvider;
use crate::scope::{self, Scope};
use crate::span::Span;

use std::path::Path;

pub fn interpret_items(items: &[Span<Item>]) {
	run_fn(items, &String::from("main")).expect("Main function not found");
}

pub fn load_scope() -> Scope<Value> {
	let mut scope = Scope::root();
	load_std(&mut scope, "src/std").unwrap();
	scope
}

fn run_fn(items: &[Span<Item>], fn_name: &String) -> Result<Value, ()> {
	let mut scope = load_scope();
	load_items_into_scope(&mut scope, &items);
	if let Ok(Value::Fn(_, boxed_body)) = scope.clone().get_value(fn_name) {
		if let Expr::Block(body) = boxed_body.as_ref() {
			return Ok(run_block(&mut scope, body))
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
			let item_type = item.as_ref().get_type();
			match item.as_ref() {
				Item::Fn(name, _, _, body) => scope.add_variable(
					name.val(),
					scope::Type::NoMut(item_type.clone()),
					Value::Fn(item_type, Box::new(body.val())),
				),
			}
			.expect("Error adding std fn");
		}
	}
	Ok(())
}

fn load_items_into_scope(scope: &mut Scope<Value>, items: &[Span<Item>]) {
	for item in items {
		let item_type = item.as_ref().get_type();
		match item.as_ref() {
			Item::Fn(name, _, _, body) => scope.add_variable(
				name.val(),
				scope::Type::NoMut(item_type.clone()),
				Value::Fn(item_type, Box::new(body.val())),
			),
		}
		.expect("Error adding item");
	}
}

fn run_block(scope: &mut Scope<Value>, block: &Block) -> Value {
	*scope = scope.clone().push();
	for e in block {
		match run_expr(scope, e.as_ref()) {
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

pub fn run_expr(scope: &mut Scope<Value>, expr: &Expr) -> RetVal {
	RetVal::Value(match expr {
		Expr::None => Value::Void,
		Expr::Value(v) => v.clone(),
		Expr::Define(id, expr) => {
			let res = match run_expr(scope, expr.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let typ = expr.get_type(scope).unwrap();
			scope.add_variable(id.clone(), scope::Type::NoMut(typ), res).unwrap();
			Value::Void
		}
		Expr::DefineMut(id, expr) => {
			let res = match run_expr(scope, expr.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			let typ = expr.get_type(scope).unwrap();
			scope.add_variable(id.clone(), scope::Type::Mut(typ), res).unwrap();
			Value::Void
		}
		Expr::Return(e) => {
			let res = match run_expr(scope, e.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			return RetVal::Return(res);
		}
		Expr::Ident(id) => {
			println!("IDENT: {}", id);
			println!("Scopeval: {:?}", scope.get_value(id));
			scope.get_value(id).unwrap().clone()
		},
		Expr::Block(b) => run_block(scope, b),
		Expr::CompilerIntrinsic(c) => return run_intrinsic(scope, c),
		Expr::Add(rhs, lhs) => {
			let op = (
				match run_expr(scope, rhs.as_ref().as_ref()) {
					RetVal::Value(v) => v,
					x => return x,
				},
				match run_expr(scope, lhs.as_ref().as_ref()) {
					RetVal::Value(v) => v,
					x => return x,
				},
			);
			// TODO: Add native trait implementations
			match op {
				(Value::Num(a), Value::Num(b)) => Value::Num(a + b),
				(Value::Str(a), Value::Str(b)) => Value::Str(format!("{}{}", a, b)),
				_ => unreachable!(),
			}
		}
		Expr::Sub(rhs, lhs) => {
			let op = (
				match run_expr(scope, rhs.as_ref().as_ref()) {
					RetVal::Value(v) => v,
					x => return x,
				},
				match run_expr(scope, lhs.as_ref().as_ref()) {
					RetVal::Value(v) => v,
					x => return x,
				},
			);
			// TODO: Add native trait implementations
			match op {
				(Value::Num(a), Value::Num(b)) => Value::Num(a - b),
				_ => unreachable!(),
			}
		}
		Expr::Mul(rhs, lhs) => {
			let op = (
				match run_expr(scope, rhs.as_ref().as_ref()) {
					RetVal::Value(v) => v,
					x => return x,
				},
				match run_expr(scope, lhs.as_ref().as_ref()) {
					RetVal::Value(v) => v,
					x => return x,
				},
			);
			// TODO: Add native trait implementations
			match op {
				(Value::Num(a), Value::Num(b)) => Value::Num(a * b),
				_ => unreachable!(),
			}
		}
		Expr::Div(rhs, lhs) => {
			let op = (
				match run_expr(scope, rhs.as_ref().as_ref()) {
					RetVal::Value(v) => v,
					x => return x,
				},
				match run_expr(scope, lhs.as_ref().as_ref()) {
					RetVal::Value(v) => v,
					x => return x,
				},
			);
			// TODO: Add native trait implementations
			match op {
				(Value::Num(a), Value::Num(b)) => Value::Num(a / b),
				_ => unreachable!(),
			}
		}
		Expr::Exp(rhs, lhs) => {
			let op = (
				match run_expr(scope, rhs.as_ref().as_ref()) {
					RetVal::Value(v) => v,
					x => return x,
				},
				match run_expr(scope, lhs.as_ref().as_ref()) {
					RetVal::Value(v) => v,
					x => return x,
				},
			);
			// TODO: Add native trait implementations
			match op {
				(Value::Num(a), Value::Num(b)) => Value::Num(a.powf(b)),
				_ => unreachable!(),
			}
		}
		Expr::Neg(e) => {
			let v = match run_expr(scope, e.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			match v {
				Value::Num(n) => Value::Num(-n),
				_ => unreachable!(),
			}
		}
		Expr::Call(callee, args) => {
			let v = match run_expr(scope, callee.as_ref().as_ref()) {
				RetVal::Value(v) => v,
				x => return x,
			};
			match v {
				Value::Fn(typ, block) => {
					let mut new_scope = scope.clone().push();
					if let Type::Fn(a, _) = typ {
						for (arg_expr, (arg_name, arg_type)) in args.iter().zip(a.iter()) {
							new_scope
								.add_variable(
									arg_name.val(),
									scope::Type::NoMut(arg_type.val()),
									match run_expr(scope, arg_expr.as_ref()) {
										RetVal::Value(v) => v,
										x => return x,
									},
								)
								.unwrap();
						}
					} else {
						unreachable!()
					}
					let ret = match run_expr(&mut new_scope, block.as_ref()) {
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
	}
}
