# .cargo/config sets our argv
# $0 = this file
# $1 = bits
# $2 = architecture

# Check if drive.img is not zeroed
if [ `cat drive.img` != "" ]; then
	# Zero it
	dd if=/dev/zero of=drive.img count=1K bs=512
fi

export ARCH=$2
export BITS=$1


if [ $GDB == "yes" ]; then
	lxterminal -e 'riscv'$BITS'-elf-gdb target/'$ARCH'-unknown-none-elf/debug/rust-0bsd-riscv-kernel\
	-ex "target remote localhost:1234"\
	-ex "break rust_0bsd_riscv_kernel::panic"
	'
	export QEMUOPTS=-s $QEMUOPTS
fi

qemu-system-riscv$BITS -s \
	-machine virt \
	-cpu rv$BITS \
	-serial stdio\
	-d unimp,guest_errors,int \
	-blockdev driver=file,filename=drive.img,node-name=hda \
	-device virtio-blk-device,drive=hda \
	-smp 1 \
	-m 128M \
	-kernel $3