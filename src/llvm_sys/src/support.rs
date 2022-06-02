use super::prelude::*;

extern "C" {
    pub fn LLVMLoadLibraryPermanently(Filename: *const i8) -> LLVMBool;
    pub fn LLVMParseCommandLineOptions(argc: i32,
                                       argv: *const *const i8,
                                       Overview: *const i8) -> ();
}
