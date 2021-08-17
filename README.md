# rust-kernel-test

Mostly 0BSD-licensed RV64G Rust kernel for Qemu's virt machine

Files not made by me (and possibly not 0BSD-licensed) are src/asm/trap.S and src/allocator.rs

## Todo

- [X] Fix issue with stack overflows overwriting trap frames in tasks (fixed by enlarging task stack)
- [ ] A way to handle what happens when a process holds a lock and an interrupt triggers, and the interrupt handler also tries to lock the same lock (for example in process.rs)
- [ ] Prevent stack overflows with a guard page
- [X] Virtio
- [ ] Virtio block driver (halfway done)
- [X] Refactor VirtioDevice interrupt API, so that instead of calling <interrupt handler> -> <VirtioDeviceType> -> <VirtioDevice> -> <Waker> -> <VirtioDeviceType>, it skips the first VirtioDeviceType step
- [ ] Fix the changed_queue mess when VirtioDevice is used as a Future
- [X] Fix bug where process trap frame would still be used after removing the process. This doesn't cause bugs until new allocations are made. (use-after-free)

## Some explanations for parts of the code

### HartMeta::boot_stack

HartMeta::boot_stack is used to own the trap frame in some sections of code (like context switching or boot) where it's possible that there is no process currently executing. This is used to prevent sscratch (which is usually held by the process struct) from being invalid in these periods. If sscractch were invalid, then the hartid could get corrupted.

## Interrupt context vs process (kernel thread) context

TODO