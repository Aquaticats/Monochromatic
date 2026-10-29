# CachyOS base distro migration

Decision:
 `doc/decision/container-base-distro.md` and AGENTS.md rule `BXD` (`0cc8c31051`, 2026-10-29).
Containers start `FROM docker.io/cachyos/cachyos:latest`, unpinned;
 Alpine only on architectures CachyOS lacks.

## In flight (2026-10-29)

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
