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
	-ex "target remote cuarto.localdomain:1234"\
	-ex "break rust_0bsd_riscv_kernel::panic"\
	-ex "alias print_hartids = p [\$mhartid, rust_0bsd_riscv_kernel::cpu::load_hartid()]"\
	-ex "alias phids = print_hartids"\
	-ex "set history save on"\
	' &
	export QEMUOPTS="-S -s $QEMUOPTS"
fi

qemu-system-riscv$BITS $QEMUOPTS \
	-machine virt \
	-cpu rv$BITS \
	-chardev stdio,id=console,mux=on \
	-d unimp,guest_errors \
	-blockdev driver=file,filename=`dirname $0`/drive.img,node-name=hda \
	-device virtio-blk-device,drive=hda \
	-smp 1 \
	-serial chardev:console \
	-monitor chardev:console \
	-nographic \
	-m 128M \
	-kernel $3
