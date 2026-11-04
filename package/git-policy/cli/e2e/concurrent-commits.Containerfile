# What:  base image for the cli-git concurrent-commit end-to-end workload replay suite
#        (`mise run //package/git-policy/cli:test:e2e:concurrent:native`, which links the native executable into it).
# Why:   concurrent mutation of shared Git state is only verified against real repositories
#        (doc/decision/cli-git-concurrent-commits.md, "Container end-to-end verification").
# Tools: all from pacman, current, with no pins. Owner decisions of 2026-11-04: "We don't need to pin git because my
#        current system is already using git latest.", "We don't need to pin Node too.", "Pacman for everything it has"
#        (an exception to AGENTS.md rule CM7 for these images), and, for the former 2.55.0 source build, "Raise minimum,
#        drop the test, merely print a warning on older git and not refuse." So the suite runs the distribution's
#        `git` at /usr/bin/git and reads its release at run time.
# Signatures: pacman checks package signatures (`SigLevel = Required DatabaseOptional` in the CachyOS image's
#        /etc/pacman.conf); nothing below turns that off.
# Base:  CachyOS on its `latest` tag (doc/decision/container-base-distro.md); bin/e2e-native-container.ts pulls the tag
#        and passes the resolved image as CACHYOS_IMAGE. Without the argument the plain image builds.
# Build context: any directory; nothing is copied from it.
# Build arguments: CACHYOS_IMAGE (the base).
ARG CACHYOS_IMAGE=docker.io/cachyos/cachyos:latest
FROM ${CACHYOS_IMAGE}

# One full system upgrade with the tools in the same transaction. --disable-sandbox: pacman's hook sandbox needs a
# network namespace a rootless podman build cannot give it (doc/troubleshooting/pacman-hook-sandbox-rootless-podman-build.md).
# `git` is the Git under test, `nodejs` runs the suite (TypeScript by Node type stripping), and `ssh-keygen` (openssh)
# backs the SSH-signing scenarios. The cache mount keeps downloaded packages between builds outside the image.
RUN --mount=type=cache,target=/var/cache/pacman/pkg,sharing=locked \
    ["pacman", "--disable-sandbox", "--sync", "--refresh", "--sysupgrade", "--noconfirm", "--needed", "git", "nodejs", "openssh"]

# The suite imports the workspace package @monochromatic-dev/caught-value-core through its `/ts` source,
# which the run task mounts read-only at /fixture/caught-value. Node resolves /fixture/e2e imports through
# /node_modules; the symlink's real path lies outside node_modules, so Node's type stripping still applies.
RUN ["mkdir", "--parents", "/node_modules/@monochromatic-dev"]
RUN ["ln", "--symbolic", "/fixture/caught-value", "/node_modules/@monochromatic-dev/caught-value-core"]

# The check proves the packaged Node and Git start.
RUN ["sh", "-c", "node --version && /usr/bin/git --version"]
WORKDIR /work
