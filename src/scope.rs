use crate::ast;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
	Mut(ast::Type),
	NoMut(ast::Type)
}

#[allow(unused)]
impl Type {
	pub fn unwrap(self) -> ast::Type {
		match self {
			Self::Mut(x) => x,
			Self::NoMut(x) => x,
		}
	}

	pub fn unwrap_ref(&self) -> &ast::Type {
		match self {
			Self::Mut(x) => x,
			Self::NoMut(x) => x,
		}
	}

	pub fn unwrap_mut(&mut self) -> &mut ast::Type {
		match self {
			Self::Mut(x) => x,
			Self::NoMut(x) => x,
		}
	}
}

#[derive(Clone)]
pub struct Scope<T: Clone> {
	parent: Option<Box<Scope<T>>>,
	variables: HashMap<String, (Type, T)>
}

impl<T: Clone> Scope<T> {
	pub fn root() -> Self {
		Self {
			parent: None,
			variables: HashMap::new()
		}
	}

	fn with_parent(parent: Scope<T>) -> Self {
		Self {
			parent: Some(Box::new(parent)),
			variables: HashMap::new()
		}
	}

	pub fn push(self) -> Self {
		Self::with_parent(self)
	}

	// pub fn is_root(&self) -> bool {
	// 	return self.parent.is_none()
	// }

	pub fn get_type<'a>(&self, variable: &'a String) -> Result<&Type, &'a String> {
		if let Some((typ, _)) = self.variables.get(variable) {
			Ok(typ)
		}else{
			if let Some(parent) = &self.parent {
				parent.get_type(variable)
			}else{
				Err(variable)
			}
		}
	}

	pub fn get_value(&self, variable: &String) -> Result<&T, ()> {
		if let Some((_, v)) = self.variables.get(variable) {
			Ok(v)
		}else{
			if let Some(parent) = &self.parent {
				parent.get_value(variable)
			}else{
				Err(())
			}
		}
	}

	pub fn set_value(&mut self, variable: &String, value: T) -> Result<(), ()> {
		if let Some((_, v)) = self.variables.get_mut(variable) {
			*v = value;
			Ok(())
		}else{
			if let Some(parent) = &mut self.parent {
				parent.set_value(variable, value)
			}else{
				Err(())
			}
		}
	}

	// pub fn get_type_value(&self, variable: &String) -> Result<&(Type, T), ()> {
	// 	if let Some(v) = self.variables.get(variable) {
	// 		Ok(v)
	// 	}else{
	// 		if let Some(parent) = &self.parent {
	// 			parent.get_type_value(variable)
	// 		}else{
	// 			Err(())
	// 		}
	// 	}
	// }

	pub fn add_variable(&mut self, variable: String, typ: Type, value: T) -> Result<(),()> {
		if self.variables.contains_key(&variable){
			return Err(())
		}
		self.variables.insert(variable, (typ, value));
		Ok(())
	}
}