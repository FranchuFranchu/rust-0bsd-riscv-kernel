# rust-kernel-test

Mostly 0BSD-licensed RV64G Rust kernel for Qemu's virt machine

Files not made by me (and possibly not 0BSD-licensed) are src/asm/trap.S and src/allocator.rs

Todo:

- [X] Fix issue with stack overflows overwriting trap frames in tasks (fixed by enlarging task stack)
- [ ] Prevent stack overflows with a guard page
- [ ] Virtio
- [ ] Virtio block driver
