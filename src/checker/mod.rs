use std::collections::HashMap;
use std::{result, fmt};
use std::ptr;

use builder::llvm;
use parser::FullToken;
use parser::ast::*;
use types::{Type, NumType};

pub struct Program {
	global_variables: HashMap<String, Variable>,
	global_functions: HashMap<String, Variable>,
}

#[derive(Clone, Copy, Debug)]
pub struct Variable {
	pub ty: Type,
	pub llvm: Option<llvm::Value>,
}
impl Variable {
	fn new(ty: Type) -> Variable {
		Variable { ty: ty, llvm: None }
	}
}

pub struct Function {
	pub params: Vec<(String, Variable)>,
	pub ret_ty: Option<Type>,
	pub scope: Scope,
	pub llvm: Option<llvm::Value>,
}

pub struct Scope {
	// name: Option<String>,
	parent: *mut Scope,
	children: Vec<Box<Scope>>,
	variables: HashMap<String, Variable>,
}
impl Scope {
	pub fn new_root() -> Scope {
		Scope {
			parent: ptr::null_mut(),
			children: Vec::new(),
			variables: HashMap::new(),
		}
	}

	pub fn new(&mut self) -> &mut Scope {
		let mut scope = Box::new(Scope::new_root());
		scope.parent = self;
		self.children.push(scope);
		let len = self.children.len();
		&mut self.children[len - 1]
	}

	pub fn create_var(&mut self, name: String, var: Variable) -> Option<Variable> {
		self.variables.insert(name, var)
	}

	pub fn get_var(&mut self, name: &str) -> Option<&mut Variable> {
		match self.variables.get_mut(name) {
			Some(var) => Some(var),
			None => if self.parent.is_null() {
				None
			} else {
				unsafe { (*self.parent).get_var(name) }
			},
		}
	}
}

pub fn passes(block: &mut Block, scope: &mut Scope) -> Result<()> {
	try!(check_types(block, scope));
	try!(complete_types(block, scope));
	Ok(())
}

pub fn check_types(block: &mut Block, scope: &mut Scope) -> Result<()> {
	for statement in block {
		match *statement {
			Statement::Declaration(ref id, ref type_name, ref mut expr) => {
				// Create new variable, assign type as merge of given type and assignment.
				let mut ty = match *type_name {
					Some(ref ty) => Type::from_str(&ty.val),
					None         => Type::Unknown,
				};
				if expr.is_some() {
					ty = ty.merge(try!(type_of(&mut expr.as_mut().unwrap(), scope)));
				}
				if scope.create_var(id.val.clone(), Variable::new(ty)).is_some() {
					return Err(Error::already_exists(&id.token));
				}
			},
			Statement::Assignment(ref id, ref mut expr) => {
				// Merge type of given variable with that of assignment.
				let ty  = try!(type_of(expr, scope));
				let var = scope.get_var(&id.val);
				let var = try!(var.ok_or(Error::doesnt_exist(&id.token)));
				var.ty = var.ty.merge(ty);
			},
			Statement::If(ref mut conditions, ref mut blocks, ref mut else_block) => {
				for condition in conditions {
					// Assert all condition types are boolean.
					let ty = try!(type_of(condition, scope));
					if ty != Type::Bool {
						return Err(Error::expected(&condition[0].token, ty, Type::Bool));
					}
				}
				for block in blocks {
					try!(check_types(&mut block.val, scope.new()));
				}
				if else_block.is_some() {
					try!(check_types(&mut else_block.as_mut().unwrap().val, scope.new()));
				}
			},
			Statement::Loop(ref mut block)  => try!(check_types(&mut block.val, scope.new())),
			Statement::Block(ref mut block) => try!(check_types(&mut block.val, scope.new())),
			Statement::Return(ref mut expr) => match *expr {
				Some(ref mut expr) => { try!(type_of(expr, scope)); },
				None => (),
			},
			Statement::Print(_)  => (),
		}
	}
	Ok(())
}
fn type_of(expr: &mut Expr, scope: &mut Scope) -> Result<Type> {
	let mut stack = Vec::new();
	for &mut Node { ref token, val: (ref e, ref mut ty) } in expr {
		match *e {
			ExprToken::Id(ref id) => match scope.get_var(id) {
				Some(var) => stack.push(var.ty),
				None      => return Err(Error::doesnt_exist(token)),
			},
			ExprToken::Op(op) => {
				match op {
					Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod => {
						*ty = try!(merge_stack(&mut stack, 2, *ty, token));
					},
					Op::Pow => unimplemented!(),
					Op::And | Op::Or => {
						try!(merge_stack(&mut stack, 2, Type::Bool, token));
					},
					Op::Gt | Op::Lt | Op::Geq | Op::Leq => {
						try!(merge_stack(&mut stack, 2, Type::Num(NumType::Unknown), token));
					},
					Op::Eq | Op::Neq => {
						try!(merge_stack(&mut stack, 2, Type::Unknown, token));
					},
					Op::Not => {
						try!(merge_stack(&mut stack, 1, Type::Bool, token));
					},
					Op::Neg => {
						*ty = try!(merge_stack(&mut stack, 1, *ty, token));
						if let Type::Num(NumType::Unsigned(_)) = *ty {
							return Err(Error::expected_sign(token));
						}
					},
					Op::Inv => {
						try!(merge_stack(&mut stack, 1, *ty, token));
					},
					Op::TempParen => unreachable!(),
				}
				stack.push(*ty);
			},
			_ => stack.push(*ty),
		}
	}
	if stack.is_empty() { return Err(Error::empty_expr()); }
	Ok(stack.pop().unwrap())
}
fn merge_stack(stack: &mut Vec<Type>, num: u32,
               req_ty: Type, token: &FullToken) -> Result<Type> {
	if stack.len() < num as usize { return Err(Error::too_few_operands(token, num)); }
	let mut ty = Type::Unknown;
	for _ in 0..num {
		let a = stack.pop().unwrap();
		let n = ty.merge(a);
		if n == Type::Invalid { return Err(Error::expected(token, ty, a)) };
		ty = n;
	}
	ty = ty.merge(req_ty);
	if ty == Type::Invalid { return Err(Error::expected(token, ty, req_ty)); }
	Ok(ty)
}

/// Finishes type inference by reverting all unknown types to defaults, when possible.
pub fn complete_types(block: &mut Block, scope: &mut Scope) -> Result<()> {
	try!(complete_var_types(block, scope));
	complete_lit_types(block, scope);
	//try!(assert_valid_types(block, scope));
	Ok(())
}

/// Sets the types of all incomplete variables to their defaults.
pub fn complete_var_types(block: &mut Block, scope: &mut Scope) -> Result<()> {
	for statement in block {
		match *statement {
			Statement::Declaration(ref id, _, _) => {
				let var = scope.get_var(&id.val).unwrap();
				var.ty = var.ty.complete();
				if var.ty == Type::Invalid { return Err(Error::type_not_determined(&id.token)); }
			},
			Statement::If(_, ref mut blocks, ref mut else_block) => {
				for block in blocks {
					try!(complete_var_types(&mut block.val, scope.new()));
				}
				if else_block.is_some() {
					try!(complete_var_types(&mut else_block.as_mut().unwrap().val, scope.new()));
				}
			},
			Statement::Loop(ref mut block) | Statement::Block(ref mut block) => {
				try!(complete_var_types(&mut block.val, scope.new()));
			},
			_ => (),
		}
	}
	Ok(())
}

/// Reverse infers assignments from the variables to the literals assigned to them.
pub fn complete_lit_types(block: &mut Block, scope: &mut Scope) {
	for statement in block {
		match *statement {
			Statement::Declaration(ref id, _, Option::Some(ref mut expr)) |
			Statement::Assignment( ref id,    ref mut expr) => {
				let ty = scope.get_var(&id.val).unwrap().ty;
				clt_expr(scope, expr, ty);
			},
			Statement::If(ref mut conditions, ref mut blocks, ref mut else_block) => {
				for condition in conditions {
					clt_expr(scope, condition, Type::Bool);
				}
				for block in blocks {
					complete_lit_types(&mut block.val, scope.new());
				}
				if else_block.is_some() {
					complete_lit_types(&mut else_block.as_mut().unwrap().val, scope.new());
				}
			},
			Statement::Loop(ref mut block) | Statement::Block(ref mut block) => {
				complete_lit_types(&mut block.val, scope.new());
			}
			Statement::Return(ref mut expr) => match *expr { // TODO: return type
				Some(ref mut expr) => clt_expr(scope, expr, Type::Unknown),
				None => (),
			},
			_ => (),
		}
	}
}

pub fn clt_expr(scope: &mut Scope, expr: &mut Expr, final_type: Type) {
	let mut stack: Vec<(Type, Vec<&mut Type>)> = Vec::new();
	for &mut Node { token: _, val: (ref e, ref mut ty) } in expr {
		match *e {
			ExprToken::Id(ref id) => {
				let var_ty = scope.get_var(id).unwrap().ty;
				assert!(var_ty.is_known());
				stack.push((var_ty, Vec::new()));
			},
			ExprToken::Op(op) => {
				match op {
					Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod |
					Op::And | Op::Or => {
						let (ty2,     tyref2) = stack.pop().unwrap();
						let (ty1, mut tyref1) = stack.pop().unwrap();

						if ty1 == Type::Unknown {
							if ty2 == Type::Unknown {
								for blah in tyref2 {
									tyref1.push(blah); // append() is unstable
								}
								stack.push((Type::Unknown, tyref1));
							} else {
								for tyref in tyref1 { *tyref = ty2; }
								stack.push((ty2, Vec::new()));
							}
						} else {
							if ty2 == Type::Unknown {
								for tyref in tyref2 { *tyref = ty1; }
								stack.push((ty1, Vec::new()));
							} else {
								stack.push((ty1, Vec::new()));
							}
						}
					},
					Op::Pow => unimplemented!(),
					Op::Gt | Op::Lt | Op::Geq | Op::Leq | Op::Eq | Op::Neq => {
						let (ty2, tyref2) = stack.pop().unwrap();
						let (ty1, tyref1) = stack.pop().unwrap();

						if ty1 == Type::Unknown {
							if ty2 == Type::Unknown {
								for tyref in tyref1 { *tyref = tyref.complete(); }
								for tyref in tyref2 { *tyref = tyref.complete(); }
							} else {
								for tyref in tyref1 { *tyref = ty2; }
							}
						} else if ty2 == Type::Unknown {
							for tyref in tyref2 { *tyref = ty1; }
						}
						stack.push((Type::Bool, Vec::new()));
					},
					Op::Not | Op::Neg => (),
					_ => unreachable!(),
				}
			},
			_ => if ty.is_known() {
				stack.push((*ty, Vec::new()));
			} else {
				stack.push((Type::Unknown, vec![ty]));
			},
		}
	}
	let (ty0, tyref0) = stack.pop().unwrap();
	if ty0 == Type::Unknown {
		for tyref in tyref0 {
			*tyref = final_type;
		}
	}
}

/*pub fn assert_valid_types(block: &mut Block, scope: &mut Scope) -> Result<()> {
	Ok(())
}*/

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
	pub token: FullToken,
	pub info: ErrorType,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorType {
	AlreadyExists,
	DoesntExist,
	TooFewOperands(u32),
	Expected(Type, Type), // expected to be mergeable
	ExpectedSign,         // expected to be negativable
	EmptyExpr,
	TypeNotDetermined,
}
impl Error {
	pub fn already_exists(  token: &FullToken) -> Error {
		Error { token: token.clone(), info: ErrorType::AlreadyExists }
	}
	pub fn doesnt_exist(    token: &FullToken) -> Error {
		Error { token: token.clone(), info: ErrorType::DoesntExist }
	}
	pub fn too_few_operands(token: &FullToken, num: u32) -> Error {
		Error { token: token.clone(), info: ErrorType::TooFewOperands(num) }
	}
	pub fn expected(       token: &FullToken, a: Type, b: Type) -> Error {
		Error { token: token.clone(), info: ErrorType::Expected(a, b) }
	}
	pub fn expected_sign(  token: &FullToken) -> Error {
		Error { token: token.clone(), info: ErrorType::ExpectedSign }
	}
	pub fn empty_expr() -> Error {
		Error { token: FullToken::none(0, 0), info: ErrorType::EmptyExpr }
	}
	pub fn type_not_determined(token: &FullToken) -> Error {
		Error { token: token.clone(), info: ErrorType::TypeNotDetermined }
	}
}
impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		try!(f.write_fmt(format_args!("At {}:{} -- ", self.token.line, self.token.column)));
		let my_token = &self.token.token;
		match self.info {
			ErrorType::AlreadyExists => {
				try!(f.write_fmt(format_args!("Redeclaration of '{}'.", my_token)))
			},
			ErrorType::DoesntExist => {
				try!(f.write_fmt(format_args!("Use of undeclared variable '{}'.", my_token)))
			},
			ErrorType::TooFewOperands(_) => {
				try!(f.write_fmt(format_args!("Misused operator '{}'", my_token)))
			},
			ErrorType::Expected(ref type1, ref type2) => {
				try!(f.write_fmt(format_args!("Types do not match: '{}' & '{}'", type1, type2)))
			},
			ErrorType::ExpectedSign => try!(f.write_str("Expected signed type.")),
			ErrorType::EmptyExpr    => try!(f.write_str("Expected expression.")),
			ErrorType::TypeNotDetermined => {
				try!(f.write_fmt(format_args!("Could not determine type: '{}'.", my_token)))
			},
		}
		Ok(())
	}
}
pub type Result<T> = result::Result<T, Error>;

// type_of
