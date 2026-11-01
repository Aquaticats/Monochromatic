# Podman inside an unprivileged `podman build`: pulling an image works with a single ID mapping, running a container does not (2026-11-01)

The repository `Dockerfile` ends with `RUN mise run prepare`,
which used to run `prepare:playwright`,
and that built `playwright.Dockerfile` with a nested `podman build`.
The nested build failed in every configuration tried.
Pulling an image inside the same build container works,
and `prepare:playwright` now pulls the published image instead
(`doc/runbook/publish-playwright-image.md`).

## Symptom

The outer command is an unprivileged,
rootless `podman build` on the Fedora-based development host,
with the Ubuntu-pinned base image of the repository `Dockerfile`,
`podman` installed from Ubuntu's packages
(the version was not recorded),
and the vfs storage driver (`/etc/containers/storage.conf`).
Inside it,
with a subordinate ID range of `root:100000:65536` in `/etc/subuid` and `/etc/subgid`:

```text
Error: command required for rootless mode with multiple IDs: exec: "newuidmap": executable file not found in $PATH
```

With the `uidmap` package installed (it provides `newuidmap`):

```text
level=error msg="running `/usr/bin/newuidmap 16 0 0 1 1 100000 65536`: newuidmap: write to uid_map failed: Operation not permitted\n"
Error: cannot set up namespace using "/usr/bin/newuidmap": exit status 1
```

Containers could not be run with any setting that was tried
(`podman build`,
`podman run`):
the failures moved from the ID map to mounts
(`proc` passes with a container configuration made for podman-in-podman,
then the `devpts` mount is refused;
with `--isolation=chroot` the remount of `/sys/fs/cgroup` fails;
binding `/dev/pts` and `/sys` from the build container fails remounting a file descriptor).

## Root cause

Measured behavior only;
the source of podman and shadow-utils was not read for this record.
The errors are consistent with a build container that runs in a user namespace of its own:
`newuidmap` is refused writing a map for the `100000:65536` range,
and a container run needs mounts (`devpts`,
`/sys/fs/cgroup`,
`proc`) that the build container is refused.
A pull needs neither:
with no subordinate IDs podman uses a single mapping into the namespace,
and only the ownership of layer files is at stake.

## Verification

Each configuration was one `Dockerfile` of five to seven steps built with
`podman build --memory=6g --cpu-period=100000 --cpu-quota=200000`,
then removed.
The base was `ubuntu:latest@sha256:f3d28607ddd78734bb7f71f117f3c6706c666b8b76cbff7c9ff6e5718d46ff64`
with `podman` and `curl` installed,
vfs storage,
and a final step that pulled a public image and ran `podman image exists` and `podman images`.

Pull works:

- Empty `/etc/subuid` and `/etc/subgid`,
  and `ignore_chown_errors = "true"` under `[storage.options.vfs]`:
  `ghcr.io/astral-sh/uv:latest` (42 MB) pulled and `podman images` listed it.
  Podman printed `Using rootless single mapping into the namespace. This might break some images.`
- The same configuration with the `uidmap` package installed:
  the same result.
- The same configuration with the real `mcr.microsoft.com/playwright:v1.63.0-noble@sha256:eff16c30e6f3f4af0a03fa4b706120d5e9b0891c344a27d64559aff5900a4a27`:
  pulled,
  tagged `localhost/monochromatic-playwright`,
  and `podman image exists monochromatic-playwright` exited 0.
  The image is 2.53 GB as listed,
  and `/var/lib/containers/storage` was 8.4 GB afterward (vfs copies every layer).

Pull fails:

- `root:100000:65536` without `uidmap`:
  the first error above.
- `root:100000:65536` with `uidmap`:
  the second error above.
- Empty `/etc/subuid` and `/etc/subgid` without `ignore_chown_errors`,
  pulling the same Playwright image:
  the layer that holds `/etc/gshadow` (group 42) fails with

  ```text
  potentially insufficient UIDs or GIDs available in user namespace (requested 0:42 for /etc/gshadow): Check /etc/subuid and /etc/subgid if configured locally and run "podman system migrate": lchown /etc/gshadow: invalid argument
  ```


## Verified workarounds

The `Dockerfile` of this repository carries the working pull configuration:
empty `/etc/subuid` and `/etc/subgid`,
and `ignore_chown_errors = "true"`.
Tradeoffs:
every layer file in the pulled image is owned by root of the build container,
so the image is only good for pulling and tagging inside the build,
not for running containers from it;
`podman run` and `podman build` stay impossible there.
A pulled image occupies several times its size in the build's storage under vfs.

## What does not work

- Installing `uidmap` and keeping the subordinate range:
  `newuidmap` is refused writing the map.
- Podman-in-podman container configuration with a range inside the build's IDs:
  the `proc` mount passes,
  then `devpts` is refused.
- `--isolation=chroot` for the nested build:
  the remount of `/sys/fs/cgroup` fails.
- Binding `/dev/pts` and `/sys` into the nested build:
  the remount of the bound file descriptor fails.

## Upstream filing decision

No upstream filing:
the refusals are the isolation an unprivileged build is meant to have
(the constraint "is it really upstream's fault" fails),
and nothing in this record is a podman defect.
Nothing was prototyped.
