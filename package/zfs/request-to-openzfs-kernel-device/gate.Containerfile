# What: the image ZFS plan step 1 runs its merge checks in: the wrapper's gate
#        base image, the two lock-row tools, the two crates' locked dependencies,
#        and the pinned browserslist data an offline `file-enforcer check` needs.
# Why:  the plan's "How it merges" runs every check in one container with no
#        network and no home folder, so every download happens here, once.
# Build: GATE_BASE_IMAGE is the gate base image the wrapper's resolver prints
#        (`mise run //package/git-policy/cli:native:image:build` names the tag).
#        It is never typed here: a CachyOS update or a change to the wrapper's
#        gate Containerfile moves it (rule BXD,
#        doc/decision/container-base-distro.md).
# Context: the two lock-row binaries at file-enforcer and monochromatic-lint,
#        and the subject and its sidecar under deps/.
ARG GATE_BASE_IMAGE
FROM ${GATE_BASE_IMAGE}
COPY file-enforcer /usr/local/bin/file-enforcer
COPY monochromatic-lint /usr/local/bin/monochromatic-lint
COPY deps/subject /deps/subject
COPY deps/sidecar /deps/sidecar
# Registry cache for the two crates, fetched once at image-build time; every
# test run is offline.
RUN ["cargo", "fetch", "--locked", "--manifest-path", "/deps/subject/Cargo.toml"]
RUN ["sh", "-c", "mkdir --parents /deps/tree/package/zfs && cp --recursive /deps/subject /deps/tree/package/zfs/request-to-openzfs-kernel-device && cp --recursive /deps/sidecar /deps/tree/package/zfs/request-to-openzfs-kernel-device.fuzz && cargo fetch --locked --manifest-path /deps/tree/package/zfs/request-to-openzfs-kernel-device.fuzz/Cargo.toml"]
# file-enforcer's browserslist rule resolves against a cache of the pinned npm
# tarballs and fetches a missing one on first use
# (package/file-enforcer/cli/src/kind_browserslist.rs); offline, that fetch is
# the check's own failure. The tarballs are therefore fetched here, where the
# plan's container rule allows the network, checked against
# browserslist-data.lock.json's sha512 digests, and left where the rule reads
# them (`DataSource::user_default()`, which is
# $HOME/.cache/browserslist for root).
# This layer leaves once the wrapper's base image carries that data; the open
# branch fix/git-policy-cli-gate-base-image-carries-the-pinned-browserslist-data
# does exactly that.
RUN ["sh", "-c", "set -eu; cache=/root/.cache/browserslist; mkdir --parents \"$cache\"; curl --fail --silent --show-error --location --output \"$cache/81f583e8cc6a542a803b098e0df65fec37215af96b03e49174d5fb71e2df837c51a9ee6cb238e917b120b250bfd2d88b88d4da8dd68d37fabc73fe6effb1b3f3.tgz\" https://registry.npmjs.org/caniuse-lite/-/caniuse-lite-1.0.30001820.tgz; curl --fail --silent --show-error --location --output \"$cache/03412afaa543408f7b21b003ea0fc7ae222ebf6e7a7a5f4df04c05e3ec6a278f69c2b7516297424a62e4d2c3b82377e74ba869604e6fdb607bdc193e580ac4a1.tgz\" https://registry.npmjs.org/node-releases/-/node-releases-2.0.62.tgz; cd \"$cache\"; printf '%s  %s\\n' 81f583e8cc6a542a803b098e0df65fec37215af96b03e49174d5fb71e2df837c51a9ee6cb238e917b120b250bfd2d88b88d4da8dd68d37fabc73fe6effb1b3f3 81f583e8cc6a542a803b098e0df65fec37215af96b03e49174d5fb71e2df837c51a9ee6cb238e917b120b250bfd2d88b88d4da8dd68d37fabc73fe6effb1b3f3.tgz 03412afaa543408f7b21b003ea0fc7ae222ebf6e7a7a5f4df04c05e3ec6a278f69c2b7516297424a62e4d2c3b82377e74ba869604e6fdb607bdc193e580ac4a1 03412afaa543408f7b21b003ea0fc7ae222ebf6e7a7a5f4df04c05e3ec6a278f69c2b7516297424a62e4d2c3b82377e74ba869604e6fdb607bdc193e580ac4a1.tgz | sha512sum --check --strict"]
RUN ["file-enforcer", "--version"]
RUN ["monochromatic-lint", "--version"]
