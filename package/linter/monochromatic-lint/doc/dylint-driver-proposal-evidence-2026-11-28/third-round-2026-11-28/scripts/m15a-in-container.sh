#!/usr/bin/env bash
# M15a (third round of 2026-11-28): Answer K1 alone on the same image and day as M12.
# The floating channel is reinstalled with rustup's minimal profile (so it carries no
# `rustc-dev`), the library holds a file naming the lock's dated toolchain with its components,
# and the proxy is first for `cargo` and `rustc`.
set -uo pipefail

short=nightly-2026-11-16
log=/work/log
mkdir -p "$log" /work/tmp
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp
export PATH=/usr/bin:/usr/sbin:/usr/local/bin:/bin

echo "== clock $(date --iso-8601=seconds)"
echo "== image $(cat /work/image.txt)"
echo "== cargo $(command -v cargo) -> $(readlink -f "$(command -v cargo)")"
echo "== rustc $(command -v rustc) -> $(readlink -f "$(command -v rustc)")"
echo "== rustup"; rustup --version 2>&1 | tail -1 | tee "$log/rustup-version.txt"
echo "== linter"; /work/monochromatic-lint --version | tee "$log/linter-version.txt"
echo "== linter digest"; sha256sum /work/monochromatic-lint | tee "$log/linter-digest.txt"
echo "== the library's file"; cat /work/repo/lint-closure/rust-toolchain.toml | tee "$log/library-file.txt"
echo "== reinstalling the floating channel with the minimal profile"
rustup toolchain uninstall nightly > "$log/uninstall.txt" 2>&1; echo "uninstall status $?"
rustup toolchain install nightly --profile minimal --no-self-update > "$log/install.txt" 2>&1; echo "install status $?"
rustup default nightly > "$log/default.txt" 2>&1; echo "default status $?"
echo "== floating components (must be the three of the minimal profile)"
rustup component list --installed --toolchain nightly | tee "$log/floating-components.txt"
echo "== floating rustc"; rustup run nightly rustc --version | tee "$log/floating-rustc.txt"
echo "== dated rustc"; rustup run "$short" rustc --version | tee "$log/dated-rustc.txt"
echo "== dated components"; rustup component list --installed --toolchain "$short" | tee "$log/dated-components.txt"
echo "== the two sysroot answers, from the linter's own working directory"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN rustc --print sysroot ) | tee "$log/probe-sysroot-unset.txt"
( cd /work/repo && RUSTUP_TOOLCHAIN=$short rustc --print sysroot ) | tee "$log/probe-sysroot-set.txt"
echo "== the toolchain name Dylint computes, in the library directory"
( cd /work/repo/lint-closure && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain ) | tee "$log/k-active-without-env.txt"
( cd /work/repo/lint-closure && RUSTUP_TOOLCHAIN=$short rustup show active-toolchain ) | tee "$log/k-active-with-env.txt"
echo "== no driver"; find /root/.dylint_drivers -maxdepth 3 2>&1 | head -3 | tee "$log/no-driver.txt"
echo "== filling the package's cache (network)"
( cd /work/repo/pkg && cargo fetch --locked ) > "$log/pkg-fetch.out" 2>&1
echo "pkg fetch status $?"

echo "== arm A: the linter, variable unset, no driver"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/a-linter-unset-before-driver.jsonl" 2> "$log/a-linter-unset-before-driver.err"
echo "arm A status $?"
head -c 500 "$log/a-linter-unset-before-driver.jsonl" | tee "$log/a-record.txt"; echo

echo "== arm B: the warm run that builds the library and the driver for the dated name"
SECONDS=0
( cd /work/repo && env --unset=CARGO_NET_OFFLINE strace -f -qq -e trace=execve -o "$log/b-warm-execve.log" cargo dylint --path lint-closure --manifest-path pkg/Cargo.toml -- --locked ) > "$log/b-warm.out" 2> "$log/b-warm.err"
echo "arm B status $? after ${SECONDS}s"
grep -n 'Building driver\|driver for toolchain' "$log/b-warm.err" | tee "$log/b-building-driver.txt"
tail -3 "$log/b-warm.err"
echo "--- git executions"; grep -c 'execve("/usr/bin/git"' "$log/b-warm-execve.log" || true
echo "--- rustc executions"; grep -o 'execve("[^"]*rustc"' "$log/b-warm-execve.log" | sort | uniq -c | sort -rn | head -5 | tee "$log/b-rustc-counts.txt"
echo "--- driver tree"; find /root/.dylint_drivers -maxdepth 3 | tee "$log/b-driver-tree.txt"
echo "--- driver version"; find /root/.dylint_drivers -name dylint-driver -exec {} -V \; | tee "$log/b-driver-version.txt"

echo "== arm C: the linter, variable set, driver present"
( cd /work/repo && RUSTUP_TOOLCHAIN=$short /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/c-linter-set-after-driver.jsonl" 2> "$log/c-linter-set-after-driver.err"
echo "arm C status $?"
cat "$log/c-linter-set-after-driver.jsonl" | tee "$log/c-record.txt"
echo "--- stderr"; cat "$log/c-linter-set-after-driver.err"

echo "== arm D: the linter, variable unset, driver present"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/d-linter-unset-after-driver.jsonl" 2> "$log/d-linter-unset-after-driver.err"
echo "arm D status $?"
head -c 500 "$log/d-linter-unset-after-driver.jsonl" | tee "$log/d-record.txt"; echo
echo "== done $(date --iso-8601=seconds)"
