#!/usr/bin/env bash
# One arm of the issue-6667 reproduction, run inside the probe container.
# Usage: arm.sh <arm>
# The fixture is /work/repo, the linter under test is /work/linter, the log is /work/log.
set -uo pipefail

arm=${1:?an arm name}
log=/work/log
repo=/work/repo
linter=/work/linter
export PATH=/usr/local/bin:/usr/bin:/usr/sbin:/bin
export TMPDIR=/work/tmp TMP=/work/tmp TEMP=/work/tmp
mkdir -p "$log" /work/tmp

# The state the issue was measured in: rustup never installs a toolchain a file names on its own.
export RUSTUP_AUTO_INSTALL=0

say () { echo "$@" | tee -a "$log/run.txt"; }

say "== arm $arm"
say "== clock $(date --iso-8601=seconds)"
say "== linter $(sha256sum "$linter" | cut -d' ' -f1) $("$linter" --version)"
say "== toolchain $(rustup show active-toolchain 2>&1 | head -1)"

# The two commands the linter builds for the driver check and for the check itself.
probe_cargo_dylint () {
  local manifest=$repo/pkg/Cargo.toml
  ( cd "$repo" && env CARGO_NET_OFFLINE=true \
      DYLINT_RUSTFLAGS='-W avoidable_closure -W unfulfilled_lint_expectations' \
      cargo dylint --path "$repo/lint-closure" --manifest-path "$manifest" -- --locked --message-format json \
      < /dev/null > "$log/dylint.stdout" 2> "$log/dylint.stderr" )
  echo "$?" > "$log/dylint.status"
  say "== cargo dylint status $(cat "$log/dylint.status")"
  say "--- cargo dylint stdout ($(wc -c < "$log/dylint.stdout") bytes, reasons: $(grep -o '"reason":"[a-z-]*"' "$log/dylint.stdout" | sort | uniq -c | tr '\n' ' '))"
  say "--- cargo dylint stderr ($(wc -c < "$log/dylint.stderr") bytes)"
  tail -12 "$log/dylint.stderr" | tee -a "$log/run.txt"
}

run_linter () {
  local name=$1
  shift
  ( cd "$repo" && "$linter" --config monochromatic-lint.config.jsonc "$@" pkg \
      < /dev/null > "$log/$name.stdout" 2> "$log/$name.stderr" )
  echo "$?" > "$log/$name.status"
  say "== linter $name status $(cat "$log/$name.status"), record $(wc -c < "$log/$name.stdout") bytes, stderr $(wc -c < "$log/$name.stderr") bytes"
  cat "$log/$name.stdout" | tee -a "$log/run.txt"
  say "--- linter $name stderr"
  cat "$log/$name.stderr" | tee -a "$log/run.txt"
}

case "$arm" in
  arm-a)
    # Cause 2: the library's own toolchain file names a toolchain that is not installed, and rustup's auto-install is off.
    printf '%s\n' '[toolchain]' 'channel = "nightly-2026-11-21"' 'components = ["rustc-dev", "llvm-tools-preview"]' \
      > "$repo/lint-closure/rust-toolchain.toml"
    ( cd "$repo/lint-closure" && env -u RUSTUP_TOOLCHAIN RUSTUP_AUTO_INSTALL=0 rustup show active-toolchain \
        > "$log/active-toolchain.txt" 2>&1 )
    say "== rustup show active-toolchain status $? in lint-closure"
    cat "$log/active-toolchain.txt" | tee -a "$log/run.txt"
    probe_cargo_dylint
    run_linter linter
    run_linter linter-debug --debug
    say "--- the --debug notes about the driver and the check"
    grep -E "Dylint driver|avoidable-closure checked" "$log/linter-debug.stderr" | tee -a "$log/run.txt"
    ;;

  arm-check)
    # The control: no toolchain file, and a current driver, so the check really runs.
    probe_cargo_dylint
    run_linter linter
    say "== the same check again, with every unit already built"
    probe_cargo_dylint
    run_linter linter-second
    ;;

  arm-no-driver-dir)
    # Cause 1: no DYLINT_DRIVER_PATH and no home directory, so no driver directory is known.
    say "== HOME is unset and this uid has no passwd entry: $(env -u HOME sh -c 'printf "%s" "${HOME:-unset}"')"
    probe_cargo_dylint
    run_linter linter
    run_linter linter-debug --debug
    grep -E "Dylint driver|avoidable-closure checked" "$log/linter-debug.stderr" | tee -a "$log/run.txt"
    ;;

  arm-no-cache-dir)
    # Cause 3: a driver directory is named and the toolchain is readable, but no cache directory is known.
    probe_cargo_dylint
    run_linter linter
    run_linter linter-debug --debug
    grep -E "Dylint driver|avoidable-closure checked" "$log/linter-debug.stderr" | tee -a "$log/run.txt"
    ;;

  arm-resolve-fails)
    # Cause 4: a driver directory and a cache directory are known, but the throwaway crate cannot be written.
    probe_cargo_dylint
    run_linter linter
    run_linter linter-debug --debug
    grep -E "Dylint driver|avoidable-closure checked" "$log/linter-debug.stderr" | tee -a "$log/run.txt"
    ;;

  *)
    say "== unknown arm $arm"
    exit 64
    ;;
esac

say "== done $(date --iso-8601=seconds)"
