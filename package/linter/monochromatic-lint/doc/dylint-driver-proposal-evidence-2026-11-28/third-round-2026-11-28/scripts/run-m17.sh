#!/usr/bin/env bash
# Host driver for M17 of the third round of 2026-11-28.
set -uo pipefail

scratch=/home/user/temp/agent/dylint-driver-proposal-revised-against-its-third-gate-read-with-the-installer-read-and-both-runs-side-by-side-scratch
stage=$scratch/measurement-2026-11-28/stage
logs=$scratch/keep/third-round-measurements-2026-11-28
img=localhost/dylint-probe-2026-11-28c
container=dylint-m17-2026-11-28c

echo "== host clock $(date --iso-8601=seconds)"
podman run --detach --name "$container" --cpus=4 --memory=8g --pids-limit=1024 \
  --blkio-weight=10 "$img" sleep infinity > /dev/null
podman exec "$container" mkdir -p /work/repo /work/repo-input /work/log /work/tmp
podman cp "$stage/repo/." "$container:/work/repo-input/"
podman cp "$stage/monochromatic-lint" "$container:/work/monochromatic-lint"
podman cp "$scratch/measurement-2026-11-28/scripts/m17-in-container.sh" "$container:/work/script.sh"
podman exec "$container" bash -c "printf '%s\n' '$img' > /work/image.txt"
podman exec "$container" bash /work/script.sh > "$logs/m17-run.txt" 2>&1
echo "== m17: script status $?"
mkdir -p "$logs/m17-log"
podman cp "$container:/work/log/." "$logs/m17-log/" > /dev/null 2>&1
podman rm --force "$container" > /dev/null
echo "== done $(date --iso-8601=seconds)"
