#!/usr/bin/env bash
# M13 (third round of 2026-11-28): Answer K1 on the same image and the same day as M12.
# The library holds a file naming the lock's dated toolchain with its components,
# the floating channel is reduced to what rustup's minimal profile installs (as M7's was),
# the components sit on the dated toolchain, and the proxy is first for `cargo` and `rustc`.
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
echo "== linter"; /work/monochromatic-lint --version | tee "$log/linter-version.txt"
echo "== linter digest"; sha256sum /work/monochromatic-lint | tee "$log/linter-digest.txt"
echo "== the library's file"; cat /work/repo/lint-closure/rust-toolchain.toml | tee "$log/library-file.txt"
echo "== toolchains"; rustup toolchain list | tee "$log/toolchains-before.txt"
echo "== removing the components Answer I installs, so the floating channel is minimal again"
rustup component remove rustc-dev llvm-tools-preview rust-src --toolchain nightly > "$log/component-remove.txt" 2>&1
echo "component remove status $?"
echo "== floating components"; rustup component list --installed --toolchain nightly | tee "$log/floating-components.txt"
echo "== dated components"; rustup component list --installed --toolchain "$short" | tee "$log/dated-components.txt"
echo "== the two sysroot answers, from the linter's own working directory"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN rustc --print sysroot ) | tee "$log/probe-sysroot-unset.txt"
( cd /work/repo && RUSTUP_TOOLCHAIN=$short rustc --print sysroot ) | tee "$log/probe-sysroot-set.txt"
echo "== the toolchain name Dylint computes, in the library directory"
( cd /work/repo/lint-closure && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain ) | tee "$log/k-active-without-env.txt"
( cd /work/repo/lint-closure && RUSTUP_TOOLCHAIN=$short rustup show active-toolchain ) | tee "$log/k-active-with-env.txt"
echo "== no driver"; find /root/.dylint_drivers -maxdepth 3 2>&1 | head -3 | tee "$log/no-driver.txt"
echo "== filling the package's cache (network), so the offline lint and the warm run can resolve it"
( cd /work/repo/pkg && cargo fetch --locked ) > "$log/pkg-fetch.out" 2>&1
echo "pkg fetch status $?"

echo "== arm A: the linter, variable unset, no driver"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/a-linter-unset-before-driver.jsonl" 2> "$log/a-linter-unset-before-driver.err"
echo "arm A status $?"
head -c 700 "$log/a-linter-unset-before-driver.jsonl" | tee "$log/a-record.txt"; echo

echo "== arm B: the warm run that builds the library and the driver for the dated name"
SECONDS=0
( cd /work/repo && env --unset=CARGO_NET_OFFLINE strace -f -qq -e trace=execve -o "$log/b-warm-execve.log" cargo dylint --path lint-closure --manifest-path pkg/Cargo.toml -- --locked ) > "$log/b-warm.out" 2> "$log/b-warm.err"
echo "arm B status $? after ${SECONDS}s"
grep -n 'Building driver\|driver for toolchain' "$log/b-warm.err" | tee "$log/b-building-driver.txt"
grep -n 'Finished' "$log/b-warm.err" | tee "$log/b-finished.txt"
tail -4 "$log/b-warm.err"
echo "--- git executions"; grep -c 'execve("/usr/bin/git"' "$log/b-warm-execve.log" || true; grep -c 'execve("/usr/sbin/git"' "$log/b-warm-execve.log" || true
echo "--- rustc executions"; grep -o 'execve("[^"]*rustc"' "$log/b-warm-execve.log" | sort | uniq -c | sort -rn | head -5 | tee "$log/b-rustc-counts.txt"
echo "--- driver tree"; find /root/.dylint_drivers -maxdepth 3 | tee "$log/b-driver-tree.txt"
echo "--- driver version"; find /root/.dylint_drivers -name dylint-driver -exec {} -V \; | tee "$log/b-driver-version.txt"
echo "--- driver dynamic section"; find /root/.dylint_drivers -name dylint-driver -exec readelf --dynamic {} \; | grep -E 'RPATH|RUNPATH|NEEDED' | tee "$log/b-driver-readelf.txt"

echo "== arm C: the linter, variable set, driver present"
( cd /work/repo && RUSTUP_TOOLCHAIN=$short /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/c-linter-set-after-driver.jsonl" 2> "$log/c-linter-set-after-driver.err"
echo "arm C status $?"
cat "$log/c-linter-set-after-driver.jsonl" | tee "$log/c-record.txt"
echo "--- stderr"; cat "$log/c-linter-set-after-driver.err"

echo "== arm D: the linter, variable unset, driver present"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/d-linter-unset-after-driver.jsonl" 2> "$log/d-linter-unset-after-driver.err"
echo "arm D status $?"
head -c 700 "$log/d-linter-unset-after-driver.jsonl" | tee "$log/d-record.txt"; echo

echo "== arm E: the same with --max-warnings 0 (the gate form)"
( cd /work/repo && RUSTUP_TOOLCHAIN=$short /work/monochromatic-lint --config monochromatic-lint.config.jsonc --max-warnings 0 pkg ) > "$log/e-linter-max-warnings.jsonl" 2> "$log/e-linter-max-warnings.err"
echo "arm E status $?"
echo "== done $(date --iso-8601=seconds)"
