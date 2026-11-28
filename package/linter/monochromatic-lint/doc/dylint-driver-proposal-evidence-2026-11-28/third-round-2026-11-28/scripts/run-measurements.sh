#!/usr/bin/env bash
# Host driver for M12, M13 and M14 of the third round of 2026-11-28.
# One container per measurement, from the image built by Containerfile, each with its own
# cgroup limits; inputs are copied in, nothing of the account's home is mounted.
set -uo pipefail

wt=/home/user/Monochromatic/.claude/worktrees/pi-dylint-driver-proposal-revised-against-its-third-gate-read-with-the-installer-read-and-both-runs-side-by-side
scratch=/home/user/temp/agent/dylint-driver-proposal-revised-against-its-third-gate-read-with-the-installer-read-and-both-runs-side-by-side-scratch
stage=$scratch/measurement-2026-11-28/stage
logs=$scratch/keep/third-round-measurements-2026-11-28
img=localhost/dylint-probe-2026-11-28c
linter=/home/user/.local/share/monochromatic-dev/toolchain/cargo-monochromatic-lint/0.5.2-alpha.25/monochromatic-lint-0.5.2-alpha.25-x86_64-unknown-linux-gnu/monochromatic-lint

mkdir -p "$stage" "$logs"

echo "== host clock $(date --iso-8601=seconds)"
echo "== image digest"; podman image inspect --format '{{.Id}}' "$img"

# The inputs, copied from this worktree.
rm --recursive --force /home/user/temp/agent/dylint-driver-proposal-revised-against-its-third-gate-read-with-the-installer-read-and-both-runs-side-by-side-scratch/measurement-2026-11-28/stage/repo
mkdir -p "$stage/repo"
cp -a "$wt/package/linter/monochromatic-lint-closure" "$stage/repo/lint-closure"
cp -a "$wt/package/wg/cli-quicker-exempt" "$stage/repo/pkg"
cp "$linter" "$stage/monochromatic-lint"
cat > "$stage/repo/monochromatic-lint.config.jsonc" <<'CONFIG'
[
  {
    "name": "rust",
    "files": [
      "**/*.rs",
    ],
    "rules": {
      "rust/avoidable-closure": {
        "severity": "warn",
        "library": "lint-closure",
      },
    },
  },
]
CONFIG
# Answer K1's file, used by M13 only.
printf '%s\n' '[toolchain]' 'channel = "nightly-2026-11-16"' 'components = ["llvm-tools-preview", "rustc-dev"]' > "$stage/rust-toolchain.k1.toml"
sha256sum "$stage/monochromatic-lint" | tee "$logs/linter-digest.txt"

run_measurement () {
  local name=$1
  local script=$2
  local container="dylint-$name-2026-11-28c"
  echo "== $name: starting $container at $(date --iso-8601=seconds)"
  podman run --detach --name "$container" --cpus=4 --memory=8g --pids-limit=1024 \
    --blkio-weight=10 "$img" sleep infinity > /dev/null
  podman exec "$container" mkdir -p /work/repo /work/log /work/tmp
  podman cp "$stage/repo/." "$container:/work/repo/"
  podman cp "$stage/monochromatic-lint" "$container:/work/monochromatic-lint"
  podman cp "$scratch/measurement-2026-11-28/scripts/$script" "$container:/work/script.sh"
  podman exec "$container" bash -c "printf '%s\n' '$img' > /work/image.txt"
  if [ "$name" = m13 ]; then
    podman cp "$stage/rust-toolchain.k1.toml" "$container:/work/repo/lint-closure/rust-toolchain.toml"
  fi
  podman exec "$container" bash /work/script.sh > "$logs/$name-run.txt" 2>&1
  echo "== $name: script status $?"
  mkdir -p "$logs/$name-log"
  podman cp "$container:/work/log/." "$logs/$name-log/" > /dev/null 2>&1
  podman rm --force "$container" > /dev/null
  echo "== $name: container removed at $(date --iso-8601=seconds)"
}

run_measurement m12 m12-in-container.sh
run_measurement m13 m13-in-container.sh
run_measurement m14 m14-in-container.sh

echo "== done $(date --iso-8601=seconds)"
echo "== containers left"; podman ps --all --format '{{.Names}}' | head
