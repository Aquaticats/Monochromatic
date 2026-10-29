# Container base distro: CachyOS, latest

## Decision

Containers this repo builds start from `docker.io/cachyos/cachyos:latest`, unpinned.
On an architecture CachyOS publishes no image for,
 the container starts from Alpine instead.

The owner's directive (2026-10-29):
 "Make cachyOS (or arch linux proper) the de-facto distro for this repo from now on.
Fallback to alpine for architectures arch doesn't support."

Asked whether to pin a dated image,
 the owner answered:
 "Must be latest, since if an upstream OS update breaks us,
 we are doing something wrong and we'd have to fix it immediately."
So base images carry no version tag and no digest;
 a build that an upstream update breaks is a bug to fix at once,
 never a reason to pin.

## Facts relied on (Docker Hub, 2026-10-29)

- `cachyos/cachyos` publishes only the `latest` tag, for `amd64`.
- `cachyos/cachyos-v3` and `cachyos/cachyos-v4` are builds for newer x86-64 CPU levels;
   `-v3` needs x86-64-v3 (AVX2) and fails on CPUs without it.
  The repo uses plain `cachyos/cachyos`, which runs on any x86-64 CPU,
   including CI runners whose CPU level is not guaranteed.
- `archlinux` (Arch proper) also publishes `amd64` only,
   so choosing Arch over CachyOS would not change the fallback.
- `alpine` publishes `amd64`, `arm64/v8`, `arm/v6`, `arm/v7`, `386`, `ppc64le`, `riscv64` and `s390x`.
  Alpine uses musl:
   official Node.js Linux builds need glibc,
   so an Alpine container installs Node.js from `apk`
   (see the `node` handling in `package/test-harness/core-matrix/src/distro.ts`).

## Where it applies

- Every Containerfile that builds a tool, test, or build environment.
- Container images named in code that runs containers (for example `podman run <image>` in tests).

## Exceptions the owner chose to keep

- Release binaries (`cargo-publish.yml`, `music-player-release.yml`) keep building on Ubuntu 24.04 runners,
   because the build host's glibc sets the minimum glibc of every released Linux binary;
   CachyOS ships the newest glibc,
   so binaries built there would not start on Ubuntu 24.04 or Debian 12.
- `package/test-harness/core-matrix` tests across several distros on purpose:
   CachyOS is added as a target,
   the other distros stay.
- The root `Dockerfile` (Ubuntu CI canary image) and the `mvm` default VM template (Ubuntu) stay as they are.
- Images that are not general bases stay:
   `package/vm-builder/cli` boots `ucore-hci`,
   `package/config/pnpr` extends the published `pnpr` image,
   `package/ide/app` extends the terminal image,
   and `package/music-player/desktop-app/Containerfile.alpine` is the deliberate musl build.

## Context

The decision followed a mutation-test image rebuild whose `dnf install` step ran at about 56 KiB/s:
 this machine's public IP geolocates to Paris,
 so Fedora's metalink handed out French mirrors that served 52 KB/s,
 while a US Fedora mirror served 1.4 MB/s and Cloudflare 24 MB/s from the same host.
