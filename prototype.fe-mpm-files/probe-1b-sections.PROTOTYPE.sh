#!/bin/sh
# PROTOTYPE probe 1b: for each section of mpm.packages.toml, does `mpm restore` install the listed version?
# Versions are deliberately NOT the latest, so a dropped version is visible.
set -u
mkdir --parents /work
cd /work
export CARGO_HOME=/work/cargo PNPM_HOME=/work/pnpm UV_TOOL_DIR=/work/uvtool UV_TOOL_BIN_DIR=/work/uvbin
export PATH=/work/pnpm/bin:/work/uvbin:/work/cargo/bin:$PATH
mkdir --parents /work/cargo /work/pnpm /work/uvtool /work/uvbin

echo "=== [uvx] section: pycowsay = 0.0.0.1 (latest is 0.0.0.2)"
printf '[uvx]\npycowsay = "0.0.0.1"\n' > uvx.toml
mpm --verbosity INFO --uvx restore uvx.toml 2>&1 | tail -6
uv tool list
echo "--- control: change the value to 0.0.0.2, reinstall"
printf '[uvx]\npycowsay = "0.0.0.2"\n' > uvx2.toml
mpm --verbosity INFO --uvx restore uvx2.toml 2>&1 | tail -4
uv tool list

echo "=== [pnpm] section: is-odd = 2.0.0 (latest is 3.0.1)"
printf '[pnpm]\nis-odd = "2.0.0"\n' > pnpm.toml
mpm --verbosity INFO --pnpm restore pnpm.toml 2>&1 | tail -8
pnpm list --global --depth 0 2>&1 | tail -5
echo "--- control: pnpm add --global is-odd@2.0.0 into a fresh PNPM_HOME"
export PNPM_HOME=/work/pnpm2; mkdir --parents /work/pnpm2; export PATH=/work/pnpm2/bin:$PATH
pnpm add --global is-odd@2.0.0 2>&1 | tail -3
pnpm list --global --depth 0 2>&1 | tail -5
export PNPM_HOME=/work/pnpm

[ -n "${SKIP_CARGO:-}" ] && exit 0
echo "=== [cargo] section: cargo-ndk = 4.1.1 (latest is 4.1.2)"
printf '[cargo]\ncargo-ndk = "4.1.1"\n' > cargo.toml
mpm --verbosity INFO --cargo restore cargo.toml 2>&1 | tail -8
cargo install --list
echo "--- control: cargo install cargo-ndk --version 4.1.1 into a fresh CARGO_HOME"
export CARGO_HOME=/work/cargo2; mkdir --parents /work/cargo2
cargo install --locked --version 4.1.1 cargo-ndk 2>&1 | tail -3
cargo install --list
