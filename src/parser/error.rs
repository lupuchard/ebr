use std::{fmt, result};

use parser::{FullToken, Token};

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
	pub token: FullToken,
	pub kind: ErrorKind,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
	Done,
	InvalidToken,
	Expected(Vec<Token>),
	InvalidOp,
	InvalidSpecial,
	UnclosedBlock,
	MismatchedParen,
}
impl Error {
	pub fn done() -> Error {
		Error { token: FullToken::none(0, 0), kind: ErrorKind::Done }
	}
	pub fn invalid_token(token: FullToken) -> Error {
		Error { token: token, kind: ErrorKind::InvalidToken }
	}
	pub fn expected(token: FullToken, possible_choices: Vec<Token>) -> Error {
		Error { token: token, kind: ErrorKind::Expected(possible_choices) }
	}
	pub fn invalid_op(token: FullToken) -> Error {
		Error { token: token, kind: ErrorKind::InvalidOp }
	}
	pub fn invalid_special(token: FullToken) -> Error {
		Error { token: token, kind: ErrorKind::InvalidSpecial }
	}
	pub fn unclosed_block(token: FullToken) -> Error {
		Error { token: token, kind: ErrorKind::UnclosedBlock }
	}
	pub fn mismatched_paren(token: FullToken) -> Error {
		Error { token: token, kind: ErrorKind::MismatchedParen }
	}
}
impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		try!(f.write_fmt(format_args!("At {}:{} -- ", self.token.line, self.token.column)));
		let my_token = &self.token.token;
		match self.kind {
			ErrorKind::Done => unreachable!(),
			ErrorKind::InvalidToken => {
				try!(f.write_str("Invalid token."));
			},
			ErrorKind::Expected(ref tokens) => {
				if tokens.len() == 1 {
					try!(f.write_fmt(format_args!("Expected '{}', found '{}'.",
					                              tokens[0], my_token)));
				} else {
					try!(f.write_str("Expected one of "));
					for token in tokens {
						try!(f.write_fmt(format_args!("'{}',", token)));
					}
					try!(f.write_fmt(format_args!(" found '{}'.", my_token)));
				}
			},
			ErrorKind::InvalidOp => {
				try!(f.write_fmt(format_args!("Invalid operator: '{}'", my_token)));
			}
			ErrorKind::InvalidSpecial => {
				try!(f.write_fmt(format_args!("Not a valid special: '{}'", my_token)));
			},
			ErrorKind::UnclosedBlock => {
				try!(f.write_str("Unclosed block."));
			},
			ErrorKind::MismatchedParen => {
				try!(f.write_str("Mismatched parenthesis."));
			},
		}
		Ok(())
	}
}
pub type Result<T> = result::Result<T, Error>;

