# What:  base image of the native wrapper gate (`mise run //package/git-policy/cli:native:test:container`) and of the
#        mutation campaigns that start from the gate image: CachyOS, the Rust toolchain, the build tools, and the audited
#        Git 2.56.0 built from source at /usr/bin/git, where the wrapper's tests expect the real Git.
# Why:   the gate image itself builds without network, so every download happens here, once per CachyOS image: the
#        build script (bin/native-test-base.ts) pulls the `latest` tag, and this image is rebuilt only when that pull
#        brings a different image or this file changes (doc/decision/container-base-distro.md).
# Git:   the version under test is a fixed release, not whatever the distribution packages: the gate's results are
#        evidence about this Git (the tag v2.56.0 points at commit a018953688f1b10bddf91bff8747068f5f4746a4).
#        Checksum: https://www.kernel.org/pub/software/scm/git/sha256sums.asc.
# Build context: any directory; nothing is copied from it.
# Build arguments: CACHYOS_IMAGE (the base, resolved by package/test-harness/core-matrix) and BUILD_JOBS (parallel
#        compile jobs, the CPUs the build container may use).
ARG CACHYOS_IMAGE=docker.io/cachyos/cachyos:latest
FROM ${CACHYOS_IMAGE}

# --disable-sandbox: pacman's hook sandbox needs a network namespace a rootless podman build cannot give it
# (doc/troubleshooting/pacman-hook-sandbox-rootless-podman-build.md). The cache mount keeps downloaded packages between
# builds outside the image. `rust` carries cargo and Clippy. `base-devel` carries the compiler and make that build Git
# and `strace`. The remaining packages are Git's optional libraries (curl, expat, openssl; the image already carries
# zlib-ng-compat, which provides zlib) and the tools the tests start (`ps`, `ssh-keygen`, perl, python).
RUN --mount=type=cache,target=/var/cache/pacman/pkg,sharing=locked \
    ["pacman", "--disable-sandbox", "--sync", "--refresh", "--sysupgrade", "--noconfirm", "--needed", \
     "base-devel", "rust", "curl", "expat", "openssl", "perl", "python", "openssh", "procps-ng"]

# The distribution's Git would own /usr/bin/git and the files below /usr/lib/git-core, so it leaves before the audited
# release installs over the same prefix.
RUN ["pacman", "--disable-sandbox", "--remove", "--nodeps", "--nodeps", "--noconfirm", "git"]

ADD --checksum=sha256:26c56c296b38c0695b26fa95f475f1d01704d2d38e73465ca30b0b2f5dc789d3 \
    https://www.kernel.org/pub/software/scm/git/git-2.56.0.tar.xz /src/git-2.56.0.tar.xz

ARG BUILD_JOBS=2

# CLI tests use English diagnostics and no Tk UI. Perl, Python, curl and Git's default Rust support stay enabled; Git's
# Rust crate has no registry dependencies, so Cargo runs offline. The source tree is removed in the same step.
RUN ["sh", "-c", "tar --extract --xz --file=/src/git-2.56.0.tar.xz --directory=/src && make --directory=/src/git-2.56.0 --jobs=${BUILD_JOBS} prefix=/usr NO_GETTEXT=YesPlease NO_TCLTK=YesPlease PYTHON_PATH=/usr/bin/python3 'CARGO_ARGS=--offline --release' install && rm --recursive --force /src"]

RUN ["git", "--version"]
