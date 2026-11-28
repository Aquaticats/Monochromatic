#!/usr/bin/env bash
# M11 (round of 2026-11-28, second round): what rustup does when a rust-toolchain.toml names a
# toolchain that is not installed, and what the linter's active_toolchain therefore answers.
set -uo pipefail
log=/work/log
mkdir -p "$log" /work/tmp
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp
export PATH=/usr/bin:/usr/sbin:/usr/local/bin:/bin
echo "== clock $(date --iso-8601=seconds)"
echo "== rustup $(rustup --version | head -1)"
echo "== channel this host's plan would name, installed or not"
rustup toolchain list | tee "$log/toolchains.txt"
mkdir -p /work/probe
printf '[toolchain]\nchannel = "nightly-2026-11-20"\ncomponents = ["llvm-tools-preview", "rustc-dev"]\n' > /work/probe/rust-toolchain.toml
echo "== rustup show active-toolchain in a directory whose file names an absent toolchain"
( cd /work/probe && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain ) > "$log/show-active.out" 2> "$log/show-active.err"
echo "show-active status $?"
cat "$log/show-active.out"; cat "$log/show-active.err"
echo "== rustc --version in the same directory (does rustup install anything?)"
( cd /work/probe && env -u RUSTUP_TOOLCHAIN rustc --version ) > "$log/rustc.out" 2> "$log/rustc.err"
echo "rustc status $?"
cat "$log/rustc.out"; cat "$log/rustc.err"
echo "== toolchains after (nothing may have been added)"
rustup toolchain list | tee "$log/toolchains-after.txt"
echo "== done $(date --iso-8601=seconds)"
