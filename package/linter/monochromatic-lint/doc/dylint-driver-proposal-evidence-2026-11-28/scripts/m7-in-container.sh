#!/usr/bin/env bash
# M7 (round of 2026-11-28, second round): option K measured alone.
# The library names the dated toolchain, the floating channel carries no component any earlier
# step added, and the rustup proxy is first for `cargo` and for `rustc` — the state that a yes to
# the "proxy for the whole cargo tree" question creates.
set -uo pipefail

short=nightly-2026-11-16
log=/work/log
mkdir -p "$log" /work/tmp
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp
export PATH=/usr/bin:/usr/sbin:/usr/local/bin:/bin

echo "== clock $(date --iso-8601=seconds)"
echo "== cargo $(command -v cargo) -> $(readlink -f "$(command -v cargo)")"
echo "== rustc $(command -v rustc) -> $(readlink -f "$(command -v rustc)")"
rustup --version | head -2
echo "== floating components before anything (must be minimal)"
rustup component list --installed --toolchain nightly | tee "$log/floating-components-before.txt"
echo "== dated components"
rustup component list --installed --toolchain "$short" | tee "$log/dated-components.txt"
echo "== rustc versions"
{ rustup run nightly rustc --version; rustup run "$short" rustc --version; } | tee "$log/rustc-versions.txt"
echo "== what the linter's rustc-dev probe reads, from the linter's own working directory"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN rustc --print sysroot ) | tee "$log/probe-sysroot-unset.txt"
( cd /work/repo && RUSTUP_TOOLCHAIN="$short" rustc --print sysroot ) | tee "$log/probe-sysroot-set.txt"
echo "== the toolchain name Dylint computes, in the library directory"
( cd /work/repo/lint-closure-k && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain ) | tee "$log/k-active-without-env.txt"
( cd /work/repo/lint-closure-k && RUSTUP_TOOLCHAIN="$short" rustup show active-toolchain ) | tee "$log/k-active-with-env.txt"
echo "== no driver before the first two arms"; find /root/.dylint_drivers -maxdepth 3 2>&1 | head -3

echo "== arm A: the linter, RUSTUP_TOOLCHAIN unset, no driver"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN /work/monochromatic-lint --config monochromatic-lint-k.config.jsonc pkg ) > "$log/a-linter-unset-before-driver.jsonl" 2> "$log/a-linter-unset-before-driver.err"
echo "arm A status $?"
head -c 600 "$log/a-linter-unset-before-driver.jsonl"; echo

echo "== arm B: the linter, RUSTUP_TOOLCHAIN set, no driver"
( cd /work/repo && RUSTUP_TOOLCHAIN="$short" /work/monochromatic-lint --config monochromatic-lint-k.config.jsonc pkg ) > "$log/b-linter-set-before-driver.jsonl" 2> "$log/b-linter-set-before-driver.err"
echo "arm B status $?"
head -c 600 "$log/b-linter-set-before-driver.jsonl"; echo

echo "== arm C: the warm run that builds the driver for the dated name"
SECONDS=0
( cd /work/repo && env --unset=CARGO_NET_OFFLINE strace -f -qq -e trace=execve -o "$log/c-warm-execve.log" cargo dylint --path lint-closure-k --manifest-path pkg/Cargo.toml -- --locked ) > "$log/c-warm.out" 2> "$log/c-warm.err"
echo "arm C status $? after ${SECONDS}s"
grep -n 'Building driver\|driver for toolchain\|Finished\|warning: unused' "$log/c-warm.err" | head -6
tail -4 "$log/c-warm.err"
echo "--- git executions in arm C"; grep -c 'execve("/usr/bin/git"' "$log/c-warm-execve.log" || true; grep -c 'execve("/usr/sbin/git"' "$log/c-warm-execve.log" || true
echo "--- rustc executions in arm C"; grep -o 'execve("[^"]*rustc"' "$log/c-warm-execve.log" | sort | uniq -c | sort -rn | head -5
echo "--- driver tree"; find /root/.dylint_drivers -maxdepth 3 | tee "$log/c-driver-tree.txt"
echo "--- driver version"; find /root/.dylint_drivers -name dylint-driver -exec {} -V \; | tee "$log/c-driver-version.txt"
echo "--- driver dynamic section"; find /root/.dylint_drivers -name dylint-driver -exec readelf --dynamic {} \; | grep -E 'RPATH|RUNPATH|NEEDED' | tee "$log/c-driver-readelf.txt"
echo "--- findings in the warm run"; grep -c 'avoidable_closure' "$log/c-warm.out" || true

echo "== arm D: the linter, RUSTUP_TOOLCHAIN set, driver present"
( cd /work/repo && RUSTUP_TOOLCHAIN="$short" /work/monochromatic-lint --config monochromatic-lint-k.config.jsonc pkg ) > "$log/d-linter-set-after-driver.jsonl" 2> "$log/d-linter-set-after-driver.err"
echo "arm D status $?"
cat "$log/d-linter-set-after-driver.jsonl"
echo "--- stderr"; cat "$log/d-linter-set-after-driver.err"

echo "== arm E: the linter, RUSTUP_TOOLCHAIN unset, driver present"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN /work/monochromatic-lint --config monochromatic-lint-k.config.jsonc pkg ) > "$log/e-linter-unset-after-driver.jsonl" 2> "$log/e-linter-unset-after-driver.err"
echo "arm E status $?"
cat "$log/e-linter-unset-after-driver.jsonl"
echo "--- stderr"; cat "$log/e-linter-unset-after-driver.err"
echo "== done $(date --iso-8601=seconds)"
