use super::super::prelude::*;

#[repr(C)]
pub struct LLVMOpaquePassManagerBuilder;
pub type LLVMPassManagerBuilderRef = *mut LLVMOpaquePassManagerBuilder;

extern "C" {
    pub fn LLVMPassManagerBuilderCreate() -> LLVMPassManagerBuilderRef;
    pub fn LLVMPassManagerBuilderDispose(PMB: LLVMPassManagerBuilderRef)
     -> ();
    pub fn LLVMPassManagerBuilderSetOptLevel(PMB: LLVMPassManagerBuilderRef,
                                             OptLevel: u32) -> ();
    pub fn LLVMPassManagerBuilderSetSizeLevel(PMB: LLVMPassManagerBuilderRef,
                                              SizeLevel: u32)
     -> ();
    pub fn LLVMPassManagerBuilderSetDisableUnitAtATime(PMB:
                                                           LLVMPassManagerBuilderRef,
                                                       Value: LLVMBool) -> ();
    pub fn LLVMPassManagerBuilderSetDisableUnrollLoops(PMB:
                                                           LLVMPassManagerBuilderRef,
                                                       Value: LLVMBool) -> ();
    pub fn LLVMPassManagerBuilderSetDisableSimplifyLibCalls(PMB:
                                                                LLVMPassManagerBuilderRef,
                                                            Value: LLVMBool)
     -> ();
    pub fn LLVMPassManagerBuilderUseInlinerWithThreshold(PMB:
                                                             LLVMPassManagerBuilderRef,
                                                         Threshold:
                                                             u32)
     -> ();
    pub fn LLVMPassManagerBuilderPopulateFunctionPassManager(PMB:
                                                                 LLVMPassManagerBuilderRef,
                                                             PM:
                                                                 LLVMPassManagerRef)
     -> ();
    pub fn LLVMPassManagerBuilderPopulateModulePassManager(PMB:
                                                               LLVMPassManagerBuilderRef,
                                                           PM:
                                                               LLVMPassManagerRef)
     -> ();
    pub fn LLVMPassManagerBuilderPopulateLTOPassManager(PMB:
                                                            LLVMPassManagerBuilderRef,
                                                        PM:
                                                            LLVMPassManagerRef,
                                                        Internalize: LLVMBool,
                                                        RunInliner: LLVMBool)
     -> ();
}
