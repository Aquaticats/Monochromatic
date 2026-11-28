#!/usr/bin/env bash
# M16 (third round of 2026-11-28): Answer K1 once the lock has moved to a nightly that
# Dylint 6.4.0 cannot compile against, which today's channel already is.
# The library's file names the dated install of today's channel compiler (2026-11-27)
# with its components. The floating channel is the image's own, carrying what its
# Containerfile installs, because this arm ran in a fresh container rather than after M15a.
set -uo pipefail

lock=/work/log
mkdir -p "$lock" /work/tmp
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp
export PATH=/usr/bin:/usr/sbin:/usr/local/bin:/bin
short=nightly-2026-11-28

echo "== clock $(date --iso-8601=seconds)"
echo "== image $(cat /work/image.txt)"
echo "== installing $short with the components the lock's row names"
SECONDS=0
rustup toolchain install "$short" --profile minimal --no-self-update \
  --component rustc-dev --component llvm-tools-preview --component rust-src > "$lock/install.txt" 2>&1
echo "install status $? after ${SECONDS}s"
tail -3 "$lock/install.txt"
echo "== its components"; rustup component list --installed --toolchain "$short" | tee "$lock/dated-components.txt"
echo "== its rustc"; rustup run "$short" rustc --version | tee "$lock/dated-rustc.txt"

mkdir -p /work/repo-b
cp -a /work/repo/. /work/repo-b/
printf '%s\n' '[toolchain]' "channel = \"$short\"" 'components = ["llvm-tools-preview", "rustc-dev"]' > /work/repo-b/lint-closure/rust-toolchain.toml
echo "== the library's file"; cat /work/repo-b/lint-closure/rust-toolchain.toml | tee "$lock/library-file.txt"
echo "== the toolchain name Dylint computes"; ( cd /work/repo-b/lint-closure && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain ) | tee "$lock/active.txt"
echo "== the two sysroot answers, from the linter's own working directory"
( cd /work/repo-b && env -u RUSTUP_TOOLCHAIN rustc --print sysroot ) | tee "$lock/probe-unset.txt"
( cd /work/repo-b && RUSTUP_TOOLCHAIN=$short rustc --print sysroot ) | tee "$lock/probe-set.txt"
echo "== no driver"; find /root/.dylint_drivers -maxdepth 3 2>&1 | head -3 | tee "$lock/no-driver.txt"
echo "== filling the caches (network)"
( cd /work/repo-b/pkg && cargo fetch --locked ) > "$lock/pkg-fetch.out" 2>&1; echo "pkg fetch status $?"
( cd /work/repo-b/lint-closure && cargo fetch --locked ) > "$lock/library-fetch.out" 2>&1; echo "library fetch status $?"
mkdir -p /work/driver-crate/src
printf '%s\n' '[package]' 'name = "dylint-driver-probe"' 'version = "0.1.0"' 'edition = "2018"' '' '[workspace]' '' '[dependencies]' 'anyhow = "1.0"' 'env_logger = "0.11"' 'dylint_driver = { version = "=6.4.0" }' > /work/driver-crate/Cargo.toml
printf '%s\n' 'fn main() {}' > /work/driver-crate/src/main.rs
( cd /work/driver-crate && cargo fetch ) > "$lock/driver-fetch.out" 2>&1; echo "driver fetch status $?"

echo "== arm A: the warm command with the variable set, on the new lock's compiler"
SECONDS=0
( cd /work/repo-b && env --unset=CARGO_NET_OFFLINE RUSTUP_TOOLCHAIN=$short cargo dylint --path lint-closure --manifest-path pkg/Cargo.toml -- --locked ) > "$lock/a-warm.out" 2> "$lock/a-warm.err"
echo "arm A status $? after ${SECONDS}s"
grep -n 'error\[E[0-9]*\]' "$lock/a-warm.err" | head -3 | tee "$lock/a-error-lines.txt"
tail -6 "$lock/a-warm.err" | tee "$lock/a-warm-tail.txt"
echo "== driver directory after arm A"; find /root/.dylint_drivers -maxdepth 3 2>&1 | head -3 | tee "$lock/a-driver-tree.txt"

echo "== arm B: the linter with the variable set, the caches filled, no driver"
SECONDS=0
( cd /work/repo-b && RUSTUP_TOOLCHAIN=$short /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$lock/b-linter.jsonl" 2> "$lock/b-linter.err"
echo "arm B status $? after ${SECONDS}s"
head -c 900 "$lock/b-linter.jsonl" | tee "$lock/b-record.txt"; echo
echo "--- stderr"; cat "$lock/b-linter.err"
echo "== done $(date --iso-8601=seconds)"
