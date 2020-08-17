use crate::ast::{Expr, HasType, Item, Type, TypeError};
use crate::error::Error;
use crate::error::ReturnValue;
use crate::file_provider::fs::FileProvider;
use crate::scope::{self, Scope};
use crate::span::{Span, SpanError};

use std::path::Path;

pub fn check(items_slice: &[Span<Item>]) -> Result<(), Error> {
	let mut scope = match load_scope() {
		Ok(v) => v,
		Err(e) => return Err(e)
	};
	for item in items_slice {
		match item.as_ref() {
			Item::Fn(name, _, _, _) => scope.add_variable(
				name.val(),
				scope::Type::NoMut(item.as_ref().get_type()),
				Some((item.clone(), false)),
			),
		}
		.expect("Error adding item");
	}
	if let Ok(scope::Type::NoMut(Type::Fn(args, ret))) = scope.get_type(&String::from("main")) {
		if args.is_empty() {
			if ret.as_ref().as_ref() == &Type::Void {
				let res = check_item(&String::from("main"), &mut scope).unwrap();
				println!(
					"{}",
					scope.map(&|x| {
						if let Some((item, checked)) = x {
							format!("([{}] {})", checked, item)
						} else {
							format!("NONE")
						}
					})
				);
				res
			} else {
				return Err(ret.error(
					"Return type should be void for main",
					ReturnValue::MainNonVoidRetType,
				));
			}
		} else {
			let spans = args.iter().map(|(_, x)| x.clone()).collect::<Vec<Span<Type>>>();
			return Err(Span::join(&spans, ()).error(
				"Main fubction shouldn't have arguments",
				ReturnValue::MainHasArguments,
			));
		}
	} else {
		return Err(items_slice
			.first()
			.unwrap()
			.error("No main function", ReturnValue::NoMain));
	}
}

pub fn load_scope() -> Result<Scope<Option<(Span<Item>, bool)>>, Error> {
	let mut scope = Scope::root();
	if let Err(e) = load_std(&mut scope, "") {
		return Err(e);
	}
	Ok(scope)
}

fn check_item(
	variable: &String,
	scope: &mut Scope<Option<(Span<Item>, bool)>>,
) -> Option<Result<(), Error>> {
	if let Ok(Some((main, checked))) = scope.get_value(variable) {
		if *checked {
			Some(Ok(()))
		} else {
			let main = main.clone();
			scope
				.set_value(variable, Some((main.clone(), true)))
				.expect("Item cant be checked");
			match main.val() {
				Item::Fn(_, args, ret, block) => {
					*scope = scope.clone().push();
					for (arg_name, arg_type) in args {
						if let Err(_) = scope.add_variable(
							arg_name.val(),
							scope::Type::NoMut(arg_type.val()),
							None,
						) {
							return Some(Err(
								arg_name.error("Name already defined", ReturnValue::NameDefined)
							));
						}
					}
					let ret_block = match block.get_type(&scope) {
						Ok(v) => v,
						Err(e) => panic!("{:?}", e.as_ref()),
					};
					if ret.as_ref() != &ret_block {
						return Some(Err(block.error(
							format!("Return type doesn't match defined return type (`{}`)", ret),
							ReturnValue::TypesDontMatch,
						)));
					}
					let res = check_expr(&block, scope);
					*scope = scope.clone().pop();
					Some(res)
				}
			}
		}
	} else {
		//unreachable!("{} is not defined or not an item", variable)
		None
	}
}

pub fn check_expr(
	expr: &Span<Expr>,
	scope: &mut Scope<Option<(Span<Item>, bool)>>,
) -> Result<(), Error> {
	match expr.as_ref() {
		Expr::Block(b) => {
			*scope = scope.clone().push();
			for e in b {
				if let Err(e) = check_expr(e, scope) {
					return Err(e);
				}
			}
			*scope = scope.clone().pop();
			Ok(())
		}
		Expr::Define(id, expr) => {
			let typ = match expr.get_type(scope) {
				Ok(v) => v,
				Err(t) => return Err(get_type_error(t)),
			};
			scope
				.add_variable(id.clone(), scope::Type::NoMut(typ), None)
				.expect("Error adding variable");
			Ok(())
		}
		Expr::DefineMut(id, expr) => {
			let typ = match expr.get_type(scope) {
				Ok(v) => v,
				Err(t) => return Err(get_type_error(t)),
			};
			scope
				.add_variable(id.clone(), scope::Type::Mut(typ), None)
				.expect("Error adding variable");
			Ok(())
		}
		_ => {
			match expr.get_type_with_call_cb(&scope.clone(), &mut |e| {
				if let Expr::Ident(id) = e.as_ref() {
					println!("ID: {}", id);
					if let Some(Err(e)) = check_item(id, scope) {
						return Err(e);
					}
				}
				Ok(())
			}) {
				Ok(_) => Ok(()),
				Err(t) => Err(get_type_error(t)),
			}
		}
	}
}

fn get_type_error(t: Span<TypeError>) -> Error {
	match t.as_ref() {
		TypeError::TraitNotImplemented(tr) => t.error(
			format!("Trait {:?} not implemented", tr),
			ReturnValue::TraitNotImplemented,
		),
		TypeError::BranchesDontMatch => {
			t.error("Branches dont match", ReturnValue::BrnchRetTypesDontMatch)
		}
		TypeError::IdentNotFound(id) => t.error(
			format!("Name `{}` not defined", id),
			ReturnValue::IdentNotDefined,
		),
		TypeError::Err(e) => e.clone(),
	}
}

fn load_std<P: AsRef<Path>>(
	scope: &mut Scope<Option<(Span<Item>, bool)>>,
	std_path: P,
) -> Result<(), Error> {
	let fp = FileProvider::new(&std_path);
	{
		let tokens = match crate::tokens::tokenize("src/std/print.lang", &fp) {
			Ok(v) => v,
			Err(e) => return Err(e),
		};
		let items = match crate::parser::parse_lines(tokens, true) {
			Ok(v) => v,
			Err(e) => return Err(e),
		};
		for item in items {
			match item.as_ref() {
				Item::Fn(name, _, _, _) => scope.add_variable(
					name.val(),
					scope::Type::NoMut(item.as_ref().get_type()),
					Some((item, true)),
				),
			}
			.expect("Error adding item");
		}
	}
	Ok(())
}
