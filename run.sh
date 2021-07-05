GDB=


if [[ 0 ]]; then
	: #lxterminal -e 'riscv64-elf-gdb -ex "target remote localhost:1234" '
fi

qemu-system-riscv64 \
	-machine virt \
	-cpu rv64\
	-serial stdio\
	-d guest_errors,unimp,int \
	-smp 1 \
	-m 128M \
	-kernel $@