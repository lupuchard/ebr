	.text
	.file	"temp.ll"
	.globl	thang_main
	.align	16, 0x90
	.type	thang_main,@function
thang_main:                             # @thang_main
	.cfi_startproc
# BB#0:                                 # %entry
	movl	$6, -4(%rsp)
	movl	$7, -8(%rsp)
	imull	$7, -4(%rsp), %eax
	movl	%eax, -4(%rsp)
	retq
.Ltmp0:
	.size	thang_main, .Ltmp0-thang_main
	.cfi_endproc


	.section	".note.GNU-stack","",@progbits
