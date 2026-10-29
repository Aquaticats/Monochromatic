# Container base distro: CachyOS, latest, by CPU level

## Decision

Containers this repo builds start from the newest CachyOS image the building host's CPU can run,
 always on the `latest` tag:
 `docker.io/cachyos/cachyos-v4:latest`,
 else `docker.io/cachyos/cachyos-v3:latest`,
 else `docker.io/cachyos/cachyos:latest`.
On a non-x86-64 host the container starts from Alpine instead.

Each Containerfile names its base through a build argument,
 so the resolver picks the image and the file stays CPU-neutral:

```dockerfile
# package/<category>/<project>/Containerfile
ARG CACHYOS_IMAGE=docker.io/cachyos/cachyos:latest
FROM ${CACHYOS_IMAGE}
```

The resolver lives in `package/test-harness/core-matrix`
 (it already owns the mapping from distro name to container image);
 image builds pass its `--build-arg CACHYOS_IMAGE=<image>`.
Without the argument the plain image builds,
 which runs on every x86-64 CPU.

## Owner directives (2026-10-29)

- "Make cachyOS (or arch linux proper) the de-facto distro for this repo from now on.
  Fallback to alpine for architectures arch doesn't support."
- On pinning:
   "Must be latest, since if an upstream OS update breaks us,
   we are doing something wrong and we'd have to fix it immediately."
  Base images carry no version tag and no digest;
   a build an upstream update breaks is a bug to fix at once,
   never a reason to pin.
- On CPU level:
   "v4, fallback to v3, and plain, and so on."
- On images that build shipped releases:
   split them (owner chose "Split dev and release").

## Facts relied on (Docker Hub and this host, 2026-10-29)

- `cachyos/cachyos`, `cachyos/cachyos-v3` and `cachyos/cachyos-v4` publish only the `latest` tag, for `amd64`.
  There is no `v2` image.
- `-v3` needs x86-64-v3 (AVX2) and `-v4` needs x86-64-v4 (AVX-512);
   on a CPU without them the image's programs fail.
  That is why the level is detected on the host that builds and runs the image.
- glibc's loader reports the supported levels:
   `/lib64/ld-linux-x86-64.so.2 --help` prints lines such as `x86-64-v4 (supported, searched)`.
  This host (AMD Ryzen 9 7950X) reports v4, v3 and v2 as supported.
- `archlinux` (Arch proper) also publishes `amd64` only,
   so choosing Arch would not change the fallback.
- `alpine` publishes `amd64`, `arm64/v8`, `arm/v6`, `arm/v7`, `386`, `ppc64le`, `riscv64` and `s390x`.
  Alpine uses musl:
   official Node.js Linux builds need glibc,
   so an Alpine container installs Node.js from `apk`
   (see the `node` handling in `package/test-harness/core-matrix/src/distro.ts`).

## The Alpine fallback contract

A pacman-based Containerfile cannot become an Alpine one through a build argument;
 the fallback needs a separate Alpine variant of that image.
Variants are written only when a non-x86-64 host actually builds that image.
Until then, resolving a CachyOS-only image on a non-x86-64 host throws an error that names the image
 and says it has no Alpine variant,
 instead of building something wrong.

## Where it applies

- Every Containerfile that builds a tool, test, or development environment.
- Container images named in code that runs containers (for example `podman run <image>` in tests).

## Exceptions

### Shipped release binaries keep their glibc floor

The build host's glibc sets the minimum glibc of every released Linux binary,
 and CachyOS ships the newest glibc,
 so binaries built there would not start on older distributions.
The owner chose to keep each release's current floor:

- Release workflows that build on GitHub runners
   (`cargo-publish.yml`, `music-player-release.yml`) keep their Ubuntu 24.04 runners.
- An image that serves both development and a shipped release is split:
   the development Containerfile moves to CachyOS,
   and a separate release Containerfile starts `FROM docker.io/library/ubuntu:24.04`,
   which every release path builds from.
  The owner chose Ubuntu 24.04 for release images ("Do Ubuntu 24.04 for release images instead."),
   so container-built releases share the floor of the Ubuntu 24.04 release runners.
  Release images carry the `24.04` tag on purpose:
   the tag is what fixes the glibc floor
   (Launchpad lists glibc `2.39-0ubuntu8.9` in noble's Updates and Security pockets on 2026-10-29,
   and `ldd --version` in `docker.io/library/ubuntu:24.04` printed `ldd (Ubuntu GLIBC 2.39-0ubuntu8.9) 2.39`).
  Example:
   IDE release binaries are compiled in `localhost/monochromatic/ide`,
   which is built from the terminal image,
   so `package/terminal/app` gets a CachyOS development image and an Ubuntu 24.04 release image.
  Before the split that image was Fedora 41
   (`ldd --version` in `localhost/monochromatic/terminal` printed `ldd (GNU libc) 2.40` on 2026-10-29),
   while the IDE release workflow's glibc floor check already targets 2.39
   (dry run 40877021793 on `e3ec1a5ff4`, reported by the IDE release owner session).

### Tools missing from the CachyOS repositories

- `osslsigncode` (Windows Authenticode signing, `package/music-player/desktop-app/Containerfile.sign`)
   is in neither the Arch nor the CachyOS repositories.
  Signing compiles nothing, so the release glibc floor does not apply;
   the owner chose "CachyOS, build from source" (2026-10-29):
   the image builds the latest upstream release from source.
  Ubuntu 24.04 was the alternative: it packages osslsigncode 2.8, older than the 2.9 the signing check verified.
- `ack` is likewise missing; the search-shadow test image installs App::Ack from CPAN.

### Binaries a development image builds for the host

A CachyOS-built binary needs CachyOS's glibc (2.44 on 2026-10-29) and does not start on an older host
 (a Fedora 44 host has 2.43).
Where a development task builds in the container and runs the result on the host
 (music-player's `run` fallback),
 it targets glibc 2.39 with cargo-zigbuild (owner chose "Build for an older glibc", 2026-10-29).

### Kept on purpose

- `package/test-harness/core-matrix` tests across several distros:
   CachyOS is added and becomes the default.
  Fedora was then dropped from the test matrices
   (owner chose "Drop Fedora from matrices" on 2026-10-29,
   after file-enforcer's Fedora cell spent 17 minutes in `dnf install` on slow mirrors);
   Ubuntu (apt) and CachyOS (pacman) cells stay,
   and an explicit `container:fedora` still resolves.
- The root `Dockerfile` (Ubuntu CI canary image) and the `mvm` default VM template (Ubuntu) stay as they are.
- Images that are not general bases stay:
   `package/vm-builder/cli` boots `ucore-hci`,
   `package/config/pnpr` extends the published `pnpr` image,
   and `package/music-player/desktop-app/Containerfile.alpine` is the deliberate musl build.

## Context

The decision followed a mutation-test image rebuild whose `dnf install` step ran at about 56 KiB/s:
 this machine's public IP geolocates to Paris,
 so Fedora's metalink handed out French mirrors that served 52 KB/s,
 while a US Fedora mirror served 1.4 MB/s and Cloudflare 24 MB/s from the same host.
