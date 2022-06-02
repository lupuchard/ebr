use std::iter::Peekable;
use std::str::{Chars, FromStr, from_utf8};
use std::ascii::AsciiExt;
use std::{f64, result, fmt};

use types::{NumType, FloatType, SignedType, UnsignedType};

pub fn tokenize(string: &str) -> Vec<FullToken> {
	Tokenizer::tokenize(string)
}

#[derive(Clone, Debug, PartialEq)]
pub struct FullToken {
	pub line: u32,
	pub column: u32,
	pub token: Token,
}
impl FullToken {
	pub fn new(token: Token, line: u32, column: u32) -> FullToken {
		FullToken { token: token, line: line, column: column }
	}
	pub fn none(line: u32, column: u32) -> FullToken {
		FullToken { token: Token::None, line: line, column: column }
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
	None,
	Float(f64, FloatType),
	Int(  u64, NumType),
	String(String),
	Ident(String),
	Symbol(char),
	Comma,
	KwIf,
	KwElse,
	KwLoop,
	KwReturn,
	KwTrue,
	KwFalse,
	Special(String),
	Invalid(TokenType, String),
}
impl fmt::Display for Token {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		match *self {
			Token::None               => try!(f.write_str("UnknownToken")),
			Token::Float( ref val, _) => try!(f.write_fmt(format_args!("{}", val))),
			Token::Int(   ref val, _) => try!(f.write_fmt(format_args!("{}", val))),
			Token::String(ref val)    => try!(f.write_fmt(format_args!("\"{}\"", val))),
			Token::Ident( ref val)    => try!(f.write_fmt(format_args!("{}", val))),
			Token::Symbol(ref val)    => try!(f.write_fmt(format_args!("{}", val))),
			Token::Comma              => try!(f.write_str(",")),
			Token::KwIf               => try!(f.write_str("if")),
			Token::KwElse             => try!(f.write_str("else")),
			Token::KwLoop             => try!(f.write_str("loop")),
			Token::KwReturn           => try!(f.write_str("return")),
			Token::KwTrue             => try!(f.write_str("true")),
			Token::KwFalse            => try!(f.write_str("false")),
			Token::Special(ref val)   => try!(f.write_fmt(format_args!("@{}", val))),
			Token::Invalid(_, _)      => try!(f.write_str("invalid"))
		}
		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq)]
enum TokenType {
	Comma,
	Number,
	Word,
	String,
	Symbol,
}

struct Tokenizer {
	raw_tokens: Vec<(TokenType, String, u32, u32)>,
	line: u32,
	column: u32,
}
impl Tokenizer {
	fn add_comma(&mut self) {
		if self.raw_tokens.is_empty() { return; }
		match self.raw_tokens[self.raw_tokens.len() - 1] {
			(TokenType::Comma, _, _, _) => (),
			(_, _, _, _) => {
				self.raw_tokens.push((TokenType::Comma, ",".to_string(), self.line, self.column))
			},
		}
	}
	fn add_number(&mut self, number: String) {
		self.raw_tokens.push((TokenType::Number, number, self.line, self.column));
	}
	fn add_word(&mut self, word: String) {
		self.raw_tokens.push((TokenType::Word, word, self.line, self.column));
	}
	fn add_string(&mut self, string: String) {
		self.raw_tokens.push((TokenType::String, string, self.line, self.column));
	}
	fn add_symbol(&mut self, sym: char) {
		let mut token = String::new();
		token.push(sym);
		self.raw_tokens.push((TokenType::Symbol, token, self.line, self.column));
	}

	fn tokenize(string: &str) -> Vec<FullToken> {
		let mut tokenizer = Tokenizer {
			raw_tokens: Vec::new(),
			line: 1,
			column: 1,
		};
		tokenizer.tokenize_raw(string);
		tokenizer.interpret_raw()
	}
	fn tokenize_raw(&mut self, string: &str) {
		self.do_whitespace(string.chars().peekable())
	}

	fn do_whitespace(&mut self, mut iter: Peekable<Chars>) {
		loop {
			match *iter.peek().unwrap_or(&'\0') {
				'\n' => {
					self.add_comma();
					self.line += 1;
					self.column = 0;
				},
				c if c.is_whitespace()             => (),
				c if c.is_alphabetic() || c == '_' => return self.do_word(iter),
				c if c.is_digit(10)    || c == '.' => return self.do_number(iter),
				_                                  => return self.do_symbol(iter),
			}
			iter.next();
			self.column += 1;
		}
	}
	fn do_word(&mut self, mut iter: Peekable<Chars>) {
		let mut word = String::new();
		word.push(iter.next().unwrap());
		self.column += 1;
		loop {
			let c = *iter.peek().unwrap_or(&'\0');
			if c.is_alphanumeric() || c == '_' {
				word.push(c);
			} else {
				self.add_word(word);
				if c.is_whitespace() {
					return self.do_whitespace(iter);
				} else {
					return self.do_symbol(iter);
				}
			}
			iter.next();
			self.column += 1;
		}
	}

	fn do_number(&mut self, mut iter: Peekable<Chars>) {
		let mut number = String::new();
		number.push(iter.next().unwrap());
		let mut prev_e   = false;
		let mut prev_dot = false;
		self.column += 1;
		loop {
			let c = *iter.peek().unwrap_or(&'\0');
			if c.is_numeric() || ((c.is_alphabetic() || c == '_') && !prev_dot) ||
			   c == '.' || (prev_e && c == '-') {
				prev_e   = c == 'e' || c == 'E';
				prev_dot = c == '.';
				number.push(c);
			} else {
				self.add_number(number);
				if c.is_whitespace() {
					return self.do_whitespace(iter);
				} else {
					return self.do_symbol(iter);
				}
			}
			iter.next();
			self.column += 1;
		}
	}
	fn do_symbol(&mut self, mut iter: Peekable<Chars>) {
		match *iter.peek().unwrap_or(&'\0') {
			'\0' => return,
			'"' => {
				// Strings!
				let mut token = String::new();
				token.push(iter.next().unwrap());
				self.column += 1;
				loop {
					self.column += 1;
					match iter.next().unwrap_or('\0') {
						'\0' => return, // TODO: error (unterminated quote)
						'\\' => {
							token.push('\\');
							token.push(iter.next().unwrap_or('\0'));
							self.column += 1;
						},
						'"' => {
							token.push('"');
							self.add_string(token);
							return self.do_symbol(iter);
						},
						c => token.push(c),
					}
				}
			},
			'/' => {
				// Comments!
				iter.next().unwrap();
				self.column += 1;
				match *iter.peek().unwrap_or(&'\0') {
					'\0' => return self.add_symbol('/'),
					'*' => {
						loop {
							self.column += 2;
							iter.next();
							match iter.next().unwrap_or('\0') {
								'\0' => return, // TODO: error (unterminated comment)
								'*'  => if iter.next().unwrap_or('\0') == '/' {
									return self.do_symbol(iter);
								},
								_ => (),
							}
						}
					},
					'/' => {
						self.column = 1;
						loop {
							match iter.next().unwrap_or('\0') {
								'\n' => return self.do_symbol(iter),
								'\0' => return,
								_    => (),
							}
						}
					},
					_ => (),
				}
			},
			',' => {
				iter.next();
				self.column += 1;
				self.add_comma();
			},
			c if c.is_whitespace()             => return self.do_whitespace(iter),
			c if c.is_alphabetic() || c == '_' => return self.do_word(iter),
			c if c.is_digit(10)    || c == '.' => return self.do_number(iter),
			_ => {
				self.column += 1;
				self.add_symbol(iter.next().unwrap())
			},
		}
		self.do_symbol(iter);
	}

	fn interpret_raw(self) -> Vec<FullToken> {
		let mut tokens = Vec::new();
		let mut prev_special = false;
		for (token_type, string, ln, mut clm) in self.raw_tokens.into_iter() {
			if clm > 0 { clm -= 1; }
			match token_type {
				TokenType::Comma => tokens.push(FullToken::new(Token::Comma, ln, clm)),
				TokenType::Word  => if prev_special {
					tokens.push(FullToken::new(Token::Special(string), ln, clm));
				} else {
					match string.as_ref() {
						"if"     => tokens.push(FullToken::new(Token::KwIf,          ln, clm)),
						"else"   => tokens.push(FullToken::new(Token::KwElse,        ln, clm)),
						"loop"   => tokens.push(FullToken::new(Token::KwLoop,        ln, clm)),
						"return" => tokens.push(FullToken::new(Token::KwReturn,      ln, clm)),
						"true"   => tokens.push(FullToken::new(Token::KwTrue,        ln, clm)),
						"false"  => tokens.push(FullToken::new(Token::KwFalse,       ln, clm)),
						_        => tokens.push(FullToken::new(Token::Ident(string), ln, clm)),
					}
				},
				TokenType::Number => tokens.push(FullToken::new(match parse_num(&string) {
					Some(token) => token,
					None        => Token::Invalid(token_type, string),
				}, ln, clm)),
				TokenType::String => {
					let s = unsafe { string.slice_unchecked(1, string.len() - 1) }; // slice_chars is unstable...
					tokens.push(FullToken::new(Token::String(s.to_string()), ln, clm));
				},
				TokenType::Symbol => {
					let c = string.chars().next().unwrap();
					if c == '@' {
						prev_special = true;
						continue;
					} else {
						tokens.push(FullToken::new(Token::Symbol(c), ln, clm));
					}
				},
			}
			prev_special = false;
		}
		tokens
	}
}

pub fn parse_num(s: &str) -> Option<Token> {
	if s == "." { return Some(Token::Symbol('.')); }
	let string = match reduce_num(s) {
		Some(string) => string,
		None         => return None,
	};
	if string.len() >= 2 && b"bqoxd".contains(&string[1]) {
		parse_int(string)
	} else if string.contains(&b'e') | string.contains(&b'.') {
		parse_float(string)
	} else {
		parse_int(string)
	}
}

fn parse_float(string: Vec<u8>) -> Option<Token> {
	let (ty, last_idx) = match rposition_elem(&string, b'f') {
		Some(idx) => (match &string[idx..] {
			b"f"   => FloatType::Unknown,
			b"f32" => FloatType::F32,
			b"f64" => FloatType::F64,
			_ => return None,
		}, idx),
		None => (FloatType::Unknown, string.len()),
	};

	let body = &string[..last_idx];
	if body == b"0in" {
		return Some(Token::Float(f64::INFINITY, ty));
	}
	match f64::from_str(from_utf8(body).unwrap()) {
		Ok(value) => Some(Token::Float(value, ty)),
		Err(_)    => None,
	}
}

fn parse_int(string: Vec<u8>) -> Option<Token> {
	let (explicit_base, base, first_idx) = if string.len() >= 2 && string[0] == b'0' {
		match string[1] {
			b'b' => (true,  2, 2),
			b'q' => (true,  4, 2),
			b'o' => (true,  8, 2),
			b'x' => (true, 16, 2),
			b'd' => (true, 10, 2),
			_    => (true, 10, 0),
		}
	} else { (false, 10, 0) };

	if let Some(idx) = rposition_elem(&string, b'f') { if !explicit_base {
		let body = &string[..idx];
		match f64::from_str(from_utf8(body).unwrap()) {
			Ok(val) => return Some(Token::Float(val, match &string[idx..] {
				b"f"   => FloatType::Unknown,
				b"f32" => FloatType::F32,
				b"f64" => FloatType::F64,
				_ => return None,
			})),
			Err(_)  => return None,
		}
	} }

	let (ty, last_idx) = match rposition_elem(&string, b'i') {
		Some(idx) => (match &string[idx..] {
			b"i"   => NumType::Signed(SignedType::Unknown),
			b"i8"  => NumType::Signed(SignedType::I8),
			b"i16" => NumType::Signed(SignedType::I16),
			b"i32" => NumType::Signed(SignedType::I32),
			b"i64" => NumType::Signed(SignedType::I64),
			_ => return None,
		}, idx),
		None => match rposition_elem(&string, b'u') {
			Some(idx) => (match &string[idx..] {
				b"u"   => NumType::Unsigned(UnsignedType::Unknown),
				b"u8"  => NumType::Unsigned(UnsignedType::U8),
				b"u16" => NumType::Unsigned(UnsignedType::U16),
				b"u32" => NumType::Unsigned(UnsignedType::U32),
				b"u64" => NumType::Unsigned(UnsignedType::U64),
				_ => return None,
			}, idx),
			None => (NumType::Unknown, string.len()),
		},
	};

	let body  = &string[first_idx..last_idx];
	let value = match u64::from_str_radix(from_utf8(body).unwrap(), base) {
		Ok(val) => val,
		Err(_)  => return None,
	};

	Some(Token::Int(value, ty))
}

fn reduce_num(string: &str) -> Option<Vec<u8>> {
	if !string.is_ascii() { return None; }
	let mut out = Vec::new();
	for c in string.bytes() {
		match c {
			b'_' => (),
			  _  => out.push(c.to_ascii_lowercase()),
		}
	}
	Some(out)
}

// position_elem is not stable
fn rposition_elem(string: &[u8], c: u8) -> Option<usize> {
	for (i, str_char) in string.iter().enumerate() {
		if *str_char == c { return Some(i); }
	}
	None
}

#[cfg(test)]
mod test {
	use super::*;
	use types::*;

	#[test]
	fn parse_num_test() {
		assert_eq!(parse_num("12"),
			Some(Token::Int(12, NumType::Unknown)));

		assert_eq!(parse_num("32u8"),
			Some(Token::Int(32, NumType::Unsigned(UnsignedType::U8))));

		assert_eq!(parse_num("_3__i6_4"),
			Some(Token::Int(3, NumType::Signed(SignedType::I64))));

		assert_eq!(parse_num("0xbEeFi"),
			Some(Token::Int(48879, NumType::Signed(SignedType::Unknown))));

		assert_eq!(parse_num("0b01010111"),
			Some(Token::Int(87, NumType::Unknown)));

		assert_eq!(parse_num("51f32"),
			Some(Token::Float(51., FloatType::F32)));

		assert_eq!(parse_num("2."),
			Some(Token::Float(2., FloatType::Unknown)));

		assert_eq!(parse_num(".5f64"),
			Some(Token::Float(0.5, FloatType::F64)));

		assert_eq!(parse_num("12e-2"),
			Some(Token::Float(12e-2, FloatType::Unknown)));
	}
}
