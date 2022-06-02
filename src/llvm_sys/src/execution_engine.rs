//! Runtime code generation and execution.

use super::ctypes;
use super::prelude::*;
use super::target::LLVMTargetDataRef;
use super::target_machine::{LLVMTargetMachineRef, LLVMCodeModel};

#[repr(C)]
pub struct LLVMOpaqueGenericValue;
#[repr(C)]
pub struct LLVMOpaqueExecutionEngine;
#[repr(C)]
pub struct LLVMOpaqueMCJITMemoryManager;

pub type LLVMGenericValueRef = *mut LLVMOpaqueGenericValue;
pub type LLVMExecutionEngineRef = *mut LLVMOpaqueExecutionEngine;
pub type LLVMMCJITMemoryManagerRef = *mut LLVMOpaqueMCJITMemoryManager;

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_snake_case)]
pub struct LLVMMCJITCompilerOptions {
    pub OptLevel: u32,
    pub CodeModel: LLVMCodeModel,
    pub NoFramePointerElim: LLVMBool,
    pub EnableFastISel: LLVMBool,
    pub MCJMM: LLVMMCJITMemoryManagerRef,
}

pub type LLVMMemoryManagerAllocateCodeSectionCallback =
    extern "C" fn(Opaque: *mut ctypes::c_void,
                  Size: usize,
                  Alignment: u32,
                  SectionID: u32,
                  SectionName: *const i8) -> *mut u8;
pub type LLVMMemoryManagerAllocateDataSectionCallback =
    extern "C" fn(Opaque: *mut ctypes::c_void,
                  Size: usize,
                  Alignment: u32,
                  SectionID: u32,
                  SectionName: *const i8,
                  IsReadOnly: LLVMBool) -> *mut u8;
pub type LLVMMemoryManagerFinalizeMemoryCallback =
    extern "C" fn(Opaque: *mut ctypes::c_void,
                  ErrMsg: *mut *mut i8) -> LLVMBool;
pub type LLVMMemoryManagerDestroyCallback =
    extern "C" fn(Opaque: *mut ctypes::c_void) -> ();

extern "C" {
    pub fn LLVMLinkInMCJIT() -> ();
    pub fn LLVMLinkInInterpreter() -> ();

    // Operations on generic values
    pub fn LLVMCreateGenericValueOfInt(Ty: LLVMTypeRef,
                                       N: u64,
                                       IsSigned: LLVMBool) -> LLVMGenericValueRef;
    pub fn LLVMCreateGenericValueOfPointer(P: *mut ctypes::c_void) -> LLVMGenericValueRef;
    pub fn LLVMCreateGenericValueOfFloat(Ty: LLVMTypeRef, N: f64) -> LLVMGenericValueRef;
    pub fn LLVMGenericValueIntWidth(GenValRef: LLVMGenericValueRef) -> u32;
    pub fn LLVMGenericValueToInt(GenVal: LLVMGenericValueRef,
                                 IsSigned: LLVMBool) -> u64;
    pub fn LLVMGenericValueToPointer(GenVal: LLVMGenericValueRef) -> *mut ctypes::c_void;
    pub fn LLVMGenericValueToFloat(TyRef: LLVMTypeRef,
                                   GenVal: LLVMGenericValueRef) -> f64;
    pub fn LLVMDisposeGenericValue(GenVal: LLVMGenericValueRef) -> ();

    // Operations on execution engines
    pub fn LLVMCreateExecutionEngineForModule(OutEE:
                                                  *mut LLVMExecutionEngineRef,
                                              M: LLVMModuleRef,
                                              OutError:
                                                  *mut *mut i8)
     -> LLVMBool;
    pub fn LLVMCreateInterpreterForModule(OutInterp:
                                              *mut LLVMExecutionEngineRef,
                                          M: LLVMModuleRef,
                                          OutError: *mut *mut i8)
     -> LLVMBool;
    pub fn LLVMCreateJITCompilerForModule(OutJIT: *mut LLVMExecutionEngineRef,
                                          M: LLVMModuleRef,
                                          OptLevel: u32,
                                          OutError: *mut *mut i8)
     -> LLVMBool;
    pub fn LLVMInitializeMCJITCompilerOptions(Options:
                                                  *mut LLVMMCJITCompilerOptions,
                                              SizeOfOptions: ctypes::size_t) -> ();

    /// Create an MCJIT execution engine for a module, with the given options.
    ///
    /// It is
    /// the responsibility of the caller to ensure that all fields in Options up to
    /// the given SizeOfOptions are initialized. It is correct to pass a smaller
    /// value of SizeOfOptions that omits some fields. The canonical way of using
    /// this is:
    ///
    /// ```c++
    /// LLVMMCJITCompilerOptions options;
    /// LLVMInitializeMCJITCompilerOptions(&options, sizeof(options));
    /// // ... fill in those options you care about
    /// LLVMCreateMCJITCompilerForModule(&jit, mod, &options, sizeof(options),
    ///                                  &error);
    /// ```
    ///
    /// Note that this is also correct, though possibly suboptimal:
    ///
    /// ```c++
    /// LLVMCreateMCJITCompilerForModule(&jit, mod, 0, 0, &error);
    /// ```
    ///
    /// 0 is returned on success, or 1 on failure.
    pub fn LLVMCreateMCJITCompilerForModule(OutJIT: *mut LLVMExecutionEngineRef,
                                            M: LLVMModuleRef,
                                            Options: *mut LLVMMCJITCompilerOptions,
                                            SizeOfOptions: ctypes::size_t,
                                            OutError: *mut *mut i8) -> LLVMBool;

    #[deprecated(reason="Use LLVMCreateExecutionEngineForModule instead")]
    pub fn LLVMCreateExecutionEngine(OutEE: *mut LLVMExecutionEngineRef,
                                     MP: LLVMModuleProviderRef,
                                     OutError: *mut *mut i8) -> LLVMBool;

    #[deprecated(reason="Use LLVMCreateInterpreterForModule instead")]
    pub fn LLVMCreateInterpreter(OutInterp: *mut LLVMExecutionEngineRef,
                                 MP: LLVMModuleProviderRef,
                                 OutError: *mut *mut i8) -> LLVMBool;

    #[deprecated(reason="Use LLVMCreateJITCompilerForModule instead")]
    pub fn LLVMCreateJITCompiler(OutJIT: *mut LLVMExecutionEngineRef,
                                 MP: LLVMModuleProviderRef,
                                 OptLevel: u32,
                                 OutError: *mut *mut i8) -> LLVMBool;

    pub fn LLVMDisposeExecutionEngine(EE: LLVMExecutionEngineRef) -> ();
    pub fn LLVMRunStaticConstructors(EE: LLVMExecutionEngineRef) -> ();
    pub fn LLVMRunStaticDestructors(EE: LLVMExecutionEngineRef) -> ();
    pub fn LLVMRunFunctionAsMain(EE: LLVMExecutionEngineRef, F: LLVMValueRef,
                                 ArgC: u32,
                                 ArgV: *const *const i8,
                                 EnvP: *const *const i8) -> i32;
    pub fn LLVMRunFunction(EE: LLVMExecutionEngineRef, F: LLVMValueRef,
                           NumArgs: u32,
                           Args: *mut LLVMGenericValueRef) -> LLVMGenericValueRef;
    pub fn LLVMFreeMachineCodeForFunction(EE: LLVMExecutionEngineRef,
                                          F: LLVMValueRef) -> ();
    pub fn LLVMAddModule(EE: LLVMExecutionEngineRef, M: LLVMModuleRef) -> ();
    pub fn LLVMAddModuleProvider(EE: LLVMExecutionEngineRef,
                                 MP: LLVMModuleProviderRef) -> ();
    pub fn LLVMRemoveModule(EE: LLVMExecutionEngineRef, M: LLVMModuleRef,
                            OutMod: *mut LLVMModuleRef,
                            OutError: *mut *mut i8) -> LLVMBool;
    #[deprecated(reason="Use LLVMRemoveModule instead")]
    pub fn LLVMRemoveModuleProvider(EE: LLVMExecutionEngineRef,
                                    MP: LLVMModuleProviderRef,
                                    OutMod: *mut LLVMModuleRef,
                                    OutError: *mut *mut i8) -> LLVMBool;
    pub fn LLVMFindFunction(EE: LLVMExecutionEngineRef,
                            Name: *const i8,
                            OutFn: *mut LLVMValueRef) -> LLVMBool;
    pub fn LLVMRecompileAndRelinkFunction(EE: LLVMExecutionEngineRef,
                                          Fn: LLVMValueRef) -> *mut ctypes::c_void;
    pub fn LLVMGetExecutionEngineTargetData(EE: LLVMExecutionEngineRef) -> LLVMTargetDataRef;
    pub fn LLVMGetExecutionEngineTargetMachine(EE: LLVMExecutionEngineRef) -> LLVMTargetMachineRef;
    pub fn LLVMAddGlobalMapping(EE: LLVMExecutionEngineRef,
                                Global: LLVMValueRef,
                                Addr: *mut ctypes::c_void) -> ();
    pub fn LLVMGetPointerToGlobal(EE: LLVMExecutionEngineRef,
                                  Global: LLVMValueRef) -> *mut ctypes::c_void;
    pub fn LLVMGetGlobalValueAddress(EE: LLVMExecutionEngineRef,
                                     Name: *const i8) -> u64;
    pub fn LLVMGetFunctionAddress(EE: LLVMExecutionEngineRef,
                                  Name: *const i8) -> u64;

    // Operations on memory managers
    /// Create a simple custom MCJIT memory manager.
    ///
    /// This memory manager can intercept allocations in a module-oblivious way. It will
    /// return NULL if any of the passed functions are NULL.
    ///
    /// `AllocateCodeSection` and `AllocateDataSection` are called to allocate blocks
    /// of memory for executable code and data, respectively. `FinalizeMemory` is called
    /// to set page permissions and flush caches, returning 0 on success and 1 on error.
    ///
    /// `Opaque` will be passed to the callbacks.
    pub fn LLVMCreateSimpleMCJITMemoryManager(Opaque: *mut ctypes::c_void,
                                              AllocateCodeSection: LLVMMemoryManagerAllocateCodeSectionCallback,
                                              AllocateDataSection: LLVMMemoryManagerAllocateDataSectionCallback,
                                              FinalizeMemory: LLVMMemoryManagerFinalizeMemoryCallback,
                                              Destroy: LLVMMemoryManagerDestroyCallback) -> LLVMMCJITMemoryManagerRef;

    pub fn LLVMDisposeMCJITMemoryManager(MM: LLVMMCJITMemoryManagerRef) -> ();
}
