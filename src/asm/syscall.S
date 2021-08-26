.global do_supervisor_syscall
do_supervisor_syscall:
	addi sp, sp, -16
	sx ra, (sp)
	mv t0, a7
	mv a7, a0
	mv a0, a1
	mv a2, a1
	mv a3, a2
	mv a4, a3
	mv a5, a4
	mv a6, a5
	mv a7, t0
	
	# Set the supervisor software interrupt pending bit (SSIP)
	csrr t0, sip
	ori t0, t0, 1 << 1
	csrw sip, t0
	
	lx ra, (sp)
	addi sp, sp, 16
	ret