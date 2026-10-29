#!/bin/sh
# PROTOTYPE probe 1a: does `mpm --rustup restore` honor the rustup section of the repo's mpm.packages.toml?
set -u
export CARGO_HOME=/work/cargo
mkdir --parents /work
cd /work
mpm --version
rustup --version 2>&1 | head -1

fresh() {
  rm -rf /work/rustup /work/cargo
  mkdir --parents /work/rustup /work/cargo
  export RUSTUP_HOME=/work/rustup
  rustup set profile minimal >/dev/null 2>&1
}

show() {
  echo "--- installed toolchains:"
  rustup toolchain list 2>&1
}

# the repo's real [rustup] section, copied verbatim from the fixture
awk '/^\[rustup\]/{p=1;print;next} /^\[/{p=0} p&&NF{print}' /proto/fixture/mpm.packages.toml > real-rustup.toml
chmod 600 real-rustup.toml
echo "=== file under test (real section):"; cat real-rustup.toml

echo "=== CONTROL 0: the real lock pin as written (nightly-2026-10-29)"
fresh
mpm --verbosity INFO --rustup restore real-rustup.toml 2>&1 | tail -8
echo "exit=$?"
show

echo "=== CASE A: key=value both nightly-2026-10-01"
fresh
printf '[rustup]\nnightly-2026-10-01 = "nightly-2026-10-01"\n' > a.toml
mpm --verbosity INFO --rustup restore a.toml 2>&1 | tail -6
show

echo "=== CASE B (positive control, change KEY and value to nightly-2026-09-15)"
fresh
printf '[rustup]\nnightly-2026-09-15 = "nightly-2026-09-15"\n' > b.toml
mpm --verbosity INFO --rustup restore b.toml 2>&1 | tail -6
show

echo "=== CASE C (change VALUE only: key nightly-2026-10-01, value nightly-2026-08-01)"
fresh
printf '[rustup]\nnightly-2026-10-01 = "nightly-2026-08-01"\n' > c.toml
mpm --verbosity INFO --rustup restore c.toml 2>&1 | tail -6
show

echo "=== CASE D (value garbage: key nightly-2026-10-01, value 99.99.99)"
fresh
printf '[rustup]\nnightly-2026-10-01 = "99.99.99"\n' > d.toml
mpm --verbosity INFO --rustup restore d.toml 2>&1 | tail -6
show

echo "=== FALLBACK: direct rustup toolchain install (what providers.rust-android.ts does without mpm)"
fresh
rustup toolchain install --profile minimal --no-self-update nightly-2026-10-01 2>&1 | tail -3
show
