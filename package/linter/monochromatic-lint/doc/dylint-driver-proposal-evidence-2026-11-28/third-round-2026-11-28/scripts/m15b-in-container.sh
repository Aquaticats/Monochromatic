#!/usr/bin/env bash
# M15b (third round of 2026-11-28): whether rustup's supported step adds a toolchain file's
# components to a toolchain that is present but lacks them.
set -uo pipefail

log=/work/log
mkdir -p "$log" /work/tmp
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp
export PATH=/usr/bin:/usr/sbin:/usr/local/bin:/bin

echo "== clock $(date --iso-8601=seconds)"
echo "== image $(cat /work/image.txt)"
echo "== rustup"; rustup --version 2>&1 | tail -1 | tee "$log/rustup-version.txt"

echo "== a toolchain installed with the minimal profile, and a file that declares components it lacks"
rustup toolchain install nightly-2026-11-19 --profile minimal --no-self-update > "$log/install-minimal.txt" 2>&1
echo "install minimal status $?"
tail -2 "$log/install-minimal.txt"
mkdir -p /work/probe-e
printf '%s\n' '[toolchain]' 'channel = "nightly-2026-11-19"' 'components = ["rustc-dev", "llvm-tools-preview"]' > /work/probe-e/rust-toolchain.toml
echo "--- components before"; rustup component list --installed --toolchain nightly-2026-11-19 | tee "$log/components-before.txt"
echo "== arm A: the supported step, rustup toolchain install with no argument, in that directory"
( cd /work/probe-e && env -u RUSTUP_TOOLCHAIN rustup toolchain install ) > "$log/a-install.txt" 2>&1
echo "a rustup toolchain install status $?"
cat "$log/a-install.txt"
echo "--- components after"; rustup component list --installed --toolchain nightly-2026-11-19 | tee "$log/components-after.txt"
echo "== arm B: what a bare rustup show active-toolchain does in the same directory"
( cd /work/probe-e && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain ) > "$log/b-show.txt" 2>&1
echo "b show status $?"
cat "$log/b-show.txt"
echo "== done $(date --iso-8601=seconds)"
