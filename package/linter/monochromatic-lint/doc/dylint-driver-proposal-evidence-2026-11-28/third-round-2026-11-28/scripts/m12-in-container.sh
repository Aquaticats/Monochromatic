#!/usr/bin/env bash
# M12 (third round of 2026-11-28): Answer I on the channel's current compiler.
# The floating channel is today's `nightly` with the components Answer I installs,
# the library has no toolchain file, the rustup proxy is first for `cargo` and `rustc`,
# no driver exists, and the three caches a warm run would fill are filled first
# (so the linter's driver probe resolves the crate and reaches the library build,
# instead of skipping for a cache it cannot fill offline).
set -uo pipefail

log=/work/log
mkdir -p "$log" /work/tmp
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp
export PATH=/usr/bin:/usr/sbin:/usr/local/bin:/bin

echo "== clock $(date --iso-8601=seconds)"
echo "== image $(cat /work/image.txt)"
echo "== cargo $(command -v cargo) -> $(readlink -f "$(command -v cargo)")"
echo "== rustc $(command -v rustc) -> $(readlink -f "$(command -v rustc)")"
echo "== rustup"; rustup --version 2>&1 | tail -1 | tee "$log/rustup-version.txt"
echo "== toolchains"; rustup toolchain list | tee "$log/toolchains.txt"
echo "== floating rustc"; rustup run nightly rustc --version | tee "$log/floating-rustc.txt"
echo "== floating components"; rustup component list --installed --toolchain nightly | tee "$log/floating-components.txt"
echo "== cargo-dylint"; cargo dylint --version 2>&1 | tee "$log/cargo-dylint-version.txt"
echo "== dylint-link"; dylint-link --version 2>&1 | head -1 | tee "$log/dylint-link-version.txt"
echo "== linter"; /work/monochromatic-lint --version | tee "$log/linter-version.txt"
echo "== linter digest"; sha256sum /work/monochromatic-lint | tee "$log/linter-digest.txt"
echo "== the toolchain name Dylint computes, in the library directory (no file there)"
( cd /work/repo/lint-closure && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain ) | tee "$log/library-active-toolchain.txt"
echo "== the sysroot the linter's probe reads, from its own working directory"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN rustc --print sysroot ) | tee "$log/probe-sysroot.txt"
echo "== no driver"; find /root/.dylint_drivers -maxdepth 3 2>&1 | head -3 | tee "$log/no-driver.txt"

echo "== filling the three caches a warm run fills (network), as Answer B's step would"
SECONDS=0
( cd /work/repo/pkg && cargo fetch --locked ) > "$log/pkg-fetch.out" 2>&1
echo "pkg fetch status $?"
( cd /work/repo/lint-closure && cargo fetch --locked ) > "$log/library-fetch.out" 2>&1
echo "library fetch status $?"
mkdir -p /work/driver-crate/src
printf '%s\n' '[package]' 'name = "dylint-driver-probe"' 'version = "0.1.0"' 'edition = "2018"' '' '[workspace]' '' '[dependencies]' 'anyhow = "1.0"' 'env_logger = "0.11"' 'dylint_driver = { version = "=6.4.0" }' > /work/driver-crate/Cargo.toml
printf '%s\n' 'fn main() {}' > /work/driver-crate/src/main.rs
( cd /work/driver-crate && cargo fetch ) > "$log/driver-fetch.out" 2>&1
echo "driver fetch status $?"
( cd /work/driver-crate && cargo metadata --offline --format-version 1 > /dev/null 2> "$log/driver-metadata.err" )
echo "driver metadata offline status $?"
echo "cache filling took ${SECONDS}s"

echo "== arm A: the linter, no exported variable, no driver, the caches filled"
SECONDS=0
( cd /work/repo && env -u RUSTUP_TOOLCHAIN /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/a-linter.jsonl" 2> "$log/a-linter.err"
echo "arm A status $? after ${SECONDS}s"
wc -c "$log/a-linter.jsonl" | tee "$log/a-size.txt"
cat "$log/a-linter.jsonl" | tee "$log/a-record.txt"
echo "--- stderr"; cat "$log/a-linter.err"

echo "== arm B: the warm command itself, to keep the first compiler error"
SECONDS=0
( cd /work/repo && env --unset=CARGO_NET_OFFLINE cargo dylint --path lint-closure --manifest-path pkg/Cargo.toml -- --locked ) > "$log/b-warm.out" 2> "$log/b-warm.err"
echo "arm B status $? after ${SECONDS}s"
grep -n 'error\[E[0-9]*\]' "$log/b-warm.err" | head -5 | tee "$log/b-error-lines.txt"
tail -12 "$log/b-warm.err" | tee "$log/b-warm-tail.txt"
echo "== driver directory after arm B"; find /root/.dylint_drivers -maxdepth 3 2>&1 | head -3 | tee "$log/b-driver-tree.txt"
echo "== done $(date --iso-8601=seconds)"
