#!/usr/bin/env bash
# Host driver for M15a and M15b of the third round of 2026-11-28.
set -uo pipefail

wt=/home/user/Monochromatic/.claude/worktrees/pi-dylint-driver-proposal-revised-against-its-third-gate-read-with-the-installer-read-and-both-runs-side-by-side
scratch=/home/user/temp/agent/dylint-driver-proposal-revised-against-its-third-gate-read-with-the-installer-read-and-both-runs-side-by-side-scratch
stage=$scratch/measurement-2026-11-28/stage
logs=$scratch/keep/third-round-measurements-2026-11-28
img=localhost/dylint-probe-2026-11-28c

echo "== host clock $(date --iso-8601=seconds)"

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
  if [ "$name" = m15a ]; then
    podman cp "$stage/rust-toolchain.k1.toml" "$container:/work/repo/lint-closure/rust-toolchain.toml"
  fi
  podman exec "$container" bash /work/script.sh > "$logs/$name-run.txt" 2>&1
  echo "== $name: script status $?"
  mkdir -p "$logs/$name-log"
  podman cp "$container:/work/log/." "$logs/$name-log/" > /dev/null 2>&1
  podman rm --force "$container" > /dev/null
  echo "== $name: container removed at $(date --iso-8601=seconds)"
}

run_measurement m15a m15a-in-container.sh
run_measurement m15b m15b-in-container.sh

echo "== done $(date --iso-8601=seconds)"
echo "== containers left"; podman ps --all --format '{{.Names}}' | head
