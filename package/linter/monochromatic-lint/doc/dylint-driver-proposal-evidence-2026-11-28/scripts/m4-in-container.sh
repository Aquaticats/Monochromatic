#!/usr/bin/env bash
# M4 (gate-review finding F10): a driver's absolute rpath, and whether it loads where the builder's
# toolchain path does not exist (a container whose home is at another path).
set -uo pipefail

log=/work/log
mkdir -p "$log" /home/other
echo "== clock $(date --iso-8601=seconds)"
echo "== control 1: the driver, run where the builder's rpath exists (/root/.rustup)"
ls -la /root/.rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu/lib/librustc_driver*.so > "$log/builder-lib.txt" 2>&1; cat "$log/builder-lib.txt"
readelf --dynamic /work/dylint-driver > "$log/driver-readelf.txt" 2>&1
grep -E 'RPATH|RUNPATH|NEEDED' "$log/driver-readelf.txt" | tee "$log/driver-rpath-and-needed.txt"
HOME=/home/other /work/dylint-driver -V > "$log/control-rpath-present.txt" 2>&1
echo "control status $?"; cat "$log/control-rpath-present.txt"

echo "== move the builder's rustup home away, install the same toolchain at /opt/rustup"
mv /root/.rustup /root/.rustup-of-the-builder
export RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo
rustup toolchain install nightly-2026-11-16 --profile minimal --component rustc-dev --component llvm-tools-preview --no-self-update > "$log/install-opt-rustup.txt" 2>&1
echo "install status $?"; tail -3 "$log/install-opt-rustup.txt"
ls -la /opt/rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu/lib/librustc_driver*.so > "$log/other-path-lib.txt" 2>&1; cat "$log/other-path-lib.txt"
echo "== the same driver, run with HOME=/home/other and the toolchain at /opt/rustup"
HOME=/home/other env -u LD_LIBRARY_PATH /work/dylint-driver -V > "$log/other-path-run.txt" 2>&1
echo "other-path status $?"; cat "$log/other-path-run.txt"
echo "== positive control: the same run with LD_LIBRARY_PATH at the other toolchain's lib"
HOME=/home/other LD_LIBRARY_PATH=/opt/rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu/lib /work/dylint-driver -V > "$log/other-path-with-ld-library-path.txt" 2>&1
echo "ld-library-path status $?"; cat "$log/other-path-with-ld-library-path.txt"
echo "== done $(date --iso-8601=seconds)"
