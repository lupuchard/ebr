//! Output of the LLVM bitcode format.

use super::prelude::*;

extern "C" {
    /// Write a module to the specified path.
    ///
    /// Returns 0 on success.
    pub fn LLVMWriteBitcodeToFile(M: LLVMModuleRef,
                                  Path: *const i8)
     -> i32;
    /// Write a module to an open file descriptor.
    ///
    /// Returns 0 on success.
    pub fn LLVMWriteBitcodeToFD(M: LLVMModuleRef, FD: i32,
                                ShouldClose: i32,
                                Unbuffered: i32) -> i32;
    #[deprecated(reason="Use LLVMWriteBitcodeToFD")]
    pub fn LLVMWriteBitcodeToFileHandle(M: LLVMModuleRef,
                                        Handle: i32)
     -> i32;
    /// Writes a module to a new memory buffer.
    pub fn LLVMWriteBitcodeToMemoryBuffer(M: LLVMModuleRef)
     -> LLVMMemoryBufferRef;
}
