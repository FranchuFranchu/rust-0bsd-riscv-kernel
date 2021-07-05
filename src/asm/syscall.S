.global do_syscall
do_syscall:
	addi sp, sp, -16
	sd ra, (sp)
	mv t0, a7
	mv a7, a0
	mv a0, a1
	mv a2, a1
	mv a3, a2
	mv a4, a3
	mv a5, a4
	mv a6, a5
	mv a7, t0
	
	call do_syscall_internal
	
	ld ra, (sp)
	addi sp, sp, 16
	ret