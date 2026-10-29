#!/bin/sh
# PROTOTYPE scenarios S1..S5 for one candidate. usage: scenario.PROTOTYPE.sh <a|b|c|d> <s123|s4|s5>
# Runs inside the base image with the prototype binaries at /out/bin (read-only) and fixtures at /proto (read-only).
set -u
CAND=$1
STAGE=$2
SH=/work/bin-shadow/proto-mpm-shadow        # shadow alone: the "no file-enforcer binary" machine
mkdir --parents /work/bin-shadow /work/bin-both
cp /out/bin/proto-mpm-shadow /work/bin-shadow/
cp /out/bin/proto-mpm-shadow /out/bin/proto-file-enforcer /work/bin-both/
BASEPATH=/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin
use_shadow_only() { export PATH=/work/bin-shadow:$BASEPATH; hash -r; }
use_both() { export PATH=/work/bin-both:$BASEPATH; hash -r; }
ms() { proto-mpm-shadow "$@" --candidate "$CAND"; }
fe() { proto-file-enforcer "$@" --candidate "$CAND"; }

fresh_env() { # empty installs: the "fresh machine" tool state
  rm -rf /work/env && mkdir --parents /work/env/rustup /work/env/cargo /work/env/uvtool /work/env/uvbin /work/env/pnpm
  export RUSTUP_HOME=/work/env/rustup CARGO_HOME=/work/env/cargo UV_TOOL_DIR=/work/env/uvtool UV_TOOL_BIN_DIR=/work/env/uvbin PNPM_HOME=/work/env/pnpm
  export PATH=/work/env/cargo/bin:/work/env/uvbin:/work/env/pnpm/bin:$PATH
  rustup set profile minimal >/dev/null 2>&1
}
listing() { # what files the candidate has in a scope dir
  (cd "$1" && ls -1 toolchain.* mpm.*.toml 2>/dev/null | tr '\n' ' '); echo
}
mkseed() { # committed state without the mpm files: plan + a lock made by update (network step stood in by --resolve)
  rm -rf "$1" && mkdir --parents "$1"
  cp /proto/fixture/scenario.plan.jsonc "$1/toolchain.plan.jsonc"
  proto-mpm-shadow update --candidate d --dir "$1" --resolve rust=nightly-2026-10-01 --resolve cargo:timeout-cli=0.1.0 >/dev/null
}
gates() { # the candidate's drift gates over a scope dir; prints exit codes
  d=$1
  ms lock --check --dir "$d" >/work/g1.out 2>&1; a=$?
  fe check --dir "$d" >/work/g2.out 2>&1; b=$?
  echo "gate mpm-shadow-lock-check=$a file-enforcer-check=$b"
  cat /work/g1.out /work/g2.out | grep -v '^OK$' | grep -v 'no toolchain rules' | sort -u | sed 's/^/    /'
}

if [ "$STAGE" = s123 ]; then
echo "################ candidate $CAND: S1 fresh machine, no file-enforcer binary ################"
use_shadow_only
command -v proto-file-enforcer >/dev/null && echo "UNEXPECTED: file-enforcer is on PATH" || echo "file-enforcer binary: absent (PATH check)"
fresh_env
mkseed /work/s1
echo "checkout before: $(listing /work/s1)"
ms lock --dir /work/s1; echo "lock exit=$?"
echo "checkout after : $(listing /work/s1)"
gates /work/s1
echo "--- sync with no fallback (exposes the mpm.packages.toml dependency of the rustup route)"
ms sync --dir /work/s1 --no-fallback 2>&1 | grep -v '^ *$' | cut -c1-200 | tail -12; echo "sync exit=${PIPESTATUS[0]}"
echo "--- sync with the fallback the TS code has (direct rustup)"
ms sync --dir /work/s1 2>&1 | grep -i 'fall\|rustup\|restore' | cut -c1-200 | head -5
echo "--- status (lock vs installed)"
ms status --dir /work/s1; echo "status exit=$?"
if [ "$CAND" = c ]; then
  use_both
  echo "--- C only: now file-enforcer is available; mpm-shadow lock again"
  ms lock --dir /work/s1; echo "lock exit=$?"; echo "checkout: $(listing /work/s1)"; gates /work/s1
fi
use_both

echo "################ candidate $CAND: S2 plan edit, then ONE command ################"
cp /proto/fixture/scenario.plan2.jsonc /work/s1/toolchain.plan.jsonc
echo "before the command: gates say:"; gates /work/s1
ms update --dir /work/s1 --resolve rust=nightly-2026-09-15; echo "update exit=$?"
echo "after the command:"; gates /work/s1
echo "lock versions: $(jq -c '[.tools|to_entries[]|{(.key):.value.version}]|add' /work/s1/toolchain.lock.json)"
if [ -f /work/s1/mpm.packages.toml ]; then
  echo "packages.toml: $(tr '\n' ' ' < /work/s1/mpm.packages.toml | cut -c1-400)"
  echo "managers ids : $(grep -o '^\[mpm.overrides.[a-z0-9-]*\]' /work/s1/mpm.managers.toml | tr '\n' ' ')"
  echo "managers mode: $(stat -c %a /work/s1/mpm.managers.toml)"
else
  echo "no mpm files exist"
fi
echo "--- install what the lock says, then compare (rustup via the candidate's route, others direct)"
fresh_env
ms sync --dir /work/s1 --no-fallback 2>&1 | grep -c '^\$ ' | sed 's/^/commands run: /'
ms status --dir /work/s1; echo "status exit=$?"
if [ -f /work/s1/mpm.packages.toml ]; then
  echo "--- the generated mpm.packages.toml used as the ONLY installer: mpm restore (all sections)"
  fresh_env
  (cd /work/s1 && mpm restore mpm.packages.toml 2>&1 | grep -i 'warning\|✓\|✘' | cut -c1-180 | sort -u | head -12)
  ms status --dir /work/s1; echo "status exit=$?  (nonzero means restore left tools different from the lock)"
fi

echo "################ candidate $CAND: S3 global scope in a disposable HOME ################"
export REALHOME=$HOME
export HOME=/work/home3
rm -rf $HOME && mkdir --parents $HOME/.config/monochromatic-dev/toolchain
G=$HOME/.config/monochromatic-dev/toolchain
cp /proto/fixture/scenario.plan.jsonc $G/toolchain.plan.jsonc
use_shadow_only
ms update --global --resolve rust=nightly-2026-10-01 --resolve cargo:timeout-cli=0.1.0; echo "update --global exit=$? (file-enforcer absent)"
echo "global dir: $(listing $G)"
if [ "$CAND" = c ]; then use_both; fe user write; echo "file-enforcer user write exit=$?"; fi
use_both
echo "global dir: $(listing $G)"
ms lock --check --global; echo "lock --check --global exit=$?"
fe user check; echo "file-enforcer user check exit=$?"
fresh_env
ms sync --global --no-fallback 2>&1 | grep -c '^\$ ' | sed 's/^/commands run: /'
ms status --global; echo "status --global exit=$?"
echo "mpm reading the global dir implicitly? managers with the global dir as cwd: $(cd $G && mpm --help 2>/dev/null | grep -c '^  --cargo-binstall-')"
export HOME=$REALHOME
fi

if [ "$STAGE" = s4 ]; then
echo "################ candidate $CAND: S4 drift ################"
use_both
D=/work/s4
mkseed $D
cp /proto/fixture/scenario.plan2.jsonc $D/toolchain.plan.jsonc
ms update --dir $D --resolve rust=nightly-2026-09-15 >/dev/null 2>&1
if [ "$CAND" = c ]; then fe write --dir $D; fi
mkdir --parents /work/s4.bak && rm -rf /work/s4.bak/* && cp -a $D/. /work/s4.bak/
restore() { rm -rf $D && mkdir --parents $D && cp -a /work/s4.bak/. $D/; }
echo "[control] untouched tree"; gates $D
echo "[plan] edit pycowsay pin in the plan, lock not regenerated"; sed -i 's/"0.0.0.2"/"0.0.0.3"/' $D/toolchain.plan.jsonc; gates $D; restore
echo "[lock pinned] hand-edit a pinned tool's version in the lock"; sed -i 's/"version": "0.0.0.2"/"version": "0.0.0.9"/' $D/toolchain.lock.json; gates $D; restore
echo "[lock resolved] hand-edit a resolved (latest-track) tool's version in the lock"; sed -i 's/"version": "0.1.0"/"version": "0.1.1"/' $D/toolchain.lock.json; gates $D; restore
if [ "$CAND" != d ]; then
echo "[packages] hand-edit mpm.packages.toml"; sed -i 's/is-odd = "2.0.0"/is-odd = "9.9.9"/' $D/mpm.packages.toml; gates $D; restore
echo "[managers] hand-edit mpm.managers.toml"; sed -i 's/"--version", "0.1.0"/"--version", "0.1.1"/' $D/mpm.managers.toml; gates $D; restore
echo "[managers mode] chmod 664 mpm.managers.toml (mpm would refuse to load it)"; chmod 664 $D/mpm.managers.toml; gates $D; restore
echo "[packages missing] delete mpm.packages.toml"; rm $D/mpm.packages.toml; gates $D; restore
else
echo "[mpm files] none exist under D; corruption of them is not possible"
fi
echo "[lock missing] delete toolchain.lock.json"; rm $D/toolchain.lock.json; gates $D; restore
fi

if [ "$STAGE" = s5 ]; then
echo "################ candidate $CAND: S5 two concurrent writers ################"
use_both
R=/work/s5
prep() { mkseed $R; cp /proto/fixture/scenario.plan2.jsonc $R/toolchain.plan.jsonc; ms update --dir $R --resolve rust=nightly-2026-10-01 --resolve cargo:timeout-cli=0.1.0 >/dev/null 2>&1; [ "$CAND" = c ] && fe write --dir $R; }
vers() { jq -c '[.tools["rust"].version, .tools["cargo:timeout-cli"].version]' $R/toolchain.lock.json; }
race2() { # $1=guard $2=title, extra env for P1 in $3; P2 starts 100 ms after P1 so a compare-and-swap can see P1's write
  prep
  echo "[$2, guard=$1] P1 resolves rust, P2 resolves timeout-cli"
  echo "  start: $(vers)"
  env PROTO_RACE_SLEEP_MS=1500 $3 proto-mpm-shadow update --candidate "$CAND" --dir $R --guard $1 --resolve rust=nightly-2026-09-15 >/work/p1.out 2>&1 &
  p1=$!
  sleep 0.1
  PROTO_RACE_SLEEP_MS=1500 ms update --dir $R --guard $1 --resolve cargo:timeout-cli=0.1.1 >/work/p2.out 2>&1 &
  p2=$!
  wait $p1; e1=$?; wait $p2; e2=$?
  echo "  P1 exit=$e1 P2 exit=$e2   final [rust, timeout-cli] = $(vers)   (both survive iff [nightly-2026-09-15, 0.1.1])"
  sed 's/^/  stderr: /' /work/p1.out /work/p2.out | head -4
}
for G in none cas mutex; do race2 $G "lost update: both read the lock, then both write" "PROTO_X=1"; done
for G in cas mutex; do race2 $G "check-to-rename window: P1 pauses 600 ms after its compare passed" "PROTO_RACE_SLEEP_AFTER_CHECK_MS=600"; done
for G in cas mutex; do
  prep
  echo "[torn files, guard=$G] P1 sleeps 1200 ms between writing the lock and writing the mpm files; P2 runs completely in between"
  PROTO_RACE_SLEEP_AFTER_LOCK_MS=1200 ms update --dir $R --guard $G --resolve rust=nightly-2026-09-15 >/work/p1.out 2>&1 &
  p1=$!
  sleep 0.3
  ms update --dir $R --guard $G --resolve cargo:timeout-cli=0.1.1 >/work/p2.out 2>&1 &
  p2=$!
  wait $p1; e1=$?; wait $p2; e2=$?
  echo "  P1 exit=$e1 P2 exit=$e2   final [rust, timeout-cli] = $(vers)"
  gates $R
done
fi
