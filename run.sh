# Check if drive.img is not zeroed
if [ `cat drive.img` != "" ]; then
	# Zero it
	dd if=/dev/zero of=drive.img count=1K bs=512
fi

if [ $GDB == "yes" ]; then
	lxterminal -e 'riscv64-elf-gdb target/riscv64gc-unknown-none-elf/debug/rust-kernel-test\
	-ex "target remote localhost:1234"\
	-ex "break rust_kernel_test::panic"
	'
	export QEMUOPTS=-s $QEMUOPTS
fi

qemu-system-riscv64 -s \
	-machine virt \
	-cpu rv64\
	-serial stdio\
	-d unimp,guest_errors \
	-blockdev driver=file,filename=drive.img,node-name=hda \
	-device virtio-blk-device,drive=hda \
	-smp 1 \
	-m 128M \
	-kernel $@