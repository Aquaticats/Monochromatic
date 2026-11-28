#!/usr/bin/env bash
# M17 (third round of 2026-11-28): a `cargo` that fails its own version probe, which the linter
# reports as `ProbeFailure::Broken` instead of skipping.
# `RUSTUP_TOOLCHAIN` names a toolchain that does not exist, and auto-install is off,
# so every `cargo` invocation fails before cargo-dylint runs.
set -uo pipefail

short=nightly-2026-11-30
log=/work/log
mkdir -p "$log" /work/tmp
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp
export PATH=/usr/bin:/usr/sbin:/usr/local/bin:/bin

echo "== clock $(date --iso-8601=seconds)"
echo "== image $(cat /work/image.txt)"
echo "== linter"; /work/monochromatic-lint --version | tee "$log/linter-version.txt"
echo "== linter digest"; sha256sum /work/monochromatic-lint | tee "$log/linter-digest.txt"
mkdir -p /work/repo
cp -a /work/repo-input/. /work/repo/
echo "== the package's dependencies are fetched first, so nothing else can mask the probe"
( cd /work/repo/pkg && cargo fetch --locked ) > "$log/pkg-fetch.out" 2>&1
echo "pkg fetch status $?"
echo "== the toolchain named, and that rustup does not have it"
( env -u RUSTUP_TOOLCHAIN rustup toolchain list ) | tee "$log/toolchains.txt"
( env -u RUSTUP_TOOLCHAIN RUSTUP_AUTO_INSTALL=0 rustup run "$short" rustc --version ) > "$log/a-rustc.txt" 2>&1
echo "a rustup run status $?"
cat "$log/a-rustc.txt"
echo "== the version probe the linter runs, with the variable exported"
( cd /work/repo && env RUSTUP_TOOLCHAIN=$short RUSTUP_AUTO_INSTALL=0 cargo dylint --version ) > "$log/b-version.out" 2> "$log/b-version.err"
echo "b cargo dylint --version status $?"
cat "$log/b-version.out"; cat "$log/b-version.err"
echo "== the linter, with the variable exported"
( cd /work/repo && env RUSTUP_TOOLCHAIN=$short RUSTUP_AUTO_INSTALL=0 /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/c-linter.jsonl" 2> "$log/c-linter.err"
echo "c linter status $?"
wc -c "$log/c-linter.jsonl" | tee "$log/c-size.txt"
cat "$log/c-linter.jsonl" | tee "$log/c-record.txt"
echo "--- stderr"; cat "$log/c-linter.err"
echo "== done $(date --iso-8601=seconds)"
