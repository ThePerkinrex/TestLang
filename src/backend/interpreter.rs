use crate::{ast::Item, interpreter, options::CodegenOptions, scope::TypeDB, span::Span, error};

pub struct Codegen;

impl super::Backend for Codegen {
    fn gen_code(&self, items: &[Span<Item>], type_db: &mut TypeDB, opts: CodegenOptions) {
        if opts.lib {
            error::non_located_error_default("Can't interpret a library, only code with a main() function is interpretable", error::ErrorKind::Error(error::ReturnValue::NoMain.into()))
        }else{
            interpreter::interpret_items(items, type_db)
        }
    }
}