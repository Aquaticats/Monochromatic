#!/bin/sh
# PROTOTYPE scratch probe (rewritten freely while debugging).
mkdir --parents /work/c1 && cp /proto/fixture/mpm.managers.toml /work/c1/ && chmod 600 /work/c1/mpm.managers.toml && chmod 700 /work/c1
mpm --config /work/c1/mpm.managers.toml managers --output-format json 2>&1 | head -c 700
echo
mpm --config /work/c1/mpm.managers.toml managers 2>&1 | head -12
echo ---
mpm --config /work/c1/mpm.managers.toml --cargo-binstall-fastmod managers 2>&1 | head -8
echo ---
mpm --config /work/c1/mpm.managers.toml --help 2>&1 | grep -i binstall | head -5
