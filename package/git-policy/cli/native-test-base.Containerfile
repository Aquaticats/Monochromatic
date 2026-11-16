# What:  base image of the native wrapper gate (`mise run //package/git-policy/cli:native:test:container`) and of the
#        mutation campaigns that start from the gate image: CachyOS with the Rust toolchain, the build tools, and every
#        tool the tests start (Git, Node, pnpm, strace, cargo-nextest).
# Why:   the gate image itself builds without network, so every download happens here, once per CachyOS image: the
#        build script (bin/native-test-base.ts) pulls the `latest` tag, and this image is rebuilt only when that pull
#        brings a different image, when this file changes, or when toolchain.lock.json moves the Rust toolchain's date
#        (doc/decision/container-base-distro.md; the arguments the tag covers).
# Pins:  none from the distribution. Owner decisions of 2026-11-04: "We don't need to pin git because my current system
#        is already using git latest.", "We don't need to pin Node too.", and, asked where the tools toolchain.lock.json
#        locks should come from in test images, "Pacman for everything it has" (an exception to AGENTS.md rule CM7 for
#        these images). So Git is whatever pacman's current `git` is, and Node, pnpm, strace and cargo-nextest are
#        pacman's current packages too; `mise run //package/git-policy/cli:test:git-image` prints the release of each the
#        image got. The wrapper's MINIMUM_GIT_VERSION (src/native/invocation/git_version.rs) follows the Git release here.
#        Rust is the one tool pacman has no dated pin for: the lock names a nightly, so the image installs exactly that
#        name, and pacman's `rustup` (which pacman has) fetches it. Owner decision of 2026-11-15: "Everything nightly.
#        This doesn't break "pacman for everything it has" if you're implying pacman doesn't have nightly. And the lock's
#        nightly is always the latest nightly anyway." The name is passed by bin/native-test-base.ts from
#        .tools.rust.version of toolchain.lock.json, so no date is written here, and the image tag covers it: a lock that
#        moves the date builds a new base instead of reusing one compiled by the night before.
#        pacman's `rustup` conflicts with pacman's `rust`, `cargo` and `rustfmt`, so this image takes Rust from the one
#        and not from the others; nothing else here installed the distribution's Rust packages.
# Signatures: pacman checks package signatures (`SigLevel = Required DatabaseOptional` in the CachyOS image's
#        /etc/pacman.conf); nothing below turns that off. rustup verifies each component against the manifest it
#        downloads, whose signature it checks with the rust-lang key its own installation pins.
# Build context: any directory; nothing is copied from it.
# Build arguments: CACHYOS_IMAGE (the base, resolved by package/test-harness/core-matrix) and RUST_TOOLCHAIN (the dated
#        nightly .tools.rust.version records; required, never defaulted here).
ARG CACHYOS_IMAGE=docker.io/cachyos/cachyos:latest
FROM ${CACHYOS_IMAGE}

ARG RUST_TOOLCHAIN

# --disable-sandbox: pacman's hook sandbox needs a network namespace a rootless podman build cannot give it
# (doc/troubleshooting/pacman-hook-sandbox-rootless-podman-build.md). The cache mount keeps downloaded packages between
# builds outside the image. `rustup` carries the toolchain installer, and the nightly it installs carries cargo, Clippy,
# rustfmt, rustdoc and the standard library. `base-devel` carries the compiler and linker that build the crates.
# `strace` injects the faults of the fault-injection tests (they check nothing without it). `nodejs` and `pnpm` serve
# the controls that run a real pnpm. `man-db` renders the packaged githooks(5) page, which the hook-event guard test
# reads when no AsciiDoc source is installed (the Git package ships the page, not the source). The remaining packages are
# the tools the tests start (`ps`, `ssh-keygen`, perl, python); the image already carries zlib-ng-compat, which provides
# zlib.
RUN --mount=type=cache,target=/var/cache/pacman/pkg,sharing=locked \
    ["pacman", "--disable-sandbox", "--sync", "--refresh", "--sysupgrade", "--noconfirm", "--needed", \
     "base-devel", "rustup", "git", "cargo-nextest", "nodejs", "pnpm", "strace", "man-db", "perl", "python", "openssh", \
     "procps-ng"]

# An absolute RUSTUP_HOME, so every stage and every user of this image finds the one toolchain installed below: the gate
# stage runs as uid 1000 with its own HOME, and the end-to-end image compiles as root.
ENV RUSTUP_HOME=/opt/rustup

# The toolchain the lock names, and nothing else: a build without it says what to pass rather than compiling with
# whatever rustup would pick.
RUN test -n "$RUST_TOOLCHAIN" || { echo 'error: --build-arg RUST_TOOLCHAIN is required; pass .tools.rust.version from toolchain.lock.json (bin/native-test-base.ts does).' >&2; exit 1; }
RUN rustup toolchain install "$RUST_TOOLCHAIN" --profile minimal --component clippy --no-self-update

# The toolchain's own programs come first on PATH, so `cargo`, `rustc`, `rustdoc` and `cargo clippy` are this night's
# compiler for every stage and every user of the image, and no step names a toolchain or a path.
ENV PATH=/opt/rustup/toolchains/${RUST_TOOLCHAIN}/bin:$PATH

# What the compiler is, and that the tools the tests start are there: printed once here, so the log of a base built on a
# new date shows which compiler its layers were cached against.
RUN ["sh", "-c", "rustc --version --verbose && cargo --version && cargo clippy --version && git --version && node --version && pnpm --version && strace -V | head --lines=1 && cargo nextest --version | head --lines=1 && man --where githooks"]
