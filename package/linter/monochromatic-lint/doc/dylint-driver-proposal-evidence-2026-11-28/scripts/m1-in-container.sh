#!/usr/bin/env bash
# M1 (gate-review finding F1): the machine's toolchain state in a container, non-proxy cargo first.
# Every command runs inside the container; nothing of the account's home is mounted.
set -uo pipefail

dated_dir=/root/.rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu
dated_full=nightly-2026-11-16-x86_64-unknown-linux-gnu
short=nightly-2026-11-16
log=/work/log
mkdir -p "$log"

echo "== clock $(date --iso-8601=seconds)"
echo "== image os: $(. /etc/os-release && echo "$PRETTY_NAME")"
echo "== rustup toolchains"; rustup toolchain list
echo "== floating components"; rustup component list --installed --toolchain nightly
echo "== dated components"; rustup component list --installed --toolchain "$dated_full"
echo "== default toolchain"; rustup default

echo "== control: rustup show active-toolchain in the library dir, without and with RUSTUP_TOOLCHAIN"
( cd /work/repo/lint-closure && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain ) > "$log/active-without-env.txt" 2>&1
( cd /work/repo/lint-closure && RUSTUP_TOOLCHAIN="$short" rustup show active-toolchain ) > "$log/active-with-env.txt" 2>&1
cat "$log/active-without-env.txt"; echo "--"; cat "$log/active-with-env.txt"
echo "== control: rustup which rustc in the library dir, without and with RUSTUP_TOOLCHAIN"
( cd /work/repo/lint-closure && env -u RUSTUP_TOOLCHAIN rustup which rustc ) > "$log/which-without-env.txt" 2>&1
( cd /work/repo/lint-closure && RUSTUP_TOOLCHAIN="$short" rustup which rustc ) > "$log/which-with-env.txt" 2>&1
cat "$log/which-without-env.txt"; echo "--"; cat "$log/which-with-env.txt"

# The machine's first cargo is not the rustup proxy: a direct link into the dated toolchain,
# the shape of the repository's .cache/toolchain-bin/cargo and .cache/cargo-shadow-bin/cargo.
mkdir -p /opt/nonproxy
ln -sfn "$dated_dir/bin/cargo" /opt/nonproxy/cargo
ln -sfn "$dated_dir/bin/rustc" /opt/nonproxy/rustc
export PATH=/opt/nonproxy:/usr/local/bin:/usr/sbin:/usr/bin:/bin
export RUSTUP_TOOLCHAIN="$short"
echo "== arm F1: which cargo and rustc"
type -a cargo || true
type -a rustc || true
echo "== rustc version the PATH gives: $(rustc --version)"

echo "== arm F1: the linter over the package, offline, with no driver"
( cd /work/repo && /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/m1-linter-before.jsonl" 2> "$log/m1-linter-before.err"
echo "linter-before status $?"
cat "$log/m1-linter-before.jsonl"
echo "--- stderr"; cat "$log/m1-linter-before.err"

echo "== arm F1: the warm command, online, traced for every execve"
( cd /work/repo && strace -f -qq -e trace=execve -o "$log/m1-warm-execve.log" env --unset=CARGO_NET_OFFLINE cargo dylint --path lint-closure --manifest-path pkg/Cargo.toml -- --locked ) > "$log/m1-warm.out" 2> "$log/m1-warm.err"
echo "warm status $?"
echo "--- stderr tail"; tail -30 "$log/m1-warm.err"
echo "--- driver tree after the failure"; find /root/.dylint_drivers -maxdepth 3 2>&1 | head -20
echo "--- execve of rustup/rustc/cargo/git"; grep -E '"(rustup|rustc|cargo|git)"' "$log/m1-warm-execve.log" | sed 's/, \[.*//' | sort | uniq -c | sort -rn | head -20

echo "== the recorded workaround shape: a driver built for the dated toolchain, proxy cargo first"
export RUSTFLAGS="-C link-args=-Wl,-rpath,$dated_dir/lib "
mkdir -p /work/driver-build/src
cp /work/inputs/main.rs /work/driver-build/src/main.rs
cat > /work/driver-build/Cargo.toml <<'TOML'
[package]
name = "dylint-driver-dated"
version = "0.1.0"
edition = "2018"

[dependencies]
anyhow = "1.0"
env_logger = "0.11"
dylint_driver = { version = "=6.4.0" }
TOML
cat > /work/driver-build/rust-toolchain.toml <<TOML
[toolchain]
channel = "$dated_full"
components = ["llvm-tools-preview", "rustc-dev"]
TOML
( cd /work/driver-build && env -u RUSTUP_TOOLCHAIN PATH=/usr/local/bin:/usr/sbin:/usr/bin:/bin strace -f -qq -e trace=execve -o "$log/m1-dated-driver-execve.log" cargo build ) > "$log/m1-dated-driver.out" 2> "$log/m1-dated-driver.err"
echo "dated driver build status $?"
tail -5 "$log/m1-dated-driver.err"
echo "--- git execve in the dated driver build (the clone, F5 dated arm)"
grep -c 'execve("/usr/sbin/git"' "$log/m1-dated-driver-execve.log" || true
grep 'execve("/usr/sbin/git"' "$log/m1-dated-driver-execve.log" | head -3
echo "--- checkout directories"
find /root/.cargo/git -maxdepth 3 -type d 2>&1 | head -10
ls -la /work/driver-build/target/debug/ | grep dylint_driver || true
driver_built=$(find /work/driver-build/target/debug -maxdepth 1 -type f -name 'dylint_driver-*' | head -1)
echo "driver built: $driver_built"
install -m 755 "$driver_built" /root/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver
/root/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver -V > "$log/m1-planted-driver-version.txt" 2>&1
echo "planted driver -V: $(cat "$log/m1-planted-driver-version.txt")"
readelf --dynamic /root/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver > "$log/m1-planted-driver-readelf.txt" 2>&1
grep -E 'RPATH|RUNPATH|NEEDED' "$log/m1-planted-driver-readelf.txt" | head -10
readelf --dynamic /work/driver-build/target/debug/dylint_driver-* > "$log/m1-planted-driver-readelf-full.txt" 2>&1 || true

echo "== arm F1 with a driver planted under the name Dylint computes: which compiler runs the check"
( cd /work/repo && strace -f -qq -e trace=execve -o "$log/m1-check-execve.log" env --unset=CARGO_NET_OFFLINE cargo dylint --path lint-closure --manifest-path pkg/Cargo.toml -- --locked --message-format json ) > "$log/m1-check.out" 2> "$log/m1-check.err"
echo "check status $?"
tail -5 "$log/m1-check.err"
echo "--- findings"; grep -c 'avoidable_closure' "$log/m1-check.out" || true
echo "--- execve of rustc/cargo/rustup/git in the check"; grep -E 'execve\("/(opt/nonproxy|usr|root)' "$log/m1-check-execve.log" | sed 's/, \[.*//' | sort | uniq -c | sort -rn | head -15

echo "== arm F1: the linter again, with the planted driver"
( cd /work/repo && /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/m1-linter-after.jsonl" 2> "$log/m1-linter-after.err"
echo "linter-after status $?"
cat "$log/m1-linter-after.jsonl"
echo "--- stderr"; cat "$log/m1-linter-after.err"

echo "== done $(date --iso-8601=seconds)"
