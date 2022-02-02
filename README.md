# rust-0bsd-riscv-kernel

Mostly 0BSD-licensed `RV64GC` and `RV32IMAC` Rust kernel for Qemu's `virt` machine

Files not made by me (and possibly not 0BSD-licensed) are `kernel/kernel_main/asm/trap.S` , `kernel/kernel_main/allocator/linkedlist.rs`, and `kernel/flat_bytes/`

Build and run with `./build_userspace_and_run.sh`

Stop QEMU with Ctrl-A and then X

## Todo

- [X] Fix issue with stack overflows overwriting trap frames in tasks (fixed by enlarging task stack)
- [X] A way to handle what happens when a process holds a lock and an interrupt triggers, and the interrupt handler also tries to lock the same lock (for example in process.rs)
- [ ] Prevent stack overflows with a guard page
- [X] Virtio
- [X] Virtio block driver
- [X] Refactor VirtioDevice interrupt API, so that instead of calling <interrupt handler> -> <VirtioDeviceType> -> <VirtioDevice> -> <Waker> -> <VirtioDeviceType>, it skips the first VirtioDeviceType step
- [X] Fix bug where process trap frame would still be used after removing the process. This doesn't cause bugs until new allocations are made. (use-after-free)
- [ ] Make the future.rs executor code cleaner
- [X] Kernel locks for kernel interrupt contexts, mixed contexts, and thread contexts (just missing performant thread-only locks that use wakers)
- [X] Userspace processes
- [ ] Elf loader: Add support for multiple segments in the same page

## Tips

### On Performance

I am not trying to achieve maximum performance for this kernel, and the kernel itself feels sluggish when doing many operations like reading a file from the disk. I thought that this slowdown might be because of the emulation. So I compared QEMU with this kernel's allocator and my own machine's performance when running a simple Sieve of Eratosthenes program (for all numbers from 0 to 10000). 

On RISC-V the program ran in 17.840218 seconds. On my machine it ran in 1.064202 second. I don't think this completely explains the sluggishness when reading a file, so I'll have to measure I/O performance.

## Some explanations for parts of the code

### TrapFrame

`TrapFrame` is a struct which holds saved registers of the current process during interrupt handlers (and other information too). A pointer to the struct is stored in the `sscratch` CSR. Programmers have to be careful to prevent the struct from being dropped while `sscratch` still holds a pointer to it.

### HartMeta::boot_stack

`HartMeta::boot_stack` is used to own the trap frame in some sections of code (like context switching or boot) where it's possible that there is no process currently executing. This is used to prevent `sscratch` (which is usually held by the `Process` struct) from being invalid in these periods. If `sscractch` were invalid, then the `hartid` stored in it could get overwritten and many operations using it would fail.

### Interrupt context vs process (kernel thread) context

RISC-V has interrupt functionality. Some functions such as driver interrupt handlers will be executed when an interrupt happens, while others are executed as kernel threads. Kernel threads are like normal processes, so they can hold handles and they might get interrupted. If a kernel thread holds a lock and an interrupt happens before it's unlocked, and the interrupt handler also tries to lock it, then a deadlock happens. This can be avoided by using the `rust_kernel_test::lock::shared` module, which includes locks that disable interrupts when they're locked.
