pub mod ast;

pub mod tokenizer;
pub use self::tokenizer::{tokenize, Token,  FullToken};

pub mod constructor;
pub use self::constructor::construct;

pub mod error;
pub use self::error::{Error, ErrorKind};

pub fn parse(string: &str) -> (ast::Block, Vec<Error>) {
	let tokens = tokenize(string);
	let mut errors = Vec::new();
	for token in tokens {
		match token.token {
			Token::Invalid(_, _) => errors.push(Error::invalid_token(token)),
			_ => (),
		}
	}
	if errors.is_empty() {
		construct(tokenize(string))
	} else {
		(Vec::new(), errors)
	}
}
