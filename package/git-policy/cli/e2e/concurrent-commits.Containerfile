# What:  base image for the cli-git concurrent-commit end-to-end workload replay suite
#        (`mise run //package/git-policy/cli:test:e2e:concurrent:native`, which links the native executable into it).
# Why:   concurrent mutation of shared Git state is only verified against real repositories
#        (doc/decision/cli-git-concurrent-commits.md, "Container end-to-end verification").
#        Distribution images ship an older Git, so the Git version under test builds from source:
#        2.55.0, the current release and the oldest the native wrapper accepts (it refuses older Gits and runs nothing).
# Build context: any directory; nothing is copied from it.
# Checksums: https://www.kernel.org/pub/software/scm/git/sha256sums.asc.
# Every RUN uses exec form: no shell logic lives in this file.

# Node 24.11.0 is the version this image has always run the suite with (the floor the retired TypeScript wrapper named);
# the suite is TypeScript run by Node type stripping, so the version is kept as it was.
FROM docker.io/library/node:24.11.0-slim AS git-build

RUN ["apt-get", "update"]
RUN ["apt-get", "install", "--yes", "--no-install-recommends", "build-essential", "zlib1g-dev", "xz-utils", "cargo"]

ADD --checksum=sha256:457fdb04dc8728e007d4688695e6912e6f680727920f2a40bf11eacc17505357 https://www.kernel.org/pub/software/scm/git/git-2.55.0.tar.xz /src/git-2.55.0.tar.xz

RUN ["tar", "--extract", "--xz", "--file=/src/git-2.55.0.tar.xz", "--directory=/src"]

# Transport, translation, and scripting-language features are unused by the suite;
# the local bare remote is reached through the file transport.
# Rust stays at 2.55.0's default (enabled, RelNotes/2.55.0.adoc), so Debian's cargo builds it (MSRV 1.49).
RUN ["make", "--directory=/src/git-2.55.0", "--jobs=2", "prefix=/opt/git/2.55.0", "NO_CURL=1", "NO_EXPAT=1", "NO_TCLTK=1", "NO_GETTEXT=1", "NO_PERL=1", "NO_PYTHON=1", "NO_OPENSSL=1", "install"]

FROM docker.io/library/node:24.11.0-slim AS consumer

# `ssh-keygen` backs the SSH-signing scenarios.
RUN ["apt-get", "update"]
RUN ["apt-get", "install", "--yes", "--no-install-recommends", "openssh-client"]

COPY --from=git-build /opt/git /opt/git

# The suite imports the workspace package @monochromatic-dev/caught-value-core through its `/ts` source,
# which the run task mounts read-only at /fixture/caught-value. Node resolves /fixture/e2e imports through
# /node_modules; the symlink's real path lies outside node_modules, so Node's type stripping still applies.
RUN ["mkdir", "--parents", "/node_modules/@monochromatic-dev"]
RUN ["ln", "--symbolic", "/fixture/caught-value", "/node_modules/@monochromatic-dev/caught-value-core"]

# The image deliberately has no /usr/bin/git: each scenario selects its Git through PATH order.
WORKDIR /work
