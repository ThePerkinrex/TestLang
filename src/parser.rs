use std::convert::TryFrom;

use crate::ast;
use crate::error::{Error, ReturnValue};
use crate::operators::Operator;
use crate::span::{HasLoc, Span, SpanError};
use crate::tokens::Token;

pub fn parse_lines(
	tokens: Vec<Span<Token>>,
	intrinsics: bool,
) -> Result<Vec<Span<ast::Item>>, Error> {
	let mut i = 0;
	let mut o = Vec::new();
	while let Some(tok) = tokens.get(i) {
		// println!("{:?}", tok.tok());
		if matches!(tok.val(), Token::EOL) {
			i += 1;
		}
		if matches!(tok.val(), Token::Ident(x) if x == String::from("fn")) {
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
			// println!("IDENT: {}", ident);
			let args = if let Some(t) = tokens.get(i) {
				if matches!(t.val(), Token::Kwd(x) if x == "(") {
					// if true {
					i += 1;
					let mut count = 0;
					let mut inner_tokens = Vec::new();
					loop {
						if i >= tokens.len() {
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
						// println!("{:?}", tokens[index + offset]);
						if matches!(tokens[i].clone().val(), Token::Kwd(x) if x == "(") {
							count += 1;
						} else if matches!(tokens[i].clone().val(), Token::Kwd(x) if x == ")") {
							if count == 0 {
								i += 1;
								break;
							}
							count -= 1;
						}
						inner_tokens.push(tokens[i].clone());
						i += 1;
					}
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
							Ok((v, _)) => v,
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
				i += 1;
				if matches!(t.val(), Token::Kwd(k) if k == "->") {
					match parse_type(&tokens, i) {
						Ok((v, off)) => {
							i += off;
							v
						}
						Err(e) => return Err(e),
					}
				} else {
					return Err(t.error("Expected `->`", ReturnValue::ExpectedReturnKwd));
				}
			} else {
				let mut l = tokens.last().unwrap().loc().clone();
				l.end.col += 1;
				l.start = l.end;
				let span = Span::new((), l);
				return Err(
					span.error("Unexpected EOI, expected `->`", ReturnValue::UnclosedParens)
				);
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
		}
	}
	Ok(o)
}

fn parse_type(tokens: &[Span<Token>], index: usize) -> Result<(Span<ast::Type>, usize), Error> {
	if let Some(t) = tokens.get(index) {
		if let Token::Ident(i) = t.val() {
			let typ = match i.as_str() {
				"number" => ast::Type::Number,
				"string" => ast::Type::String,
				"void" => ast::Type::Void,
				_ => return Err(t.error("Unexpected type", ReturnValue::UnexpectedType)),
			};
			return Ok((t.clone().map(typ), 1));
		} else if matches!(t.val(), Token::Kwd(k) if k == "[") {
			let (s, offset) = match parse_type(tokens, index + 1) {
				Ok(r) => r,
				Err(e) => return Err(e),
			};
			if let Some(t_end) = tokens.get(index + offset + 1) {
				if matches!(t_end.val(), Token::Kwd(k) if k == "]") {
					return Ok((
						t.clone()
							.join_with(&[t_end.clone()], ast::Type::Array(Box::new(s))),
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
		return Err(span.error("Unexpected EOI, expected Type", ReturnValue::UnclosedParens));
	};
}

/// Parse an expression
pub fn parse_expr(tokens: &[Span<Token>], intrinsics: bool) -> Result<Span<ast::Expr>, Error> {
	let len = tokens.len();
	match parse_expr_with_priority(&tokens, 0, 0, intrinsics) {
		Ok((v, offset)) => {
			assert_eq!(offset, len, "Didn't parse all the tokens");
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
				lhs = match parse_expr_with_priority(&tokens, index + offset, next_op.0.priority(), intrinsics)
				{
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
		index: usize, intrinsics: bool
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
					if let Some((expr, off)) = match parse_block(&tokens, index + offset, intrinsics) {
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
						central = Some(match super::parse_expr(&inner_tokens, intrinsics) {
							Ok(v) => v,
							Err(e) => return Err(e),
						});
					} else {
						//println!("TOKENS: {:?}", tokens);
						central = Some(match tokens.get(index + offset) {
							Some(v) => match v.val() {
								Token::Number(n) => v.clone().map(ast::Expr::Num(n)),
								Token::Ident(id) => if intrinsics && id.starts_with("INSTRINSIC_") {
									if let Some(i) = ast::intrinsics::Intrinsic::from_str(&id) {
										v.clone().map(ast::Expr::CompilerIntrinsic(i))
									}else{
										return Err(v.error("Intrinsic not defined", ReturnValue::IntrinsicNotDefined))
									}
								} else {v.clone().map(ast::Expr::Ident(id))},
								Token::String(s) => v.clone().map(ast::Expr::Str(s)),

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
							.map(|x| super::parse_expr(x.into(), intrinsics))
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
		index: usize, intrinsics: bool
	) -> Result<Option<(Span<ast::Expr>, usize)>, Error> {
		let mut offset = 0;
		if matches!(tokens.get(index + offset).map(|x| x.val()), Some(Token::Kwd(x)) if x == "{") {
			// println!("Parsing block: {:?}", &tokens[index..]);
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
					return Err(
						span.error("Unexpected EOI, expected `}`", ReturnValue::UnclosedParens)
					);
					//panic!("Unexpected EOI")
				}
				// println!("{:?}", tokens[index + offset]);
				if matches!(tokens[index + offset].clone().val(), Token::Kwd(x) if x == "{") {
					count += 1;
				} else if matches!(tokens[index + offset].clone().val(), Token::Kwd(x) if x == "}")
				{
					if count == 0 {
						break;
					}
					count -= 1;
				}
				inner_tokens.push(tokens[index + offset].clone());
				offset += 1;
			}
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
				let (expr, i) = match parse_expr_with_priority(next_tokens, inner_offset, 0, intrinsics) {
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
