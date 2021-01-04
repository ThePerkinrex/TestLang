use std::path::PathBuf;

use crate::{ast::Item, options::CodegenOptions, scope::TypeDB, span::Span};

pub struct Codegen;

impl super::Backend for Codegen {
    fn gen_code(&self, items: &[Span<Item>], type_db: &mut TypeDB, options: CodegenOptions) {
        
    }
}