#!/usr/bin/env bash
# M2: the proxy-first state, the two requirements isolated, the `nightly`-channel driver build
# (F5 arm 1), the dated-name driver build (F5 arm 2) and the non-proxy driver build (F1 arm).
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
echo "== toolchains"; rustup toolchain list
echo "== floating components before"; rustup component list --installed --toolchain nightly | tee "$log/floating-components-before.txt"
echo "== control: active toolchain in the library dir without RUSTUP_TOOLCHAIN"
( cd /work/repo/lint-closure && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain ) | tee "$log/active-without-env.txt"
echo "== control: with RUSTUP_TOOLCHAIN=$short"
( cd /work/repo/lint-closure && RUSTUP_TOOLCHAIN="$short" rustup show active-toolchain ) | tee "$log/active-with-env.txt"

mkdir -p /opt/nonproxy
ln -sfn "$dated_dir/bin/cargo" /opt/nonproxy/cargo
ln -sfn "$dated_dir/bin/rustc" /opt/nonproxy/rustc

echo "== option I's change: add rustc-dev, llvm-tools-preview, rust-src to the floating nightly"
export RUSTUP_TOOLCHAIN="$short"
SECONDS=0
rustup component add rustc-dev llvm-tools-preview rust-src --toolchain nightly > "$log/component-add.txt" 2>&1
echo "component add status $? after ${SECONDS}s"
tail -4 "$log/component-add.txt"
rustup component list --installed --toolchain nightly | tee "$log/floating-components-after.txt"

echo "== isolation 1: the library build with the proxy first, sanitized environment"
SECONDS=0
( cd /work/repo/lint-closure && env -u RUSTUP_TOOLCHAIN DYLINT_BUILDING_LIBRARIES=1 strace -f -qq -e trace=execve -o "$log/lib-proxy-execve.log" cargo build --release --target-dir /work/lib-target-proxy ) > "$log/lib-proxy.out" 2> "$log/lib-proxy.err"
echo "proxy library build status $? after ${SECONDS}s"
tail -3 "$log/lib-proxy.err"
echo "--- rustc paths in the proxy library build"
grep -o 'execve("\(/[^"]*\)\?rustc"' "$log/lib-proxy-execve.log" | sort | uniq -c | head
grep -o 'execve("[^"]*rustc"' "$log/lib-proxy-execve.log" | sed 's/execve("//; s/"//' | sort | uniq -c | sort -rn | head -5

echo "== isolation 2: the same library build with the non-proxy cargo first"
SECONDS=0
( cd /work/repo/lint-closure && env -u RUSTUP_TOOLCHAIN DYLINT_BUILDING_LIBRARIES=1 PATH=/opt/nonproxy:/usr/local/bin:/usr/sbin:/usr/bin:/bin strace -f -qq -e trace=execve -o "$log/lib-nonproxy-execve.log" cargo build --release --target-dir /work/lib-target-nonproxy ) > "$log/lib-nonproxy.out" 2> "$log/lib-nonproxy.err"
echo "non-proxy library build status $? after ${SECONDS}s"
grep -E 'dylint-link|RUSTUP_TOOLCHAIN|error' "$log/lib-nonproxy.err" | head -8
echo "--- rustc paths in the non-proxy library build"
grep -o 'execve("[^"]*rustc"' "$log/lib-nonproxy-execve.log" | sed 's/execve("//; s/"//' | sort | uniq -c | sort -rn | head -5

echo "== arm F2 (both requirements met): the whole warm command, online, traced"
find /root/.dylint_drivers -maxdepth 2 2>&1 | head -3
SECONDS=0
( cd /work/repo && env --unset=CARGO_NET_OFFLINE strace -f -qq -e trace=execve -o "$log/m2-warm-execve.log" cargo dylint --path lint-closure --manifest-path pkg/Cargo.toml -- --locked ) > "$log/m2-warm.out" 2> "$log/m2-warm.err"
echo "warm status $? after ${SECONDS}s"
head -5 "$log/m2-warm.err"; echo "..."; tail -8 "$log/m2-warm.err"
echo "--- findings in the warm run"; grep -c 'avoidable_closure' "$log/m2-warm.out" || true
echo "== driver tree"; find /root/.dylint_drivers -maxdepth 3 | tee "$log/driver-tree.txt"
echo "== driver version"; /root/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver -V | tee "$log/driver-version.txt"
echo "== driver rpath"; readelf --dynamic /root/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver | grep -E 'RPATH|RUNPATH|NEEDED' | tee "$log/driver-readelf.txt"
echo "== F5 nightly arm: git execve count (a clone would be one)"
grep -c 'execve("/usr/sbin/git"' "$log/m2-warm-execve.log" || true
echo "== F5 nightly arm: checkouts"; find /root/.cargo/git -maxdepth 3 -type d 2>&1 | head -3
echo "== rustc/cargo/rustup execve counts in the warm run"
grep -o 'execve("[^"]*\(rustc\|cargo\|rustup\)"' "$log/m2-warm-execve.log" | sed 's/execve("//; s/"//' | sort | uniq -c | sort -rn | head -12
echo "== the driver-build line"
grep -n 'Building driver\|driver for toolchain' "$log/m2-warm.err" | head -3

echo "== arm F2: the linter over the package, offline, with the driver present"
( cd /work/repo && /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/m2-linter.jsonl" 2> "$log/m2-linter.err"
echo "linter status $?"
cat "$log/m2-linter.jsonl"
echo "--- stderr"; cat "$log/m2-linter.err"

echo "== F5 arm 2: a driver build for the dated name (proxy cargo), the same instrument"
export RUSTFLAGS="-C link-args=-Wl,-rpath,$dated_dir/lib "
mkdir -p /work/driver-build-dated/src
cp /work/inputs/main.rs /work/driver-build-dated/src/main.rs
cat > /work/driver-build-dated/Cargo.toml <<'TOML'
[package]
name = "dylint-driver-dated"
version = "0.1.0"
edition = "2018"

[dependencies]
anyhow = "1.0"
env_logger = "0.11"
dylint_driver = { version = "=6.4.0" }
TOML
cat > /work/driver-build-dated/rust-toolchain.toml <<TOML
[toolchain]
channel = "$dated_full"
components = ["llvm-tools-preview", "rustc-dev"]
TOML
SECONDS=0
( cd /work/driver-build-dated && env -u RUSTUP_TOOLCHAIN strace -f -qq -e trace=execve -o "$log/m2-dated-driver-execve.log" cargo build ) > "$log/m2-dated-driver.out" 2> "$log/m2-dated-driver.err"
echo "dated driver build status $? after ${SECONDS}s"
tail -4 "$log/m2-dated-driver.err"
echo "--- git execve count in the dated driver build"
grep -c 'execve("/usr/sbin/git"' "$log/m2-dated-driver-execve.log" || true
grep 'execve("/usr/sbin/git"' "$log/m2-dated-driver-execve.log" | head -3
echo "--- checkouts and target"
find /root/.cargo/git -maxdepth 3 -type d 2>&1 | head -5
find /work/driver-build-dated/target/debug -maxdepth 1 -name 'dylint_driver-*' 2>&1
cp "$(find /work/driver-build-dated/target/debug -maxdepth 1 -type f -name 'dylint_driver-*' | head -1)" /work/dylint-driver-dated
chmod 755 /work/dylint-driver-dated
/work/dylint-driver-dated -V | tee "$log/m2-dated-driver-version.txt"
readelf --dynamic /work/dylint-driver-dated | grep -E 'RPATH|RUNPATH|NEEDED' | tee "$log/m2-dated-driver-readelf.txt"

echo "== F1 sub-arm: the same driver build with the non-proxy cargo first"
SECONDS=0
env -u RUSTUP_TOOLCHAIN PATH=/opt/nonproxy:/usr/local/bin:/usr/sbin:/usr/bin:/bin strace -f -qq -e trace=execve -o "$log/m2-nonproxy-driver-execve.log" bash -c 'cd /work/driver-build-dated && cargo build --target-dir /work/driver-build-nonproxy' > "$log/m2-nonproxy-driver.out" 2> "$log/m2-nonproxy-driver.err"
echo "non-proxy driver build status $? after ${SECONDS}s"
grep -E 'environment variable not found|RUSTUP_TOOLCHAIN|error' "$log/m2-nonproxy-driver.err" | head -6
tail -6 "$log/m2-nonproxy-driver.err"
echo "== done $(date --iso-8601=seconds)"
