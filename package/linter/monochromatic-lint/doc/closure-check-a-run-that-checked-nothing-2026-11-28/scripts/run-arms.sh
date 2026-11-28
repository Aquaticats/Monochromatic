#!/usr/bin/env bash
# Host driver for the issue-6667 arms: one container per arm and binary, each with its own cgroup limits.
# Usage: run-arms.sh <label> <linter binary> <arm> [<arm>...]
# Inputs are copied in; nothing of this account is mounted.
set -uo pipefail

label=${1:?a label for the log folder}
binary=${2:?the linter binary}
shift 2
arms=("$@")
probe=/home/user/temp/agent/monochromatic-lint-a-package-the-closure-rule-did-not-check-never-ends-as-clean-scratch/probe
keep=/home/user/temp/agent/monochromatic-lint-a-package-the-closure-rule-did-not-check-never-ends-as-clean-scratch/keep/arms
image=localhost/monochromatic-lint-closure-issue-6667-20261128
uid=61234:61234

mkdir -p "$keep"
printf '%s\n' "== host clock $(date --iso-8601=seconds)" "== image $(podman image inspect --format '{{.Id}}' "$image")" \
  "== label $label" "== binary $binary sha256 $(sha256sum "$binary" | cut -d' ' -f1)" \
  "== arms ${arms[*]}" | tee "$keep/$label-run.txt"

for arm in "${arms[@]}"; do
  container="issue6667-$label-$arm"
  logs="$keep/$label-$arm"
  mkdir -p "$logs"
  echo "== $arm at $(date --iso-8601=seconds)" | tee -a "$keep/$label-run.txt"
  podman rm --force "$container" > /dev/null 2>&1
  podman run --detach --name "$container" --cpus 4 --memory 6g --pids-limit 1024 --blkio-weight 10 --rm \
    "$image" sleep infinity > /dev/null || exit 1
  podman exec "$container" mkdir -p /work/repo /work/log /work/tmp
  podman cp "$probe/fixture/." "$container:/work/repo/"
  podman cp "$binary" "$container:/work/linter"
  podman cp "$probe/arm.sh" "$container:/work/arm.sh"
  podman exec "$container" chmod 755 /work/linter
  podman exec "$container" chmod --recursive a+rwX /work
  case "$arm" in
    arm-a)
      podman exec "$container" mkdir -p /work/home
      podman exec "$container" bash /work/arm.sh "$arm" > /dev/null 2>&1
      ;;
    arm-check)
      podman exec "$container" mkdir -p /work/home
      podman exec "$container" env DYLINT_DRIVER_PATH=/opt/drivers bash /work/arm.sh "$arm" > /dev/null 2>&1
      ;;
    arm-no-driver-dir)
      podman exec --user "$uid" "$container" env -u HOME -u DYLINT_DRIVER_PATH -u XDG_CACHE_HOME \
        bash /work/arm.sh "$arm" > /dev/null 2>&1
      ;;
    arm-no-cache-dir)
      podman exec "$container" mkdir -p /work/drivers-empty
      podman exec "$container" chmod 777 /work/drivers-empty
      podman exec --user "$uid" "$container" env -u HOME -u XDG_CACHE_HOME DYLINT_DRIVER_PATH=/work/drivers-empty \
        bash /work/arm.sh "$arm" > /dev/null 2>&1
      ;;
    arm-resolve-fails)
      podman exec "$container" mkdir -p /work/drivers-empty /work/home/.cache
      podman exec "$container" chmod 777 /work/drivers-empty
      podman exec "$container" chmod 500 /work/home/.cache
      podman exec --user "$uid" "$container" env HOME=/work/home -u XDG_CACHE_HOME DYLINT_DRIVER_PATH=/work/drivers-empty \
        bash /work/arm.sh "$arm" > /dev/null 2>&1
      ;;
    *)
      echo "== unknown arm $arm" | tee -a "$keep/$label-run.txt"
      ;;
  esac
  podman cp "$container:/work/log/." "$logs/" > /dev/null 2>&1
  podman rm --force "$container" > /dev/null 2>&1
  cat "$logs/run.txt" >> "$keep/$label-run.txt" 2>/dev/null
  echo "-- $arm container removed at $(date --iso-8601=seconds)" | tee -a "$keep/$label-run.txt"
done

printf '%s\n' "== done $(date --iso-8601=seconds)" "== containers left" | tee -a "$keep/$label-run.txt"
podman ps --all --format '{{.Names}}' | tee -a "$keep/$label-run.txt"
