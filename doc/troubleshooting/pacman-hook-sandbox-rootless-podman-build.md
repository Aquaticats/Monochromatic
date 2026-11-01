# pacman 7.1.0: post-transaction hooks fail inside a rootless `podman build`

Investigated on 2026-10-29 while moving four Containerfiles from Fedora to `docker.io/cachyos/cachyos` (see
[container-base-distro](../decision/container-base-distro.md)).

Short answer: pass `--disable-sandbox` to `pacman` in every `RUN` step of a rootless podman build.
Without it `pacman` exits 0 and the packages install, but every hook that runs under its sandbox fails,
so effects that live in hooks are silently missing from the image.

## Symptom

Build step: `RUN pacman --sync --refresh --sysupgrade --noconfirm --needed postgresql`, image
`docker.io/cachyos/cachyos:latest` (pacman v7.1.0, libalpm v16.0.1), rootless podman 5.8.7.
The transaction output ends with:

```text
could not isolate the network (Operation not permitted)
refusing to run "/usr/share/libalpm/scripts/systemd-hook" with network access; set DisableSandboxNetwork in pacman.conf to override
error: command failed to execute correctly
```

and `ldconfig` prints `warning: could not isolate the network for "/usr/bin/ldconfig" (Operation not permitted)`.
A later step `install --owner=postgres` then fails with `install: invalid user 'postgres'`,
because the `systemd-sysusers` hook is one of the hooks that failed, so the package's `postgres` user was never created.

## Cause

`pacman` runs hooks and scriptlets in a sandbox that unshares a network namespace.
A rootless container build lacks the capability to create one (`Operation not permitted`),
and the hook runner refuses to run a hook with network access rather than run it unsandboxed.

## Fix

`pacman --disable-sandbox --sync ...`; the option is accepted by pacman 7.1.0 in that image.
Verified with a one-step build of `pacman --disable-sandbox --sync --refresh --sysupgrade --noconfirm --needed postgresql`
followed by `id postgres`, which printed `uid=968(postgres) gid=968(postgres) groups=968(postgres)`;
the hook output showed `Creating user 'postgres' (PostgreSQL user) with UID 968 and GID 968.`

The container is already the isolation boundary for a build step, so nothing is lost by disabling the inner sandbox.
