use std::path::PathBuf;

use crate::{ast::Item, scope::TypeDB, span::Span, options::CodegenOptions};

pub mod js;
pub mod interpreter;

pub trait Backend {
	fn gen_code(&self, items: &[Span<Item>], type_db: &mut TypeDB, options: CodegenOptions);
}