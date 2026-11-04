# What:  base image of the native wrapper gate (`mise run //package/git-policy/cli:native:test:container`) and of the
#        mutation campaigns that start from the gate image: CachyOS with the Rust toolchain, the build tools, and every
#        tool the tests start (Git, Node, pnpm, strace, cargo-nextest), all installed from pacman.
# Why:   the gate image itself builds without network, so every download happens here, once per CachyOS image: the
#        build script (bin/native-test-base.ts) pulls the `latest` tag, and this image is rebuilt only when that pull
#        brings a different image or this file changes (doc/decision/container-base-distro.md).
# Pins:  none. Owner decisions of 2026-11-04: "We don't need to pin git because my current system is already using git
#        latest.", "We don't need to pin Node too.", and, asked where the tools toolchain.lock.json locks should come
#        from in test images, "Pacman for everything it has" (an exception to AGENTS.md rule CM7 for these images).
#        So Git is whatever pacman's current `git` is, and Node, pnpm, strace, cargo-nextest and Rust are pacman's
#        current packages too; `mise run //package/git-policy/cli:test:git-image` prints the release of each the image
#        got. The wrapper's MINIMUM_GIT_VERSION (src/native/invocation/git_version.rs) follows the Git release here.
# Signatures: pacman checks package signatures (`SigLevel = Required DatabaseOptional` in the CachyOS image's
#        /etc/pacman.conf); nothing below turns that off.
# Build context: any directory; nothing is copied from it.
# Build arguments: CACHYOS_IMAGE (the base, resolved by package/test-harness/core-matrix).
ARG CACHYOS_IMAGE=docker.io/cachyos/cachyos:latest
FROM ${CACHYOS_IMAGE}

# --disable-sandbox: pacman's hook sandbox needs a network namespace a rootless podman build cannot give it
# (doc/troubleshooting/pacman-hook-sandbox-rootless-podman-build.md). The cache mount keeps downloaded packages between
# builds outside the image. `rust` carries cargo and Clippy. `base-devel` carries the compiler and linker that build the
# crates. `strace` injects the faults of the fault-injection tests (they check nothing without it). `nodejs` and `pnpm`
# serve the controls that run a real pnpm. `man-db` renders the packaged githooks(5) page, which the hook-event guard
# test reads when no AsciiDoc source is installed (the Git package ships the page, not the source). The remaining
# packages are the tools the tests start (`ps`, `ssh-keygen`, perl, python); the image already carries zlib-ng-compat,
# which provides zlib.
RUN --mount=type=cache,target=/var/cache/pacman/pkg,sharing=locked \
    ["pacman", "--disable-sandbox", "--sync", "--refresh", "--sysupgrade", "--noconfirm", "--needed", \
     "base-devel", "rust", "git", "cargo-nextest", "nodejs", "pnpm", "strace", "man-db", "perl", "python", "openssh", \
     "procps-ng"]

RUN ["sh", "-c", "git --version && node --version && pnpm --version && strace -V | head --lines=1 && cargo nextest --version | head --lines=1 && man --where githooks"]
