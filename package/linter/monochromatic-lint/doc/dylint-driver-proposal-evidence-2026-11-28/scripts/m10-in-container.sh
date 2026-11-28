#!/usr/bin/env bash
# M10 (round of 2026-11-28, second round): the statuses a skip returns, with the status lines kept.
# Arm A measures the machine-wide rustc-dev skip; arm B the driver-gap skip. The earlier round kept
# the JSONL records of both and not the statuses, which the second gate read flagged.
set -uo pipefail

short=nightly-2026-11-16
log=/work/log
mkdir -p "$log" /work/tmp
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp
export PATH=/usr/bin:/usr/sbin:/usr/local/bin:/bin

echo "== clock $(date --iso-8601=seconds)"
echo "== cargo $(command -v cargo) -> $(readlink -f "$(command -v cargo)")"
echo "== rustc $(command -v rustc) -> $(readlink -f "$(command -v rustc)")"
echo "== floating components"; rustup component list --installed --toolchain nightly | tee "$log/floating-components.txt"
echo "== no driver"; find /root/.dylint_drivers -maxdepth 3 2>&1 | head -3

echo "== arm A: the machine-wide rustc-dev skip, RUSTUP_TOOLCHAIN unset"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/a-default.jsonl" 2> "$log/a-default.err"
echo "a-default status $?"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN /work/monochromatic-lint --config monochromatic-lint.config.jsonc --max-warnings 0 pkg ) > "$log/a-max-warnings.jsonl" 2> "$log/a-max-warnings.err"
echo "a-max-warnings status $?"
( cd /work/repo && env -u RUSTUP_TOOLCHAIN /work/monochromatic-lint --config monochromatic-lint.config.jsonc --quiet pkg ) > "$log/a-quiet.jsonl" 2> "$log/a-quiet.err"
echo "a-quiet status $?"
wc -c "$log/a-default.jsonl" "$log/a-max-warnings.jsonl" "$log/a-quiet.jsonl" | tee "$log/a-sizes.txt"
head -c 300 "$log/a-default.jsonl"; echo

echo "== arm B: the driver-gap skip, RUSTUP_TOOLCHAIN set to the dated name"
( cd /work/repo && RUSTUP_TOOLCHAIN="$short" /work/monochromatic-lint --config monochromatic-lint.config.jsonc pkg ) > "$log/b-default.jsonl" 2> "$log/b-default.err"
echo "b-default status $?"
( cd /work/repo && RUSTUP_TOOLCHAIN="$short" /work/monochromatic-lint --config monochromatic-lint.config.jsonc --max-warnings 0 pkg ) > "$log/b-max-warnings.jsonl" 2> "$log/b-max-warnings.err"
echo "b-max-warnings status $?"
( cd /work/repo && RUSTUP_TOOLCHAIN="$short" /work/monochromatic-lint --config monochromatic-lint.config.jsonc --quiet pkg ) > "$log/b-quiet.jsonl" 2> "$log/b-quiet.err"
echo "b-quiet status $?"
wc -c "$log/b-default.jsonl" "$log/b-max-warnings.jsonl" "$log/b-quiet.jsonl" | tee "$log/b-sizes.txt"
head -c 300 "$log/b-default.jsonl"; echo
echo "== done $(date --iso-8601=seconds)"
