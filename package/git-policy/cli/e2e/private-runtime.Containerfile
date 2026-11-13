# What:
#  An image with a Git, a D-Bus daemon, a counting stand-in for a service manager's program, and a wrapper under
#  test as `git`, for `bin/proof-private-runtime-directory.ts`.
# Why:
#  Issue 6293's reproduction runs in a container whose bus and manager are its own, so nothing in this image can
#  reach this machine's manager or session bus.
#  The stand-in is the program the wrapper looks for and its only manager interaction: it records every call, then
#  does what `systemd-run --user --scope` does, which is run the command as its child in the caller's environment.
#  The wrapper is copied in rather than compiled here, so the two arms (before and after the fix) are one build of
#  this branch's executable each and the image needs no Rust.
# Base:
#  CachyOS on its `latest` tag (AGENTS.md rule BXD, doc/decision/container-base-distro.md);
#  `bin/proof-private-runtime-directory.ts` pulls the tag and passes the resolved image as CACHYOS_IMAGE.
#  Without the argument the plain image builds.
# Build context:
#  the image's own, made by the program: `git-under-test`, `stand-in-systemd-run`, `repro-in-a-container.sh` and a
#  copy of this file.
# Network:
#  the build needs one for pacman; the proof itself does not.
ARG CACHYOS_IMAGE=docker.io/cachyos/cachyos:latest
FROM ${CACHYOS_IMAGE}

# One full system upgrade with the tools in the same transaction. --disable-sandbox: pacman's hook sandbox needs a
# network namespace a rootless podman build cannot give it
# (doc/troubleshooting/pacman-hook-sandbox-rootless-podman-build.md).
# `dbus` brings dbus-daemon, dbus-send and dbus-monitor, which are the bus and its log; `git` is the real Git the
# wrapper forwards to.
RUN --mount=type=cache,target=/var/cache/pacman/pkg,sharing=locked \
    ["pacman", "--disable-sandbox", "--sync", "--refresh", "--sysupgrade", "--noconfirm", "--needed", "git", "dbus"]

RUN ["mkdir", "--parents", "/proof"]
COPY git-under-test /usr/local/bin/git
COPY stand-in-systemd-run /usr/local/bin/systemd-run
COPY repro-in-a-container.sh /proof/repro-in-a-container.sh
RUN ["chmod", "0555", "/usr/local/bin/git", "/usr/local/bin/systemd-run", "/proof/repro-in-a-container.sh"]
CMD ["/bin/sh", "/proof/repro-in-a-container.sh"]
