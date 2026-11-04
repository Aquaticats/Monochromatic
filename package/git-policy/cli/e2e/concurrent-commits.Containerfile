# What:  base image for the cli-git concurrent-commit end-to-end workload replay suite
#        (`mise run //package/git-policy/cli:test:e2e:concurrent:native`, which links the native executable into it).
# Why:   concurrent mutation of shared Git state is only verified against real repositories
#        (doc/decision/cli-git-concurrent-commits.md, "Container end-to-end verification").
#        The Git version under test is a fixed release, not whatever the distribution packages, so it builds from source:
#        2.55.0, the oldest the native wrapper accepts (it refuses older Gits and runs nothing).
# Base:  CachyOS on its `latest` tag (doc/decision/container-base-distro.md); bin/e2e-native-container.ts pulls the tag
#        and passes the resolved image as CACHYOS_IMAGE. Without the argument the plain image builds.
# Build context: any directory; nothing is copied from it.
# Checksums: https://www.kernel.org/pub/software/scm/git/sha256sums.asc and https://nodejs.org/dist/v24.11.0/SHASUMS256.txt.
# Build arguments: CACHYOS_IMAGE (the base) and BUILD_JOBS (parallel compile jobs, the CPUs the build may use).
# Every RUN uses exec form, except the two that need `&&` or an argument expansion.
ARG CACHYOS_IMAGE=docker.io/cachyos/cachyos:latest

# One full system upgrade, shared by both stages below: the Git built in one stage runs in the other, so both must link
# the same C library. --disable-sandbox: pacman's hook sandbox needs a network namespace a rootless podman build cannot
# give it (doc/troubleshooting/pacman-hook-sandbox-rootless-podman-build.md). `ssh-keygen` (openssh) backs the
# SSH-signing scenarios. The cache mount keeps downloaded packages between builds outside the image.
FROM ${CACHYOS_IMAGE} AS system

RUN --mount=type=cache,target=/var/cache/pacman/pkg,sharing=locked \
    ["pacman", "--disable-sandbox", "--sync", "--refresh", "--sysupgrade", "--noconfirm", "--needed", "openssh"]

# The CachyOS image carries a Git, and this image deliberately has none at /usr/bin/git: each scenario selects its Git
# through PATH order, so the distribution's Git leaves before any stage builds on this one.
RUN ["pacman", "--disable-sandbox", "--remove", "--nodeps", "--nodeps", "--noconfirm", "git"]

FROM system AS git-build

# The package databases of the stage above are current, so installing without a refresh keeps every package on the
# versions the system upgrade chose. `rust` carries the Cargo that builds Git's Rust part.
RUN --mount=type=cache,target=/var/cache/pacman/pkg,sharing=locked \
    ["pacman", "--disable-sandbox", "--sync", "--noconfirm", "--needed", "base-devel", "rust"]

ADD --checksum=sha256:457fdb04dc8728e007d4688695e6912e6f680727920f2a40bf11eacc17505357 https://www.kernel.org/pub/software/scm/git/git-2.55.0.tar.xz /src/git-2.55.0.tar.xz

RUN ["tar", "--extract", "--xz", "--file=/src/git-2.55.0.tar.xz", "--directory=/src"]

# Transport, translation, and scripting-language features are unused by the suite;
# the local bare remote is reached through the file transport.
# Rust stays at 2.55.0's default (enabled, RelNotes/2.55.0.adoc), so the distribution's cargo builds it (MSRV 1.49).
ARG BUILD_JOBS=2
RUN ["sh", "-c", "make --directory=/src/git-2.55.0 --jobs=${BUILD_JOBS} prefix=/opt/git/2.55.0 NO_CURL=1 NO_EXPAT=1 NO_TCLTK=1 NO_GETTEXT=1 NO_PERL=1 NO_PYTHON=1 NO_OPENSSL=1 install"]

FROM system AS consumer

# Node 24.11.0 is the version this image has always run the suite with (the floor the retired TypeScript wrapper named);
# the suite is TypeScript run by Node type stripping, so the version is kept as it was, as the official Linux x64 build.
ADD --checksum=sha256:46da9a098973ab7ba4fca76945581ecb2eaf468de347173897044382f10e0a0a https://nodejs.org/dist/v24.11.0/node-v24.11.0-linux-x64.tar.xz /opt/node/node-v24.11.0-linux-x64.tar.xz

RUN ["sh", "-c", "tar --extract --xz --file=/opt/node/node-v24.11.0-linux-x64.tar.xz --directory=/opt/node --strip-components=1 && rm /opt/node/node-v24.11.0-linux-x64.tar.xz && ln --symbolic /opt/node/bin/node /usr/local/bin/node"]

COPY --from=git-build /opt/git /opt/git

# The suite imports the workspace package @monochromatic-dev/caught-value-core through its `/ts` source,
# which the run task mounts read-only at /fixture/caught-value. Node resolves /fixture/e2e imports through
# /node_modules; the symlink's real path lies outside node_modules, so Node's type stripping still applies.
RUN ["mkdir", "--parents", "/node_modules/@monochromatic-dev"]
RUN ["ln", "--symbolic", "/fixture/caught-value", "/node_modules/@monochromatic-dev/caught-value-core"]

# The image deliberately has no /usr/bin/git (see the system stage); the check fails the build if one is there, and
# proves the pinned Node and Git start.
RUN ["sh", "-c", "test ! -e /usr/bin/git && node --version && /opt/git/2.55.0/bin/git --version"]
WORKDIR /work
