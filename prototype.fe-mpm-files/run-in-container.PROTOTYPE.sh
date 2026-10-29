#!/bin/sh
# PROTOTYPE helper (throwaway; deviates from the repo's no-shell-script rule on purpose, because it only drives probes).
# Runs one probe script inside the base image, bounded to 2g memory and 2 cpus.
# usage: run-in-container.PROTOTYPE.sh <script-name-in-this-dir> [extra podman args before the image]
set -eu
here=$(cd "$(dirname "$0")" && pwd)
script=$1
shift
exec systemd-run --user --unit="fe-proto-probe-$$" --wait --pipe --collect \
  --property=MemoryMax=4G --property=CPUQuota=200% \
  podman run --rm --memory=2g --cpus=2 --security-opt label=disable \
  --volume "$here":/proto:ro "$@" localhost/fe-proto-base:2 bash "/proto/$script" ${PROTO_ARGS:-}
