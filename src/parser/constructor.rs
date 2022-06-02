use std::vec::IntoIter;
use std::iter::Peekable;

use types::*;
use parser::ast::*;
use parser::{FullToken, Token};
use parser::error::{Error, ErrorKind, Result};

pub fn construct(mut tokens: Vec<FullToken>) -> (Block, Vec<Error>) {
	let mut con = Constructor { errors: Vec::new(), line: 0, column: 0 };

	// place extra block close at the end of the tokens
	let ln = tokens[tokens.len() - 1].line;
	tokens.push(FullToken::new(Token::Symbol('}'), ln + 1, 0));

	let mut iter  = tokens.into_iter().peekable();

	let mut block = Vec::new();
	con.do_block(&mut iter, &mut block);

	(block, con.errors)
}

macro_rules! next {
	($slf: expr, $iter: expr) => { {
		let token = try!($iter.next().ok_or(
			Error::unclosed_block(FullToken::none($slf.line, $slf.column))));
		$slf.line   = token.line;
		$slf.column = token.column;
		token
	} }
}
macro_rules! peek {
	($slf: expr, $iter:expr) => { {
		let token = try!($iter.peek().ok_or(
			Error::unclosed_block(FullToken::none($slf.line, $slf.column))));
		$slf.line   = token.line;
		$slf.column = token.column;
		token
	} }
}
macro_rules! sym {
	[$($sym:expr),+] => {
		vec![$(Token::Symbol($sym)),+]
	}
}

type Iter = Peekable<IntoIter<FullToken>>;
struct Constructor {
	errors: Vec<Error>,
	line: u32,
	column: u32,
}
impl Constructor {

	// Do each statement in the block.
	fn do_block(&mut self, iter: &mut Iter, block: &mut Block) {
		loop {
			match self.do_statement(iter) {
				Ok(s)  => block.push(s),
				Err(e) => match e.kind {
					ErrorKind::Done => break,
					ErrorKind::UnclosedBlock => {
						self.errors.push(e);
						break;
					},
					_ => {
						self.errors.push(e);
						match self.close_block(iter) {
							Ok(()) => (),
							Err(e) => self.errors.push(e),
						}
					},
				},
			}
		}
	}

	// Throw away tokens until ending '}' is found.
	fn close_block(&mut self, iter: &mut Iter) -> Result<()> {
		loop {
			match next!(self, iter).token {
				Token::Symbol('{') => return self.close_block(iter),
				Token::Symbol('}') => return Ok(()),
				_ => (),
			}
		}
	}

	fn do_statement(&mut self, iter: &mut Iter) -> Result<Statement> {
		self.trim_commas(iter);
		let token = next!(self, iter);
		match token.token {
			Token::Ident(ref id) => {
				// Statements beginning with an identifier are either assignments or declarations.
				let id = id.to_string();
				let token2 = next!(self, iter);
				match token2.token {
					Token::Symbol(':') => self.do_declare(iter, Node::new(id, token.clone())),
					Token::Symbol('=') => {
						self.do_assign(iter, Node::new(id, token.clone()), Op::Not)
					},
					Token::Symbol(op) => {
						// Possibly an operator assignment.
						let eq_token = next!(self, iter);
						match eq_token.token {
							Token::Symbol('=') => (),
							_ => return Err(Error::expected(eq_token, vec![Token::Symbol('=')])),
						}
						match op {
							'+' => self.do_assign(iter, Node::new(id, token2), Op::Add),
							'-' => self.do_assign(iter, Node::new(id, token2), Op::Sub),
							'*' => self.do_assign(iter, Node::new(id, token2), Op::Mul),
							'/' => self.do_assign(iter, Node::new(id, token2), Op::Div),
							'%' => self.do_assign(iter, Node::new(id, token2), Op::Mod),
							'^' => self.do_assign(iter, Node::new(id, token2), Op::Pow),
							'&' => self.do_assign(iter, Node::new(id, token2), Op::And),
							'|' => self.do_assign(iter, Node::new(id, token2), Op::Or),
							_ => Err(Error::expected(token2, sym![':','=','+','-','*','/',
							                                     '^','%','&','|'])),
						}
					}
					_ => Err(Error::expected(token2, sym![':','=','+','-','*','/',
					                                     '^','%','&','|'])),
				}
			},
			Token::KwIf     => self.do_if(    iter),
			Token::KwLoop   => self.do_loop(  iter),
			Token::KwReturn => self.do_return(iter),
			Token::Symbol('{') => {
				let mut block = Vec::new();
				self.do_block(iter, &mut block);
				Ok(Statement::Block(Node::new(block, token)))
			},
			Token::Symbol('}') => Err(Error::done()),
			Token::Special(command) => self.do_special(&command[..], iter),
			_ => Err(Error::expected(token, vec![Token::Ident("".to_string()),
			                                     Token::KwIf,
			                                     Token::KwLoop,
			                                     Token::KwReturn,
			                                     Token::Symbol('{'),
			                                     Token::Symbol('}')])),
		}
	}

	fn do_declare(&mut self, iter: &mut Iter, id: Node<String>) -> Result<Statement> {
		let token = next!(self, iter);
		let type_id = match token.token {
			Token::Symbol('=') => None, // var := val
			Token::Ident(_) => {        // var: type
				let eq_token = next!(self, iter);
				let type_id = if let Token::Ident(ref id) = token.token {
					Some(Node::new(id.clone(), token.clone()))
				} else { unreachable!() };
				match eq_token.token {
					Token::Symbol('=') => (), // var: type = val
					Token::Comma       => return Ok(Statement::Declaration(id, type_id, None)),
					_ => return Err(Error::expected(eq_token, vec![Token::Symbol('=')])),
				}
				type_id
			},
			_ => return Err(Error::expected(token, vec![Token::Symbol('='),
			                                            Token::Ident("".to_string())])),
		};
		self.trim_commas(iter);
		let mut expr = Vec::new();
		try!(self.do_expr(iter, '}', &mut expr));
		Ok(Statement::Declaration(id, type_id, Some(expr)))
	}

	fn do_assign(&mut self, iter: &mut Iter, id: Node<String>, op: Op) -> Result<Statement> {
		self.trim_commas(iter);
		let mut expr: Expr = Vec::new();
		if op != Op::Not {
			// If it is an operator assignment (like +=) then the expression has the variable
			// appended to the front and the operator appended to the back.
			let (l, c) = (self.line, self.column);
			expr.push(Node::newn((ExprToken::Id(id.val.clone()), Type::Unknown), l, c));
			try!(self.do_expr(iter, '}', &mut expr));
			expr.push(Node::newn((ExprToken::Op(op,), op.return_type()), l, c));
		} else {
			try!(self.do_expr(iter, '}', &mut expr));
		}
		Ok(Statement::Assignment(id, expr))
	}

	fn do_if(&mut self, iter: &mut Iter) -> Result<Statement> {
		let mut exprs  = Vec::new();
		let mut blocks = Vec::new();
		let else_block;
		loop {
			let mut expr = Vec::new();
			try!(self.do_expr(iter, '{', &mut expr));
			self.trim_commas(iter);
			exprs.push(expr);

			let token = iter.next().unwrap();
			let mut block = Vec::new();
			self.do_block(iter, &mut block);
			blocks.push(Node::new(block, token));

			self.trim_commas(iter);
			match peek!(self, iter).token {
				Token::KwElse => {
					iter.next();
					self.trim_commas(iter);
					let token = next!(self, iter);
					match token.token {
						Token::KwIf => (),
						Token::Symbol('{') => {
							let mut block = Vec::new();
							self.do_block(iter, &mut block);
							else_block = Some(Node::new(block, token));
							break;
						},
						_ => return Err(Error::expected(token, vec![Token::KwIf,
						                                            Token::Symbol('{')])),
					}
				},
				_ => {
					else_block = None;
					break;
				},
			}
		}
		Ok(Statement::If(exprs, blocks, else_block))
	}

	fn do_loop(&mut self, iter: &mut Iter) -> Result<Statement> {
		if peek!(self, iter).token == Token::KwIf {
			/*iter.next();
			self.trim_commas(iter);

			let mut expr = Vec::new();
			try!(self.do_expr(iter, '{', &mut expr));
			self.trim_commas(iter);
			//if iter.peek().unwrap().token == Token::Symbol('{') { iter.next(); }
			expr.push(Node::newn(ExprToken::Op(Op::Not), self.line, self.column));

			let token = iter.next().unwrap();
			let mut block = Vec::new();
			let break_block = Node::newn(vec![Statement::Break], self.line, self.column);
			block.push(Statement::If(vec![expr], vec![break_block], None));
			self.do_block(iter, &mut block);

			iter.next();
			Ok(Statement::Loop(Node::new(block, token)))*/
			unimplemented!();
		} else {
			self.trim_commas(iter);

			let token = next!(self, iter);
			if token.token != Token::Symbol('{') {
				return Err(Error::expected(token, vec![Token::Symbol('{')]));
			}

			let mut block = Vec::new();
			self.do_block(iter, &mut block);

			Ok(Statement::Loop(Node::new(block, token)))
		}
	}

	fn do_return(&mut self, iter: &mut Iter) -> Result<Statement> {
		{
			let token = peek!(self, iter);
			if token.token == Token::Comma || token.token == Token::Symbol('}') {
				return Ok(Statement::Return(None));
			}
		}
		let mut expr = Vec::new();
		try!(self.do_expr(iter, '}', &mut expr));
		Ok(Statement::Return(Some(expr)))
	}

	fn do_special(&mut self, command: &str, iter: &mut Iter) -> Result<Statement> {
		// TODO: make command lowercase (its unstable!)
		match command {
			"print" => {
				let token = next!(self, iter);
				match token.token {
					Token::Ident(id) => Ok(Statement::Print(id)),
					_ => Err(Error::expected(token, vec![Token::Ident("".to_string())]))
				}
			},
			_ => Err(Error::invalid_special(FullToken::new(Token::Special(command.to_string()),
			                                               self.line,
			                                               self.column))),
		}
	}

	// shunting yard
	fn do_expr(&mut self, iter: &mut Iter, term: char, output: &mut Expr) -> Result<()> {
		let mut ops: Vec<(Op, FullToken)> = Vec::new();
		let mut prev_was_op = true;
		loop {
			{
				let token = peek!(self, iter);
				if token.token == Token::Comma || token.token == Token::Symbol(term) {
					// The expression has terminated.
					while !ops.is_empty() {
						let (op, ftoken) = ops.pop().unwrap();
						if op == Op::TempParen {
							return Err(Error::expected((*token).clone(), vec![Token::Symbol(')')]));
						}
						output.push(Node::new((ExprToken::Op(op), op.return_type()), ftoken));
					}
					return Ok(());
				}
			}
			let pwo = prev_was_op;
			prev_was_op = false;
			let token = next!(self, iter);
			match token.token {
				Token::Int(val, ty) => {
					output.push(Node::new((ExprToken::IntLit(val), Type::Num(ty)), token));
				},
				Token::Float(val, ty) => {
					let ty = Type::Num(NumType::Float(ty));
					output.push(Node::new((ExprToken::FloatLit(val), ty), token));
				},
				Token::String(_) => unimplemented!(),
					/*let val = if let Token::String(ref val) = token.token {
						val.clone()
					} else { unreachable!() };
					output.push(Node::new(ExprToken::StringLit(val), token));*/
				Token::Ident(_)   => {
					let id = if let Token::Ident(ref id) = token.token {
						id.clone()
					} else { unreachable!() };
					output.push(Node::new((ExprToken::Id(id), Type::Unknown), token));
				},
				Token::KwTrue  => {
					output.push(Node::new((ExprToken::BoolLit(true),  Type::Bool), token));
				},
				Token::KwFalse => {
					output.push(Node::new((ExprToken::BoolLit(false), Type::Bool), token));
				},
				Token::Symbol('(') => ops.push((Op::TempParen, FullToken::none(0, 0))),
				Token::Symbol(')') => loop {
					match ops.pop() {
						None => return Err(Error::mismatched_paren(token)),
						Some((Op::TempParen, _)) => break,
						Some((op, t)) => {
							output.push(Node::new((ExprToken::Op(op), op.return_type()), t));
						},
					}
				},
				Token::Symbol(c) => {
					let op = match c {
						'!' => match peek!(self, iter).token {
							Token::Symbol('=') => { iter.next(); Op::Neq },
							_ => Op::Not,
						},
						'+' => Op::Add,
						'-' if pwo => Op::Neg,
						'-' => Op::Sub,
						'*' => Op::Mul,
						'/' if pwo => Op::Inv,
						'/' => Op::Div,
						'%' => Op::Mod,
						'^' => Op::Pow,
						'&' => Op::And,
						'|' => Op::Or,
						'>' => match peek!(self, iter).token {
							Token::Symbol('=') => { iter.next(); Op::Geq },
							_ => Op::Gt,
						},
						'<' => match peek!(self, iter).token {
							Token::Symbol('=') => { iter.next(); Op::Leq },
							_ => Op::Lt,
						},
						'=' => match peek!(self, iter).token {
							Token::Symbol('=') => { iter.next(); Op::Eq },
							_ => return Err(Error::invalid_op(token)),
						},
						_ => return Err(Error::invalid_op(token)),
					};
					loop {
						if ops.is_empty() { break; }
						{
							let (ref op2, _) = ops[ops.len() - 1];
							if !((op.left_assoc() && op.prec() <= op2.prec()) ||
							    (!op.left_assoc() && op.prec() <  op2.prec())) {
								break;
							}
						}
						let (op, t) = ops.pop().unwrap();
						output.push(Node::new((ExprToken::Op(op), op.return_type()), t));
					}
					ops.push((op, token));
					prev_was_op = true;
					self.trim_commas(iter);
				},
				_ => return Err(Error::expected(token, vec![Token::Int(0, NumType::Unknown),
				                                            Token::Float(0.0, FloatType::Unknown),
				                                            Token::String("".to_string()),
				                                            Token::Ident("".to_string()),
				                                            Token::Symbol(term),
				                                            Token::Comma])),
			}
		}
	}

	fn trim_commas(&mut self, iter: &mut Iter) {
		loop {
			let _ = match iter.peek() {
				Some(&FullToken { token: Token::Comma, line: _, column: _ }) => iter.next(),
				_ => break,
			};
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use parser::tokenize;

	#[test]
	fn test() {
		let (_, e) = construct(tokenize("x123 := 6.4"));
		assert_eq!(e, Vec::new());

		let (_, e) = construct(tokenize("_X: bool = y"));
		assert_eq!(e, Vec::new());

		let (_, e) = construct(tokenize("y=- -(1 + 2 - 3 * 4 % 5 ^ 6) > 7 & 8 < a == b != c"));
		assert_eq!(e, Vec::new());

		let (_, e) = construct(tokenize("y= !(d >= f | g <= h)"));
		assert_eq!(e, Vec::new());

		let (_, e) = construct(tokenize("loop{ if false {{}}, return }"));
		assert_eq!(e, Vec::new());

		let (b1, e) = construct(tokenize("if true{y=0}else if x<1{y=1}else{return x}"));
		assert_eq!(e, Vec::new());

		let (b2, e) = construct(tokenize(r#"
			if true {
		y =
		0
			,,} else if (x <,,
				1)
			{ y = 1 },,,,,
			else
			{ return
			}"#));
		assert_eq!(e, Vec::new());
	}
}
