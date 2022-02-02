#!/bin/bash

set -e

export ARCH=$2
export BITS=$1
export FILE=`dirname $0`/../target/$2-unknown-none-elf/release/program
#doas mount -o loop `dirname $0`/../../drive.img `dirname $0`/../../drive-loopback
cp $FILE `dirname $0`/../../drive-loopback/main
echo Copied successfully
#doas umount `dirname $0`/../../drive-loopback
if mountpoint `dirname $0`/../../drive-loopback; then
    : # ... things which should happen if command's result code was 0
	sync `dirname $0`/../../drive-loopback/main
else
	echo "Error: Run \`sudo mount -o loop drive.img drive-loopback\` to mount the drive image first"
    exit 1
fi