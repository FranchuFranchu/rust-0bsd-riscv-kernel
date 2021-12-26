
.set stack_end, 0x80000000
.set stack_size, 0x4000
.set stack_start, (0x80000000 - stack_size)

.extern main
.global _start
_start:
# Allocate a stack
# by using the AllocPages syscall
li a7, 3
li a0, stack_start
li a1, stack_size
li a2, 1 << 1 | 1 << 2 # R + W

ecall

li sp, stack_end

call main

# Exit
li a7, 1
ecall

.global __libc_fini_array
__libc_fini_array: