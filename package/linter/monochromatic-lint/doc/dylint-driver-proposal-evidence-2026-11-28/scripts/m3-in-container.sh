#!/usr/bin/env bash
# M3 (gate-review finding F3): with a current driver present, which selectable lock directories
# still skip, and for which cause. One linter invocation per directory, each bounded.
set -uo pipefail

log=/work/log
mkdir -p "$log" /work/tmp
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp CARGO_BUILD_JOBS=4
export RUSTUP_TOOLCHAIN=nightly-2026-11-16
cd /work/repo || exit 1

echo "== clock $(date --iso-8601=seconds)"
echo "== cargo: $(command -v cargo) -> $(readlink -f "$(command -v cargo)")"
echo "== rustc: $(readlink -f "$(command -v rustc)")"
echo "== lock directories to run: $(wc -l < /work/selectable.txt)"
cat > /work/repo/package/linter/monochromatic-lint-closure/rust-toolchain.toml <<'TOML'
[toolchain]
channel = "nightly-2026-11-16"
components = ["llvm-tools-preview", "rustc-dev"]
TOML
echo "== the library now names the dated toolchain (option K; no pin is proposed here, only a measured working state)"
( cd /work/repo/package/linter/monochromatic-lint-closure && env -u RUSTUP_TOOLCHAIN rustup show active-toolchain )
echo "== registry cache entries copied in: $(ls /root/.cargo/registry/cache/*/ | wc -l)"

echo "== warm the driver and one package first"
SECONDS=0
env --unset=CARGO_NET_OFFLINE cargo dylint --path package/linter/monochromatic-lint-closure --manifest-path package/wg/cli-quicker-exempt/Cargo.toml -- --locked > "$log/warm.out" 2> "$log/warm.err"
echo "warm status $? after ${SECONDS}s"
tail -3 "$log/warm.err"
echo "driver: $(find /root/.dylint_drivers -name dylint-driver | head -1)"
find /root/.dylint_drivers -name dylint-driver -exec {} -V \; | tee "$log/driver-version.txt"

: > "$log/results.tsv"
while IFS= read -r dir; do
  echo "== $dir $(date --iso-8601=seconds)"
  meta=ok
  timeout 300 cargo metadata --offline --locked --format-version 1 --manifest-path "$dir/Cargo.toml" > /dev/null 2> "$log/meta-$(echo "$dir" | tr '/' '_').err" || meta=failed
  if [ "$meta" = failed ]; then
    echo "   offline metadata failed; fetching with network"
    timeout 900 cargo fetch --locked --manifest-path "$dir/Cargo.toml" > /dev/null 2> "$log/fetch-$(echo "$dir" | tr '/' '_').err" || echo "   fetch failed"
    cargo metadata --offline --locked --format-version 1 --manifest-path "$dir/Cargo.toml" > /dev/null 2>> "$log/meta-$(echo "$dir" | tr '/' '_').err" || meta=still-failed
  fi
  SECONDS=0
  timeout 600 /work/monochromatic-lint --config monochromatic-lint.config.jsonc "$dir" > "$log/lint-$(echo "$dir" | tr '/' '_').jsonl" 2> "$log/lint-$(echo "$dir" | tr '/' '_').err"
  status=$?
  elapsed=$SECONDS
  skipped=$(grep -c ' skipped ' "$log/lint-$(echo "$dir" | tr '/' '_').jsonl" 2>/dev/null || true)
  printf '%s\t%s\t%s\t%s\t%s\n' "$dir" "$status" "$elapsed" "$meta" "$skipped" >> "$log/results.tsv"
  echo "   status $status after ${elapsed}s, metadata $meta, skip records $skipped"
  grep -o '"message":"[^"]*' "$log/lint-$(echo "$dir" | tr '/' '_').jsonl" 2>/dev/null | head -2
done < /work/selectable.txt

echo "== summary"
cat "$log/results.tsv"
echo "== every skip message"
rg --no-filename -o 'rust/avoidable-closure skipped [^"]*' "$log"/lint-*.jsonl | sort | uniq -c | sort -rn | head -60
echo "== done $(date --iso-8601=seconds)"
