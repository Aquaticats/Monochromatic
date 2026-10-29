#!/bin/sh
# PROTOTYPE probe 1d: does mpm 8.2.0 read mpm.managers.toml / mpm.packages.toml without --config or restore?
# Measure: how many manager ids containing "cargo-binstall" mpm lists under each placement.
set -u
mkdir --parents /work/case
cd /work
count() { # prints the number of cargo-binstall managers mpm knows
  "$@" --help 2>/dev/null | grep -c "^  --cargo-binstall-"
}
echo "mpm: $(mpm --version | head -1)"
echo "bare mpm, no files anywhere: $(count mpm) cargo-binstall managers (negative baseline)"

rm -rf /work/c1 && mkdir --parents /work/c1 && cp /proto/fixture/mpm.managers.toml /proto/fixture/mpm.packages.toml /work/c1/ && chmod 600 /work/c1/*.toml && chmod 700 /work/c1
echo "C1 files in cwd with the repo's names, no flag: $(cd /work/c1 && count mpm)"
echo "C2 --config /work/c1/mpm.managers.toml (positive control): $(count mpm --config /work/c1/mpm.managers.toml)"
echo "C3 MPM_CONFIG env: $(count env MPM_CONFIG=/work/c1/mpm.managers.toml mpm)"

echo "C4 mpm.managers.toml in the XDG app dir:"
mpm --help 2>/dev/null | grep -A3 -e "--config LOCATION" | head -5
mkdir --parents /root/.config/mpm && cp /proto/fixture/mpm.managers.toml /root/.config/mpm/ && chmod 600 /root/.config/mpm/mpm.managers.toml && chmod 700 /root/.config/mpm
echo "    ~/.config/mpm/mpm.managers.toml, run from an empty dir: $(cd /work/case && count mpm) managers"
rm -rf /root/.config/mpm

echo "C5 mpm.packages.toml in the XDG app dir (is it treated as config?)"
mkdir --parents /root/.config/mpm && cp /proto/fixture/mpm.packages.toml /root/.config/mpm/ && chmod 700 /root/.config/mpm
(cd /work/case && mpm --verbosity INFO managers 2>&1 | grep -i 'config\|packages.toml\|unrecogn\|ignored' | head -5)
echo "    exit=$(cd /work/case && mpm managers >/dev/null 2>&1; echo $?)"
rm -rf /root/.config/mpm

echo "C6 cwd pyproject.toml with [tool.mpm.overrides.*] (same definitions, table renamed):"
rm -rf /work/c6 && mkdir --parents /work/c6 && sed 's/^\[mpm\./[tool.mpm./' /proto/fixture/mpm.managers.toml > /work/c6/pyproject.toml && chmod 600 /work/c6/pyproject.toml && chmod 700 /work/c6
echo "    $(cd /work/c6 && count mpm) managers"

echo "C7 trust gate: same --config file made group-writable (0664)"
cp /proto/fixture/mpm.managers.toml /work/c1/loose.toml && chmod 664 /work/c1/loose.toml
mpm --config /work/c1/loose.toml managers 2>&1 | grep -i 'trust\|warn\|untrusted' | head -3
echo "    managers registered: $(count mpm --config /work/c1/loose.toml)"
echo "C8 trust gate: file 0600 but world-writable parent dir (0777, not sticky)"
rm -rf /work/c8 && mkdir --parents /work/c8 && cp /proto/fixture/mpm.managers.toml /work/c8/ && chmod 600 /work/c8/mpm.managers.toml && chmod 777 /work/c8
echo "    managers registered: $(count mpm --config /work/c8/mpm.managers.toml)"
