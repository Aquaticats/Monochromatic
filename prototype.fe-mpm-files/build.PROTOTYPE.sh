#!/bin/sh
# PROTOTYPE: build the two prototype binaries in the container; /out is a host directory mounted read-write.
set -eu
mkdir --parents /work
cp -r /proto/proto-mpm-files.PROTOTYPE /work/crate
cd /work/crate
CARGO_TARGET_DIR=/work/target cargo build --release 2>&1 | tail -25
mkdir --parents /out/bin
cp /work/target/release/proto-mpm-shadow /work/target/release/proto-file-enforcer /out/bin/
cp /work/crate/Cargo.lock /out/Cargo.lock
ls -l /out/bin
