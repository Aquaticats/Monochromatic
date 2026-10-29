#!/bin/sh
# PROTOTYPE probe 1c: does the version baked into mpm.managers.toml take effect through
# `mpm --config mpm.managers.toml --cargo-binstall-<slug> install <crate>`?
set -u
mkdir --parents /work/cfg
cp /proto/fixture/mpm.managers.toml /work/cfg/mpm.managers.toml
chmod 600 /work/cfg/mpm.managers.toml
chmod 700 /work/cfg
cd /work
fresh() { rm -rf /work/cargo-home && mkdir --parents /work/cargo-home; export CARGO_HOME=/work/cargo-home; export PATH=/work/cargo-home/bin:$PATH; }
export PATH=/usr/local/cargo/bin:$PATH
cargo-binstall -V

echo "=== T1 smallest tool: timeout-cli 0.1.0 (only version that exists), file as generated"
fresh
mpm --config /work/cfg/mpm.managers.toml --verbosity INFO --cargo-binstall-timeout-cli install timeout-cli 2>&1 | cut -c1-240 | tail -12
cargo install --list
sha256sum /work/cargo-home/bin/timeout | cut -c1-64 > /work/sha-mpm.txt
echo "--- same crate by the direct command the file wraps:"
fresh
cargo binstall --no-confirm --version 0.1.0 timeout-cli 2>&1 | tail -3
sha256sum /work/cargo-home/bin/timeout | cut -c1-64 > /work/sha-direct.txt
echo "sha via mpm override : $(cat /work/sha-mpm.txt)"
echo "sha via direct binstall: $(cat /work/sha-direct.txt)"

[ -n "${ONLY_T1:-}" ] && exit 0
echo "=== T2 cargo-ndk: file as generated (baked 4.1.2 == latest, so no signal on its own)"
fresh
mpm --config /work/cfg/mpm.managers.toml --cargo-binstall-cargo-ndk install cargo-ndk 2>&1 | cut -c1-240 | tail -3
cargo install --list | head -1
echo "=== T3 POSITIVE CONTROL: edit the version inside the managers file to 4.1.1 (both the name and --version arg)"
sed 's/"4\.1\.2"/"4.1.1"/; s/cargo-ndk 4\.1\.2/cargo-ndk 4.1.1/' /work/cfg/mpm.managers.toml > /work/cfg/edited.toml
chmod 600 /work/cfg/edited.toml
grep -n '4\.1\.1' /work/cfg/edited.toml
fresh
mpm --config /work/cfg/edited.toml --verbosity INFO --cargo-binstall-cargo-ndk install cargo-ndk 2>&1 | cut -c1-240 | grep -i 'binstall\|install' | head -6
cargo install --list | head -1

echo "=== T4 same package through the stock cargo manager with an explicit version (how restore would route it): version?"
fresh
mpm --verbosity INFO --cargo install cargo-ndk@4.1.1 2>&1 | cut -c1-240 | tail -4
cargo install --list | head -1
