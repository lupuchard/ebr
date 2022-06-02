use std::{fmt, result};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Type {
	Invalid,
	Unknown,
	Num(NumType),
	Bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumType {
	Unknown,
	Signed(SignedType),
	Unsigned(UnsignedType),
	Float(FloatType),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SignedType   { Unknown, I8, I16, I32, I64 }
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnsignedType { Unknown, U8, U16, U32, U64 }
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloatType    { Unknown,          F32, F64 }
impl Type {
	pub fn from_str(string: &str) -> Type {
		// TODO: to upper case
		match string {
			"U8"          => Type::Num(NumType::Unsigned(UnsignedType::U8)),
			"U16"         => Type::Num(NumType::Unsigned(UnsignedType::U16)),
			"U32"         => Type::Num(NumType::Unsigned(UnsignedType::U32)),
			"U64"         => Type::Num(NumType::Unsigned(UnsignedType::U64)),
			"I8"          => Type::Num(NumType::Signed(    SignedType::I8)),
			"I16"         => Type::Num(NumType::Signed(    SignedType::I16)),
			"I32" | "Int" => Type::Num(NumType::Signed(    SignedType::I32)),
			"I64"         => Type::Num(NumType::Signed(    SignedType::I64)),
			"F32"         => Type::Num(NumType::Float(      FloatType::F32)),
			"F64" |"Float"=> Type::Num(NumType::Float(      FloatType::F64)),
			"Bool"        => Type::Bool,
			_             => Type::Invalid,
		}
	}

	/// Merges the types together.
	/// If they cannot be the same, this returns invalid.
	pub fn merge(self, right: Type) -> Type {
		match self.merge_left_to_right(right) {
			Type::Invalid => right.merge_left_to_right(self),
			ty            => ty,
		}
	}
	fn merge_left_to_right(self, right: Type) -> Type {
		match self {
			Type::Unknown      => right,
			_ if self == right => right,
			Type::Num(num) => match num {
				NumType::Unknown => match right {
					Type::Num(_) => right,
					_ => Type::Invalid,
				},
				NumType::Unsigned(_) => match right {
					Type::Num(NumType::Unsigned(_)) => right,
					_ => Type::Invalid,
				},
				NumType::Signed(_) => match right {
					Type::Num(NumType::Signed(_)) => right,
					_ => Type::Invalid,
				},
				NumType::Float(_) => match right {
					Type::Num(NumType::Float(_)) => right,
					_ => Type::Invalid,
				},
			},
			_ => Type::Invalid,
		}
	}

	/// Completes a type.
	/// If it is already known, nothing changes.
	/// If it is completely unknown, it becomes invalid.
	/// If it is partially known, it can revert to defaults.
	pub fn complete(self) -> Type {
		match self {
			Type::Unknown  => Type::Invalid,
			Type::Num(num) => Type::Num(match num {
				NumType::Unknown                          => NumType::Signed(    SignedType::I32),
				NumType::Unsigned(UnsignedType::Unknown)  => NumType::Unsigned(UnsignedType::U32),
				NumType::Signed(    SignedType::Unknown)  => NumType::Signed(    SignedType::I32),
				NumType::Float(      FloatType::Unknown)  => NumType::Float(      FloatType::F64),
				_ => num,
			}),
			_ => self,
		}
	}

	pub fn is_known(&self) -> bool {
		match* self {
			Type::Unknown => false,
			Type::Num(NumType::Unknown) => false,
			Type::Num(NumType::Unsigned(UnsignedType::Unknown)) => false,
			Type::Num(NumType::Signed(    SignedType::Unknown)) => false,
			Type::Num(NumType::Float(      FloatType::Unknown)) => false,
			_ => true,
		}
	}
}
impl fmt::Display for Type {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		match *self {
			Type::Invalid                                      => f.write_str("Invalid"),
			Type::Unknown                                      => f.write_str("Unknown"),
			Type::Bool                                         => f.write_str("Bool"),
			Type::Num(NumType::Unknown)                        => f.write_str("Number"),
			Type::Num(NumType::Signed(    SignedType::Unknown))=> f.write_str("Signed"),
			Type::Num(NumType::Signed(    SignedType::I8 ))    => f.write_str("I8"),
			Type::Num(NumType::Signed(    SignedType::I16))    => f.write_str("I16"),
			Type::Num(NumType::Signed(    SignedType::I32))    => f.write_str("I32"),
			Type::Num(NumType::Signed(    SignedType::I64))    => f.write_str("I64"),
			Type::Num(NumType::Unsigned(UnsignedType::Unknown))=> f.write_str("Unsigned"),
			Type::Num(NumType::Unsigned(UnsignedType::U8 ))    => f.write_str("U8"),
			Type::Num(NumType::Unsigned(UnsignedType::U16))    => f.write_str("U16"),
			Type::Num(NumType::Unsigned(UnsignedType::U32))    => f.write_str("U32"),
			Type::Num(NumType::Unsigned(UnsignedType::U64))    => f.write_str("U64"),
			Type::Num(NumType::Float(      FloatType::Unknown))=> f.write_str("Float"),
			Type::Num(NumType::Float(      FloatType::F32))    => f.write_str("F32"),
			Type::Num(NumType::Float(      FloatType::F64))    => f.write_str("F64"),
		}.unwrap();
		Ok(())
	}
}
