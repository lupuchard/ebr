//! Target machine information, to generate assembly or object files.

use super::prelude::*;
use super::target::LLVMTargetDataRef;

#[repr(C)]
pub struct LLVMOpaqueTargetMachine;
pub type LLVMTargetMachineRef = *mut LLVMOpaqueTargetMachine;

#[repr(C)]
pub struct LLVMTarget;
pub type LLVMTargetRef = *mut LLVMTarget;

#[repr(C)]
pub enum LLVMCodeGenOptLevel {
    LLVMCodeGenLevelNone = 0,
    LLVMCodeGenLevelLess = 1,
    LLVMCodeGenLevelDefault = 2,
    LLVMCodeGenLevelAggressive = 3
}

#[repr(C)]
pub enum LLVMRelocMode {
    LLVMRelocDefault = 0,
    LLVMRelocStatic = 1,
    LLVMRelocPIC = 2,
    LLVMRelocDynamicNoPic = 3,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum LLVMCodeModel {
    LLVMCodeModelDefault = 0,
    LLVMCodeModelJITDefault = 1,
    LLVMCodeModelSmall = 2,
    LLVMCodeModelKernel = 3,
    LLVMCodeModelMedium = 4,
    LLVMCodeModelLarge = 5,
}

#[repr(C)]
pub enum LLVMCodeGenFileType {
    LLVMAssemblyFile = 0,
    LLVMObjectFile = 1,
}

extern "C" {
    pub fn LLVMGetFirstTarget() -> LLVMTargetRef;
    pub fn LLVMGetNextTarget(T: LLVMTargetRef) -> LLVMTargetRef;
    pub fn LLVMGetTargetFromName(Name: *const i8)
     -> LLVMTargetRef;
    pub fn LLVMGetTargetFromTriple(Triple: *const i8,
                                   T: *mut LLVMTargetRef,
                                   ErrorMessage: *mut *mut i8)
     -> LLVMBool;
    pub fn LLVMGetTargetName(T: LLVMTargetRef) -> *const i8;
    pub fn LLVMGetTargetDescription(T: LLVMTargetRef)
     -> *const i8;
    pub fn LLVMTargetHasJIT(T: LLVMTargetRef) -> LLVMBool;
    pub fn LLVMTargetHasTargetMachine(T: LLVMTargetRef) -> LLVMBool;
    pub fn LLVMTargetHasAsmBackend(T: LLVMTargetRef) -> LLVMBool;
    pub fn LLVMCreateTargetMachine(T: LLVMTargetRef,
                                   Triple: *const i8,
                                   CPU: *const i8,
                                   Features: *const i8,
                                   Level: LLVMCodeGenOptLevel,
                                   Reloc: LLVMRelocMode,
                                   CodeModel: LLVMCodeModel)
     -> LLVMTargetMachineRef;
    pub fn LLVMDisposeTargetMachine(T: LLVMTargetMachineRef) -> ();
    pub fn LLVMGetTargetMachineTarget(T: LLVMTargetMachineRef)
     -> LLVMTargetRef;
    pub fn LLVMGetTargetMachineTriple(T: LLVMTargetMachineRef)
     -> *mut i8;
    pub fn LLVMGetTargetMachineCPU(T: LLVMTargetMachineRef)
     -> *mut i8;
    pub fn LLVMGetTargetMachineFeatureString(T: LLVMTargetMachineRef)
     -> *mut i8;
    pub fn LLVMGetTargetMachineData(T: LLVMTargetMachineRef)
     -> LLVMTargetDataRef;
    pub fn LLVMSetTargetMachineAsmVerbosity(T: LLVMTargetMachineRef,
                                            VerboseAsm: LLVMBool) -> ();
    pub fn LLVMTargetMachineEmitToFile(T: LLVMTargetMachineRef,
                                       M: LLVMModuleRef,
                                       Filename: *mut i8,
                                       codegen: LLVMCodeGenFileType,
                                       ErrorMessage: *mut *mut i8)
     -> LLVMBool;
    pub fn LLVMTargetMachineEmitToMemoryBuffer(T: LLVMTargetMachineRef,
                                               M: LLVMModuleRef,
                                               codegen: LLVMCodeGenFileType,
                                               ErrorMessage:
                                                   *mut *mut i8,
                                               OutMemBuf:
                                                   *mut LLVMMemoryBufferRef)
     -> LLVMBool;
    pub fn LLVMGetDefaultTargetTriple() -> *mut i8;
    pub fn LLVMAddAnalysisPasses(T: LLVMTargetMachineRef,
                                 PM: LLVMPassManagerRef) -> ();
}
