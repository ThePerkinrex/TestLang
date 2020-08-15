use crate::span::{File as SpanFile, Span, SpanError};
use crate::span::location::{FilePosition, Location};
use crate::file_provider::{FileProvider, FileReader};
use crate::error::{ReturnValue, Error};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
	Number(f64),
	String(String),
	Ident(String),
	EOL,
	Colon,
	Comma,
	Semicolon,
	Kwd(String),
}

enum State {
	Normal,
	Number(String, FilePosition),
	String(String, bool, FilePosition),
	Ident(String, FilePosition),
}

fn line_col((line, line_i): (usize, usize), i: &usize) -> (usize, usize) {
	(line, i - line_i + 1)
}

fn token<F, S, E>(token: Token, file: F, start: S, end: E) -> Span<Token>
where
	F: Into<SpanFile>,
	S: Into<FilePosition>,
	E: Into<FilePosition>,
{
	Span::new(token, Location::new(file, start, end))
}

pub fn tokenize<FProv: FileProvider<File>, File: FileReader>(file: &str, fprov: &FProv) -> Result<Vec<Span<Token>>, Error> {
	let f = fprov.get_file(&file);
	let s = f.contents();
	let mut tokens = Vec::new();
	let mut state = State::Normal;
	let mut chars = s.chars().enumerate().peekable();
	let mut kwd = String::new();
	let mut kwd_start = None;
	let mut line = (1, 0);
	while let Some((i, peeked_char)) = chars.peek() {
		if matches!(peeked_char, ' ' | '\t') && !matches!(state, State::String(_, _, _)) {
			if let State::Ident(name, start) = &state {
				tokens.push(token(
					Token::Ident(name.clone()),
					file,
					*start,
					line_col(line, i),
				));
				state = State::Normal;
			} else if matches!(state, State::Normal) {
				if !kwd.is_empty() {
					tokens.push(token(Token::Kwd(kwd), file, kwd_start.expect("Kwd start not defined"), line_col(line, i)));
					kwd = String::new();
					kwd_start = None;
				}
			}
			chars.next();
			continue;
		}
		//print!("{}", peeked_char);
		match &mut state {
			State::Normal => {
				// println!("NORMAL: {}", peeked_char);
				if peeked_char.is_ascii_digit() {
					if !kwd.is_empty() {
						tokens.push(token(Token::Kwd(kwd), file, kwd_start.expect("Kwd start not defined"), line_col(line, i)));
						kwd = String::new();
						kwd_start = None;
					}
					state = State::Number(String::new(), line_col(line, i).into());
					continue;
				}
				// if peeked_char == &'(' {
				// 	state = State::Parens(String::new(), 0);
				// 	chars.next();
				// 	continue;
				// }
				if peeked_char == &'"' {
					if !kwd.is_empty() {
						tokens.push(token(Token::Kwd(kwd), file, kwd_start.expect("Kwd start not defined"), line_col(line, i)));
						kwd = String::new();
						kwd_start = None;
					}
					state = State::String(String::new(), false, line_col(line, i).into());
					chars.next();
					continue;
				}
				// if let Ok(op) = Operator::try_from(peeked_char) {
				// 	tokens.push(Token::BinaryOp(op));
				// 	chars.next();
				// 	continue;
				// }
				if peeked_char == &'\n' {
					if !kwd.is_empty() {
						tokens.push(token(Token::Kwd(kwd), file, kwd_start.expect("Kwd start not defined"), line_col(line, i)));
						kwd = String::new();
						kwd_start = None;
					}
					tokens.push(token(Token::EOL, file, line_col(line, i), line_col(line, i)));
					line = (line.0 + 1, *i + 1);
					chars.next();
					continue;
				}
				if peeked_char == &';'{
					if !kwd.is_empty() {
						tokens.push(token(Token::Kwd(kwd), file, kwd_start.expect("Kwd start not defined"), line_col(line, i)));
						kwd = String::new();
						kwd_start = None;
					}
					tokens.push(token(Token::Colon, file, line_col(line, i), line_col(line, i)));
					chars.next();
					continue;
				}
				if peeked_char == &',' {
					if !kwd.is_empty() {
						tokens.push(token(Token::Kwd(kwd), file, kwd_start.expect("Kwd start not defined"), line_col(line, i)));
						kwd = String::new();
						kwd_start = None;
					}
					tokens.push(token(Token::Comma, file, line_col(line, i), line_col(line, i)));
					chars.next();
					continue;
				}
				if peeked_char == &':' {
					if !kwd.is_empty() {
						tokens.push(token(Token::Kwd(kwd), file, kwd_start.expect("Kwd start not defined"), line_col(line, i)));
						kwd = String::new();
						kwd_start = None;
					}
					tokens.push(token(Token::Semicolon, file, line_col(line, i), line_col(line, i)));
					chars.next();
					continue;
				}
				//tokens.push(Token::Other(*peeked_char));
				if peeked_char.is_alphanumeric() {
					if !kwd.is_empty() {
						tokens.push(token(Token::Kwd(kwd), file, kwd_start.expect("Kwd start not defined"), line_col(line, i)));
						kwd = String::new();
						kwd_start = None;
					}
					// println!("IDENT START:({:?}:{}) -> {:?}", line, i, line_col(line, i));
					state = State::Ident(String::new(), line_col(line, i).into());
					continue;
				}
				match *peeked_char {
					'(' | ')' => {
						if !kwd.is_empty() {
							tokens.push(token(Token::Kwd(kwd), file, kwd_start.expect("Kwd start not defined"), line_col(line, i)));
							kwd = String::new();
							kwd_start = None;
						}
						let mut k = String::new();
						k.push(*peeked_char);
						tokens.push(token(Token::Kwd(k), file, line_col(line, i), line_col(line, i)))
					}
					_ => {
						kwd.push(*peeked_char);
						if kwd_start.is_none() {
							kwd_start = Some(line_col(line, i))
						}
					}
				}
				
				chars.next();
			}
			State::Ident(id, start) => {
				if peeked_char.is_alphabetic() || peeked_char == & '_' {
					id.push(*peeked_char);
					chars.next();
				} else {
					// println!("Ident end ({})", peeked_char);
					tokens.push(token(Token::Ident(id.clone()), file, *start, line_col(line, &(i-1))));
					state = State::Normal;
				}
			}
			State::Number(num, start) => {
				if peeked_char.is_ascii_digit() || (peeked_char == &'.' && !num.contains('.')) {
					num.push(*peeked_char);
					chars.next();
				} else {
					if let Ok(n) = num.parse() {
						tokens.push(token(Token::Number(n), file, *start, line_col(line, &(*i-1))))
					} else {
						let err_span = Span::new(num.clone(), Location::new(file, *start, line_col(line, i)));
						return Err(err_span.error("Number wrongly formatted", ReturnValue::WrongNumberFormat));
					}
					state = State::Normal;
				}
			}
			State::String(s, escaping, start) => {
				if *escaping {
					*escaping = false;
					if let Some(c) = get_escape_char(peeked_char) {
						s.pop();
						s.push(c);
						chars.next();
						continue;
					} else {
						s.push(*peeked_char);
						chars.next();
						continue;
					}
				}
				if peeked_char == &'\\' {
					s.push(*peeked_char);
					chars.next();
					*escaping = true;
					continue;
				}
				if peeked_char == &'"' {
					tokens.push(token(Token::String(s.clone()), file, *start, line_col(line, i)));
					state = State::Normal;
					chars.next();
					continue;
				}
				s.push(*peeked_char);
				chars.next();
			}
		};
	}
	if let State::Number(num, start) = state {
		if let Ok(n) = num.parse() {
			tokens.push(token(Token::Number(n), file, start, line_col(line, &(s.len()-1))))
		} else {
			let err_span = Span::new(num, Location::new(file, start, line_col(line, &(s.len()-1))));
			return Err(err_span.error("Number wrongly formatted", ReturnValue::WrongNumberFormat));
			//panic!("Error parsing number")
		}
	}else if let State::Ident(name, start) = state {
		tokens.push(token(Token::Ident(name), file, start, line_col(line, &(s.len()-1))))
	} else if let State::String(v, _escaping, start) = state {
		let err_span = Span::new(v, Location::new(file, start, line_col(line, &(s.len()-1))));
		return Err(err_span.error("Unclosed string", ReturnValue::UnclosedString));
	}else{
		if let Some(start) = kwd_start {
			tokens.push(token(Token::Kwd(kwd), file, start, line_col(line, &(s.len()-1))))
		}
	}
	//println!("");
	Ok(tokens)
}

fn get_escape_char(escaped: &char) -> Option<char> {
	match escaped {
		'n' => Some('\n'),
		'\\' => Some('\\'),
		'"' => Some('"'),
		_ => None,
	}
}
