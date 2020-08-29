use crate::color::{color, colorln};
use crate::file_provider::{FileProvider, FileReader};
use crate::span::location::Location;
use crate::span::Span;

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

#[repr(i32)]
pub enum ReturnValue {
	#[allow(unused)]
	Ok,
	WrongNumberFormat,
	UnclosedString,
	UnclosedParens,
	UnclosedBracket,
	UnexpectedNonValue,
	UnexpectedNonIdentifier,
	// UnexpectedType,
	// ExpectedType,
	// ExpectedReturnKwd,
	NoMain,
	MainHasArguments,
	MainNonVoidRetType,
	NameDefined,
	TypesDontMatch,
	TraitNotImplemented,
	// BrnchRetTypesDontMatch,
	IdentNotDefined,
	IntrinsicNotDefined,
	ExpectedSemicolon,
	UnexpectedToken,
}

impl Into<i32> for ReturnValue {
	fn into(self) -> i32 {
		self as i32
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorKind {
	Error(i32),
	Warning,
	Info,
}

impl std::fmt::Display for ErrorKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		match self {
			Self::Error(errno) => write!(f, "error[{}]", errno),
			Self::Warning => write!(f, "warning"),
			Self::Info => write!(f, "info"),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
	loc: Location,
	info: String,
	kind: ErrorKind,
	notes: Vec<String>,
}

impl Error {
	pub fn new(loc: Location, info: String, kind: ErrorKind) -> Self{
		Self{
			loc,
			info,
			kind,
			notes: Vec::new(),
		}
	}

	// pub fn note<T: ToString>(mut self, n: T) -> Self {
	// 	self.notes.push(n.to_string());
	// 	self
	// }

	pub fn span<T: Clone>(self, v: T) -> Span<T> {
		Span::new(v, self.loc)
	}

	pub fn default_display<FProv: FileProvider<T>, T: FileReader>(
		&self,
		file_provider: &FProv,
		exit: bool,
	) {
		let buffer_writer = BufferWriter::stdout(ColorChoice::Always);
		self.display(file_provider, &buffer_writer);
		if exit {
			if let ErrorKind::Error(r) = self.kind {
				std::process::exit(r);
			}
		}
	}

	pub fn display<FProv: FileProvider<T>, T: FileReader>(
		&self,
		file_provider: &FProv,
		buffer_writer: &BufferWriter,
	) {
		let notes = self.notes.clone();
		let file = file_provider.get_file(self.loc.file());
		//let mut r = String::new();
		let mut base_color = ColorSpec::new();
		base_color.set_bold(true);
		base_color.set_fg(Some(match self.kind {
			ErrorKind::Info => Color::Blue,
			ErrorKind::Warning => Color::Yellow,
			ErrorKind::Error(_) => Color::Red,
		}));
		let mut line_color = ColorSpec::new();
		line_color.set_bold(true);
		line_color.set_fg(Some(Color::Cyan));
		let mut bold = ColorSpec::new();
		bold.set_bold(true);
		bold.set_fg(Some(Color::White));
		let mut default = ColorSpec::new();
		default.set_bold(false);
		default.set_fg(Some(Color::White));
		let mut buffer = buffer_writer.buffer();
		buffer
			.set_color(&mut default)
			.expect("Error setting buffer color");
		color(&mut buffer, self.kind.to_string(), &base_color).expect("Error setting color");
		colorln(&mut buffer, format!(": {}", self.info), &bold).expect("Error setting color");
		//writeln!(buffer, "{}: {}", self.kind, self.info);
		color(&mut buffer, format!("{:>5}--> ", ""), &line_color).expect("Error setting color");
		colorln(&mut buffer, &self.loc.to_string(), &default).expect("Error setting color");
		//writeln!(buffer, "{:>5}--> {}", "", self.loc);
		colorln(&mut buffer, format!("{:>5} |", ""), &line_color).expect("Error setting color");
		
		if self.loc.start.line == self.loc.end.line {
			let col_diff = self.loc.end.col - self.loc.start.col;
			let mut line: String = file.line(self.loc.start.line).unwrap().into();
			let line_num = format!("{:>5}", self.loc.start.line);
			let empty_num = format!("{:1$}", "", line_num.len());
			let mut space = format!("{:1$}", "", self.loc.start.col - 1);
			for (i, c) in &mut line.clone().chars().enumerate() {
				if i >= space.len() {
					break;
				}
				if c == '\t' {
					line.replace_range(i..i + 1, "    ");
					space.replace_range(i..i + 1, "    ");
				}
			}
			color(&mut buffer, format!("{} | ", line_num), &line_color)
				.expect("Error setting color");
			colorln(&mut buffer, line, &default).expect("Error setting color");
			//writeln!(buffer, "{} | {}", line_num, line);

			color(&mut buffer, format!("{} | ", empty_num), &line_color)
				.expect("Error setting color");
			colorln(
				&mut buffer,
				format!("{}{:^<2$}", space, "", col_diff + 1),
				&base_color,
			)
			.expect("Error setting color");
		} else {
			{
				// first line
				let mut line: String = file.line(self.loc.start.line).unwrap().into();
				let line_num = format!("{:>5}", self.loc.start.line);
				let mut space = format!("{:_<1$}", "", self.loc.start.col - 1);
				for (i, c) in &mut line.clone().chars().enumerate() {
					if c == '\t' {
						line.replace_range(i..i + 1, "    ");
						if i < space.len() {
							space.replace_range(i..i + 1, "____");
						}
					}
				}

				color(&mut buffer, format!("{} |   ", line_num), &line_color)
					.expect("Error setting color");
				colorln(&mut buffer, line, &default).expect("Error setting color");
				color(&mut buffer, format!("{:>5} | ", ""), &line_color)
					.expect("Error setting color");
				colorln(&mut buffer, format!(" _{}^", space), &base_color)
					.expect("Error setting color");
			}
			for line_n in self.loc.start.line + 1..=self.loc.end.line {
				let mut line: String = file.line(line_n).unwrap().into();
				let line_num = format!("{:>5}", line_n);
				for (i, c) in &mut line.clone().chars().enumerate() {
					if c == '\t' {
						line.replace_range(i..i + 1, "    ");
					}
				}
				color(&mut buffer, format!("{} | ", line_num), &line_color)
					.expect("Error setting color");
				color(&mut buffer, "| ", &base_color).expect("Error setting color");
				colorln(&mut buffer, line, &default).expect("Error setting color");
			}
			{
				// last line
				let line: String = file.line(self.loc.end.line).unwrap().into();
				let mut space = format!("{:_<1$}", "", self.loc.end.col - 1);
				for (i, c) in &mut line.clone().chars().enumerate() {
					if i >= space.len() {
						break;
					}
					if c == '\t' {
						space.replace_range(i..i + 1, "____");
					}
				}

				color(&mut buffer, format!("{:>5} | ", ""), &line_color)
					.expect("Error setting color");
				colorln(&mut buffer, format!("|_{}^", space), &base_color)
					.expect("Error setting color");
			}
			colorln(&mut buffer, format!("{:>5} |", ""), &line_color).expect("Error setting color");
		}
		for n in &notes {
			color(&mut buffer, format!("{:>5} = ", ""), &line_color).expect("Error setting color");
			color(&mut buffer, "note: ", &bold).expect("Error setting color");
			colorln(&mut buffer, n, &default).expect("Error setting color");
		}
		

		buffer.reset().expect("Error resetting color");
		buffer_writer
			.print(&buffer)
			.expect("Error printing the buffer");
	}
}