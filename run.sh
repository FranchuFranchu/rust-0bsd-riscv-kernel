if [ $GDB == "yes" ]; then
	lxterminal -e 'riscv64-elf-gdb target/riscv64gc-unknown-none-elf/debug/rust-kernel-test -ex "target remote localhost:1234" '
	export QEMUOPTS=-s $QEMUOPTS
fi

qemu-system-riscv64 -s \
	-machine virt \
	-cpu rv64\
	-serial stdio\
	-d guest_errors,unimp \
	-smp 1 \
	-m 128M \
	-kernel $@