#!/usr/bin/env bash
# M6: a clean driver artifact for the rpath measurement (F10), and the driver probe's blind spot
# once its offline resolve can succeed (F3/F8 option F).
set -uo pipefail

dated_full=nightly-2026-11-16-x86_64-unknown-linux-gnu
log=/work/log
mkdir -p "$log" /work/tmp
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp
echo "== clock $(date --iso-8601=seconds)"
echo "== cargo: $(readlink -f "$(command -v cargo)")"
echo "== tests: 0) control 1) clean dated driver 2) probe cache 3) probe says ready, run fails"
rustup component add rustc-dev llvm-tools-preview rust-src --toolchain nightly > "$log/component-add.txt" 2>&1 || true
rustup component list --installed --toolchain nightly | tee "$log/floating-components.txt"

echo "== a clean driver for the dated name, no inherited RUSTFLAGS"
mkdir -p /work/driver-clean/src
cp /work/inputs/main.rs /work/driver-clean/src/main.rs
cat > /work/driver-clean/Cargo.toml <<'TOML'
[package]
name = "dylint-driver-clean"
version = "0.1.0"
edition = "2018"

[dependencies]
anyhow = "1.0"
env_logger = "0.11"
dylint_driver = { version = "=6.4.0" }
TOML
cat > /work/driver-clean/rust-toolchain.toml <<TOML
[toolchain]
channel = "$dated_full"
components = ["llvm-tools-preview", "rustc-dev"]
TOML
SECONDS=0
( cd /work/driver-clean && env -u RUSTUP_TOOLCHAIN env -u RUSTFLAGS strace -f -qq -e trace=execve -o "$log/clean-execve.log" cargo build ) > "$log/clean.out" 2> "$log/clean.err"
echo "clean driver build status $? after ${SECONDS}s"
tail -3 "$log/clean.err"
cp /work/driver-clean/target/debug/dylint-driver-clean /work/dylint-driver-clean
chmod 755 /work/dylint-driver-clean
/work/dylint-driver-clean -V | tee "$log/clean-driver-version.txt"
readelf --dynamic /work/dylint-driver-clean | grep -E 'RPATH|RUNPATH|NEEDED' | tee "$log/clean-driver-readelf.txt"
echo "--- git execve count"; grep -c 'execve("/usr/sbin/git"' "$log/clean-execve.log" || true
mkdir -p "/root/.dylint_drivers/$dated_full"
install -m 755 /work/dylint-driver-clean "/root/.dylint_drivers/$dated_full/dylint-driver"
"/root/.dylint_drivers/$dated_full/dylint-driver" -V | tee "$log/planted-driver-version.txt"

echo "== fill the driver probe's own resolution offline, every platform"
mkdir -p /work/probe/src
cp /work/inputs/probe-Cargo.toml /work/probe/Cargo.toml
printf 'fn main() {}\n' > /work/probe/src/main.rs
SECONDS=0
( cd /work/probe && env --unset=CARGO_NET_OFFLINE cargo fetch ) > "$log/probe-fetch.out" 2> "$log/probe-fetch.err"
echo "probe fetch status $? after ${SECONDS}s"
tail -3 "$log/probe-fetch.err"
( cd /work/probe && env -u RUSTUP_TOOLCHAIN cargo metadata --offline --format-version 1 > "$log/probe-metadata.json" 2> "$log/probe-metadata.err" )
echo "offline metadata status $?"
head -2 "$log/probe-metadata.err"

echo "== control: the linter over the package with the K library and the clean driver planted under the dated name"
( cd /work/repo && /work/monochromatic-lint --config monochromatic-lint-k.config.jsonc pkg ) > "$log/control-k.jsonl" 2> "$log/control-k.err"
echo "control status $?; findings: $(wc --lines < "$log/control-k.jsonl")"
head -1 "$log/control-k.jsonl"

echo "== probe versus run: the original library (floating name), the probe's resolve now succeeds"
( cd /work/repo && /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/probe-vs-run.jsonl" 2> "$log/probe-vs-run.err"
echo "no-file status $?"
cat "$log/probe-vs-run.jsonl" | head -3
echo "--- stderr"; head -5 "$log/probe-vs-run.err"
echo "== done $(date --iso-8601=seconds)"
