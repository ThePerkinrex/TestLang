use std::path::{PathBuf, Path};
use std::io::Read;
use std::fs::File as OsFile;

pub trait FileReader {
	fn contents(&self) -> &str;
	fn line(&self, line: usize) -> Option<&str>;
	fn name(&self) -> Option<&Path>;
}

pub trait FileProvider<File: FileReader> {
	fn get_file<P: AsRef<Path>>(&self, p: &P) -> File;
}


pub mod fs {
	use super::*;
	use super::FileProvider as FileProviderTrait;

	pub struct File {
		name: PathBuf,
		contents: String
	}

	impl File {
		fn new(p: PathBuf) -> Self {
			let mut s = String::new();
			let mut f = OsFile::open(&p).expect(&format!("Error opening file {}", p.display()));
			f.read_to_string(&mut s).expect(&format!("Error reading file {}", p.display()));
			Self {
				name: p,
				contents: s
			}
		}
	}

	impl FileReader for File {
		fn contents(&self) -> &str {
			&self.contents
		}
	
		fn line(&self, line: usize) -> Option<&str> {
			// println!("Querying {}:{}",self.name.display(), line);
			self.contents.split('\n').nth(line-1).map(|line| line.trim_end())
		}

		fn name(&self) -> Option<&Path> {
			Some(&self.name)
		}
	}

	pub struct FileProvider{
		base_path: PathBuf
	}

	impl FileProvider{
		pub fn new<P: AsRef<Path>>(path: &P) -> Self {
			Self {
				base_path: path.as_ref().into()
			}
		}
	}

	impl FileProviderTrait<File> for FileProvider {
		fn get_file<P: AsRef<Path>>(&self, p: &P) -> File {
			File::new(self.base_path.join(p))
		}
	}
}

pub mod repl {
	use super::*;
	use super::FileProvider as FileProviderTrait;

	pub struct File {
		contents: String
	}

	impl File {
		fn new(contents: String) -> Self {
			Self {contents}
		}
	}

	impl FileReader for File {
		fn contents(&self) -> &str {
			&self.contents
		}
	
		fn line(&self, line: usize) -> Option<&str> {
			self.contents.split('\n').nth(line-1).map(|line| line.trim_end())
		}

		fn name(&self) -> Option<&Path> {
			None
		}
	}

	pub struct FileProvider{
		file: String
	}

	impl FileProvider{
		pub fn new() -> Self {
			Self {
				file: String::new()
			}
		}

		pub fn new_line(&mut self, line: String) {
			self.file = line;
		}
	}

	impl FileProviderTrait<File> for FileProvider {
		fn get_file<P: AsRef<Path>>(&self, _: &P) -> File {
			File::new(self.file.clone())
		}
	}
}

