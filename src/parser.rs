use std::convert::TryFrom;

use crate::ast;
use crate::error::{Error, ReturnValue};
use crate::operators::Operator;
use crate::span::{HasLoc, Span, SpanError};
use crate::tokens::Token;

fn parse_inner(
	i: &mut usize,
	tokens: &[Span<Token>],
	start: &str,
	end: &str,
) -> Result<Vec<Span<Token>>, Error> {
	let mut count = 0;
	let mut inner_tokens = Vec::new();
	loop {
		if *i >= tokens.len() {
			let mut l = tokens.last().unwrap().loc().clone();
			l.end.col += 1;
			l.start = l.end;
			let span = Span::new((), l);
			return Err(span.error(&format!("Unexpected EOI, expected `{}`", end), ReturnValue::UnclosedParens));
			//panic!("Unexpected EOI")
		}
		// println!("{:?}", tokens[index + offset]);
		if matches!(tokens[*i].clone().val(), Token::Kwd(x) if x == start) {
			count += 1;
		} else if matches!(tokens[*i].clone().val(), Token::Kwd(x) if x == end) {
			if count == 0 {
				*i += 1;
				break;
			}
			count -= 1;
		}
		inner_tokens.push(tokens[*i].clone());
		*i += 1;
	}
	Ok(inner_tokens)
}

pub fn parse_lines(
	tokens: Vec<Span<Token>>,
	intrinsics: bool,
) -> Result<Vec<Span<ast::Item>>, Error> {
	let mut i = 0;
	let mut o = Vec::new();
	while let Some(tok) = tokens.get(i) {
		//println!("{:?}", tok.as_ref());
		if matches!(tok.val(), Token::EOL) {
			i += 1;
		} else if matches!(tok.val(), Token::Ident(x) if x == String::from("impl")) {
			//println!("Matching impl");
			i += 1;
			let ident = if let Some(t) = tokens.get(i) {
				if let Token::Ident(id) = t.val() {
					i += 1;
					t.clone().map(id)
				} else {
					return Err(t.error("Expected identfier", ReturnValue::UnexpectedNonIdentifier));
				}
			} else {
				let mut l = tokens.last().unwrap().loc().clone();
				l.end.col += 1;
				l.start = l.end;
				let span = Span::new((), l);
				return Err(span.error(
					"Unexpected EOI, expected identifier (trait name)",
					ReturnValue::UnclosedParens,
				));
			};
			let defining_types: Vec<Span<ast::Type>> = if let Some(t) = tokens.get(i) {
				match t.val() {
					Token::Kwd(x) if x == "<" => {
						// if true {
						i += 1;
						let inner_tokens = parse_inner(&mut i, &tokens, "<", ">")?;
						let mut v = vec![];
						for n in inner_tokens
							.split(|x| matches!(x.as_ref(), Token::Comma))
						{
							//dbg!(&n);
							let (nf, offset) = parse_type(&n, 0)?;
							//dbg!(offset, n.len());
							if n.len() == offset {
								v.push(nf)
							} else {
								let mut l = nf.loc().clone();
								l.end.col += 1;
								l.start = l.end;
								let span = Span::new((), l);
								return Err(span.error(
									"Unexpected tokens, expected only a type",
									ReturnValue::UnexpectedNonIdentifier,
								));
							}
							
							// println!("{}", n.iter().map(|x|format!("{:?},", x)).fold(String::new(), |x, y| format!("{}{}", x, y)));
						}
						v
					}
					_ => vec![],
				}
			} else {
				let mut l = tokens.last().unwrap().loc().clone();
				l.end.col += 1;
				l.start = l.end;
				let span = Span::new((), l);
				return Err(span.error(
					"Unexpected EOI, expected `<` or `{`",
					ReturnValue::UnclosedParens,
				));
			};
			//println!("Name: {}", ident);
			//println!("Defining types: {:?}", defining_types);

		} else if matches!(tok.val(), Token::Ident(x) if x == String::from("trait")) {
			//println!("Matching trait");
			i += 1;
			let ident = if let Some(t) = tokens.get(i) {
				if let Token::Ident(id) = t.val() {
					i += 1;
					t.clone().map(id)
				} else {
					return Err(t.error("Expected identfier", ReturnValue::UnexpectedNonIdentifier));
				}
			} else {
				let mut l = tokens.last().unwrap().loc().clone();
				l.end.col += 1;
				l.start = l.end;
				let span = Span::new((), l);
				return Err(span.error(
					"Unexpected EOI, expected identifier (trait name)",
					ReturnValue::UnclosedParens,
				));
			};
			let defining_types: Vec<Span<ast::Ident>> = if let Some(t) = tokens.get(i) {
				match t.val() {
					Token::Kwd(x) if x == "<" => {
						// if true {
						i += 1;
						let inner_tokens = parse_inner(&mut i, &tokens, "<", ">")?;
						let mut v = vec![];
						for (i, n) in inner_tokens
							.split(|x| matches!(x.as_ref(), Token::Comma))
							.enumerate()
						{
							if let Some(nf) = n.get(0) {
								if n.len() == 1 {
									if let Token::Ident(id) = nf.as_ref() {
										v.push(nf.clone().map(id.clone()))
									} else {
										return Err(nf.error(
											"Expected identifier",
											ReturnValue::UnexpectedNonIdentifier,
										));
									}
								} else {
									let mut l = nf.loc().clone();
									l.end.col += 1;
									l.start = l.end;
									let span = Span::new((), l);
									return Err(span.error(
										"Unexpected tokens, expected only an identifier",
										ReturnValue::UnexpectedNonIdentifier,
									));
								}
							} else {
								return Err(inner_tokens[i * 2 + 1].error(
									"Expected identifier",
									ReturnValue::UnexpectedNonIdentifier,
								));
							}
							// println!("{}", n.iter().map(|x|format!("{:?},", x)).fold(String::new(), |x, y| format!("{}{}", x, y)));
						}
						v
					}
					_ => vec![],
				}
			} else {
				let mut l = tokens.last().unwrap().loc().clone();
				l.end.col += 1;
				l.start = l.end;
				let span = Span::new((), l);
				return Err(span.error(
					"Unexpected EOI, expected `<` or `{`",
					ReturnValue::UnclosedParens,
				));
			};
			//println!("Trait {}{:?}", ident, defining_types);
			if let Some(t) = tokens.get(i) {
				if matches!(t.as_ref(), Token::Kwd(x) if x == "{") {
					// if true {
					i += 1;
					
					let inner_tokens = parse_inner(&mut i, &tokens, "{", "}")?;
					
					let mut idx = 0;
					let mut type_defs = Vec::new();
					use std::collections::HashMap;
					let mut methods = HashMap::new();
					while idx < inner_tokens.len() {
						//println!("[{}] => {:?}", idx, inner_tokens[idx]);
						if let Token::EOL = inner_tokens[idx].as_ref() {
							idx += 1;
							continue;
						} else if matches!(inner_tokens[idx].as_ref(), Token::Ident(x) if x == "type")
						{
							idx += 1;
							type_defs.push(if let Some(tok) = inner_tokens.get(idx) {
								if let Token::Ident(id) = tok.val() {
									idx += 1;
									if let Some(tok) = inner_tokens.get(idx) {
										if let Token::Colon = tok.as_ref() {
											//println!("TYPEDEF: {}", id);
											id
										} else {
											return Err(tok.error(
												"Expected `;`",
												ReturnValue::ExpectedSemicolon,
											));
										}
									} else {
										let mut l = inner_tokens.last().unwrap().loc().clone();
										l.end.col += 1;
										l.start = l.end;
										let span = Span::new((), l);
										return Err(span.error(
											"Unexpected EOI, expected `;`",
											ReturnValue::UnclosedParens,
										));
									}
								} else {
									return Err(tok.error(
										"Expected identifier",
										ReturnValue::UnexpectedNonIdentifier,
									));
								}
							} else {
								let mut l = inner_tokens.last().unwrap().loc().clone();
								l.end.col += 1;
								l.start = l.end;
								let span = Span::new((), l);
								return Err(span.error(
									"Unexpected EOI, expected identifier",
									ReturnValue::UnclosedParens,
								));
							});
						} else if matches!(inner_tokens[idx].as_ref(), Token::Ident(x) if x == "fn")
						{
							idx += 1;
							//println!("METHOD {} => {:?}", idx, inner_tokens[idx]);
							let ident = if let Some(t) = inner_tokens.get(idx) {
								if let Token::Ident(id) = t.val() {
									idx += 1;
									t.clone().map(id)
								} else {
									return Err(t.error(
										"Expected identfier",
										ReturnValue::UnexpectedNonIdentifier,
									));
								}
							} else {
								let mut l = inner_tokens.last().unwrap().loc().clone();
								l.end.col += 1;
								l.start = l.end;
								let span = Span::new((), l);
								return Err(span.error(
									"Unexpected EOI, expected identifier (fn signature name)",
									ReturnValue::UnclosedParens,
								));
							};
							//println!("METHOD: {}", ident);
							if methods.contains_key(ident.as_ref()) {
								return Err(
									ident.error("Name already defined", ReturnValue::NameDefined)
								);
							}
							let args = if let Some(t) = inner_tokens.get(idx) {
								if matches!(t.val(), Token::Kwd(x) if x == "(") {
									// if true {
									idx += 1;
									let inner_inner_tokens = parse_inner(&mut idx, &inner_tokens, "(", ")")?;
									let mut res = Vec::new();
									for arg in inner_inner_tokens
										.split(|tok| matches!(tok.val(), Token::Comma))
										.filter(|x| !x.is_empty())
									{
										// println!("ARG: {:?}", arg);
										let name = if let Some(t) = arg.get(0) {
											if let Token::Ident(id) = t.val() {
												t.clone().map(id)
											} else {
												return Err(t.error(
													"Expected identfier",
													ReturnValue::UnexpectedNonIdentifier,
												));
											}
										} else {
											unreachable!(); // No empty arrays
										};
										let typ = if name.as_ref() == "self" {
											name.clone().map(ast::TypeData::SelfRef)
										} else {
											if let Some(t) = arg.get(1) {
												if let Token::Semicolon = t.val() {
												} else {
													return Err(t.error(
														"Expected semicolon",
														ReturnValue::UnexpectedNonIdentifier,
													));
												}
											} else {
												let mut l = arg.last().unwrap().loc().clone();
												l.end.col += 1;
												l.start = l.end;
												let span = Span::new((), l);
												return Err(span.error(
													"Unexpected EOI, expected semicolon",
													ReturnValue::UnclosedParens,
												));
											};
											match parse_type(&arg, 2) {
												Ok((v, _)) => {
													v.clone().map(v.as_ref().type_data().clone())
												}
												Err(e) => return Err(e),
											}
										};
										res.push((name, typ))
									}
									res
								} else {
									return Err(t.error(
										"Expected `(`",
										ReturnValue::UnexpectedNonIdentifier,
									));
								}
							} else {
								let mut l = inner_tokens.last().unwrap().loc().clone();
								l.end.col += 1;
								l.start = l.end;
								let span = Span::new((), l);
								return Err(span.error(
									"Unexpected EOI, expected `(`",
									ReturnValue::UnclosedParens,
								));
							};
							//println!("ARGS: {:?}", args);
							let ret_type = if let Some(t) = inner_tokens.get(idx) {
								if matches!(t.val(), Token::Kwd(k) if k == "->") {
									idx += 1;
									let ty = match parse_type(&inner_tokens, idx) {
										Ok((v, off)) => {
											i += off;
											v
										}
										Err(e) => return Err(e),
									};
									ty.clone().map(ty.as_ref().type_data().clone())
								} else {
									t.clone().map(ast::TypeData::Void)
								}
							} else {
								let mut l = inner_tokens.last().unwrap().loc().clone();
								l.end.col += 1;
								l.start = l.end;
								let span = Span::new((), l);
								return Err(span.error(
									"Unexpected EOI, expected `->` or `;`",
									ReturnValue::UnclosedParens,
								));
							};
							idx += 1;
							if let Some(tok) = inner_tokens.get(idx) {
								if let Token::Colon = tok.as_ref() {
									let fn_sig = ast::FnSignature(args, Box::new(ret_type));
									methods.insert(ident.unwrap(), fn_sig);
								} else {
									return Err(
										tok.error("Expected `;`", ReturnValue::ExpectedSemicolon)
									);
								}
							} else {
								let mut l = inner_tokens.last().unwrap().loc().clone();
								l.end.col += 1;
								l.start = l.end;
								let span = Span::new((), l);
								return Err(span.error(
									"Unexpected EOI, expected `;`",
									ReturnValue::UnclosedParens,
								));
							}
						}

						idx += 1;
					}
					//println!("Methods: {:?}", methods);
					//println!("TYPEDEFS: {:?}", type_defs);
				}
			} else {
				let mut l = tokens.last().unwrap().loc().clone();
				l.end.col += 1;
				l.start = l.end;
				let span = Span::new((), l);
				return Err(span.error("Unexpected EOI, expected `{`", ReturnValue::UnclosedParens));
			}
		} else if matches!(tok.val(), Token::Ident(x) if x == String::from("fn")) {
			//println!("Matching fn");
			i += 1;
			let ident = if let Some(t) = tokens.get(i) {
				if let Token::Ident(id) = t.val() {
					i += 1;
					t.clone().map(id)
				} else {
					return Err(t.error("Expected identfier", ReturnValue::UnexpectedNonIdentifier));
				}
			} else {
				let mut l = tokens.last().unwrap().loc().clone();
				l.end.col += 1;
				l.start = l.end;
				let span = Span::new((), l);
				return Err(span.error(
					"Unexpected EOI, expected identifier (fn name)",
					ReturnValue::UnclosedParens,
				));
			};
			// println!("FN NAME: {}", ident);
			let args = if let Some(t) = tokens.get(i) {
				if matches!(t.val(), Token::Kwd(x) if x == "(") {
					// if true {
					i += 1;
					let inner_tokens = parse_inner(&mut i, &tokens, "(", ")")?;
					//println!("{:?}", tokens.get(i));
					let mut res = Vec::new();
					for arg in inner_tokens
						.split(|tok| matches!(tok.val(), Token::Comma))
						.filter(|x| !x.is_empty())
					{
						// println!("ARG: {:?}", arg);
						let name = if let Some(t) = arg.get(0) {
							if let Token::Ident(id) = t.val() {
								t.clone().map(id)
							} else {
								return Err(t.error(
									"Expected identfier",
									ReturnValue::UnexpectedNonIdentifier,
								));
							}
						} else {
							unreachable!(); // No empty arrays
						};
						if let Some(t) = arg.get(1) {
							if let Token::Semicolon = t.val() {
							} else {
								return Err(t.error(
									"Expected semicolon",
									ReturnValue::UnexpectedNonIdentifier,
								));
							}
						} else {
							let mut l = arg.last().unwrap().loc().clone();
							l.end.col += 1;
							l.start = l.end;
							let span = Span::new((), l);
							return Err(span.error(
								"Unexpected EOI, expected semicolon",
								ReturnValue::UnclosedParens,
							));
						};
						let typ = match parse_type(&arg, 2) {
							Ok((v, _)) => v.clone().map(v.as_ref().type_data().clone()),
							Err(e) => return Err(e),
						};
						res.push((name, typ))
					}
					res
				} else {
					return Err(t.error("Expected `(`", ReturnValue::UnexpectedNonIdentifier));
				}
			} else {
				let mut l = tokens.last().unwrap().loc().clone();
				l.end.col += 1;
				l.start = l.end;
				let span = Span::new((), l);
				return Err(span.error("Unexpected EOI, expected `(`", ReturnValue::UnclosedParens));
			};
			// println!("ARGS: {:?}", args);

			let ret_type = if let Some(t) = tokens.get(i) {
				if matches!(t.val(), Token::Kwd(k) if k == "->") {
					i += 1;
					let ty = match parse_type(&tokens, i) {
						Ok((v, off)) => {
							i += off;
							v
						}
						Err(e) => return Err(e),
					};
					ty.clone().map(ty.as_ref().type_data().clone())
				} else {
					t.clone().map(ast::TypeData::Void)
				}
			} else {
				let mut l = tokens.last().unwrap().loc().clone();
				l.end.col += 1;
				l.start = l.end;
				let span = Span::new((), l);
				return Err(span.error(
					"Unexpected EOI, expected `->` or `{`",
					ReturnValue::UnclosedParens,
				));
			};
			let body = match match value::parse_block(&tokens, i, intrinsics) {
				Ok(v) => v,
				Err(e) => return Err(e),
			} {
				Some((v, off)) => {
					i += off;
					v
				}
				None => {
					let mut l = tokens.last().unwrap().loc().clone();
					l.end.col += 1;
					l.start = l.end;
					let span = Span::new((), l);
					return Err(span.error(
						"Unexpected EOI, expected block",
						ReturnValue::UnclosedParens,
					));
				}
			};

			o.push(Span::join(
				&[ident.clone(), body.clone().map(String::new())],
				ast::Item::Fn(ident, args, ret_type, body),
			))
		} else {
			return Err(tok.error("Unexpected token", ReturnValue::UnexpectedToken));
		}
	}
	Ok(o)
}

fn parse_type(tokens: &[Span<Token>], index: usize) -> Result<(Span<ast::Type>, usize), Error> {
	if let Some(t) = tokens.get(index) {
		if let Token::Ident(i) = t.val() {
			let typ = match i.as_str() {
				"number" => ast::TypeData::Number.default_type(),
				"string" => ast::TypeData::String.default_type(),
				"void" => ast::TypeData::Void.default_type(),
				_ => ast::TypeData::Other(i).default_type(),
			};
			return Ok((t.clone().map(typ), 1));
		} else if matches!(t.val(), Token::Kwd(k) if k == "[") {
			let (s, offset) = match parse_type(tokens, index + 1) {
				Ok((r, offset)) => (r.clone().map(r.as_ref().type_data().clone()), offset),
				Err(e) => return Err(e),
			};
			if let Some(t_end) = tokens.get(index + offset + 1) {
				if matches!(t_end.val(), Token::Kwd(k) if k == "]") {
					return Ok((
						t.clone().join_with(
							&[t_end.clone()],
							ast::TypeData::Array(Box::new(s.val())).default_type(),
						),
						offset + 2,
					));
				} else {
					return Err(t_end.error("Expected `]`", ReturnValue::UnclosedBracket));
				}
			} else {
				let mut l = tokens.last().unwrap().loc().clone();
				l.end.col += 1;
				l.start = l.end;
				let span = Span::new((), l);
				return Err(span.error("Unexpected EOI, expected `]`", ReturnValue::UnclosedParens));
			};
		} else {
			println!("Expected type: {:?}", t);
			println!("Expected type (tokens): {:?}", &tokens[index..]);
			return Err(t.error("Expected type", ReturnValue::UnexpectedNonIdentifier));
		}
	} else {
		let mut l = tokens.last().unwrap().loc().clone();
		l.end.col += 1;
		l.start = l.end;
		let span = Span::new((), l);
		return Err(span.error("Unexpected EOI, expected type", ReturnValue::UnclosedParens));
	};
}

/// Parse an expression
pub fn parse_expr(
	tokens: &[Span<Token>],
	intrinsics: bool,
	last: usize,
) -> Result<Span<ast::Expr>, Error> {
	let len = tokens.len();
	match parse_expr_with_priority(&tokens, 0, 0, intrinsics) {
		Ok((v, offset)) => {
			//println!("{:?}", tokens);
			//println!("{:?}", v);
			assert_eq!(offset, len + last, "Didn't parse all the tokens");
			Ok(v)
		}
		Err(e) => Err(e),
	}
}

fn parse_expr_with_priority(
	tokens: &[Span<Token>],
	index: usize,
	priority: u8,
	intrinsics: bool,
) -> Result<(Span<ast::Expr>, usize), Error> {
	let mut offset = 0;
	let mut rhs = match next_value(&tokens, index, intrinsics) {
		Ok(v) => v,
		Err(e) => return Err(e),
	};
	offset += rhs.1;
	while let Some(curr_op) = next_op(&tokens, index + offset) {
		//dbg!(curr_op);
		if curr_op.0.priority() < priority {
			break;
		}
		offset += curr_op.1;
		//println!("Index: {} + {} ({:?})", index, offset, curr_op.0);
		let mut lhs = match next_value(&tokens, index + offset, intrinsics) {
			Ok(v) => v,
			Err(e) => return Err(e),
		};

		if let Some(next_op) = next_op(&tokens, index + offset) {
			//dbg!(next_op);
			if next_op.0.priority() > curr_op.0.priority() {
				lhs = match parse_expr_with_priority(
					&tokens,
					index + offset,
					next_op.0.priority(),
					intrinsics,
				) {
					Ok(v) => v,
					Err(e) => return Err(e),
				}
			}
		}
		offset += lhs.1;
		if offset == tokens.len() {
			rhs = (
				rhs.0.clone().join_with(
					&[lhs.0.clone()],
					ast::Expr::from_op(curr_op.0, rhs.0, lhs.0),
				),
				offset,
			);
			break;
		}
		if curr_op.0.priority() == priority {
			rhs = (
				rhs.0.clone().join_with(
					&[lhs.0.clone()],
					ast::Expr::from_op(curr_op.0, rhs.0, lhs.0),
				),
				offset,
			);
			continue;
		}
		if curr_op.0.priority() > priority {
			rhs = match parse_expr_with_priority(&tokens, index, curr_op.0.priority(), intrinsics) {
				Ok(v) => v,
				Err(e) => return Err(e),
			};
			continue;
		}
	}
	return Ok(rhs);
}

fn next_op(tokens: &[Span<Token>], index: usize) -> Option<(Operator, usize)> {
	//println!("Next op: {:?}", tokens.get(index..));
	// if index >= tokens.len() {
	// 	return None;
	// }
	let mut i = 0;
	while index + i < tokens.len() {
		if let Some(Token::Kwd(c)) = tokens.get(i + index).map(|x| x.val()) {
			if let Ok(op) = Operator::try_from(c.as_str()) {
				//println!("[{}] = {}", i + index, c);
				return Some((op, i + 1));
			}
		};
		i += 1;
	}
	return None;
}

pub use value::next_value;

mod value {
	use super::{ast, parse_expr_with_priority, Error, ReturnValue, Span, SpanError, Token};
	use crate::{span::HasLoc, RecursiveFn};

	enum State {
		LhsUnary,
		Central,
		RhsUnary,
	}

	pub fn next_value(
		tokens: &[Span<Token>],
		index: usize,
		intrinsics: bool,
	) -> Result<(Span<ast::Expr>, usize), Error> {
		//println!("VALUE: {:?}", tokens.get(index..));
		let mut offset = 0;
		let mut state = State::LhsUnary;
		let mut neg = RecursiveFn::new(|x: Span<ast::Expr>| {
			let mut loc = x.loc().clone();
			loc.start.col -= 1;
			Span::new(ast::Expr::Neg(Box::new(x)), loc)
		});
		let mut central = None;
		'main: loop {
			match state {
				State::LhsUnary => {
					while let Some(Token::Kwd(x)) = tokens.get(index + offset).map(|x| x.val()) {
						//matches!(tokens.get(index + offset), Some(Token::Other(x)) if x == &'-' || x == &'!') {

						match x.as_str() {
							"-" => neg.add(),
							_ => break,
						};
						offset += 1;
					}
					state = State::Central;
				}
				State::Central => {
					//println!("Central: {:?}", &tokens.get(index + offset..));
					if let Some((expr, off)) =
						match parse_block(&tokens, index + offset, intrinsics) {
							Ok(v) => v,
							Err(e) => return Err(e),
						} {
						central = Some(expr);
						offset += off;
					} else if matches!(tokens.get(index + offset).map(|x| x.val()), Some(Token::Kwd(x)) if x == "(")
					{
						// if true {
						offset += 1;
						let mut count = 0;
						let mut inner_tokens = Vec::new();
						loop {
							if index + offset >= tokens.len() {
								let mut l = tokens.last().unwrap().loc().clone();
								l.end.col += 1;
								l.start = l.end;
								let span = Span::new((), l);
								return Err(span.error(
									"Unexpected EOI, expected `)`",
									ReturnValue::UnclosedParens,
								));
								//panic!("Unexpected EOI")
							}
							if matches!(
								tokens.get(index + offset).map(|x| x.val()),
								Some(Token::EOL)
							) {
								offset += 1;
							}
							// println!("{:?}", tokens[index + offset]);
							if matches!(tokens[index + offset].clone().val(), Token::Kwd(x) if x == "(")
							{
								count += 1;
							} else if matches!(tokens[index + offset].clone().val(), Token::Kwd(x) if x == ")")
							{
								if count == 0 {
									break;
								}
								count -= 1;
							}
							inner_tokens.push(tokens[index + offset].clone());
							offset += 1;
						}
						central = Some(match super::parse_expr(&inner_tokens, intrinsics, 0) {
							Ok(v) => v,
							Err(e) => return Err(e),
						});
					} else {
						//println!("TOKENS: {:?}", tokens);
						central = Some(match tokens.get(index + offset) {
							Some(v) => match v.val() {
								Token::Number(n) => {
									v.clone().map(ast::Expr::Value(ast::Value::Num(n)))
								}
								Token::Ident(id) => {
									if intrinsics && id.starts_with("INTRINSIC_") {
										if let Some(i) = ast::intrinsics::Intrinsic::from_str(&id) {
											v.clone().map(ast::Expr::CompilerIntrinsic(i))
										} else {
											return Err(v.error(
												"Intrinsic not defined",
												ReturnValue::IntrinsicNotDefined,
											));
										}
									} else {
										v.clone().map(ast::Expr::Ident(id))
									}
								}
								Token::String(s) => {
									v.clone().map(ast::Expr::Value(ast::Value::Str(s)))
								}

								x => {
									return Err(v.error(
										format!("Expected value, not {:?}", x),
										ReturnValue::UnexpectedNonValue,
									))
								}
							},
							None => {
								let mut l = tokens.last().unwrap().loc().clone();
								l.end.col += 1;
								l.start = l.end;
								let span = Span::new((), l);
								return Err(span.error(
									"Unexpected EOI, expected value",
									ReturnValue::UnclosedParens,
								));
							}
						});
					}
					offset += 1;
					state = State::RhsUnary;
				}
				State::RhsUnary => {
					while matches!(tokens.get(index + offset).map(|x|x.val()), Some(Token::Kwd(x)) if x == "(")
					{
						offset += 1;
						let mut count = 0;
						let mut inner_tokens: Vec<Span<Token>> = Vec::new();
						loop {
							if index + offset >= tokens.len() {
								let mut l = tokens.last().unwrap().loc().clone();
								l.end.col += 1;
								l.start = l.end;
								let span = Span::new((), l);
								return Err(span.error(
									"Unexpected EOI, expected `)`",
									ReturnValue::UnclosedParens,
								));
							}
							if matches!(
								tokens.get(index + offset).map(|x| x.val()),
								Some(Token::EOL)
							) {
								offset += 1;
							}
							if matches!(tokens.get(index + offset).map(|x|x.val()), Some(Token::Kwd(x)) if x == "(")
							{
								count += 1;
							} else if matches!(tokens.get(index + offset).map(|x|x.val()), Some(Token::Kwd(x)) if x == ")")
							{
								if count == 0 {
									break;
								}
								count -= 1;
							}
							inner_tokens.push(tokens[index + offset].clone());
							offset += 1;
						}
						offset += 1;
						let i: Vec<Result<Span<ast::Expr>, Error>> = inner_tokens
							.split(|x| x.val() == Token::Comma)
							.filter(|x| !x.is_empty())
							.map(|x| super::parse_expr(x.into(), intrinsics, 0))
							.collect();
						let mut i_unwrapped = Vec::with_capacity(i.len());
						for x in i {
							match x {
								Ok(v) => i_unwrapped.push(v),
								Err(e) => return Err(e),
							}
						}
						central = central.map(|x| {
							x.clone().join_with(
								&i_unwrapped.clone(),
								ast::Expr::Call(Box::new(x), i_unwrapped),
							)
						})
					}
					break 'main;
				}
			}
			//offset += 1;
		}
		central = central.map(|x| neg.call(x));
		//println!("Result: {:?}", central);
		Ok((central.unwrap(), offset))
	}

	pub fn parse_block(
		tokens: &[Span<Token>],
		index: usize,
		intrinsics: bool,
	) -> Result<Option<(Span<ast::Expr>, usize)>, Error> {
		let mut offset = 0;
		if matches!(tokens.get(index + offset).map(|x| x.val()), Some(Token::Kwd(x)) if x == "{") {
			// println!("Parsing block: {:?}", &tokens[index..]);
			// if true {
			offset += 1;
			let mut idx = offset + index;
			let inner_tokens = super::parse_inner(&mut idx, &tokens, "{", "}")?;
			offset = idx - index;
			let mut body = Vec::new();
			let mut inner_offset = 0;
			while inner_offset < inner_tokens.len() {
				let next_colon = inner_tokens.iter().enumerate().position(|(i, x)| {
					// println!("[{}] = {:?}", i,x);
					i > inner_offset && x.val() == Token::Colon
				});
				let next_tokens = if let Some(pos) = next_colon {
					&inner_tokens[..pos]
				} else {
					&inner_tokens
				};

				// println!("-> {:?}", next_tokens);
				while let Some(Token::EOL) = next_tokens.get(inner_offset).map(|x| x.val()) {
					inner_offset += 1;
				}
				let (expr, i) =
					match parse_expr_with_priority(next_tokens, inner_offset, 0, intrinsics) {
						Ok(v) => v,
						Err(e) => return Err(e),
					};
				inner_offset += i;
				// println!("{:?}", inner_tokens.get(inner_offset));
				if matches!(
					inner_tokens.get(inner_offset).map(|x| x.val()),
					Some(Token::EOL) | None
				) {
					// println!("Pushing return {}", expr);
					body.push(expr.clone().map(ast::Expr::Return(Box::new(expr))));
					inner_offset += 1;
				} else if matches!(
					inner_tokens.get(inner_offset).map(|x| x.val()),
					Some(Token::Colon)
				) {
					body.push(expr);
					inner_offset += 1;
					if matches!(
						inner_tokens.get(inner_offset).map(|x| x.val()),
						Some(Token::EOL)
					) {
						inner_offset += 1;
						// println!("> {:?}", inner_tokens.get(inner_offset));
					}
				}
			}
			body.reverse();
			// println!("{:?}", body);
			return Ok(Some((
				Span::join(&tokens[..index + offset], ast::Expr::Block(body)),
				offset + 1,
			)));
		// end
		} else {
			return Ok(None);
		}
	}
}
