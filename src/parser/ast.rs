use parser::FullToken;
use types::*;

// The structure of the language's abstract syntax tree.

/// Nodes are used to store extra token data along with the AST node.
#[derive(Clone, Debug, PartialEq)]
pub struct Node<T> {
	pub token: FullToken,
	pub val: T,
}
impl<T> Node<T> {
	pub fn new(val: T, token: FullToken) -> Node<T> {
		Node { token: token, val: val }
	}
	pub fn newn(val: T, line: u32, column: u32) -> Node<T> {
		Node { token: FullToken::none(line, column), val: val }
	}
}

/// Items are the outmost structure of a program.
/// A program is basically a Vec<Item>.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
	/// Constant global declaration and initialization.
	/// `const Id: [Id] = Expr`
	Const(Node<Id>, Option<Node<Id>>, Expr),

	/// Function global declaration and definition.
	/// `fn Id ( [Id: Id,]... ) [-> Id] { Block }
	Function(Node<Id>, Option<Node<Id>>, Block),
}

/// Variable identifier.
pub type Id    = String;
pub type Block = Vec<Statement>;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
	/// `Id: [Id] = Expr`
	Declaration(Node<Id>, Option<Node<Id>>, Option<Expr>),

	/// `Id = Expr`
	Assignment(Node<Id>, Expr),

	/// `if Expr { Block } [else if Expr { Block }]... [else { Block }]`
	If(Vec<Expr>, Vec<Node<Block>>, Option<Node<Block>>),

	/// `loop { Block }`
	Loop(Node<Block>),

	/// `{ Block }`
	Block(Node<Block>),

	/// `return [Expr]`
	Return(Option<Expr>),

	/// `@print id`
	Print(Id),
}

pub type Expr = Vec<Node<(ExprToken, Type)>>;

#[derive(Clone, Debug, PartialEq)]
pub enum ExprToken {
	IntLit(  u64),
	FloatLit(f64),
	BoolLit(bool),
	StringLit(String),
	Id(Id),
	Op(Op),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Op {
	Neg, Inv, Not, Add, Sub, Mul, Div, Mod, Pow, And, Or, Gt, Lt, Geq, Leq, Eq, Neq, TempParen
}
impl Op {
	pub fn return_type(&self) -> Type {
		match *self {
			Op::Neg | Op::Add | Op::Sub | Op::Mul | Op::Inv |
			Op::Not | Op::Div | Op::Mod | Op::Pow => Type::Num(NumType::Unknown),
			Op::And | Op::Or | Op::Gt | Op::Lt |
			Op::Geq | Op::Leq | Op::Eq | Op::Neq  => Type::Bool,
			Op::TempParen                         => Type::Invalid,
		}
	}
	pub fn left_assoc(&self) -> bool {
		match *self {
			Op::Neg | Op::Inv | Op::Not | Op::Pow => false,
			_ => true,
		}
	}
	pub fn prec(&self) -> i32 {
		match *self {
			Op::Neg | Op::Not | Op::Inv => 7,
			Op::Pow                     => 6,
			Op::Mul | Op::Div | Op::Mod => 5,
			Op::Add | Op::Sub           => 4,
			Op::Eq | Op::Neq | Op::Lt |
			Op::Gt | Op::Leq | Op::Geq  => 3,
			Op::And                     => 2,
			Op::Or                      => 1,
			Op::TempParen               => 0,
		}
	}
	pub fn is_binary(&self) -> bool {
		match *self {
			Op::Neg | Op::Not | Op::Inv => false,
			_ => true,
		}
	}
}
