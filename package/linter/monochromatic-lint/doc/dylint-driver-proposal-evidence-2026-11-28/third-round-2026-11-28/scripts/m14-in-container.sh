#!/usr/bin/env bash
# M14 (third round of 2026-11-28): what rustup does with a toolchain file's components,
# on the rustup this machine and the image carry, and what the linter returns when the
# toolchain a file names is absent and auto-install is off.
set -uo pipefail

log=/work/log
mkdir -p "$log" /work/tmp
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp
export PATH=/usr/bin:/usr/sbin:/usr/local/bin:/bin

echo "== clock $(date --iso-8601=seconds)"
echo "== image $(cat /work/image.txt)"
echo "== rustup"; rustup --version 2>&1 | tail -1 | tee "$log/rustup-version.txt"
echo "== toolchains at the start"; rustup toolchain list | tee "$log/toolchains-before.txt"
echo "== linter"; /work/monochromatic-lint --version | tee "$log/linter-version.txt"
echo "== linter digest"; sha256sum /work/monochromatic-lint | tee "$log/linter-digest.txt"

# ---------------------------------------------------------------------------
echo "== arm A: the linter with auto-install off and a file naming a toolchain that is absent"
mkdir -p "/work/repo-a"
cp -a /work/repo/. "/work/repo-a/"
printf '%s\n' '[toolchain]' 'channel = "nightly-2026-11-21"' 'components = ["rustc-dev", "llvm-tools-preview"]' > "/work/repo-a/lint-closure/rust-toolchain.toml"
( cd "/work/repo-a/lint-closure" && env -u RUSTUP_TOOLCHAIN RUSTUP_AUTO_INSTALL=0 rustup show active-toolchain ) > "$log/a-active.err" 2>&1
echo "a show active-toolchain status $?"
cat "$log/a-active.err"
( cd /work/repo-a/pkg && cargo fetch --locked ) > "$log/a-pkg-fetch.out" 2>&1
echo "a pkg fetch status $?"
( cd /work/repo-a && env -u RUSTUP_TOOLCHAIN RUSTUP_AUTO_INSTALL=0 /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/a-linter.jsonl" 2> "$log/a-linter.err"
echo "a linter status $?"
cat "$log/a-linter.jsonl" | tee "$log/a-record.txt"
echo "--- stderr"; cat "$log/a-linter.err"
echo "--- with --debug, the probe's own note"
( cd /work/repo-a && env -u RUSTUP_TOOLCHAIN RUSTUP_AUTO_INSTALL=0 /work/monochromatic-lint --config monochromatic-lint.config.jsonc --debug pkg ) > "$log/a-linter-debug.jsonl" 2> "$log/a-linter-debug.err"
echo "a linter --debug status $?"
cat "$log/a-linter-debug.err" | head -20

# ---------------------------------------------------------------------------
echo "== arm B: a file naming an absent toolchain, auto-install on: what gets installed"
mkdir -p /work/probe-b
printf '%s\n' '[toolchain]' 'channel = "nightly-2026-11-20"' 'components = ["rustc-dev", "llvm-tools-preview"]' > /work/probe-b/rust-toolchain.toml
( cd /work/probe-b && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain ) > "$log/b-show.txt" 2>&1
echo "b show active-toolchain status $?"
cat "$log/b-show.txt"
echo "--- the components that toolchain now holds (itemised, not counted)"
rustup component list --installed --toolchain nightly-2026-11-20 | tee "$log/b-installed-components.txt"

# ---------------------------------------------------------------------------
echo "== arm C: a present toolchain that lacks a component the file declares"
mkdir -p /work/probe-c
printf '%s\n' '[toolchain]' 'channel = "nightly-2026-11-17"' 'components = ["rustc-dev", "llvm-tools-preview"]' > /work/probe-c/rust-toolchain.toml
echo "--- components before"; rustup component list --installed --toolchain nightly-2026-11-17 | tee "$log/c-components-before.txt"
( cd /work/probe-c && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain ) > "$log/c-show.txt" 2>&1
echo "c show active-toolchain status $?"
cat "$log/c-show.txt"
echo "--- components after show"; rustup component list --installed --toolchain nightly-2026-11-17 | tee "$log/c-components-after-show.txt"
( cd /work/probe-c && env -u RUSTUP_TOOLCHAIN cargo --version ) > "$log/c-cargo.txt" 2>&1
echo "c cargo --version status $?"
cat "$log/c-cargo.txt"
echo "--- components after cargo --version"; rustup component list --installed --toolchain nightly-2026-11-17 | tee "$log/c-components-after-cargo.txt"

# ---------------------------------------------------------------------------
echo "== arm D: the supported step, rustup toolchain install with no argument, in that directory"
( cd /work/probe-c && env -u RUSTUP_TOOLCHAIN rustup toolchain install ) > "$log/d-install.txt" 2>&1
echo "d rustup toolchain install status $?"
cat "$log/d-install.txt"
echo "--- components after rustup toolchain install"; rustup component list --installed --toolchain nightly-2026-11-17 | tee "$log/d-components-after-install.txt"

echo "== toolchains at the end"; rustup toolchain list | tee "$log/toolchains-after.txt"
echo "== done $(date --iso-8601=seconds)"
