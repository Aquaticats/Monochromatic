# CachyOS base distro migration

Decision:
 `doc/decision/container-base-distro.md` and AGENTS.md rule `BXD` (`0cc8c31051`, 2026-10-29).
Containers start `FROM docker.io/cachyos/cachyos:latest`, unpinned;
 Alpine only on architectures CachyOS lacks.

Owner follow-ups the same day:
 CPU-level ladder ("v4, fallback to v3, and plain, and so on.") through `ARG CACHYOS_IMAGE`,
 and images that build shipped releases are split into a CachyOS dev image and a release image on the current base.

## In flight (2026-10-29)

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
- Fedora dropped from the test matrices (owner decision, `8627135ed7`; code `a055954b40`).
- `test-harness/core-matrix`: `cachyos` distro (`98c49b0603`), CPU-level resolver and `cachyos-image` bin (`6204200b67`),
   pacman installs with a full sync and no hook sandbox (`038e3d9077`; lint, types and 19 test suites pass).
- file-enforcer ensure-package container matrix: CachyOS cells (`15cfc9951d`, `04d9489934`, `241bc0c9b0`, `8c4ad0e59d`);
   final run `4 cells passed` (ubuntu and cachyos, root and user).
- Survey: `ide-release.yml:193` builds the IDE image natively on `ubuntu-24.04-arm`,
   so the IDE release path must use the multi-arch Ubuntu 24.04 release image;
   no workflow moves images between machines.
- Known gap left alone: `container:arch` resolves to `arch:latest`, which does not exist (the image is `archlinux`).

## Found during verification, not caused by the base change

- `nested-wayland-session/cli` `verify:container`:
   two `appearance_portal` tests fail with `XDG_RUNTIME_DIR is not set`
   since `45cec40b27` made the private D-Bus need it;
   the container task never sets it (fails on Fedora too).
  Fixed in `de17d1f557` (owner approved): `containerCargo` mounts a private tmpfs at `/run/user/0` as `XDG_RUNTIME_DIR`;
   `test:container` then printed `279 tests run: 279 passed, 0 skipped`.
- `test-fixture/file-enforcer-perf` `perf:constrained`:
   `StalenessManifestPersistenceError` (staleness manifest lock timeout) from file-enforcer-core,
   reproduced on the host outside any container; filed as issue #1813 (owner chose an issue over a fix now).
- `test-fixture/file-enforcer-perf/TODO.md` asked to pin Node, against the "latest" decision; removed in `c1444df68b`.
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
