use crate::error::Error;
use crate::scope::{Scope, TypeDB};
use crate::span::Span;

use super::expr::Expr;
use super::{Block, Ident};

use std::collections::HashMap;

// pub type TypeDB = HashMap<TypeData, Type>;

#[derive(Clone, Hash, Eq)]
pub struct FnSignature(
	pub Vec<(Span<Ident>, Span<TypeData>)>,
	pub Box<Span<TypeData>>,
);

impl FnSignature {
	pub fn matches_args(&self, args_other: &[TypeData]) -> bool {
		let FnSignature(args_self, _) = self;
		let mut ret = true;
		let mut i = 0;
		while let (Some((_, typ1)), Some(typ2), true) = (args_self.get(i), args_other.get(i), ret) {
			ret = ret && typ1.as_ref() == typ2;
			i += 1;
		}
		ret
	}

	pub fn args_types_tuple(&self) -> Type {
		TypeData::Tuple(self.0.iter().map(|(_, t)| t.val()).collect()).default_type()
	}
}

impl PartialEq for FnSignature {
	fn eq(&self, rhs: &Self) -> bool {
		let FnSignature(args_self, ret_self) = self;
		let FnSignature(args_other, ret_other) = rhs;
		let mut ret = ret_self.as_ref().as_ref() == ret_other.as_ref().as_ref();
		let mut i = 0;
		ret = ret && args_self.len() == args_other.len();
		while let (Some((_, typ1)), Some((_, typ2)), true) =
			(args_self.get(i), args_other.get(i), ret)
		{
			ret = ret && typ1 == typ2;
			i += 1;
		}
		ret
	}
}

impl std::fmt::Debug for FnSignature {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(
			f,
			"fn({}) -> {}",
			self.0
				.iter()
				.map(|(name, x)| format!("{}: {}", name, x))
				.collect::<Vec<String>>()
				.join(", "),
			self.1.as_ref()
		)
	}
}

#[derive(Clone)]
pub struct Type {
	data: TypeData,
	imp: Impl,
	traits: Vec<ImplTrait>,
}

impl Type {
	pub fn add_impl_trait(&mut self, impl_trait: ImplTrait) {
		self.traits.push(impl_trait)
	}

	pub fn get_impl_trait(&self, name: &str, defining_types: &[&Type]) -> Option<&ImplTrait> {
		for t in &self.traits {
			if t.trait_name() == name && t.matches(defining_types) {
				return Some(t);
			}
		}
		None
	}

	pub fn type_data(&self) -> &TypeData {
		&self.data
	}
}

impl PartialEq for Type {
	fn eq(&self, rhs: &Self) -> bool {
		self.data == rhs.data
	}
}

impl std::fmt::Debug for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{}", self.data)
	}
}

impl std::fmt::Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{}", self.data)
	}
}

#[derive(Clone, Debug, Hash, Eq)]
pub enum TypeData {
	Number,
	String,
	Array(Box<TypeData>),
	Fn(FnSignature),
	Tuple(Vec<TypeData>),
	Void,
	Err,
	Never,
	SelfRef,
	Other(String),
}

impl TypeData {
	pub fn default_type(self) -> Type {
		Type {
			data: self,
			imp: Default::default(),
			traits: vec![],
		}
	}
}

impl PartialEq for TypeData {
	fn eq(&self, rhs: &Self) -> bool {
		match (self, rhs) {
			(Self::Number, Self::Number) => true,
			(Self::String, Self::String) => true,
			(Self::Void, Self::Void) => true,
			(Self::Err, Self::Err) => true,
			(Self::Never, Self::Never) => true,
			(Self::Array(t1), Self::Array(t2)) => t1 == t2,
			(Self::Fn(f1), Self::Fn(f2)) => f1 == f2,
			(Self::Tuple(v1), Self::Tuple(v2)) => {
				v1.iter().zip(v2).fold(true, |b, (v1, v2)| b && v1 == v2)
			}
			(Self::SelfRef, Self::SelfRef) => true,
			(Self::Other(o1), Self::Other(o2)) => o1 == o2,
			_ => false,
		}
	}
}

impl std::fmt::Display for TypeData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		match self {
			Self::Number => write!(f, "number"),
			Self::String => write!(f, "string"),
			Self::Array(typ) => write!(f, "[{}]", typ),
			Self::Fn(x) => write!(f, "{:?}", x),
			Self::Void => write!(f, "void"),
			Self::Err => write!(f, "ERROR (Type not known)"),
			Self::Never => write!(f, "!"),
			Self::Tuple(t) => write!(
				f,
				"({})",
				t.iter()
					.map(|x| format!("{}", x))
					.collect::<Vec<String>>()
					.join(", ")
			), // Self::Generic(g) => write!(f, "{}", g),
			Self::SelfRef => write!(f, "Self"),
			Self::Other(o) => write!(f, "{}", o),
		}
	}
}

#[derive(Debug, Clone)]
pub enum TypeError {
	TraitNotImplemented(String, Vec<Type>, TypeData),
	BranchesDontMatch,
	IdentNotFound(Ident),
	Err(Error),
}

pub trait HasType<T: Clone> {
	fn get_type_with_call_cb<F: FnMut(&Span<Expr>) -> Result<(), Error>>(
		&self,
		scope: &Scope<T>,
		type_db: &mut TypeDB,
		f: &mut F,
	) -> Result<TypeData, Span<TypeError>>;

	fn get_type(
		&self,
		scope: &Scope<T>,
		type_db: &mut TypeDB,
	) -> std::result::Result<TypeData, Span<TypeError>> {
		self.get_type_with_call_cb(scope, type_db, &mut |_| Ok(()))
	}
}

#[derive(Clone, Default)]
pub struct Impl {
	methods: HashMap<Ident, (Type, Span<Block>)>,
	// functions: HashMap<Ident, (Type, Span<Block>)>,
}

#[derive(Clone, Debug)]
pub struct ImplTrait {
	based_on: Span<Ident>,
	defining_types: Vec<Span<Type>>,
	typedefs: HashMap<Ident, Span<Type>>,
	methods: HashMap<Ident, (FnSignature, Span<Expr>)>,
	// functions: HashMap<Ident, (FnSignature, Span<Expr>)>,
}

impl ImplTrait {
	pub fn new(
		based_on: Span<Ident>,
		defining_types: Vec<Span<Type>>,
		typedefs: HashMap<Ident, Span<Type>>,
		methods: HashMap<Ident, (FnSignature, Span<Expr>)>,
	) -> Self {
		Self {
			based_on,
			defining_types,
			typedefs,
			methods,
		}
	}

	// pub fn methods(&self) -> &HashMap<Ident, (FnSignature, Span<Expr>)> {
	// 	&self.methods
	// }

	pub fn get_method(&self, id: &Ident) -> Option<&(FnSignature, Span<Expr>)> {
		self.methods.get(id)
	}

	pub fn matches(&self, definfing_types: &[&Type]) -> bool {
		let mut ret = self.defining_types.len() == definfing_types.len();
		let mut i = 0;
		while let (Some(v1), Some(v2), true) =
			(self.defining_types.get(i), definfing_types.get(i), ret)
		{
			ret = ret && &v1.as_ref() == v2;
			i += 1;
		}
		ret
	}

	fn trait_name(&self) -> &str {
		&self.based_on.as_ref()
	}

	pub fn trait_name_string(&self) -> &String {
		self.based_on.as_ref()
	}

	pub fn get_typedef<T: Into<Ident>>(&self, name: T) -> Option<&Type> {
		self.typedefs.get(&name.into()).map(|x| x.as_ref())
	}

	pub fn matches_trait<
		F: FnMut(
			&TypeDB,
			Vec<(Span<Ident>, Span<TypeData>)>,
			Span<TypeData>,
			Span<Expr>,
		) -> Result<(), Error>,
	>(
		&self,
		t: &Trait,
		type_db: &TypeDB,
		mut checker_fn: F,
	) -> Result<bool, Error> {
		let mut type_db = type_db.clone().push();
		println!(
			"Checking if impl trait {} matches {}",
			self.trait_name(),
			t.name
		);
		let mut ret = self.trait_name() == t.name.as_ref()
			&& self.defining_types.len() == t.defining_types.len();
		for (name, t) in t.defining_types.iter().zip(self.defining_types.iter()) {
			type_db.set(TypeData::Other(name.val()), t.val())
		}
		// TODO give better errors
		if ret == false {
			println!("Names or defining types len don't match");
			return Ok(false);
		}
		for (self_k, self_t) in &self.typedefs {
			ret = ret
				&& t.typedefs
					.iter()
					.position(|x| x.as_ref() == self_k)
					.is_some();
			if ret == false {
				return Ok(false);
			}
			type_db.set(TypeData::Other(self_k.clone()), self_t.val())
		}
		for (self_method_name, (self_signature, self_body)) in &self.methods {
			println!(
				"Checking {} {:?} is present in trait",
				self_method_name, self_signature
			);
			ret = ret
				&& t.methods
					.get(self_method_name)
					.map(|fn_sig| {
						println!(
							"[METHOD SIGNATURE] {} {:?} is present in trait",
							self_method_name, self_signature
						);
						let FnSignature(args_other, ret_other) = fn_sig.clone();
						let FnSignature(args_self, ret_self) = self_signature.clone();
						let mut ret = type_db.get_no_mut(ret_other.as_ref().as_ref())
							== type_db.get_no_mut(ret_self.as_ref().as_ref());
						println!(
							"{} (({}){:?} == ({}){:?})",
							ret,
							ret_other,
							type_db.get_no_mut(ret_other.as_ref().as_ref()),
							ret_self,
							type_db.get_no_mut(ret_self.as_ref().as_ref())
						);
						for ((_, other_type), (_, self_type)) in
							args_other.iter().zip(args_self.iter())
						{
							ret = ret
								&& type_db.get_no_mut(other_type.as_ref())
									== type_db.get_no_mut(self_type.as_ref());
							println!(
								"{} (({}){:?} == ({}){:?})",
								ret,
								other_type,
								type_db.get_no_mut(other_type.as_ref()),
								self_type,
								type_db.get_no_mut(self_type.as_ref())
							);
						}
						ret
					})
					.unwrap_or(false);
			//println!("{}", ret);
			if ret == false {
				return Ok(false);
			}
			{
				let FnSignature(args, ret) = self_signature;
				checker_fn(&type_db, args.clone(), *ret.clone(), self_body.clone())?;
			}
		}
		//println!("{}", ret);
		Ok(ret)
	}
}

impl PartialEq for ImplTrait {
	fn eq(&self, rhs: &Self) -> bool {
		let mut ret = self.based_on == rhs.based_on;
		for (k, v) in &self.typedefs {
			if let Some(v2) = rhs.typedefs.get(k) {
				ret = ret && v.as_ref() == v2.as_ref();
			} else {
				ret = false;
			}
			if !ret {
				break;
			}
		}
		ret
	}
}

impl std::fmt::Display for ImplTrait {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(
			f,
			"{}<{}>",
			self.trait_name(),
			self.defining_types
				.iter()
				.map(|x| format!("{}", x))
				.collect::<Vec<String>>()
				.join(", ")
		)
	}
}

#[derive(Clone)]
pub struct Trait {
	name: Span<Ident>,
	defining_types: Vec<Span<Ident>>,
	typedefs: Vec<Span<Ident>>,
	methods: HashMap<Ident, FnSignature>,
	// functions: HashMap<Ident, FnSignature>,
}

impl Trait {
	pub fn new(
		name: Span<Ident>,
		defining_types: Vec<Span<Ident>>,
		typedefs: Vec<Span<Ident>>,
		methods: HashMap<Ident, FnSignature>,
	) -> Self {
		Self {
			name,
			typedefs,
			defining_types,
			methods,
		}
	}
}

impl std::fmt::Debug for Trait {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{}", self)
	}
}

impl std::fmt::Display for Trait {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(
			f,
			"{}<{}>",
			self.name.as_ref(),
			self.defining_types
				.iter()
				.map(|x| format!("{}", x))
				.collect::<Vec<String>>()
				.join(", ")
		)
	}
}

impl PartialEq for Trait {
	fn eq(&self, rhs: &Self) -> bool {
		self.name == rhs.name
	}
}
