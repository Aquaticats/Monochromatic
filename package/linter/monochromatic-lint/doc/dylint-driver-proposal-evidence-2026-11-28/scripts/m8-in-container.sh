#!/usr/bin/env bash
# M8 (round of 2026-11-28, second round): option I measured alone.
# The floating channel carries the compiler this host's floating channel carries
# (rustc 1.102.0-nightly (5df8c02af 2026-11-16), the nightly of 2026-11-17) and the components,
# the library has no toolchain file, and the rustup proxy is first for `cargo` and `rustc`.
set -uo pipefail

hostlike=nightly-2026-11-17-x86_64-unknown-linux-gnu
log=/work/log
mkdir -p "$log" /work/tmp
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp
export PATH=/usr/bin:/usr/sbin:/usr/local/bin:/bin

echo "== clock $(date --iso-8601=seconds)"
echo "== toolchains before"; rustup toolchain list | tee "$log/toolchains-before.txt"

# The floating channel is made to hold the build the host's floating channel holds: rustup names a
# toolchain by its directory, so the dated install of 2026-11-17 is copied over the floating name.
echo "== making the floating channel the 2026-11-17 build"
rm -rf /root/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu
cp -a "/root/.rustup/toolchains/$hostlike" /root/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu
rustup default nightly > "$log/default-set.txt" 2>&1; echo "rustup default status $?"
echo "== toolchains after"; rustup toolchain list | tee "$log/toolchains-after.txt"
echo "== active toolchain"; rustup show active-toolchain | tee "$log/active.txt"
echo "== floating rustc"; rustup run nightly rustc --version | tee "$log/floating-rustc-version.txt"
echo "== floating components"; rustup component list --installed --toolchain nightly | tee "$log/floating-components.txt"
echo "== cargo $(command -v cargo) -> $(readlink -f "$(command -v cargo)")"
echo "== rustc $(command -v rustc) -> $(readlink -f "$(command -v rustc)")"
echo "== sysroot the linter's probe reads, from the linter's working directory"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN rustc --print sysroot ) | tee "$log/probe-sysroot.txt"
echo "== the toolchain name Dylint computes, in the library directory (no toolchain file)"
( cd /work/repo/lint-closure && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain ) | tee "$log/library-active-toolchain.txt"
echo "== no driver before the first arm"; find /root/.dylint_drivers -maxdepth 3 2>&1 | head -3

echo "== arm A: the linter, no driver, the original library"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/a-linter-before-driver.jsonl" 2> "$log/a-linter-before-driver.err"
echo "arm A status $?"
head -c 700 "$log/a-linter-before-driver.jsonl"; echo

echo "== arm B: the warm run that builds the library and the driver for the floating name"
SECONDS=0
( cd /work/repo && env --unset=CARGO_NET_OFFLINE strace -f -qq -e trace=execve -o "$log/b-warm-execve.log" cargo dylint --path lint-closure --manifest-path pkg/Cargo.toml -- --locked ) > "$log/b-warm.out" 2> "$log/b-warm.err"
echo "arm B status $? after ${SECONDS}s"
grep -n 'Building driver\|driver for toolchain' "$log/b-warm.err" | tee "$log/b-building-driver.txt"
grep -n 'Finished' "$log/b-warm.err" | tee "$log/b-finished.txt"
tail -4 "$log/b-warm.err"
echo "--- git executions in arm B"; grep -c 'execve("/usr/bin/git"' "$log/b-warm-execve.log" || true; grep -c 'execve("/usr/sbin/git"' "$log/b-warm-execve.log" || true
echo "--- rustc executions in arm B"; grep -o 'execve("[^"]*rustc"' "$log/b-warm-execve.log" | sort | uniq -c | sort -rn | head -5
echo "--- driver tree"; find /root/.dylint_drivers -maxdepth 3 | tee "$log/b-driver-tree.txt"
echo "--- driver version"; find /root/.dylint_drivers -name dylint-driver -exec {} -V \; | tee "$log/b-driver-version.txt"
echo "--- driver dynamic section"; find /root/.dylint_drivers -name dylint-driver -exec readelf --dynamic {} \; | grep -E 'RPATH|RUNPATH|NEEDED' | tee "$log/b-driver-readelf.txt"
echo "--- findings in the warm run"; grep -c 'avoidable_closure' "$log/b-warm.out" || true

echo "== arm C: the linter, offline, driver present, no exported toolchain"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/c-linter-after-driver.jsonl" 2> "$log/c-linter-after-driver.err"
echo "arm C status $?"
cat "$log/c-linter-after-driver.jsonl"
echo "--- stderr"; cat "$log/c-linter-after-driver.err"

echo "== arm D: the same with --max-warnings 0"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN /work/monochromatic-lint --config monochromatic-lint.config.jsonc --max-warnings 0 pkg ) > "$log/d-linter-max-warnings.jsonl" 2> "$log/d-linter-max-warnings.err"
echo "arm D status $?"
head -c 300 "$log/d-linter-max-warnings.jsonl"; echo
echo "== done $(date --iso-8601=seconds)"
