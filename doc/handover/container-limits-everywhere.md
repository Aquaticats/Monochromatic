# Container limits everywhere

Branch `feat/container-limits-everywhere`,
worktree `.claude/worktrees/container-limits-everywhere`.

## Purpose

Every container start in the repository gets the lowest disk weight and CPU,
memory and process limits sized to its job,
with the disk weight coming from one shared constant.
Owner,
2026-11-07,
answering the question in `doc/handover/questions-for-the-owner.md`:
"All of them (Recommended)":
"every container start gets the lowest disk weight and limits sized to its job,
through one shared constant,
including `vm-builder`'s rootful run".

The six packages in the owner's earlier answer "Both flags in all six" were done first
(`git log --grep='disk weight'`:
`78df0f33c3`,
`373a2834d3`,
`a683ef5986`,
`f455e3b8f0`,
`a7e4161d23`,
`27d4d3b3c8`,
and `f256b35b32` for the verification runners).

## Facts about the tools, probed on this host (2026-11-07)

Command:
`podman run --help`,
`podman create --help`,
`podman build --help`,
each filtered for the option names.

- `podman` is 6.1.3.
- `podman run` and `podman create` accept `--blkio-weight` (10 to 1000),
  `--cgroup-conf`,
  `--cpus`,
  `--memory`,
  `--pids-limit` (default 2048,
  the value `containers.conf` supplies when the option is absent).
- `podman build` accepts `--memory`,
  `--cpu-period`,
  `--cpu-quota`,
  `--memory-swap`.
  It has no `--blkio-weight`,
  no `--cgroup-conf` and no `--pids-limit`,
  so an image build cannot be put at the lowest disk weight or given a process cap by an option.
  Builds get the CPU and memory bound only.
- The block devices here use the `bfq` scheduler (`/sys/block/*/queue/scheduler`).
  `--blkio-weight` sets `io.bfq.weight` and `--cgroup-conf=io.weight=` sets `io.weight` (read by iocost),
  so both are passed,
  as `doc/runbook/claude-cpu-cap.md` records for the launcher's own unit.
- `docker` run (the `pnpr-publish` workflow) accepts `--blkio-weight 10` and has no `--cgroup-conf`.
- No Rust source in the repository starts a container.
  Command:
  `git ls-files -z '*.rs' | xargs -0 rg -n -i 'podman|docker|blkio'`
  found only file-type names (`dockerfile`,
  `docker-compose`) in the IDE's syntax tables and a language-server name list,
  and doc comments.
  So the shared constant exists for TypeScript only.

## Shared constant

New package `package/container-limit/core` (`@monochromatic-dev/container-limit-core`,
private,
no dependencies,
TypeScript source importable by node's type stripping so inline `mise.toml` scripts of packages without a `package.json`
can import it by relative path,
as they already import `package/test-harness/core-matrix/src/bin/*.ts`).
It exports `LOWEST_DISK_WEIGHT`,
`diskWeightArguments()`,
`containerLimitArguments()` (memory,
CPUs,
processes and the
weight) and `imageBuildLimitArguments()` (memory and CFS quota only,
because the build has no weight option).

Copies it replaces:
- `package/mutation-test/cli-cargo-mutants/src/container-limits.ts` (`lowestDiskWeight`,
  `diskWeightArguments`),
  which `git-policy/cli`,
  `forbidden-strings/cli`,
  `forbidden-strings/core`,
  `linter/monochromatic-lint` and the fuzz
  sidecars reach through `runLimitArguments`.
- `package/mutation-test/cli/src/host/podman.ts` (`LOWEST_DISK_WEIGHT`).
- `package/vm-builder/cli/src/disk-weight.ts` and the `lowest_disk_weight` variable of `package/vm-builder/cli/mise.toml`.
- `package/ide/app/bin/container-bounds.ts` (`lowestDiskWeight`).
- the inline `lowestDiskWeight` of `package/nested-wayland-session/cli/mise.toml` and `package/search-shadow/cli/mise.toml`.
- the literal `--blkio-weight=10` of `.github/workflows/pnpr-publish.yml` (docker has no second option;
  kept literal,
  checked against the constant by a test).

## Sizing rule

No number below is invented.
Each is either measured (none yet;
marked) or adopted from a documented sibling and marked "adopted".
Process limits:
where nothing documents a size,
the value is `2048`,
which is what `podman run --help` prints as the
default,
so the container's effective cap does not change and becomes explicit (marked "adopted,
default").
Threads count against `pids.max`,
and browsers and package managers start many.

Update 2026-11-08T17:10:51-05:00:
every size marked "adopted" now says beside it whether the repository holds a measurement of that site's own
workload,
because the audit of the queue
(`queue-audit-report.md`,
item 16)
found that some numbers came from a sibling package's documentation.
What was searched:
every Markdown file under `doc/` for each site's workload name beside a peak,
RSS,
OOM or `memory.peak` term,
uncapped,
and the comment beside each size in this inventory and in the task that sets it.
What that search found:
no adopted size is backed by a measurement of its own bounded run.
Three sites say so in their own comments
(`mise.toml:350`,
`package/browserslist/core-rust/bin/regenerate-conformance.ts`,
`package/test-harness/core-matrix/src/container.ts`).
The one own-workload record is the matrix cell's growth to 42 GB RSS and the OOM kill of 2026-10-26,
which records the unbounded failure and says nothing about the cell's CPU or process caps.
Those searches cannot prove that no measurement exists outside the files they read,
so "unmeasured here" means "none found in `doc/` or beside the size".
No container was built or started for this check;
`## What each adopted size would need measured` lists the runs that would take.

Documented sizes cited:
- `package/mutation-test/cli-cargo-mutants/src/container-limits.ts`:
  2g,
  2 CPUs,
  512 processes (`defaultPidsLimit`,
  with its measured rationale:
  128 failed on `fork`).
- `package/ide/app/bin/container-bounds.ts`:
  process limit `max(512, 256 per CPU)`,
  so 1024 for 4 CPUs;
  `.github/workflows/ide-release.yml` already passes `--pids-limit=1024` for a 4 CPU leg.
- `package/vm-builder/cli/mise.toml` `image:check`:
  2g,
  2 CPUs,
  128 processes for single small commands;
  `image:build`:
  12g,
  8 CPUs (`--cpu-quota 800000`).
- `package/music-player/desktop-app/mise.toml` Alpine guard:
  12g,
  4 CPUs;
  `.github/workflows/music-player-release.yml`:
  4 vCPU,
  16 GB runners,
  `BUILD_MEMORY: 12g`,
  `BUILD_CPUS: '4'`.
- `package/terminal/app/mise.toml`:
  4g,
  4 CPUs for the Ghostty plus Slint compile.
- `package/test-harness/core/mise.toml` `test:browser`:
  "bounded to 2 GiB RAM and 2 CPUs".
- `package/config/pnpr` and `pnpr-publish.yml`:
  2g,
  2 CPUs,
  256 processes for verification containers.
- `package/repository-move/cli/src/guided-cutover-stages.data.ts`:
  256m,
  1 CPU,
  64 processes for a one-command run.

## Inventory

For each site:
where,
today's flags,
the flags needed,
sizing source.
"weight" means `--blkio-weight=10 --cgroup-conf=io.weight=10`.
"build bound" means memory and CFS period and quota,
which is all `podman build` accepts.
"unmeasured here" means the repository holds no record of a peak measured at that site,
so the number beside it was adopted from the sibling the parentheses name.

### Already complete before this branch

These start containers through `runLimitArguments`,
`limitArguments` or a literal that already carries the weight.
They change only by taking the shared constant (the copies the "Shared constant" section lists),
never by gaining flags.

- `package/git-policy/cli/bin/*.ts` (`build-git-test-image.ts`,
  `e2e-native-container.ts`,
  `test-native-container.ts`,
  `native-planted-controls*.ts`,
  `native-test-base.ts`,
  `native-verification-process.ts`) and
  `package/git-policy/cli.fuzz/bin/*.ts`:
  `runLimitArguments` and `buildLimitArguments` from
  `mutation-test-cli-cargo-mutants`.
  Not edited by this branch (another agent owns the wrapper).
- `package/linter/monochromatic-lint/{bin,mise.toml}`,
  `package/linter/monochromatic-lint.fuzz/bin`:
  `diskWeightArguments` from `mutation-test-cli-cargo-mutants`.
- `package/forbidden-strings/cli/bin`,
  `package/forbidden-strings/cli.fuzz/bin/smoke-container.ts`,
  `package/forbidden-strings/core/bin`:
  the same helpers.
- `package/mutation-test/cli-cargo-mutants/src` (`campaign-*`,
  `podman.ts`):
  `runLimitArguments`.
- `package/mutation-test/cli/src/host/podman.ts:41` shard `run`:
  weight present (own constant).
- `package/ide/app/{bin,mise.toml,task.ts}`:
  `containerBounds().limitArguments` (commit `27d4d3b3c8`).
- `package/nested-wayland-session/cli/mise.toml:55` and `package/search-shadow/cli/mise.toml:67`:
  weight present
  (inline constants).
- `package/vm-builder/cli` `image:check` (three `podman run`,
  `mise.toml:20` to `22`) and the `bootc-image-builder` run's
  weight (`src/build-and-import.ts`):
  weight present.
- `.github/workflows/pnpr-publish.yml:99` and `:178` (`docker run`):
  `--cpus=2 --memory=2g --pids-limit=256
  --blkio-weight=10`.

### Containers with no limit at all

- Root `mise.toml:350` (`runPlaywright`,
  behind `test:browser` and `test:e2e`):
  `podman run --rm --security-opt
  label=disable`.
  Needs:
  2g,
  2 CPUs (adopted from `test-harness/core` `test:browser`;
  unmeasured here),
  2048 processes (adopted,
  the Podman default;
  unmeasured here),
  weight.
- `package/music-player/desktop-app/mise.toml:114` and the same body in `task.ts`,
  and
  `package/monorepo-task/cli/src/task-script-prelude.ts:155` (the `task.ts` runtime prelude):
  `containerCargo` runs with `limits: []` by default.
  Needs:
  12g,
  4 CPUs (adopted from the file's Alpine guard;
  unmeasured here),
  1024 processes (adopted from `container-bounds.ts`;
  unmeasured here),
  weight.
  The Windows GNU leg (`gnuContainer`,
  6g,
  4 CPUs) gains 1024 processes and the weight.
- `package/music-player/desktop-app/mise.toml:597` icon tools and `:713` to `:723` signing (`podman run`):
  no limits.
  Needs:
  2g,
  2 CPUs,
  128 processes (adopted from `vm-builder` `image:check`;
  those three numbers are unmeasured here),
  weight.
- `package/vm-builder/cli/src/build-and-import.ts` `bootc-image-builder` run (rootful,
  `--privileged`):
  weight only.
  Needs:
  12g and 8 CPUs (adopted from `image:build`;
  unmeasured here,
  and this run needs root to measure at all),
  2048 processes (adopted,
  the Podman default;
  unmeasured here),
  weight kept.
  `--pids-limit` on a privileged `osbuild` partitioning step is unverified:
  it cannot be exercised without root.
- `package/vm-builder/cli/src/build-and-import.ts` rootful `podman build`:
  no limits.
  Needs:
  build bound 12g and 8 CPUs,
  as `mise.toml:15` (adopted there too;
  unmeasured here,
  and rootful).

### Partial limits, no weight

- `package/terminal/app/mise.toml:68` and the twelve copies in `task.ts` (`--memory=4g --cpus=4`):
  the twelve `task.ts` copies now pass `containerLimitArguments(containerBound)`, and so does the `mise.toml` body
  since this pass: before that, `mise run //package/terminal/app:build:container` started the
  compiler container with the two literal flags and no `--pids-limit` (the observed `pids.max` was Podman's 2048)
  and no weight, while the inventory said it had them.
  The 4g is measured (2.97 GiB, 74 percent of it, on a build that reused a warm target directory) and 1024 processes
  is still adopted,
  though the same run reached 235 threads of it.
- `package/test-harness/core/mise.toml:64` (2g,
  2 CPUs,
  unmeasured here):
  add 2048 processes (adopted,
  the Podman default;
  unmeasured here) and the weight.
- `package/test-harness/core-matrix/src/container.ts:391` (every matrix cell,
  reached from `env-local-json` and the other
  `*:test:container` tasks;
  2g,
  2 CPUs,
  adopted after the unbounded 42 GB incident of 2026-10-26;
  no bounded run's peak is recorded,
  so these two are unmeasured here):
  add 2048 processes (adopted,
  the Podman default;
  unmeasured here) and the weight.
  Gets an exported argument builder and an argv test.
- `package/upstream-fork/conf/mise.toml:71` and the sixteen `spawn('podman'` sites of
  `package/upstream-fork/deepmerge-ts.fuzz/mise.toml` (2g,
  2 CPUs):
  add 512 processes (`defaultPidsLimit`,
  whose 128-fork failure was measured on the mutation runner,
  not at these sites;
  unmeasured here) and the weight.
- `package/config/pnpr/{mise.toml:53,task.ts:44}` (2g,
  2 CPUs):
  add 256 processes (as `pnpr-publish.yml`;
  unmeasured here) and the weight.
- `package/browserslist/core-rust/bin/regenerate-conformance.ts:287` (`ORACLE_MEMORY`,
  `ORACLE_CPUS`):
  add 512 processes and the weight (the file says the 512 is adopted for one Node.js process and not measured;
  the memory and CPU figures there are unmeasured here too).
- `package/repository-move/cli/src/guided-cutover-stages.data.ts:597` (256m,
  1 CPU,
  64 processes,
  shown to the owner as a
  runbook command):
  add the weight.
- `.github/workflows/music-player-release.yml` (`podman run` at the lines that read `--memory="$BUILD_MEMORY"`,
  the
  smoke test and the runtime image checks) and `.github/workflows/ide-release.yml:298` to `:308`:
  see the "Workflow decisions" section.

### Image builds (`podman build`, no weight option exists)

Builds get the memory and CPU bound only.
- `package/test-harness/core-matrix/src/container-image-rebuild-on-changed-inputs.ts` `buildWithPodman`:
  one place that every `containerImageBuild` caller reaches (`ide/app`,
  `music-player/desktop-app`,
  `terminal/app`,
  `nested-wayland-session/cli`);
  no bound today.
  Needs:
  build bound 4g and 4 CPUs (adopted from `terminal/app`;
  unmeasured here),
  applied unless the caller passes its own `--memory`.
- `package/music-player/desktop-app/mise.toml:242` (`--memory=4g` only),
  `:594` and `:705` (none).
- `package/nested-wayland-session/cli/mise.toml:54` and `:271` (none;
  the second is the same build).
- `package/forbidden-strings/core/mise.toml:50` (none).
- `package/upstream-fork/deepmerge-ts.fuzz/mise.toml:244` (none).
- `package/mutation-test/cli/src/host/image.ts` (none).
- `package/config/pnpr/{mise.toml:48,task.ts:39}` (`--memory=2g --cpu-quota=200000`,
  no period).
- `package/monorepo-task/core-exec/src/playwright-image-build.ts` (CI build of `playwright.Dockerfile`;
  none).
  Needs:
  12g and 4 CPUs (the 4 vCPU,
  16 GB runner of `music-player-release.yml`;
  a runner size,
  not a measurement of this build).
- `.github/workflows/music-player-release.yml` `podman build` (two jobs):
  none.

## What each adopted size would need measured

What each one needs is a measurement of that site's own workload under its own bound,
from a representative run and recorded beside the size with the command that produced it:
the container cgroup's `memory.peak`,
whether the run reached its bound (a limit hit shows in the run's own error or in `pids.current` reaching
`pids.max`),
the elapsed time and the CPU throttling in `cpu.stat`,
and which workload variants the run covered,
because one short run does not establish that a bound is adequate.
`AGENTS.md` rule LQM asks for a workload measurement or an owner decision beside a shipped limit,
and rule NQH keeps the measurement at the load the host already has.
The audit that produced this list
(`queue-audit-report.md`,
item 16)
ran no container;
those runs are the separate work.

- The Playwright browser run of the root `mise.toml` `runPlaywright` task,
  for the memory,
  CPU and process peaks a browser set reaches.
  Still needed: the run is blocked on this host until the Playwright image and the shared pnpm store reach the
  container;
  `### Measured runs on this host` records what was tried.
- The `containerCargo` cargo build of `package/music-player/desktop-app`
  (build,
  clippy and nextest),
  whose bound was copied from that package's Alpine guard.
  Its Windows GNU leg stays paused with the rest of Windows.
  The release build is measured (5.3 GiB of 12 GiB);
  clippy and nextest still need their own run.
- The music player's icon and signing one-shot tool containers,
  whose bound was copied from `vm-builder` `image:check`.
  The icon containers are measured (111 MiB of 2 GiB);
  the signing containers still need a run.
- The `test-harness/core` `test:browser` run,
  whose 2048 processes is the Podman default.
- One matrix cell of `package/test-harness/core-matrix`,
  which bounds every cell from the one `CELL_LIMITS` constant and is the one site already known to have grown to
  42 GB RSS unbounded,
  with 2048 processes that is the Podman default.
  Still needed: no cell on this host can resolve its dependencies inside the container,
  so the bound has no runnable workload here;
  `### Measured runs on this host` records the failure.
- The `upstream-fork/conf` and `deepmerge-ts.fuzz` container runs,
  whose 512 processes is the mutation runner's constant.
- The `config/pnpr` verification containers,
  whose 256 processes was copied from the publish workflow.
- The browserslist oracle container,
  whose 512 processes the file itself calls adopted and unmeasured.
- The `terminal/app` compile containers,
  whose 4g and 4 CPUs predate this branch and whose 1024 processes is the IDE's 4-CPU bound.
- The image builds:
  `core-matrix`'s `buildWithPodman` (4g and 4 CPUs from `terminal/app`),
  `playwright-image-build` (12g and 4 CPUs from the release runner's size),
  and the `vm-builder` `bootc-image-builder` run and rootful build,
  which need root before any figure can be read.

Each is one run,
not a benchmark:
the rule asks for a peak that a bound can be read against,
and `AGENTS.md` rule NQH keeps the measurement at the load the host already has.

### Measured runs on this host

Pass of 2026-11-08T19:18:31-05:00 to 19:57:43-05:00 on branch
`docs/container-limit-sizes-measured-from-their-own-workloads`.
Each entry is one run of a site's own task,
at the load the host already had,
with the bound the task passes.
Every command ran inside `systemd-run --user --scope` with `CPUQuota`,
`MemoryMax`,
`TasksMax` and `IOWeight=10` (rule `RXI`),
and a watcher polled every live `libpod-*.scope` under
`/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/user.slice` for `memory.peak`,
`memory.current`,
`pids.current`,
`pids.max` and `cpu.stat`
(every 200 ms,
20 ms for the one-shot tool containers).
Rootless Podman gives each container its own scope there,
so those figures are the container's own,
and `memory.peak` is the kernel's watermark rather than a single sample.
One run does not prove a bound is adequate for every variant of a workload,
and each entry names the variants its run covered.
Excluded by the brief that set this pass:
the `vm-builder` runs,
which need root before any figure can be read,
and every Windows and macOS leg.

- `package/music-player/desktop-app` `containerCargo`,
  command `mise run //package/music-player/desktop-app:build:container`,
  started 2026-11-08T19:21:30-05:00 and finished 19:31:10-05:00
  (9 min 40 s,
  exit 0),
  host load 13.64 at the start on 12 CPUs.
  The `cargo build --release` container (12 GiB,
  4 CPUs,
  1024 processes) ran 7 min 12 s (19:23:58 to 19:31:10;
  cargo reported "Finished in 7m 10s") and peaked at 5,651,558,400 bytes (5.3 GiB),
  44 percent of its 12 GiB.
  It reached 61 threads of 1024,
  was throttled in 293 of 4251 CPU periods (25.3 s of throttled time),
  and killed nothing.
  Variants covered:
  the release build only.
  `lint:clippy:container` (clippy) and `test:container` (nextest) share this bound and were not run.
  The image build of the same run (`podman build` with `--memory=12g --cpu-quota=400000`) crested at
  3,598,233,600 bytes (3.35 GiB).
  That figure is the launching scope's watermark and also covers the wrapper of the later cargo container,
  so it bounds the build without isolating it.
- `package/music-player/desktop-app` icon tool containers,
  command `mise run //package/music-player/desktop-app:gen:icons`,
  started 2026-11-08T19:37:39-05:00 and finished 19:39:19-05:00
  (1 min 40 s,
  exit 0),
  host load 32.67.
  Ten containers,
  each bounded to 2 GiB,
  2 CPUs and 128 processes;
  the largest peak was 116,862,976 bytes (111 MiB) in the `png2icns` step,
  the eight `rsvg-convert` sizes (16 to 1024 pixels) peaked between 700,416 and 7,864,320 bytes,
  and `icotool` peaked at 1,310,720 bytes.
  One thread of 128 was busy at each sample,
  no container was throttled,
  and nothing was killed.
  Variants covered:
  eight `rsvg-convert` sizes,
  `png2icns` and `icotool`.
  The signing containers share this bound and were not run,
  because they need a Windows executable.
- `package/terminal/app` compile containers,
  command `mise run //package/terminal/app:build:container`,
  started 2026-11-08T19:53:09-05:00 and finished 19:57:43-05:00
  (4 min 34 s,
  exit 0),
  host load 40.33 at the start.
  The container (4 GiB,
  4 CPUs,
  and no process option at all,
  because `mise.toml` still spelled `--memory=4g --cpus=4` by hand when it ran;
  see the bullet in `### Open`) peaked at 3,188,219,904 bytes (2.97 GiB),
  74 percent of its 4 GiB,
  and cargo reported "Finished in 4m 18s".
  It reached 235 threads,
  was throttled in 207 of 2260 CPU periods (164.8 s of throttled time),
  and killed nothing.
  Two limits of this figure:
  the build reused the target directory the failed attempt named later in this bullet had already filled,
  so a cold build can go higher,
  and the variants covered are the release build only.
  `lint:clippy:container` and `test:container` share this bound and were not run.
  An earlier attempt of the same task (19:45:55 to 19:51:07)
  stopped in the `libghostty-vt-sys` build script because `target/ghostty-offline/source` was empty in this worktree,
  after peaking at 3,007,496,192 bytes (2.8 GiB) of 4 GiB while compiling dependencies.
  Neither failed nor warm run makes 4 GiB look loose.
- `package/test-harness/core-matrix` `CELL_LIMITS`:
  not measured,
  because the only `*:test:container` task that reaches the matrix here cannot run its workload on this host.
  `mise run //package/env-local-json/core:test:container` of 2026-11-08T19:18:31-05:00 failed in both of its
  combinations (`container:ubuntu` as `root` and as `user`,
  runtime `node`) after 42 s:
  the container's node could not resolve `p-limit`,
  imported by `/workspace/package/test-harness/core/src/describe.ts`,
  with `ERR_MODULE_NOT_FOUND`,
  because this repository's `node_modules` holds links into `${HOME}/.local/share/pnpm/store`
  (`storeDir` and `virtualStoreType: global` in `pnpm-workspace.yaml`),
  a path the cell's `/workspace` mount does not contain.
  The two failed runs peaked at 435,101,696 and 283,648,000 bytes of 2 GiB,
  with 9 and 10 threads of 2048,
  no throttling and no kills;
  those are set-up figures,
  not the workload's.
  A hand-built copy of the `root` cell with the same options reproduced the same error at 439,230,464 bytes.
- The root `mise.toml` `runPlaywright` browser run:
  not measured.
  `mise run test:browser` of 2026-11-08T19:40:29-05:00 failed after 7 s before any browser started,
  with the same `ERR_MODULE_NOT_FOUND` into the pnpm store,
  raised by `npx playwright`.
  Repeating the container with the store mounted read-only let Playwright start and fail all 84 tests with
  `browserType.launch: Executable doesn't exist at /ms-playwright/chromium_headless_shell-1254/...`:
  the local `monochromatic-playwright` image carries Playwright 1.63.0 while the workspace pins 1.65.0,
  and `prepare:playwright` cannot replace the image for the same store reason.
  That run peaked at 1,833,377,792 bytes (1.7 GiB) of 2 GiB with 183 threads of 2048 and 182 throttled periods of
  268,
  and it measured node and the harness,
  not a browser.

## Workflow decisions

`ide-release.yml`,
`music-player-release.yml`,
`playwright-image.yml`,
`env-local-json.yml` and `pnpr-publish.yml`
are the five workflows that start containers (the handover said two).

Update 2026-11-08,
after the first dry runs of the two release workflows on `main`:
- Hosted runners refuse the disk weight.
  Run 42447009237 of `music-player-release.yml` failed in `test-alpine-x86_64` and `test-alpine-aarch64`,
  step "Read the
  toolchain",
  with `crun: opening file 'io.weight' for writing: No such file or directory`:
  the `io` controller is not
  delegated.
  `ide-release.yml`,
  `music-player-release.yml` and `env-local-json.yml` now keep `--pids-limit=1024` (or the matrix
  default) and set `CONTAINER_LIMIT_DISK_WEIGHT=off`,
  which `package/container-limit/core` reads to leave the weight out
  of every container a task of that job starts.
  The ledger test requires the switch,
  the reason and the run id in each of them.
- `pnpr-publish.yml` keeps `docker run --blkio-weight=10`:
  the runner's root docker daemon accepted it (run 42446813126,
  success,
  on the merge).
- Run 42447007710 of `ide-release.yml` failed in "Build the base image (terminal release)" with
  `ERR_MODULE_NOT_FOUND` for `@monochromatic-dev/container-limit-core`.
  The job installs no dependencies,
  and the same script already imported `@monochromatic-dev/logger-core` by name,
  which
  is the next missing package (a fresh clone of `a9a3d48d53`,
  before this branch,
  stops on it),
  so the job was already
  broken by the image rebuild helper (`3d01ccd564`);
  this branch only changed which package is named first.
  `task-cli-setup` has an `install-filter` input and the job installs `@monochromatic-dev/test-harness-core-matrix...`.
- The `build-android` failure of that run is a `cargo binstall` timeout fetching `cargo-ndk` 4.1.2 from GitHub
  (`Timeout reached while checking fetcher`),
  and the `notices` failure is the Slint license file checksum check of
  `package/music-player/desktop-app/bin/third-party-notices.mjs`;
  neither starts a container or reads this branch's files.

Earlier reasoning,
kept for the record (superseded by the update dated 2026-11-08 in "Workflow decisions"):
- On a dedicated hosted runner the job is the only tenant,
  so the weight has no sibling to yield disk time to.
- `playwright-image.yml` builds through `core-exec`'s `playwright-image-build.ts` (build bound only).

## Not edited

- `package-paused/dev-script/inference-canary/src/container-runtime.ts` (podman or docker run of `docker.io/oven/bun:1.3`)
  and `package-paused/webapp-edu/paper2vn/mise.toml:37` (`podman run` with no limits):
  paused packages;
  proposed,
  not changed (the narrower reading of the request).
- `package-paused/dev-script/inference-canary-viewer/docker-compose.yml` and `package/config/pnpr/compose.yaml`:
  Coolify compose resources the deployer starts,
  not repository commands.
- `package/git-policy/**`:
  the wrapper's runners already use the shared helpers;
  the owner named the package as
  being changed by two other agents.

## Result (branch `feat/container-limits-everywhere`, 2026-11-08)

Every site in the inventory now carries the lowest disk weight (containers) or a memory and CPU bound (builds),
through `@monochromatic-dev/container-limit-core`,
except the sites listed under "Not edited" and "Open".

### Evidence

- Tests,
  each red before the change and green after (red shown by running against the unchanged source or without the
  implementation):
  `package/container-limit/core/src/index.unit.test.ts`,
  `package/vm-builder/cli/src/container-arguments.unit.test.ts`,
  `package/config/pnpr/src/container-limits.unit.test.ts`,
  `package/test-harness/core-matrix/src/container-limits.unit.test.ts`,
  `package/mutation-test/cli/src/host/image.unit.test.ts`,
  `package/monorepo-task/core-exec/src/playwright-image-steps.unit.test.ts` (the build step's expected argv),
  `package/repository-move/cli/src/guided-container-bind-limits.unit.test.ts`.
- The ledger `package/container-limit/core/src/container-start-sites.unit.test.ts` lists every tracked file that names
  `podman` or `docker` as a program.
  It was shown to fail twice:
  a listed file with its helper name removed,
  and a new tracked file with
  `spawn('podman')` that is not listed.
- A real `podman run` with `containerLimitArguments({ memory: '256m', cpus: 1, pidsLimit: 64 })` on `podman` 6.1.3
  printed inside the container `memory.max=268435456`,
  `pids.max=64`,
  `cpu.max=100000 100000`,
  `io.bfq.weight=default 10`,
  `io.weight=default 10`.
- A real image build through `container-image-build-if-inputs-changed.ts` (the matrix build helper that every
  `containerImageBuild` caller reaches) succeeded with the default bound;
  the image was removed afterwards.
- A real run of `mise run //package/music-player/desktop-app:gen:icons` (image build with the build bound,
  then eight
  `rsvg-convert` containers and the icon tools at 2g,
  2 CPUs,
  128 processes and the weight) ran to the end;
  it
  regenerated `asset/music-player.icns`,
  which was restored from `HEAD`.
- `mise run //package/test-harness/core:test:browser` started its container with
  `--memory=2g --cpus=2 --pids-limit=2048 --blkio-weight=10 --cgroup-conf=io.weight=10`,
  and failed inside it with
  `Cannot find module '/.local/share/pnpm/store/v11/links/@playwright/test/...'`:
  the worktree's `node_modules` links
  into the global pnpm store under the home directory,
  which the container does not mount.
  The failure does not involve the limits;
  the same mount is used with or without them.

### Gates

Passed:
lint,
types and tests of `container-limit-core`,
`vm-builder-cli`,
`monorepo-task-core-exec`,
`monorepo-task-cli`,
`mutation-test-cli` (types,
tests,
build),
`repository-move-cli`,
`test-harness-core-matrix` (lint,
types,
`test`);
lint and types of `mutation-test-cli-cargo-mutants`,
`config-pnpr`;
`test:bin` of `ide/app`;
`lint:scripts:types` of `browserslist-core-rust`,
`forbidden-strings-cli`,
`monochromatic-lint`;
`lint:types` of `git-policy-cli` (read only).

Failures that this branch did not cause (rerun on the tree after `main` was merged in at 2026-11-08T04:25-05:00;
`git diff main HEAD` over `package/mutation-test/cli-cargo-mutants/src/golden`,
`package/linter/monochromatic-lint/bin/mutation-scopes.ts` and `package/config/pnpr/config.yaml` is empty,
so the
branch changes none of the inputs these tests compare):
- `mutation-test-cli-cargo-mutants` `test:unit`:
  four product goldens (`linter-semantic`,
  `linter-reject-unknown-scope`,
  `linter-reject-two-scopes`,
  `linter-coverage`) list `src/rust_file_engine_syntax.rs` among the semantic scope's files,
  which `package/linter/monochromatic-lint/bin/mutation-scopes.ts` no longer emits there (commits `00bb13db2c` and
  `40e24c8cda` came after golden commit `a49dd27a52`).
  The timing test `acquireCampaignLock refuses at once` measured 15268 ms against a 5000 ms bound inside the first suite
  run,
  and passed in the rerun and when `campaign-lock.unit.test.ts` ran alone:
  a contention or ordering defect until
  measured otherwise,
  cause not measured.
- `config-pnpr` `test:unit`:
  three `real workspace` tests fail because the checked-in `config.yaml` lacks
  `@monochromatic-dev/repository-link-check` and `@monochromatic-dev/repository-move-cli` (171 published names,
  file
  holds 169);
  `container-limit-core` is not among them.
- `mutation-test-cli` `lint`:
  two errors and six warnings in `src/host/shards.ts`,
  `src/host/selection.ts` and
  `src/scratch-parent.ts`,
  files this branch does not touch.
- `monorepo-task-core-exec` `lint:parse` failed while `disk-weight.ts` was a deleted,
  uncommitted file;
  `lint` passes now
  that the deletion is committed.
- `git-policy-cli` `bin/native-gate-proof.unit.test.ts`,
  which consumes `diskWeightArguments` through the re-export
  chain,
  passes (read only run).

### Releases

None is needed.
The packages whose behaviour changed are repository task scripts and runners installed from this checkout
(`task-run` is built from source in CI),
not rows of `toolchain.lock.json`.
The lock rows of crates this branch touches (`monochromatic-nested-wayland-session`,
`monochromatic-search-shadow`)
change only in `mise.toml`,
which the published crates do not use.

### Open

- The `bootc-image-builder` run and the rootful build in `vm-builder` have the new argument lists and unit tests,
  and
  have not been run:
  both need `sudo`,
  and no root step was taken.
  Owner decision needed:
  run `mise run //package/vm-builder/cli:run` as root once,
  to see that `--pids-limit=2048`,
  `--memory=12g` and `--cpus=8` do not stop `osbuild`.
- The workflow flags ran on a hosted runner in `ide-release` dry run 42448765561,
  which succeeded on `08c50b0bbb`,
  the commit with the disk-weight fix.
  `music-player-release` dry run 42525780309 on `b7ecf6e22f` fails only in its macOS job,
  `bundle:macos` throwing on a `versionLine` it did not find,
  and macOS is paused by the owner:
  "Ignore Win and mac until I say so."
  Its Android job,
  which failed in run 42521054342,
  now succeeds.
- Sizes marked "adopted" in the inventory are documented siblings,
  not measurements.
  Update 2026-11-08T16:49:36-05:00:
  each one now carries "unmeasured here" beside it,
  and `## What each adopted size would need measured` lists the run each still needs.
  The check found no measurement of any adopted site's own workload,
  and the ledger gains one only when such a run is recorded.
  Update 2026-11-08T19:46:11-05:00:
  the first measured pass ran four sites and its figures sit in
  `### Measured runs on this host`.
  Measured:
  the `containerCargo` release build of `package/music-player/desktop-app`
  (5.3 GiB of 12 GiB, 61 of 1024 processes),
  the music-player image build of the same run (3.35 GiB of 12 GiB),
  the music-player icon one-shot containers (111 MiB of 2 GiB, 1 of 128 processes),
  and the `package/terminal/app` compile container (2.97 GiB of 4 GiB, 235 threads,
  two of whose 4 GiB and weight were missing from the file `mise run` reads until this pass).
  Two sites could not be measured at all,
  because this host's `node_modules` holds links into `${HOME}/.local/share/pnpm/store`,
  which the containers do not mount:
  the `package/test-harness/core-matrix` cell and the root `runPlaywright` browser run.
  Neither figure was changed by this pass:
  the measurements leave the memory values between 2 and 19 times the peak of one variant,
  and every bound covers variants that were not run.
  Still unmeasured after this pass,
  each needing the run its bullet in `## What each adopted size would need measured` names:
  the music-player clippy and nextest containers and its signing containers,
  the `package/test-harness/core` `test:browser` run,
  the `upstream-fork/conf` and `deepmerge-ts.fuzz` container runs,
  the `config/pnpr` verification containers,
  the browserslist oracle container,
  every image build except the music-player one,
  and the rootful `vm-builder` runs.
- The `package/terminal/app` compiler container shipped two different bounds.
  `task.ts` passes `containerLimitArguments(containerBound)` in all twelve copies,
  while `mise.toml` spelled `--memory=4g --cpus=4` by hand,
  so `mise run //package/terminal/app:build:container` (the path `mise` takes) started the container with no process
  option and no disk weight:
  the measured run of 2026-11-08T19:53:09-05:00 recorded `pids.max` at Podman's default 2048,
  not the 1024 the inventory claimed.
  Fixed on this branch:
  the `mise.toml` body now passes the same helper.
  The ledger test in `package/container-limit/core` did not catch it,
  because it accepts a file that merely names a helper somewhere,
  and this file imported one without calling it.
- Two sizes are left as they are with a measurement that suggests they could shrink,
  and the choice belongs to the owner:
  the icon tool bound (2 GiB against a measured 111 MiB)
  and the music-player cargo bound's 1024 processes (against 61 threads).
  Neither is lowered here,
  because each run covered one variant of its workload and the signing containers were not run at all.
- The 4 GiB of the `package/terminal/app` compile container is 74 percent used by a build that reused a warm target
  directory.
  It is not lowered and not raised;
  a cold build measuring it again is the run that would settle it.
- `package-paused/` and `package/git-policy/**` are unchanged.
