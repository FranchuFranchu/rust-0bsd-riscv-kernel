#!/bin/bash

set -ex
cd `dirname $0`
cd userspace
cargo run --release
cd ..
cd kernel
cargo run