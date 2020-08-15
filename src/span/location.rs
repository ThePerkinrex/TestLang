use super::File;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct FilePosition {
	pub line: usize,
	pub col: usize,
}

impl std::fmt::Display for FilePosition {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		write!(f, "{}:{}", self.line, self.col)
	}
}

impl From<(usize, usize)> for FilePosition {
	fn from((line, col): (usize, usize)) -> Self {
		Self { line, col }
	}
}

#[derive(Clone, PartialEq, Eq)]
pub struct Location {
	file: File,
	pub start: FilePosition,
	pub end: FilePosition,
}

impl PartialOrd for Location {
	fn partial_cmp(&self, rhs: &Self) -> std::option::Option<std::cmp::Ordering> {
		if self.file != rhs.file {
			None
		} else {
			if self.start < rhs.start && self.end < rhs.end {
				return Some(std::cmp::Ordering::Less);
			} else if self.start > rhs.start && self.end > rhs.end {
				return Some(std::cmp::Ordering::Greater);
			} else {
				return None;
			}
		}
	}
}

impl Location {
	pub fn new<F, S, E>(file: F, start: S, end: E) -> Self
	where
		F: Into<File>,
		S: Into<FilePosition>,
		E: Into<FilePosition>,
	{
		let s = start.into();
		let e = end.into();
		assert!(
			s <= e,
			format!(
				"[{}] Location start ({}) isn't smaller than location end ({})",
				file.into().display(),
				s,
				e
			)
		);
		Self {
			file: file.into(),
			start: s,
			end: e,
		}
	}

	pub fn file(&self) -> &File {
		&self.file
	}
}

impl std::fmt::Debug for Location {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		write!(f, "{}:{{{}-{}}}", self.file.display(), self.start, self.end)
	}
}

impl std::fmt::Display for Location {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		write!(f, "{}:{}", self.file.display(), self.start)
	}
}