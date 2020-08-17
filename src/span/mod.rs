use crate::ast;
use crate::tokens::Token;
use crate::error::{Error, ErrorKind};

use std::convert::{AsMut, AsRef};
use std::path::PathBuf;

pub mod location;
use location::*;

pub type File = PathBuf;

#[derive(Clone, PartialEq)]
pub struct Span<T>
where
	T: Clone,
{
	v: T,
	loc: Location,
}

impl<T> Span<T>
where
	T: Clone,
{
	pub fn new(v: T, loc: Location) -> Self {
		Self { v, loc }
	}

	pub fn val(&self) -> T {
		self.v.clone()
	}

	pub fn unwrap(self) -> T {
		self.v
	}

	pub fn join_with<U>(self, other: &[Span<T>], line: U) -> Span<U>
	where
		U: Clone,
	{
		let start = other.iter().fold(self.loc.start, |min, x| {
			if x.loc.start < min {
				x.loc.start
			} else {
				min
			}
		});
		let end = other.iter().fold(
			self.loc.end,
			|max, x| if x.loc.end > max { x.loc.end } else { max },
		);
		for x in other {
			assert_eq!(
				self.loc.file(), x.loc.file(),
				"Files don't match when joining tokens"
			)
		}
		Span::new(line, Location::new(self.loc.file(), start, end))
	}

	pub fn map<U: Clone>(self, line: U) -> Span<U> {
		self.join_with(&[], line)
	}

	pub fn join<U>(other: &[Span<T>], line: U) -> Span<U>
	where
		U: Clone,
	{
		other[0].clone().join_with(&other[1..], line)
	}
}

impl<T: Clone> AsRef<T> for Span<T> {
	fn as_ref(&self) -> &T {
		&self.v
	}
}

impl<T: Clone> AsMut<T> for Span<T> {
	fn as_mut(&mut self) -> &mut T {
		&mut self.v
	}
}

impl std::fmt::Debug for Span<Token> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		write!(f, "{:?}[{}]", self.v, self.loc)
	}
}

impl std::fmt::Display for Span<ast::Expr> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		write!(f, "{}", self.v)
	}
}

impl std::fmt::Debug for Span<ast::Expr> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		write!(f, "{:?}", self.v)
	}
}

impl std::fmt::Display for Span<ast::Item> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		write!(f, "{}", self.v)
	}
}

impl std::fmt::Debug for Span<ast::Item> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		write!(f, "{:?}", self.v)
	}
}

impl std::fmt::Display for Span<ast::Ident> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		write!(f, "{}", self.v)
	}
}

impl std::fmt::Debug for Span<ast::Ident> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		write!(f, "{:?}", self.v)
	}
}

impl std::fmt::Display for Span<ast::Type> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		write!(f, "{}", self.v)
	}
}

impl std::fmt::Debug for Span<ast::Type> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		write!(f, "{:?}", self.v)
	}
}

impl std::fmt::Debug for Span<ast::TypeError> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		write!(f, "{:?}", self.v)
	}
}

pub trait HasLoc {
	fn loc(&self) -> &Location;
}

impl<T> HasLoc for Span<T>
where
	T: Clone,
{
	fn loc(&self) -> &Location {
		&self.loc
	}
}

pub trait SpanError {
	fn error<T: ToString, R: Into<i32>>(&self, info: T, r: R) -> Error;
	fn warn<T: ToString>(&self, info: T) -> Error;
	fn info<T: ToString>(&self, info: T) -> Error;
}

impl<T> SpanError for T
where
	T: HasLoc,
{
	fn error<S: ToString, R: Into<i32>>(&self, info: S, r: R) -> Error {
		Error::new(
			self.loc().clone(),
			info.to_string(),
			ErrorKind::Error(r.into()),
		)
	}
	fn warn<S: ToString>(&self, info: S) -> Error {
		Error::new(
			self.loc().clone(),
			info.to_string(),
			ErrorKind::Warning,
		)
	}
	fn info<S: ToString>(&self, info: S) -> Error {
		Error::new(
			self.loc().clone(),
			info.to_string(),
			ErrorKind::Info,
		)
	}
}
