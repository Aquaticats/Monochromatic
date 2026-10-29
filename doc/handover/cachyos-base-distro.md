# CachyOS base distro migration

Decision:
 `doc/decision/container-base-distro.md` and AGENTS.md rule `BXD` (`0cc8c31051`, 2026-10-29).
Containers start `FROM docker.io/cachyos/cachyos:latest`, unpinned;
 Alpine only on architectures CachyOS lacks.

Owner follow-ups the same day:
 CPU-level ladder ("v4, fallback to v3, and plain, and so on.") through `ARG CACHYOS_IMAGE`,
 and images that build shipped releases are split into a CachyOS dev image and a release image on the current base.

## In flight (2026-10-29)

- `terminal/app` and `ide/app` split into CachyOS dev images and Ubuntu 24.04 release images:
   monochromatic-ec's IDE release dry run 40887621364 passed (detect, build arm64, build x64, checksums, release),
   newest glibc symbol 2.39 on both legs like baseline 40877021793.
  It needed `cb03f20350` (monochromatic-ec): the arm64 executable now names `ld-linux-aarch64.so.1`,
   added to the arm64 shared-library allow list in `ide-release.yml`.
  The compositor release in `cargo-publish.yml` builds on ubuntu-24.04 runners, so the CachyOS image does not touch it.
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

- Desktop images (subagent, 2026-10-29):
   terminal split `265c55d90c`, `024f973075`, `6168f6c2e2`;
   ide split `fd1bd8caf8`, `150dca7169`, `e5d55a8d94`;
   music-player `397b35d77a`, `c094db67d6` (Arch has no `wine64`), `7b1caae071` (`noto-fonts` for Slint under Wine).
  Only `ide-release.yml` builds shipped artifacts in these images.
  Local reproduction of the IDE release gates passed (glibc 2.39, needed libraries match, bundle inspections pass).
  music-player `test:windows-gnu`: same 4 failed and 2 timed out as the recorded Fedora run.

## music-player follow-ups (owner decisions 2026-10-29)

- `Containerfile.sign` on CachyOS, osslsigncode 2.14 built from the latest upstream release (`bf3cf4220f`, `8ff6727144`):
   `sign:windows` signed a mingw-built PE with a throwaway PFX (`Signature verification: ok`).
  Upstream publishes no checksum or signature for its source; TLS to github.com is the only check.
  Not run: the PKCS#11 path (needs a real certificate) and `verify:signing`.
- `run` fallback built with cargo-zigbuild for glibc 2.39 (`4723d28bc9`, `dc88d327e4`):
   prints `music-player 0.1.13` on this glibc 2.43 host, newest symbol `GLIBC_2.39`.
  Extended to `run-quick` and `run-debug` (owner choice; `21e72de5c0`, docs `ba5582e60c`):
   both print `music-player 0.1.13` on this host from the container path; the container path of `run-quick`
   also stopped ignoring its own `CARGO_TARGET_DIR` and LTO settings.
- `XDG_RUNTIME_DIR` tmpfs in container cargo runs (`26146d128a`): the three `unplayable_notification` tests pass.
- `ui_lists_render_tests::a_search_result_title_wider_than_the_page_ends_in_an_ellipsis` fails on both images
   and stops nextest before 267 tests: owner chose an issue, #1815.
- Docs: `doc/troubleshooting/cachyos-container-binary-needs-newer-glibc-than-host.md`,
   `doc/troubleshooting/cachyos-image-migration-package-differences.md` (`533ed23205`).
- `doc/decision/ide-releases.md` and `doc/runbook/release-the-ide.md` now name the Ubuntu 24.04 release image
   (monochromatic-ec, `8e843b1aa5`); remaining Fedora 41 mentions there are dated history notes.

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

## Mutation-test runtime image and the work-tree copy

- Runtime image on CachyOS: Containerfile committed, `image.ts` passes `--build-arg CACHYOS_IMAGE`
   from the resolver and hashes the base image into the tag (lockfile change 3 lines).
  Lint findings in `shards.ts`, `selection.ts`, `scratch-parent.ts` come from `2972f3c6a0` (another session).
- Every shard copies the checkout into a 2 GiB tmpfs with rsync and fixed excludes;
   the checkout is 14 GB after excludes (11 GB of ignored `.claude/worktrees`), so rsync exits 11 in every container.
  The p-map run (55 of 55 shards failed) and the p-memoize rerun were stopped.
  Copying only `git ls-files --cached --others --exclude-standard` is 804 MB.
- Fixed: each shard gets the `git ls-files` list and copies with `--files-from` (`8b4374358e`, tests `b7c1773fd1`).
  Narrow p-map probe on the `cachyos-v4` runtime image: 19515 files copied, 6 shards, no copy failure;
   `src/errors.ts` Killed 5, Survived 5, CompileError 11.
- `.claude` left out of the image build context (`containerignore`), which had made `COPY . /baked` take about 25 minutes.
- Running since 11:13 local: units `mutation-p-map-3` (551 mutants) and `mutation-p-memoize-3` (363 mutants),
   reusing image `baa75398fb47`; logs `~/temp/agent/package-category-elimination/mutation-p-{map,memoize}.log`.
  Finished 11:19 (p-memoize) and 11:30 (p-map), no infrastructure errors:
   p-map killed 219, survived 89, timeout 9, compileError 234, ignored 58 (135 shards);
   p-memoize killed 142, survived 15, compileError 206, ignored 9 (37 shards).
  Survivors by file: p-map `p-map-iterable.ts` 40, `map-iterator.ts` 25, `p-map.ts` 16, `errors.ts` 5,
   `map-options.ts`, `p-map-keyed.ts`, `p-map-stop.ts` 1 each;
   p-memoize `p-memoize-decorator.ts` 6, `memoized-cache-store.ts` 5, `p-memoize-clear.ts` 4.
  Correction (2026-10-29): the report's survivor entries carry no field saying whether tests ran,
   so an earlier claim here that every survivor ran against its own tests had no basis;
   p-memoize's `memoized-cache-store.ts` had no stem-matching test file, so its 5 survivors were never tested.
  A second claim, that every p-map file with survivors had a stem-matching test, was also wrong:
   it was checked after the survivor agent had added `p-map-stop.unit.test.ts`, and several p-map survivors
   came from tests living under another stem (the agent's report lists them).
  Owner chose "Kill or justify all": one subagent per package adds killing tests or justified
   `mutation-test-disable` suppressions, then reruns `test:mutation`
   (units `pmap-survivors-final`, `pmemoize-survivors-final`; report copies in `~/temp/agent/mutation-survivors/`).
  p-memoize done: final run killed 151, survived 0, compileError 210, ignored 11 (`6fb171e256`, `d511882404`);
   new `memoized-cache-store.unit.test.ts`, clear-thenable and decorator-receiver tests,
   one suppression at `p-memoize-decorator.ts:69` with an equivalence proof. No fork bug against upstream 9.0.0.
  p-map: final run killed 253, survived 0, timeout 9, compileError 247, ignored 114, no infrastructure errors
   (`f8dd9b3f07`, `d89c8ce63a`, `3245bea575`, `d47af933c6`, `15e29d6f63`, `2911e59e01`, `f38fcf5656`, `0f1b7e349e`);
   no fork bug found.
  Gap: suppression directives work per line and mutation family, so proofs for equivalent mutants also hide killable
   ones on the same line (p-map ignored 58 to 114, p-memoize 9 to 11).
  Owner chose "Split lines in source": both agents resumed to isolate each proven-equivalent mutant on its own line
   and family, kill what reappears, and rerun.
  p-memoize after that round: killed 149, survived 0, ignored 7 (`b6f224be40` removed the decorator check and its
   suppression; `e179a0d72b` dropped an equality guard that hid killable mutants).
  Four older suppressions (`mimic-function.ts` 310/312/314, `p-memoize.ts:620`, the `CACHE_DISABLED` description)
   were inherited unverified; narrowed runs checked each.
  p-memoize final: killed 150, survived 0, compileError 207, ignored 3, each with a proof checked against the code
   (`mimic-function.ts:287` unreachable guard, its forced-true twin a compile error;
   `p-memoize.ts:620` `ignoreNonConfigurable` unobservable at that call site).
  `25603427ff` exports `CACHE_DISABLED` from `src/index.ts` so a built-artifact test can pin its description
   (allowed by AGENTS.md XPT; the package is private and unpublished); `c9365744bb` rewrote the unreachable fallbacks.

## music-player `run`: zigbuild adopted for now

Owner adopted cargo-zigbuild "for now" for the `run` fallback (glibc 2.39) and asked for issue #1814 to replace it
 after a choosing-technology pass, because Zig's upstream cannot take this project's issue reports.
Declined for now: distrobox ("Must be podman"), an Ubuntu 24.04 build image and running inside podman ("Neither").

## Out of scope by owner choice

Release workflows (Ubuntu 24.04 glibc floor), the root `Dockerfile`, `mvm` default VM,
 `vm-builder` (`ucore-hci`), `config/pnpr`, `music-player/desktop-app/Containerfile.alpine`.
