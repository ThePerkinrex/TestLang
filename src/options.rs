use std::path::PathBuf;

use super::backend;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "testlang")]
pub struct Options {
    /// Path to the entry point for compilation or execution
    pub input_file: String,
    /// Is it a library (has no main function)
    #[structopt(short, long)]
    pub lib: bool,
    /// Output file path
    #[structopt(short, required_ifs(&Backend::out_required(&["interpret"], &[])))]
    out: Option<PathBuf>,
    /// Backend to use, interpret doesn't use the out file, as it doesn't perform any kind of codegen
    #[structopt(name = "backend", short, long, default_value, possible_values(&Backend::variants()))]
    backend: Backend,
}

impl Options {
	pub fn into_codegen_options(self) -> (CodegenOptions, Backend) {
		(CodegenOptions {
			lib: self.lib,
			out: self.out
		}, self.backend)
	}
}

pub struct CodegenOptions {
	pub lib: bool,
    pub out: Option<PathBuf>,
}

use structopt::clap::arg_enum;

arg_enum! {
    #[derive(PartialEq, Debug)]
    #[allow(non_camel_case_types)]
    pub enum Backend {
        interpret,
        js
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::interpret
    }
}

impl Backend {
    pub fn get_codegen(&self) -> Box<dyn backend::Backend> {
        match self {
            Self::interpret => Box::new(backend::interpreter::Codegen),
            Self::js => Box::new(backend::js::Codegen),
        }
	}

	fn out_required<'a>(ignored: &[&'a str], other: &[(&'a str, &'a str)]) -> Vec<(&'a str, &'a str)> {
		let mut r: Vec<(&str, &str)> = Self::variants().to_vec().into_iter().filter(|x| !ignored.contains(x)).map(|x| ("backend", x)).collect();
		r.extend_from_slice(other);
		r
	}
}
