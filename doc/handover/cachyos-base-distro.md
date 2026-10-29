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

## Done

- Tool and test images on CachyOS, verified on `cachyos-v4` (subagent report, 2026-10-29):
   `search-shadow/cli` `b2b8297ff6` (test:container 44 PASS, rerun independently by monochromatic-71;
   ack is not in Arch or CachyOS repos, so it installs App::Ack from CPAN),
   `file-enforcer/core/generator` `6985201202`, `d371bc2e04`, `1e523469fb`
   (Arch PostgreSQL wants clang for JIT bitcode, so the libversion build uses `with_llvm=no`),
   `test-fixture/file-enforcer-perf` `856bb719ae`, `cbc7292378`,
   `nested-wayland-session/cli` `7c448d8a3e`, `2f68ce13f8`.
- pacman hooks need `--disable-sandbox` in rootless podman builds:
   `doc/troubleshooting/pacman-hook-sandbox-rootless-podman-build.md` (`bd06f4c440`).
- Fedora dropped from the test matrices (owner decision, `8627135ed7`).

## Found during verification, not caused by the base change

- `nested-wayland-session/cli` `verify:container`:
   two `appearance_portal` tests fail with `XDG_RUNTIME_DIR is not set`
   since `45cec40b27` made the private D-Bus need it;
   the container task never sets it (fails on Fedora too).
  With `--tmpfs /xdg --env XDG_RUNTIME_DIR=/xdg`, 279 of 279 tests pass.
- `test-fixture/file-enforcer-perf` `perf:constrained`:
   `StalenessManifestPersistenceError` (staleness manifest lock timeout) from file-enforcer-core,
   reproduced on the host outside any container.
- `test-fixture/file-enforcer-perf/TODO.md` asks to pin Node, which conflicts with the "latest" decision.
- The first p-memoize mutation run failed building its image:
   `pnpmfile hook 'updateConfig' timed out after 30 seconds` at load average 58;
   rerun as unit `mutation-p-memoize-2`.

## Queued for this session

- `package/mutation-test/cli/runtime/Containerfile` (`FROM fedora:latest`):
   migrate after the running `mutation-p-memoize` and `mutation-p-map` systemd units finish,
   because changing it changes the runtime image hash mid-run.
  Then rebuild `mutation-test/cli` and run one package's `test:mutation` on the new image.

## Out of scope by owner choice

Release workflows (Ubuntu 24.04 glibc floor), the root `Dockerfile`, `mvm` default VM,
 `vm-builder` (`ucore-hci`), `config/pnpr`, `music-player/desktop-app/Containerfile.alpine`.
