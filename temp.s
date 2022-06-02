	.text
	.file	"temp.ll"
	.globl	thang_main
	.align	16, 0x90
	.type	thang_main,@function
thang_main:                             # @thang_main
	.cfi_startproc
# BB#0:                                 # %entry
	movl	$4, -4(%rsp)
	movl	$-4, -4(%rsp)
	movl	$-4, %eax
	retq
.Ltmp0:
	.size	thang_main, .Ltmp0-thang_main
	.cfi_endproc


	.section	".note.GNU-stack","",@progbits
