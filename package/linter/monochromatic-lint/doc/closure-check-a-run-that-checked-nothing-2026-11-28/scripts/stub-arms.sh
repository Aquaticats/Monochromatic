#!/usr/bin/env bash
# The stub-cargo arms of issue 6667, at the built executable's surface.
# Usage: stub-arms.sh <label> <linter binary> <scenario>
# Scenarios:
#   probe-fails   a driver directory and a cache directory are known, and the throwaway driver crate
#                 cannot be written, so the driver check cannot be made; `cargo dylint` then exits 0
#                 and prints none of cargo's records (the shape a real run has when it finds no library).
#   checked       the same driver-check state, and the run printed the one record a real build prints.
set -uo pipefail

label=${1:?a label}
binary=${2:?the linter binary}
scenario_name=${3:?the scenario}
scratch=/home/user/temp/agent/monochromatic-lint-a-package-the-closure-rule-did-not-check-never-ends-as-clean-scratch
work=$scratch/keep/stub-arms/$label-$scenario_name
stub=$scratch/stub-cargo
mkdir -p "$work" "$stub"

if [ ! -x "$stub/cargo" ]; then
  rustc --edition=2024 --crate-name=closure_stub_cargo \
    /home/user/Monochromatic/.claude/worktrees/pi-monochromatic-lint-a-package-the-closure-rule-did-not-check-never-ends-as-clean/package/linter/monochromatic-lint/src/closure_stub_cargo.rs \
    -o "$stub/cargo"
  echo "stub compiled at $stub/cargo" > "$stub/compiled.txt"
fi

# The fixture: a library directory, a package with one source file, and the configuration.
mkdir -p "$work/repo/lib" "$work/repo/pkg/src" "$work/cargo" "$work/programs" "$work/drivers"
printf '[package]\n' > "$work/repo/lib/Cargo.toml"
printf '[package]\n' > "$work/repo/pkg/Cargo.toml"
printf 'fn a() -> bool {\n  run(|x| x)\n}\n' > "$work/repo/pkg/src/lib.rs"
cat > "$work/repo/monochromatic-lint.config.jsonc" <<'CONFIG'
[
  {
    "files": ["**/*.rs"],
    "rules": {
      "rust/avoidable-closure": {
        "severity": "warn",
        "library": "lib"
      }
    }
  }
]
CONFIG

# The stub `cargo`: its version answer, and the record the check's own run leaves.
ln -f "$stub/cargo" "$work/cargo/cargo"
printf 'cargo-dylint 6.3.0\n' > "$work/cargo/dylint-version.stdout"
if [ "$scenario_name" = checked ]; then
  printf '{"reason":"build-finished","success":true}\n' > "$work/cargo/dylint.stdout"
fi

# The stub `rustup`, which names a toolchain that exists.
ln -f "$stub/cargo" "$work/programs/rustup"
printf 'nightly-fixture (default)\n' > "$work/programs/show.stdout"

# The cache directory the throwaway driver crate would be written in: a directory that exists and
# cannot be written, so the write fails and the driver check cannot be made.
mkdir -p "$work/cache/monochromatic-lint"
chmod 500 "$work/cache/monochromatic-lint"

(
  cd "$work/repo" || exit 1
  env MONOCHROMATIC_LINT_CARGO="$work/cargo/cargo" \
    DYLINT_DRIVER_PATH="$work/drivers" \
    XDG_CACHE_HOME="$work/cache" \
    PATH="$work/programs:/usr/local/bin:/usr/bin:/bin" \
    "$binary" --config monochromatic-lint.config.jsonc --debug pkg \
    < /dev/null > "$work/run.stdout" 2> "$work/run.stderr"
)
echo "$?" > "$work/run.status"
echo "label $label scenario $scenario_name binary $(sha256sum "$binary" | cut -d' ' -f1) status $(cat "$work/run.status") record $(wc -c < "$work/run.stdout") bytes"
cat "$work/run.stdout"
echo "--- stderr"
grep -E "Dylint driver|avoidable-closure checked" "$work/run.stderr"
chmod 700 "$work/cache/monochromatic-lint"
