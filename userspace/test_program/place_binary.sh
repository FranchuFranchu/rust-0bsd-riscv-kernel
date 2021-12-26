#!/bin/bash
export ARCH=$2
export BITS=$1
export FILE=`dirname $0`/../target/$2-unknown-none-elf/debug/program
doas mount -o loop `dirname $0`/../../drive.img `dirname $0`/../../drive-loopback
cp $FILE `dirname $0`/../../drive-loopback/main
echo copied
doas umount `dirname $0`/../../drive-loopback
