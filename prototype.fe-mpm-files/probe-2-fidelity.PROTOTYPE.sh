#!/bin/sh
# PROTOTYPE probe 2.0: is each mpm file a pure function of the LOCK? Render both from toolchain.lock.json with the
# ported renderer and the legacy header, then byte-compare with the files the TypeScript generator wrote.
set -u
export PATH=/out/bin:$PATH
mkdir --parents /work/render
proto-mpm-shadow render --lock /proto/fixture/toolchain.lock.json --out /work/render --legacy-header
cmp /work/render/mpm.packages.toml /proto/fixture/mpm.packages.toml && echo "packages.toml: byte-identical to the TS output (exit 0)"
cmp /work/render/mpm.managers.toml /proto/fixture/mpm.managers.toml && echo "managers.toml: byte-identical to the TS output (exit 0)"
wc -c /work/render/*.toml /proto/fixture/mpm.*.toml
echo "--- positive control: change one locked version, expect the render to differ"
sed 's/"version": "10.5.0"/"version": "10.4.2"/' /proto/fixture/toolchain.lock.json > /work/lock-edited.json
mkdir --parents /work/render2
proto-mpm-shadow render --lock /work/lock-edited.json --out /work/render2 --legacy-header
( [ -s /work/render2/mpm.packages.toml ] && ! cmp -s /work/render2/mpm.packages.toml /proto/fixture/mpm.packages.toml ) && echo "packages.toml differs after the lock edit (exit $?): good"
( [ -s /work/render2/mpm.managers.toml ] && ! cmp -s /work/render2/mpm.managers.toml /proto/fixture/mpm.managers.toml ) && echo "managers.toml differs after the lock edit: good"
diff /work/render2/mpm.managers.toml /proto/fixture/mpm.managers.toml | head -6
