use std::{mem, ptr};
use std::ffi::CString;

use llvm_sys::*;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::analysis::*;

fn to_cstr(bstr: &[u8]) -> *const i8 {
	let cstr = CString::new(bstr).unwrap();
	unsafe { &mem::transmute::<&[u8], &[i8]>(cstr.as_bytes_with_nul())[0] }
}

macro_rules! type_constructor {
	($func: ident, $kind: expr, $llvm: ident) => {
		pub fn $func() -> Type {
			unsafe { Type { kind: $kind, r: $llvm() } }
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct Type {
	kind: LLVMTypeKind,
	r: LLVMTypeRef,
}
impl Type {
	fn new(ty: LLVMTypeRef) -> Type {
		unsafe { Type { kind: LLVMGetTypeKind(ty), r: ty } }
	}

	type_constructor!(bool, LLVMTypeKind::LLVMIntegerTypeKind, LLVMInt1Type);
	type_constructor!(i8  , LLVMTypeKind::LLVMIntegerTypeKind, LLVMInt8Type);
	type_constructor!(i16 , LLVMTypeKind::LLVMIntegerTypeKind, LLVMInt16Type);
	type_constructor!(i32 , LLVMTypeKind::LLVMIntegerTypeKind, LLVMInt32Type);
	type_constructor!(i64 , LLVMTypeKind::LLVMIntegerTypeKind, LLVMInt64Type);
	type_constructor!(f32 , LLVMTypeKind::LLVMFloatTypeKind  , LLVMFloatType);
	type_constructor!(f64 , LLVMTypeKind::LLVMDoubleTypeKind , LLVMDoubleType);
	type_constructor!(void, LLVMTypeKind::LLVMVoidTypeKind   , LLVMVoidType);

	pub fn is_int(&self) -> bool {
		self.kind == LLVMTypeKind::LLVMIntegerTypeKind
	}
	pub fn is_real(&self) -> bool {
		match self.kind {
			LLVMTypeKind::LLVMHalfTypeKind      => true,
			LLVMTypeKind::LLVMFloatTypeKind     => true,
			LLVMTypeKind::LLVMDoubleTypeKind    => true,
			LLVMTypeKind::LLVMX86_FP80TypeKind  => true,
			LLVMTypeKind::LLVMFP128TypeKind     => true,
			LLVMTypeKind::LLVMPPC_FP128TypeKind => true,
			_ => false,
		}
	}

	pub fn function(ret: Type, params: &mut [Type]) -> Type {
		if params.is_empty() {
			unsafe {
				return Type {
					kind: LLVMTypeKind::LLVMFunctionTypeKind,
					r: LLVMFunctionType(ret.r, ptr::null_mut(), 0, 0),
				};
			}
		}
		let mut params_vec = Vec::new();
		for ref param in params {
			params_vec.push(param.r);
		}
		unsafe {
			Type {
				kind: LLVMTypeKind::LLVMFunctionTypeKind,
				r: LLVMFunctionType(ret.r, &mut params_vec[..][0], params_vec.len() as u32, 0),
			}
		}
	}
	pub fn is_function(&self) -> bool {
		self.kind == LLVMTypeKind::LLVMFunctionTypeKind
	}
}

#[derive(Copy, Clone, Debug)]
pub struct Value {
	ty: Type,
	r: LLVMValueRef,
}
impl Value {
	fn new(val: LLVMValueRef) -> Value {
		unsafe { Value { ty: Type::new(LLVMTypeOf(val)), r: val } }
	}

	pub fn const_int(ty: Type, val: u64) -> Value {
		assert!(ty.is_int());
		unsafe { Value { ty: ty, r: LLVMConstInt(ty.r, val, 0) } }
	}
	pub fn const_real(ty: Type, val: f64) -> Value {
		assert!(ty.is_real());
		unsafe { Value { ty: ty, r: LLVMConstReal(ty.r, val) } }
	}

	pub fn get_type(&self) -> Type {
		self.ty
	}
	/*pub fn set_name(&mut self, name: &[u8]) {
		unsafe { LLVMSetValueName(self.r, to_cstr(name)); }
	}

	pub fn get_param(&mut self, index: u32) -> Value {
		assert!(self.ty.is_function());
		unsafe {
			assert!(index < LLVMCountParams(self.r));
			Value::new(LLVMGetParam(self.r, index))
		}
	}*/

	pub fn append_basic_block(&mut self, name: &[u8]) -> BasicBlock {
		unsafe {
			assert!(self.ty.is_function());
			BasicBlock { r: LLVMAppendBasicBlock(self.r, to_cstr(name)) }
		}
	}
}

/*pub struct Context { r: LLVMContextRef }
impl Context {
	pub fn new() -> Context {
		unsafe { Context { r: LLVMContextCreate() } }
	}
}
impl Drop for Context {
	fn drop(&mut self) {
		unsafe { LLVMContextDispose(self.r) };
	}
}*/

pub struct Module { r: LLVMModuleRef }
impl Module {
	/// Creates a new LLVM Module with the given name and context.
	pub fn new(name: &[u8]) -> Module {
		unsafe { Module { r: LLVMModuleCreateWithName(to_cstr(name)) } }
	}
	pub fn verify(&mut self) -> bool {
		unsafe {
			LLVMVerifyModule(self.r,
				LLVMVerifierFailureAction::LLVMPrintMessageAction,
				ptr::null_mut()) == 0
		}
	}
	pub fn print(&mut self, filename: &[u8]) -> bool {
		unsafe {
			LLVMPrintModuleToFile(self.r, to_cstr(filename), ptr::null_mut()) == 0
		}
	}
	pub fn add_function(&mut self, name: &[u8], func: Type) -> Value {
		assert_eq!(func.kind, LLVMTypeKind::LLVMFunctionTypeKind);
		unsafe {
			let val = LLVMAddFunction(self.r, to_cstr(name), func.r);
			LLVMSetFunctionCallConv(val, LLVMCallConv::LLVMFastCallConv as u32);
			Value { ty: func, r: val }
		}
	}
}
impl Drop for Module {
	fn drop(&mut self) {
		unsafe { LLVMDisposeModule(self.r) }
	}
}

pub struct BasicBlock { r: LLVMBasicBlockRef }
impl BasicBlock {
	pub fn insert(&mut self, name: &[u8]) -> BasicBlock {
		unsafe { BasicBlock { r: LLVMInsertBasicBlock(self.r, to_cstr(name)) } }
	}
}

pub const INT_EQ:    LLVMIntPredicate =  LLVMIntPredicate::LLVMIntEQ;
pub const INT_NE:    LLVMIntPredicate =  LLVMIntPredicate::LLVMIntNE;
pub const INT_UGT:   LLVMIntPredicate =  LLVMIntPredicate::LLVMIntUGT;
pub const INT_UGE:   LLVMIntPredicate =  LLVMIntPredicate::LLVMIntUGE;
pub const INT_ULT:   LLVMIntPredicate =  LLVMIntPredicate::LLVMIntULT;
pub const INT_ULE:   LLVMIntPredicate =  LLVMIntPredicate::LLVMIntULE;
pub const INT_SGT:   LLVMIntPredicate =  LLVMIntPredicate::LLVMIntSGT;
pub const INT_SGE:   LLVMIntPredicate =  LLVMIntPredicate::LLVMIntSGE;
pub const INT_SLT:   LLVMIntPredicate =  LLVMIntPredicate::LLVMIntSLT;
pub const INT_SLE:   LLVMIntPredicate =  LLVMIntPredicate::LLVMIntSLE;
pub const REAL_OEQ: LLVMRealPredicate = LLVMRealPredicate::LLVMRealOEQ;
pub const REAL_OGT: LLVMRealPredicate = LLVMRealPredicate::LLVMRealOGT;
pub const REAL_OGE: LLVMRealPredicate = LLVMRealPredicate::LLVMRealOGE;
pub const REAL_OLT: LLVMRealPredicate = LLVMRealPredicate::LLVMRealOLT;
pub const REAL_OLE: LLVMRealPredicate = LLVMRealPredicate::LLVMRealOLE;
pub const REAL_ONE: LLVMRealPredicate = LLVMRealPredicate::LLVMRealONE;

macro_rules! builder_binop {
	($func: ident, $llvm: ident) => {
		pub fn $func(&mut self, lhs: Value, rhs: Value, name: &[u8]) -> Value {
			unsafe { Value::new($llvm(self.r, lhs.r, rhs.r, to_cstr(name))) }
		}
	}
}
macro_rules! builder_unop {
	($func: ident, $llvm: ident) => {
		pub fn $func(&mut self, val: Value, name: &[u8]) -> Value {
			unsafe { Value::new($llvm(self.r, val.r, to_cstr(name))) }
		}
	}
}
pub struct Builder { r: LLVMBuilderRef }
impl Builder {
	pub fn new(block: BasicBlock) -> Builder {
		unsafe {
			let builder = Builder { r: LLVMCreateBuilder() };
			LLVMPositionBuilderAtEnd(builder.r, block.r);
			builder
		}
	}

	pub fn ret_void(&mut self) -> Value {
		unsafe { Value::new(LLVMBuildRetVoid(self.r)) }
	}
	pub fn ret(&mut self, val: Value) -> Value {
		unsafe { Value::new(LLVMBuildRet(self.r, val.r)) }
	}

	builder_binop!(nsw_add, LLVMBuildNSWAdd);
	builder_binop!(nuw_add, LLVMBuildNUWAdd);
	builder_binop!(  f_add, LLVMBuildFAdd);
	builder_binop!(nsw_sub, LLVMBuildNSWSub);
	builder_binop!(nuw_sub, LLVMBuildNUWSub);
	builder_binop!(  f_sub, LLVMBuildFSub);
	builder_binop!(nsw_mul, LLVMBuildNSWMul);
	builder_binop!(nuw_mul, LLVMBuildNUWMul);
	builder_binop!(  f_mul, LLVMBuildFMul);
	builder_binop!(  s_div, LLVMBuildSDiv);
	builder_binop!(  u_div, LLVMBuildUDiv);
	builder_binop!(  f_div, LLVMBuildFDiv);
	builder_binop!(  s_rem, LLVMBuildSRem);
	builder_binop!(  u_rem, LLVMBuildURem);
	builder_binop!(  f_rem, LLVMBuildFRem);
	builder_binop!(    and, LLVMBuildAnd);
	builder_binop!(     or, LLVMBuildOr);

	builder_unop!(    not, LLVMBuildNot);
	builder_unop!(nsw_neg, LLVMBuildNSWNeg);
	builder_unop!(nuw_neg, LLVMBuildNUWNeg);
	builder_unop!(  f_neg, LLVMBuildFNeg);

	pub fn i_cmp(&mut self, op: LLVMIntPredicate,   l: Value, r: Value, name: &[u8]) -> Value {
		unsafe { Value::new(LLVMBuildICmp(self.r, op, l.r, r.r, to_cstr(name))) }
	}
	pub fn f_cmp(&mut self, op: LLVMRealPredicate, l: Value, r: Value, name: &[u8]) -> Value {
		unsafe { Value::new(LLVMBuildFCmp(self.r, op, l.r, r.r, to_cstr(name))) }
	}

	pub fn alloca(&mut self, ty: Type, name: &[u8]) -> Value {
		unsafe { Value::new(LLVMBuildAlloca(self.r, ty.r, to_cstr(name))) }
	}
	pub fn load(&mut self, ptr: Value, name: &[u8]) -> Value {
		assert!(ptr.ty.kind == LLVMTypeKind::LLVMPointerTypeKind);
		unsafe { Value::new(LLVMBuildLoad(self.r, ptr.r, to_cstr(name))) }
	}
	pub fn store(&mut self, val: Value, ptr: Value) -> Value {
		assert!(ptr.ty.kind == LLVMTypeKind::LLVMPointerTypeKind);
		unsafe { Value::new(LLVMBuildStore(self.r, val.r, ptr.r)) }
	}
}
impl Drop for Builder {
	fn drop(&mut self) {
		unsafe { LLVMDisposeBuilder(self.r) }
	}
}
