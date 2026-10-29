# CachyOS base distro migration

Decision:
 `doc/decision/container-base-distro.md` and AGENTS.md rule `BXD` (`0cc8c31051`, 2026-10-29).
Containers start `FROM docker.io/cachyos/cachyos:latest`, unpinned;
 Alpine only on architectures CachyOS lacks.

Owner follow-ups the same day:
 CPU-level ladder ("v4, fallback to v3, and plain, and so on.") through `ARG CACHYOS_IMAGE`,
 and images that build shipped releases are split into a CachyOS dev image and a release image on the current base.

## In flight (2026-10-29)

- Subagent in `test-harness/core-matrix`: the CPU-level resolver (`--build-arg CACHYOS_IMAGE=<image>`),
   plus a check for non-x86-64 container builds and images built in one place and run elsewhere.
  When it lands, the tool and desktop agents wire it into their image builds.
- `terminal/app` and `ide/app` are held until the dev/release split lands;
   monochromatic-ec dry-runs the IDE release on that commit before any real release, so send them the hash.
- monochromatic-71 wants the `search-shadow/cli` `test:container` result line.

- Subagent: tool and test Containerfiles
   (`search-shadow/cli`, `file-enforcer/core/generator`, `test-fixture/file-enforcer-perf`, `nested-wayland-session/cli`).
- Subagent: desktop Containerfiles
   (`music-player/desktop-app` `Containerfile`, `.sign`, `.icons`, `.windows-gnu`; `terminal/app`; check `ide/app`, which builds on the terminal image).
  Verification builds run in a throwaway worktree because other sessions have uncommitted test edits in those packages.
- Subagent: CachyOS target in `test-harness/core-matrix` (default container distro)
   and a CachyOS cell in file-enforcer's ensure-package container matrix.

## Queued for this session

- `package/mutation-test/cli/runtime/Containerfile` (`FROM fedora:latest`):
   migrate after the running `mutation-p-memoize` and `mutation-p-map` systemd units finish,
   because changing it changes the runtime image hash mid-run.
  Then rebuild `mutation-test/cli` and run one package's `test:mutation` on the new image.

## Out of scope by owner choice

Release workflows (Ubuntu 24.04 glibc floor), the root `Dockerfile`, `mvm` default VM,
 `vm-builder` (`ucore-hci`), `config/pnpr`, `music-player/desktop-app/Containerfile.alpine`.
