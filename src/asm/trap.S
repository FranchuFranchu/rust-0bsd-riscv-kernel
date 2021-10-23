# trap.S
# Trap handler and global context
# Steve Operating System
# Stephen Marz
# 24 February 2019
.option norvc
.altmacro
.set NUM_GP_REGS, 32  # Number of registers per context


# Use macros for saving and restoring multiple registers
.macro save_gp i, basereg=t6
	sx	x\i, ((\i)*XLEN)(\basereg)
.endm
.macro load_gp i, basereg=t6
	lx	x\i, ((\i)*XLEN)(\basereg)
.endm
.macro save_fp i, basereg=t6
	fsx	f\i, ((NUM_GP_REGS+(\i))*XLEN)(\basereg)
.endm
.macro load_fp i, basereg=t6
	flx	f\i, ((NUM_GP_REGS+(\i))*XLEN)(\basereg)
.endm

# IN: t6: Trap frame pointer
.macro store_to_trap_frame 
.endm


.macro store_volatile_registers


	# All registers are volatile here, we need to save them
	# before we do anything.
	csrrw	t6, sscratch, t6
	# csrrw will atomically swap t6 into sscratch and the olx
	# value of sscratch into t6. This is nice because we just
	# switched values and didn't destroy anything -- all atomically!
	# in cpu.rs we have a structure of:
	#  32 gp regs		0
	#  32 fp regs		256
	# We use t6 as the temporary register because it is the very
	# bottom register (x31)
	.set 	i, 0
	.rept	31
		save_gp	%i
		.set	i, i+1
	.endr

	# Save the actual t6 register, which we swapped into
	# sscratch
	mv		t5, t6
	csrr	t6, sscratch
	save_gp 31, t5

	# Restore the kernel trap frame into sscratch
	csrw	sscratch, t5

	# TODO add fp registers to trap frame
	j 1f

	csrr	t1, sstatus
	srli	t0, t1, 13
	andi	t0, t0, 3
	li		t3, 3
	bne		t0, t3, 1f
	# Save floating point registers
	.set 	i, 0
	.rept	32
		save_fp	%i, t5
		.set	i, i+1
	.endr
	
	1:
.endm

# IN: t6: Trap frame pointer
.macro load_from_trap_frame
	j 1f
	csrr	t1, sstatus
	srli	t0, t1, 13
	andi	t0, t0, 3
	li		t3, 3
	bne		t0, t3, 1f
	.set	i, 0
	.rept	32
		load_fp %i
		.set i, i+1
	.endr
1: # no_f_extension:
	# Restore all GP registers
	.set	i, 1
	.rept	31
		load_gp %i
		.set	i, i+1
	.endr
.endm

.macro restore_volatile_registers
	# Now load the trap frame back into t6
	csrr	t6, sscratch

	load_from_trap_frame
	# Since we ran this loop 31 times starting with i = 1,
	# the last one loaded t6 back to its original value.
.endm

.section .text
.global s_trap_vector
# This must be aligned by 4 since the last two bits
# of the mtvec register do not contribute to the address
# of this vector.
.align 4
s_trap_vector:
	store_volatile_registers
	# Now t5 and sscratch have the trap frame pointer
	
	# Get ready to go into Rust (trap.rs)
	# csrw	sie, zero
	
	lx 		sp, XLEN*35(t5)
	csrr	a0, sepc
	sx		a0, XLEN*32(t5)
	csrr	a1, stval
	csrr	a2, scause
	lx		a0, XLEN*32(t5)
	lx		a3, XLEN*33(t5)
	csrr	a4, sstatus
	csrr	a5, sscratch
	
	# Usually, We don't want to write into the user's stack or whomever
	# messed with us here.
	# We haven't gotten userspace to work yet, so we can assume that this interrupt was triggered in kernel mode
	
	# la		t0, KERNEL_STACK_END
	# lx		sp, 0(t0)
	call	trap_handler
	
	# When we get here, we've returned from m_trap, restore registers
	# and return.
	# m_trap will return the return address via a0.

	csrw	sepc, a0
	
	restore_volatile_registers

trap_exit:
	sret

.global switch_to_supervisor_frame	
switch_to_supervisor_frame:
	// Load the trap frame
	csrw sscratch, a0
	// When SRET is executed, set PC to the old PC
	lx t0, XLEN*32(a0)
	csrw sepc, t0
	// If this process got interrupted, it means interrupts were enabled
	li t0, 0x222
	csrw sie, t0
	
	restore_volatile_registers
.global switch_to_supervisor_frame_end
switch_to_supervisor_frame_end:
	sret


# Essentially like s_trap_vector, but smode-to-smode
do_syscall_internal:
	
	store_volatile_registers
	# Now t5 and sscratch have the trap frame pointer
	
	# Get ready to go into Rust (trap.rs)
	# csrw	sie, zero
	
	la 		sp, _stack_start
	mv	a0, ra
	sx		a0, XLEN*32(t5)
	li		a1, 0
	csrr	a2, 9
	lx		a0, XLEN*32(t5)
	lx		a3, XLEN*33(t5)
	csrr	a4, sstatus
	csrr	a5, sscratch
	
	# Usually, We don't want to write into the user's stack or whomever
	# messed with us here.
	# We haven't gotten userspace to work yet, so we can assume that this interrupt was triggered in kernel mode
	
	# la		t0, KERNEL_STACK_END
	# lx		sp, 0(t0)
	call	trap_handler
	
	# When we get here, we've returned from m_trap, restore registers
	# and return.
	# m_trap will return the return address via a0.

	mv	ra, a0
	
	restore_volatile_registers
	ret

.global store_to_trap_frame

store_to_trap_frame:
	csrrw a0, sscratch, a0
	store_volatile_registers
	csrrw a0, sscratch, a0
	ret