#!/usr/bin/env bash
# M5: the skip-return probes, the `nightly`-channel driver build (F5 arm 1), and the only
# configuration measured to run the rule end to end: a rust-toolchain.toml naming the dated
# toolchain in the library directory (option K) with the proxy first (F1/F2/F5/F10).
set -uo pipefail

dated_dir=/root/.rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu
dated_full=nightly-2026-11-16-x86_64-unknown-linux-gnu
short=nightly-2026-11-16
log=/work/log
mkdir -p "$log" /work/tmp
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp

echo "== clock $(date --iso-8601=seconds)"
echo "== cargo: $(command -v cargo) -> $(readlink -f "$(command -v cargo)")"
echo "== rustc: $(readlink -f "$(command -v rustc)")"
echo "== rustc versions: floating $(rustup run nightly rustc --version); dated $(rustup run "$short" rustc --version)" | tee "$log/rustc-versions.txt"
echo "== floating components before"; rustup component list --installed --toolchain nightly | tee "$log/floating-components-before.txt"

echo "== skip-return probes: no driver yet, cold registry"
( cd /work/repo && /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/skip-default.jsonl" 2> "$log/skip-default.err"; echo "default status $?"
( cd /work/repo && /work/monochromatic-lint --config monochromatic-lint.config.jsonc --max-warnings 0 pkg ) > "$log/skip-max-warnings.jsonl" 2> "$log/skip-max-warnings.err"; echo "max-warnings-0 status $?"
( cd /work/repo && /work/monochromatic-lint --config monochromatic-lint.config.jsonc --quiet pkg ) > "$log/skip-quiet.jsonl" 2> "$log/skip-quiet.err"; echo "quiet status $?"
wc -c "$log/skip-default.jsonl" "$log/skip-max-warnings.jsonl" "$log/skip-quiet.jsonl"

echo "== option I's change: components on the floating nightly (needed to build a driver for the name `nightly`)"
SECONDS=0
rustup component add rustc-dev llvm-tools-preview rust-src --toolchain nightly > "$log/component-add.txt" 2>&1
echo "component add status $? after ${SECONDS}s"
rustup component list --installed --toolchain nightly | tee "$log/floating-components-after.txt"

echo "== F5 arm 1: a driver built for the channel name `nightly` (direct build, proxy first)"
mkdir -p /work/driver-nightly/src
cp /work/inputs/main.rs /work/driver-nightly/src/main.rs
cat > /work/driver-nightly/Cargo.toml <<'TOML'
[package]
name = "dylint-driver-nightly"
version = "0.1.0"
edition = "2018"

[dependencies]
anyhow = "1.0"
env_logger = "0.11"
dylint_driver = { version = "=6.4.0" }
TOML
cat > /work/driver-nightly/rust-toolchain.toml <<'TOML'
[toolchain]
channel = "nightly"
components = ["llvm-tools-preview", "rustc-dev"]
TOML
export RUSTFLAGS="-C link-args=-Wl,-rpath,/root/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib "
SECONDS=0
( cd /work/driver-nightly && env -u RUSTUP_TOOLCHAIN strace -f -qq -e trace=execve -o "$log/nightly-driver-execve.log" cargo build ) > "$log/nightly-driver.out" 2> "$log/nightly-driver.err"
echo "nightly driver build status $? after ${SECONDS}s"
tail -3 "$log/nightly-driver.err"
echo "--- git execve count"; grep -c 'execve("/usr/sbin/git"' "$log/nightly-driver-execve.log" || true
echo "--- checkouts"; find /root/.cargo/git -maxdepth 3 -type d 2>&1 | head -3
find /work/driver-nightly/target/debug -maxdepth 1 -name 'dylint_driver-*' 2>&1

echo "== option K: a rust-toolchain.toml naming the dated toolchain in the library directory"
cp -a /work/repo/lint-closure /work/repo/lint-closure-k
cat > /work/repo/lint-closure-k/rust-toolchain.toml <<TOML
[toolchain]
channel = "$short"
components = ["llvm-tools-preview", "rustc-dev"]
TOML
printf '%s\n' '[{ "files": ["**/*.rs"], "rules": { "rust/avoidable-closure": { "severity": "warn", "library": "lint-closure-k" } } }]' > /work/repo/monochromatic-lint-k.config.jsonc
echo "--- the control: what rustup reports in the library dir with the file, env removed and set"
( cd /work/repo/lint-closure-k && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain ) | tee "$log/k-active-without-env.txt"
( cd /work/repo/lint-closure-k && RUSTUP_TOOLCHAIN="$short" rustup show active-toolchain ) | tee "$log/k-active-with-env.txt"
echo "--- empty driver cache"; find /root/.dylint_drivers -maxdepth 3 2>&1 | head -3
SECONDS=0
( cd /work/repo && env --unset=CARGO_NET_OFFLINE strace -f -qq -e trace=execve -o "$log/k-warm-execve.log" cargo dylint --path lint-closure-k --manifest-path pkg/Cargo.toml -- --locked ) > "$log/k-warm.out" 2> "$log/k-warm.err"
echo "K warm status $? after ${SECONDS}s"
head -3 "$log/k-warm.err"; echo ...; tail -6 "$log/k-warm.err"
echo "--- findings"; grep -c 'avoidable_closure' "$log/k-warm.out" || true
echo "--- driver tree"; find /root/.dylint_drivers -maxdepth 3 | tee "$log/k-driver-tree.txt"
echo "--- driver version"; find /root/.dylint_drivers -name dylint-driver -exec {} -V \; | tee "$log/k-driver-version.txt"
echo "--- driver rpath"; find /root/.dylint_drivers -name dylint-driver -exec readelf --dynamic {} \; | grep -E 'RPATH|RUNPATH|NEEDED' | tee "$log/k-driver-readelf.txt"
echo "--- git execve count in the K run (the dated driver build)"; grep -c 'execve("/usr/sbin/git"' "$log/k-warm-execve.log" || true
echo "--- git clone lines"; grep 'execve("/usr/sbin/git"' "$log/k-warm-execve.log" | head -3
echo "--- rustc paths in the K run"; grep -o 'execve("[^"]*rustc"' "$log/k-warm-execve.log" | sed 's/execve("//; s/"//' | sort | uniq -c | sort -rn | head -5
echo "--- driver-build line"; grep -n 'Building driver\|driver for toolchain' "$log/k-warm.err" | head -3
cp "$(find /root/.dylint_drivers -name dylint-driver | head -1)" /work/dylint-driver-k
chmod 755 /work/dylint-driver-k

echo "== the linter, offline, with the library's toolchain file (option K)"
( cd /work/repo && /work/monochromatic-lint --config monochromatic-lint-k.config.jsonc pkg ) > "$log/k-linter.jsonl" 2> "$log/k-linter.err"
echo "K linter status $?"
cat "$log/k-linter.jsonl"
echo "--- stderr"; cat "$log/k-linter.err"

echo "== the linter, offline, with the original library (no toolchain file): probe versus run"
( cd /work/repo && /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/no-file-linter.jsonl" 2> "$log/no-file-linter.err"
echo "no-file linter status $?"
cat "$log/no-file-linter.jsonl"
echo "--- stderr"; cat "$log/no-file-linter.err"
echo "== done $(date --iso-8601=seconds)"
