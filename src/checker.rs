use crate::ast::{Expr, FnSignature, HasType, Ident, Item, Trait, TypeData, TypeError};
use crate::error::Error;
use crate::error::ReturnValue;
use crate::file_provider::fs::FileProvider;
use crate::scope::{self, Scope, TypeDB};
use crate::span::{Span, SpanError};

use std::collections::HashMap;
use std::path::Path;

pub type TraitDB = HashMap<Ident, Trait>;

pub fn check(items_slice: &[Span<Item>], type_db: &mut TypeDB) -> Result<(), Error> {
	let mut trait_db = TraitDB::new();
	let mut scope = match load_scope(type_db, &mut trait_db) {
		Ok(v) => v,
		Err(e) => return Err(e),
	};
	load_items(items_slice, &mut scope, type_db, &mut trait_db)?;
	println!("Loaded items, checking main");
	if let Ok(TypeData::Fn(FnSignature(args, ret))) = scope
		.get_type(&String::from("main"))
		.map(|x| x.unwrap_ref())
	{
		if args.is_empty() {
			if ret.as_ref().as_ref() == &TypeData::Void {
				println!("main found");
				let res = check_item(&String::from("main"), &mut scope, type_db).unwrap();
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
			let spans = args
				.iter()
				.map(|(_, x)| x.clone())
				.collect::<Vec<Span<TypeData>>>();
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

pub fn load_scope(
	type_db: &mut TypeDB,
	trait_db: &mut TraitDB,
) -> Result<Scope<Option<(Span<Item>, bool)>>, Error> {
	let mut scope = Scope::root();
	if let Err(e) = load_std(&mut scope, type_db, trait_db, "") {
		return Err(e);
	}
	Ok(scope)
}

fn check_item(
	variable: &String,
	scope: &mut Scope<Option<(Span<Item>, bool)>>,
	type_db: &mut TypeDB,
) -> Option<Result<(), Error>> {
	if let Ok(Some((main, checked))) = scope.get_value(variable) {
		if *checked {
			Some(Ok(()))
		} else {
			let main = main.clone();
			scope
				.set_value(variable, Some((main.clone(), true)))
				.expect("Item cant be checked off");
			match main.val() {
				Item::Fn(_, args, ret, block) => Some(check_fn(scope, type_db, args, ret, block)),

				_ => None,
			}
		}
	} else {
		//unreachable!("{} is not defined or not an item", variable)
		None
	}
}

pub fn check_fn(
	scope: &mut Scope<Option<(Span<Item>, bool)>>,
	type_db: &mut TypeDB,
	args: Vec<(Span<Ident>, Span<TypeData>)>,
	ret: Span<TypeData>,
	block: Span<Expr>,
) -> Result<(), Error> {
	*scope = scope.clone().push();
	for (arg_name, arg_type) in args {
		if let Err(_) = scope.add_variable(arg_name.val(), scope::Type::NoMut(arg_type.val()), None)
		{
			return Err(
				arg_name.error("Name already defined", ReturnValue::NameDefined)
			);
		}
	}
	let ret_block = match block.get_type(&scope, type_db) {
		Ok(v) => v,
		Err(e) => panic!("{:?}", e.as_ref()),
	};
	if ret.as_ref() != &ret_block {
		return Err(block.error(
			format!("Return type doesn't match defined return type (`{}`)", ret),
			ReturnValue::TypesDontMatch,
		));
	}
	println!("Check item's block");
	let res = check_expr(&block, scope, type_db);
	*scope = scope.clone().pop();
	res
}

pub fn check_expr(
	expr: &Span<Expr>,
	scope: &mut Scope<Option<(Span<Item>, bool)>>,
	type_db: &mut TypeDB,
) -> Result<(), Error> {
	println!("Checking expr: {}", expr);
	match expr.as_ref() {
		Expr::Block(b) => {
			*scope = scope.clone().push();
			let mut reversed_block = b.clone();
			reversed_block.reverse();
			for e in &reversed_block {
				if let Err(e) = check_expr(e, scope, type_db) {
					return Err(e);
				}
			}
			*scope = scope.clone().pop();
			Ok(())
		}
		Expr::Define(id, expr) => {
			let typ = match expr.get_type(scope, type_db) {
				Ok(v) => v,
				Err(t) => return Err(get_type_error(t)),
			};
			scope
				.add_variable(id.clone(), scope::Type::NoMut(typ), None)
				.expect("Error adding variable");
			Ok(())
		}
		Expr::DefineMut(id, expr) => {
			let typ = match expr.get_type(scope, type_db) {
				Ok(v) => v,
				Err(t) => return Err(get_type_error(t)),
			};
			scope
				.add_variable(id.clone(), scope::Type::Mut(typ), None)
				.expect("Error adding variable");
			Ok(())
		}
		_ => {
			match expr.get_type_with_call_cb(&scope.clone(), &mut type_db.clone(), &mut |e| {
				if let Expr::Ident(id) = e.as_ref() {
					// println!("ID: {}", id);
					if let Some(Err(e)) = check_item(id, scope, type_db) {
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
		TypeError::TraitNotImplemented(name, defining_types, for_type) => t.error(
			format!(
				"Trait `{}<{}>` not implemented for type `{}`",
				name,
				defining_types
					.iter()
					.map(|x| format!("{}", x))
					.collect::<Vec<String>>()
					.join(", "),
				for_type
			),
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
	type_db: &mut TypeDB,
	trait_db: &mut TraitDB,
	std_path: P,
) -> Result<(), Error> {
	let fp = FileProvider::new(&std_path);
	let files = ["src/std/print.lang", "src/std/ops.lang"];
	for f in &files {
		println!("Loading file {}", f);
		println!("Tokenizing");
		let tokens = match crate::tokens::tokenize(f, &fp) {
			Ok(v) => v,
			Err(e) => return Err(e),
		};
		println!("Parsing");
		let items = match crate::parser::parse_lines(tokens, true) {
			Ok(v) => v,
			Err(e) => return Err(e),
		};
		println!("Loading items into scope");
		load_items(&items, scope, type_db, trait_db)?;
	}
	Ok(())
}

pub fn load_items(
	items: &[Span<Item>],
	scope: &mut Scope<Option<(Span<Item>, bool)>>,
	type_db: &mut TypeDB,
	trait_db: &mut TraitDB,
) -> Result<(), Error> {
	for item in items {
		match item.as_ref() {
			Item::Fn(name, _, _, _) => match scope.add_variable(
				name.val(),
				scope::Type::NoMut(item.as_ref().get_type().unwrap()),
				Some((item.clone(), false)),
			) {
				Ok(v) => Ok(v),
				Err(_) => Err("Error adding scope to variable".into()),
			}
			.and_then(|_| match type_db.add(item.as_ref().get_type().unwrap()) {
				Ok(v) => Ok(v),
				Err(_) => Err(format!(
					"Error adding type `{}` to db",
					item.as_ref().get_type().unwrap()
				)),
			}),
			Item::TraitDef(name, t) => match trait_db.insert(name.val(), t.clone()) {
				Some(_) => Err(format!(
					"Name `{}` already defined for a trait",
					name.as_ref()
				)),
				None => Ok(()),
			},
			Item::ImplTrait(for_type_data, impl_trait) => {
				*type_db = type_db.clone().push();
				let for_type = type_db.get(for_type_data.as_ref());
				type_db.set(TypeData::SelfRef, for_type);
				let res = if let Some(t) = trait_db.get(impl_trait.trait_name_string()) {
					if impl_trait.matches_trait(t, &type_db, |type_db, args, ret, body|{
						let mut type_db = type_db.clone().push();
						check_fn(scope, &mut type_db, args, ret, body)
					})? {
						match type_db.get_mut(for_type_data.as_ref()) {
							Some(x) => {
								x.add_impl_trait(impl_trait.clone());
								Ok(())
							}
							None => Err("Error adding trait to type".into()),
						}
					} else {
						Err("Trait doesn't match impl trait".into())
					}
				} else {
					Err("Trait not defined".into())
				};
				*type_db = type_db.clone().pop();
				res
			}
		}
		.expect("Error adding item");
	}
	Ok(())
}
