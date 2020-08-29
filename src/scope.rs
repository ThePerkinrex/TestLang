use crate::ast;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
	Mut(ast::TypeData),
	NoMut(ast::TypeData),
}

#[allow(unused)]
impl Type {
	pub fn unwrap(self) -> ast::TypeData {
		match self {
			Self::Mut(x) => x,
			Self::NoMut(x) => x,
		}
	}

	pub fn unwrap_ref(&self) -> &ast::TypeData {
		match self {
			Self::Mut(x) => x,
			Self::NoMut(x) => x,
		}
	}

	pub fn unwrap_mut(&mut self) -> &mut ast::TypeData {
		match self {
			Self::Mut(x) => x,
			Self::NoMut(x) => x,
		}
	}
}

impl std::fmt::Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Mut(t) => write!(f, "mut {}", t),
			Self::NoMut(t) => write!(f, "{}", t),
		}
	}
}

#[derive(Clone)]
pub struct Scope<T: Clone> {
	parent: Option<Box<Scope<T>>>,
	variables: HashMap<String, (Type, T)>,
}

impl<T: Clone> Scope<T> {
	pub fn root() -> Self {
		Self {
			parent: None,
			variables: HashMap::new(),
		}
	}

	fn with_parent(parent: Scope<T>) -> Self {
		Self {
			parent: Some(Box::new(parent)),
			variables: HashMap::new(),
		}
	}

	pub fn push(self) -> Self {
		Self::with_parent(self)
	}

	pub fn pop(self) -> Self {
		if let Some(p) = self.parent {
			*p
		}else{
			self
		}
	}

	// pub fn is_root(&self) -> bool {
	// 	return self.parent.is_none()
	// }

	pub fn get_type<'a>(&self, variable: &'a String) -> Result<&Type, &'a String> {
		if let Some((typ, _)) = self.variables.get(variable) {
			Ok(typ)
		} else {
			if let Some(parent) = &self.parent {
				parent.get_type(variable)
			} else {
				Err(variable)
			}
		}
	}

	pub fn get_value(&self, variable: &String) -> Result<&T, ()> {
		if let Some((_, v)) = self.variables.get(variable) {
			Ok(v)
		} else {
			if let Some(parent) = &self.parent {
				parent.get_value(variable)
			} else {
				Err(())
			}
		}
	}

	pub fn set_value(&mut self, variable: &String, value: T) -> Result<(), ()> {
		if let Some((_, v)) = self.variables.get_mut(variable) {
			*v = value;
			Ok(())
		} else {
			if let Some(parent) = &mut self.parent {
				parent.set_value(variable, value)
			} else {
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

	pub fn add_variable(&mut self, variable: String, typ: Type, value: T) -> Result<(), ()> {
		if self.variables.contains_key(&variable) {
			return Err(());
		}
		self.variables.insert(variable, (typ, value));
		Ok(())
	}

	pub fn map<U: Clone, F: Fn(T) -> U>(mut self, f: &F) -> Scope<U> {
		Scope {
			parent: self.parent.map(|p| Box::new(p.map(f))),
			variables: self
				.variables
				.drain()
				.map(|(k, (typ, v))| (k, (typ, f(v))))
				.collect(),
		}
	}
}

impl<T: Clone + std::fmt::Display> std::fmt::Display for Scope<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		if let Some(p) = &self.parent {
			writeln!(f, "{}", p)?;
			writeln!(f, "\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/")?;
		}
		for (k, (typ, v)) in &self.variables {
			writeln!(f, "{}: {} = {}", k, typ, v)?;
		}
		Ok(())
	}
}

use std::hash::Hash;
// use std::ops::Index;

#[derive(Clone)]
pub struct GenericScope<K: Hash + Eq, V: Clone> {
	parent: Option<Box<Self>>,
	variables: HashMap<K, V>,
}

impl<K: Hash + Eq, V: Clone> GenericScope<K, V> {
	pub fn root() -> Self {
		Self {
			parent: None,
			variables: HashMap::new(),
		}
	}

	fn with_parent(parent: GenericScope<K, V>) -> Self {
		Self {
			parent: Some(Box::new(parent)),
			variables: HashMap::new(),
		}
	}

	pub fn get_mut_root(&mut self) -> &mut Self {
		let mut root = false;
		if let None = self.parent {
			root = true;
		}
		if root {
			self
		}else{
			if let Some(parent) = &mut self.parent {
				parent.get_mut_root()
			}else{
				unreachable!()
			}
		}
	}

	pub fn push(self) -> Self {
		Self::with_parent(self)
	}

	pub fn pop(self) -> Self {
		if let Some(p) = self.parent {
			*p
		}else{
			self
		}
	}

	pub fn get(&self, variable: &K) -> Result<&V, ()> {
		if let Some(v) = self.variables.get(variable) {
			Ok(v)
		} else {
			if let Some(parent) = &self.parent {
				parent.get(variable)
			} else {
				Err(())
			}
		}
	}

	pub fn get_mut(&mut self, variable: &K) -> Result<&mut V, ()> {
		if let Some(v) = self.variables.get_mut(variable) {
			Ok(v)
		} else {
			if let Some(parent) = &mut self.parent {
				parent.get_mut(variable)
			} else {
				Err(())
			}
		}
	}

	pub fn set(&mut self, variable: &K, value: V) -> Result<(), ()> {
		if let Some(v) = self.variables.get_mut(variable) {
			*v = value;
			Ok(())
		} else {
			if let Some(parent) = &mut self.parent {
				parent.set(variable, value)
			} else {
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

	pub fn add(&mut self, variable: K, value: V) -> Result<(), &V> {
		if self.variables.contains_key(&variable) {
			return Err(self.variables.get(&variable).unwrap());
		}
		self.variables.insert(variable, value);
		Ok(())
	}

	// pub fn map<U: Clone, F: Fn(V) -> U>(mut self, f: &F) -> GenericScope<K, U> {
	// 	GenericScope {
	// 		parent: self.parent.map(|p| Box::new(p.map(f))),
	// 		variables: self
	// 			.variables
	// 			.drain()
	// 			.map(|(k, v)| (k, f(v)))
	// 			.collect(),
	// 	}
	// }
}

impl<K: Hash + Eq + std::fmt::Display, V: Clone + std::fmt::Display> std::fmt::Display for GenericScope<K, V> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		if let Some(p) = &self.parent {
			writeln!(f, "{}", p)?;
			writeln!(f, "\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/")?;
		}
		for (k, v) in &self.variables {
			writeln!(f, "{} = {}", k, v)?;
		}
		Ok(())
	}
}

#[derive(Clone)]
pub struct TypeDB(GenericScope<ast::TypeData, ast::Type>);

impl TypeDB {
	pub fn new() -> Self {
		let mut scope = GenericScope::root();
		scope.add(ast::TypeData::Number, ast::TypeData::Number.default_type()).unwrap();
		scope.add(ast::TypeData::String, ast::TypeData::String.default_type()).unwrap();
		Self(scope)
	}

	pub fn push(self) -> Self {
		Self(self.0.push())
	}

	pub fn pop(self) -> Self {
		Self(self.0.pop())
	}

	pub fn get_no_mut(&self, data: &ast::TypeData) -> Result<&ast::Type, ()> {
		self.0.get(data)
	}

	pub fn get(&mut self, data: &ast::TypeData) -> ast::Type {
		self.clone().get_or(data, |d| self.add(d).unwrap())
	}

	pub fn get_or_add_to_root(&mut self, data: &ast::TypeData) -> ast::Type {
		self.clone().get_or(data, |d| self.add_root(d).unwrap())
	}

	fn get_or<'a, F: FnMut(ast::TypeData)>(&self, data: &ast::TypeData, mut f: F) -> ast::Type {
		// println!("Getting: {}", data);
		#[allow(unused_assignments)]
		let mut add = false;
		match self.0.clone().get(data) {
			Ok(v) => return v.clone(),
			Err(_) => {
				add = true;
			}
		};
		if add {
			// println!("Adding: {}", data);
			// let t = data.clone().default_type();
			f(data.clone());
			data.clone().default_type()
		}else{
			unreachable!()
		}
	}

	pub fn get_mut(&mut self, data: &ast::TypeData) -> Option<&mut ast::Type> {
		self.0.get_mut(data).ok()
	}

	pub fn add(&mut self, data: ast::TypeData) -> Result<(), &ast::Type> {
		self.0.add(data.clone(), data.default_type())
	}

	pub fn add_root(&mut self, data: ast::TypeData) -> Result<(), &ast::Type> {
		self.0.get_mut_root().add(data.clone(), data.default_type())
	}

	pub fn set(&mut self, data: ast::TypeData, t: ast::Type) {
		// println!("Setting: {}", data);
		match self.0.set(&data, t.clone()) {
			Ok(_) => (),
			Err(_) => self.0.add(data, t).unwrap()
		}
	}

	// pub fn map<U: Clone, F: Fn(ast::Type) -> U>(self, f: &F) -> GenericScope<ast::TypeData, U> {
	// 	self.0.map(f)
	// }
}