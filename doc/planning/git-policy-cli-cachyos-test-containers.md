# Git policy wrapper test containers on CachyOS latest

Status:
 implemented on branch `build/git-policy-cli-cachyos-test-containers`;
the tool picks were then replaced by the owner's
 decisions of 2026-11-04 (section "Owner decisions of 2026-11-04",
on branch
 `feat/git-policy-cli-test-images-from-pacman`).
Where a bullet of "Picks (agent's pick,
veto open)" names a source build,
a checksum or a version pin,
the owner decisions supersede it.

## Why

The gate image (`native:test:container`) started from a content-addressed local image of Debian trixie
 (the official Rust image plus a Git 2.56.0 built on top),
 and the end-to-end image (`test:e2e:concurrent:image`) started from `docker.io/library/node:24.11.0-slim` (Debian)
 with the compile stage in `docker.io/library/rust:1.97-bookworm`.
`doc/decision/container-base-distro.md` makes every image that builds a tool,
test or development environment start from
 the newest CachyOS image the host CPU runs,
on its `latest` tag,
with no version tag and no digest.
None of its exceptions apply here:
 these are not release images (nothing shipped is built in them),
 no tool is missing from the CachyOS repositories (`strace` and `cargo-nextest` were already built or downloaded by
 checksum,
and Rust,
the compiler and `ssh-keygen` are packaged),
 and no binary built in the images runs on the host.

## The conflict and the options

The decision forbids pinning,
and the gate image builds without network from a content-addressed base.
A pin by digest satisfies the second and violates the first;
 a `FROM <latest>` with network satisfies the first and loses the second.

Option A:
resolve `latest` once per run,
derive a base keyed on what the pull returned.
Each run pulls the tag with `podman pull --policy=newer`,
 names a derived base `<first digits of the pulled image ID>-<first digits of the Containerfile hash>`,
 builds it with network only when that tag is missing,
 and builds the gate image from the derived base's ID with no network,
as before.
Pros:
 follows the rule literally (nothing is pinned,
a CachyOS republish is picked up on the next run),
 keeps every gate layer offline and cached on the base ID,
 and a run that finds its base spends one registry request.
Cons:
 a CachyOS republish rebuilds the derived base and every gate layer above it once,
 and the first run on a host needs the network.

Option B:
`FROM` the CachyOS image inside the gate Containerfile and install packages there with network.
Pros:
 one Containerfile,
no derived image.
Cons:
 every layer that follows is keyed on a base that the pull may have changed,
 the compile layers lose the no-network guarantee that proves the vendored sources are complete,
 and the package step runs wherever the cache key moves.

Option C:
pin the base by digest or version.
Pros:
 fully reproducible and offline.
Cons:
 forbidden by the owner ("Must be latest,
since if an upstream OS update breaks us,
we are doing something wrong and we'd
 have to fix it immediately.").

Ranking:
A > B > C.
A beats B because it keeps the gate image's offline,
cached build and costs the same network use as B only when CachyOS
 changed.
B beats C because C is excluded by the owner's directive while B only costs speed.

## Picks (agent's pick, veto open)

- Option A,
  with the derived-base tag as above (`localhost/git-policy-native-test-base` for the gate,
  `localhost/cli-git-concurrent-e2e` for the end-to-end suite).
- A registry that cannot be reached is reported and the run continues on the CachyOS copy the host holds,
  because a gate that cannot start offline teaches people to stop running it;
  with no local copy the run fails.
  This is the one pick that brushes against "must be latest":
  the other choice is to fail the run whenever the pull fails.
- The gate keeps Git 2.56.0 built from the kernel.org tarball (checksum pinned) at `/usr/bin/git`,
  after removing the CachyOS Git package from the base,
  because the gate's results are evidence about that release and the tests name `/usr/bin/git`.
  The CachyOS Git package is also 2.56.0 today but moves with each pacman sync.
  Ranking:
  pinned release > package Git,
  because a gate whose Git moves under it cannot say which Git it verified.
- The end-to-end executable is compiled in the gate's base image (same CachyOS image,
  Rust included) instead of a second
  Rust image,
  so one pull gives both stages the same C library.
- Node 24.11.0 for the end-to-end suite comes from the official tarball (checksum pinned),
  not the packaged Node,
  because the suite has always run on that version.
- Rust,
  Cargo and Clippy in the images are the CachyOS packages (the Debian images carried 1.97.x),
  so the gate compiles with whatever stable release CachyOS ships.
- `strace` stays at the pinned source release but builds with its bundled kernel headers and without `-Werror`,
  because the newest `linux-api-headers` and compiler stop the unmodified build.
  This is the cost of `latest` the decision names:
  the build broke on an upstream update and was fixed here.

## Owner decisions of 2026-11-04

Asked to veto the picks in the `Picks (agent's pick, veto open)` section
(Git 2.56.0 built from source in the gate image,
Rust from pacman,
Node 24.11.0 from the
upstream tarball,
`strace` 6.13 downloaded and checked by checksum),
the owner answered "Make as many pkgs as possible
come from podman."
and,
asked which was meant,
"Sorry,
typo.
Yes I meant from pacman."
Then,
in order:

- "We don't need to pin git because my current system is already using git latest."
- "We don't need to pin Node too."
- Asked where the test images should get the tools `toolchain.lock.json` locks (Node,
  cargo-nextest,
  Rust,
  pnpm),
  against `AGENTS.md` rule CM7,
  the owner chose "Pacman for everything it has":
  Node,
  cargo-nextest and Rust from pacman's
  current packages,
  and pnpm from the lock only if pacman lacks it.
  This is an exception to CM7 for these test images.
- Asked about Git 2.55.0 in the end-to-end image (the oldest Git the wrapper accepted;
  it refused older Git):
  "Raise minimum,
  drop the test,
  merely print a warning on older git and not refuse."

What changed,
each with its evidence (probed 2026-11-04 in `docker.io/cachyos/cachyos-v4:latest` after `pacman -Sy`):

- Git:
  the gate and end-to-end images install pacman's current `git`;
  no source build,
  no checksum.
  `pacman -Si git` listed `git 2.56.0-1.1` (repository `cachyos-extra-v4`) and `git 2.56.0-1` (`extra`);
  the image
  installed `git 2.56.0-1.1`.
  The host's `/usr/bin/git --version` printed `git version 2.56.0` on the same day.
  No exact-version route (an Arch Linux Archive package,
  a dated snapshot) is used.
- The Git source tree at `/work/git-source` is gone.
  The package ships the rendered page `/usr/share/man/man5/githooks.5.gz` and no `githooks.adoc`,
  so the hook-event guard
  test (`invocation::hooks_off::tests::every_documented_hook_event_is_listed`) reads the page through `man`,
  which the
  image gets from the `man-db` package.
- Node:
  `nodejs` from pacman (26.12.1-2 from `cachyos-v4` when probed;
  `extra` had 26.13.0-1,
  and the host runs
  v26.13.0),
  instead of the 24.11.0 tarball.
  The suite is TypeScript run by Node type stripping,
  and nothing in the package reads the Node version.
  `nodejs-lts-krypton` (24.22.2-1 in `extra`) exists if the owner wants the 24 line.
- pnpm:
  pacman carries it (`pnpm 12.17.0-1.1`,
  `cachyos-extra-v4`;
  the lock row records 12.16.1),
  so the gate image no
  longer extracts the lock row's archive and `bin/native-pnpm-snapshot.ts` is deleted.
- cargo-nextest:
  pacman carries it (`cargo-nextest 0.9.150-1.1`,
  `cachyos-extra-v4`;
  the former pin was 0.9.148 by
  checksum).
  `bin/native-nextest-snapshot.ts` stays only because `package/git-policy/cli.fuzz` and `package/linter/monochromatic-lint`
  (and its fuzz sidecar) still import it for their own images.
- strace:
  pacman's `strace` (7.3-1 in `extra`) replaces the statically linked 6.13 build;
  the gate image no longer needs
  a build stage,
  a source archive or `--enable-bundled`/`--disable-gcc-Werror`,
  and `bin/native-strace-snapshot.ts` is
  deleted.
  The Arch Linux Archive lists `strace-6.13-1-x86_64.pkg.tar.zst` should the owner ever want 6.13 back through pacman.
- Rust:
  unchanged,
  pacman's `rust` (1:1.99.1-1 in `extra` when probed).
- End-to-end Git:
  the 2.55.0 source build and its scenario list are gone;
  the suite runs the image's `/usr/bin/git` and
  reads its release at run time.
  The wrapper no longer refuses an older Git;
  see `package/git-policy/cli/SPEC.md`,
  "Compatibility and degradation".
  Its `MINIMUM_GIT_VERSION` is 2.56.0,
  the release the gate runs with,
  and is raised by hand with the image's Git.
- Signatures:
  the CachyOS image's `/etc/pacman.conf` sets `SigLevel = Required DatabaseOptional`,
  and the Containerfiles
  never lower it;
  `pacman --sync` downloaded `git-2.56.0-1.1-x86_64_v4.pkg.tar.zst` with its `.sig` beside it.
  Reproducibility is the cost:
  the tools move with each CachyOS republish or pacman sync,
  which the owner accepted by
  rejecting pins ("if an upstream OS update breaks us,
  we are doing something wrong and we'd have to fix it immediately").
  `mise run //package/git-policy/cli:test:git-image` prints the release of each tool the base image got.

### Verified on 2026-11-04 (gate and end-to-end suite on the pacman images)

- Gate on this branch,
  second run (log `/var/home/user/temp/agent/gate/gpc-gate-test-images-from-pacman-second-20261104T090950-0500.log`):
  4754 tests run,
  4754 passed,
  7 skipped,
  `gate-exit-status=0`,
  nextest 465.8 s,
  no "NOT RUN" lines (the fault-injection
  controls ran under pacman's `strace` 7.3).
  The first run (`gpc-gate-test-images-from-pacman-20261104T085700-0500.log`) failed one test,
  `package_repair::a_missing_pnpm_never_fails_the_add`:
  it assumed no `pnpm` in `/usr/bin`,
  and pacman put one there.
  The control now builds its own `PATH` (the wrapper's directory and a directory holding only real Git).
- Control gate on main `d4979bab1b` (log `gpc-gate-main-control-for-pacman-images-20261104T092122-0500.log`):
  4753 tests,
  4753 passed,
  7 skipped,
  nextest 464.6 s.
  The sorted lists of passing test names differ in the names the owner's decision changed,
  and nowhere else.
  Only in main:
  `git_version::tests::an_older_release_is_refused`,
  `git_version::tests::the_unsupported_message_names_the_git_and_the_remedy`
  and `git_version_cache::tests::a_cached_old_release_is_refused_like_a_live_one`.
  Only on this branch:
  `git_version::tests::an_older_release_is_accepted`,
  `git_version::tests::an_older_release_draws_a_warning_that_names_both_releases`,
  `git_version::tests::the_unreadable_message_names_the_git_and_the_remedy` and
  `git_version_cache::tests::a_cached_old_release_is_accepted_like_a_live_one`.
- Release of each tool the gate ran with,
  from `mise run //package/git-policy/cli:test:git-image`:
  `git 2.56.0-1.1` (`git version 2.56.0`),
  `nodejs 26.12.1-2`,
  `pnpm 12.17.0-1.1`,
  `strace 7.3-1`,
  `cargo-nextest 0.9.150-1.1`,
  `rust 1:1.99.1-1`,
  on CachyOS image `50d89fb6ea06` (`cachyos-v4:latest`).
  The host's `/usr/bin/git --version` printed `git version 2.56.0` and its `node --version` printed `v26.13.0`.
- End-to-end suite on this branch and on main,
  same seed 2910071103 (logs `e2e.log` and `e2e-control.log` under
  `/var/home/user/temp/agent/pacman-probe/`):
  38 scenario runs each,
  35 passed,
  2 skipped (`replay-unavailable-*`),
  1 failed,
  and the sorted lists of status and scenario name are identical.
  The failure is `sigkill-service-during-work` on both (the service settled 0 push records within 30 s),
  so it does not
  change.
  The report lines name `git 2.56.0` now and `git 2.55.0` on main.

## Left open

- `package/git-policy/cli.fuzz/bin/fuzz-images.ts` (base `62ba2f7ce22b...`,
  Debian Rust image),
  `package/forbidden-strings/cli/bin/snapshot-constants.ts` (base `6ec87f6d290a...`,
  the old gate base) and
  `package/linter/monochromatic-lint/test.Containerfile` (base `84f24e75017a...`) still start from Debian images.
  They are other packages,
  so this change does not touch them.
- The CachyOS Git package and the derived gate base could serve those packages too once their owners move them.
- Flagged for the owner,
  veto open:
  package state drifts between CachyOS republishes.
  Each derived base runs a full `pacman` system upgrade when it is built,
  and it is rebuilt only when the CachyOS image or the Containerfile changes.
  The packages inside a cached base are therefore the ones that were newest when it was built,
  not the ones newest today,
  until the next republish or Containerfile edit.
  The end-to-end image check looks for `not found` lines of `ldd`,
  so a compile base rebuilt later than the end-to-end base cannot silently link against a newer C library.
  Forcing a fresh system upgrade on every run would restore "newest" literally and cost the cached gate layers each time.
- `strace` stays at 6.13 by checksum while upstream has released 7.3;
  it is a fixed test tool,
  not a base image,
  so the base rule does not move it.

## Measured on 2026-11-04 (this host under its ordinary load, two-CPU containers)

Gate image build with the cache switched off,
alternating old and new:
 old 293.4 s and 260.6 s,
new 276.3 s and 267.4 s (the spread of the old pair is larger than the difference).
Gate image build with the cache warm:
old 27.0 s,
new 11.2 s plus 2.0 s for the pull check and base lookup.
New base image,
built once per CachyOS republish or Containerfile edit:
107 s at two CPUs,
with the package cache warm.
Gate test run:
old control 451.8 s (earlier gates of this size:
442 s to 460 s);
new 458.4 s when the host was otherwise
 quiet and 511.0 s while lint and type checks ran beside it,
so the new image adds no time that the load did not explain.
Both gates ran 4707 tests with 7 skipped,
and the lists of passing test names are identical.
End-to-end suite:
old and new both run the same 38 scenario runs,
with the same one failure
 (`sigkill-service-during-work`,
the service settles no push records within 30 s) on the old and the new containers,
 so that failure is not caused by the base image.
Its image build:
compile stage 284 s old,
277 s new;
suite run 47 s old,
48 s new;
base cold build 131 s old,
94 s new.

## Where the release gate's wall time goes (measured 2026-11-10)

A wrapper release waits for two things:
the container gate,
which `launch-gate.ts` starts as one systemd user unit named `gpc-gate-<what it covers>-<timestamp>`
and which runs `native:test:container`,
and the btrfs host controls,
which `native:release` runs through the crate's `dispatchGate` registry entry
(`native:test:host:btrfs`).
This section splits the gate's wall time into its phases from the logs and the unit journal,
names the slowest tests,
says which parts wait on the host's other work,
and records the two changes that cut the wait and the measurement of each.

### The phases of the last gates

Timestamps are second-precise local time with this host's offset,
read from the gate's log name,
the `Main process exited` line of `journalctl --user -u 'gpc-gate-*'`,
and the podman records between them;
the test-run figure is nextest's own `Summary` line.
Startup is everything before the first podman record:
mise,
the sidecar lock check,
the source snapshot,
the dependency stand-ins and the vendored sources.

- `gpc-gate-wrapper-next-alpha-on-630cba63d4-20261110T174626-0500`:
  2026-11-10T17:46:26-05:00 to 18:06:40-05:00,
  1,214 s.
  Startup 43 s.
  Image build 90 s,
  every layer cached and 0 s of compile.
  Test run 1,105 s,
  nextest 1,102.1 s.
  No Clippy because the suite failed.
  Exit 1.
- `gpc-gate-prewarm-control-incremental-20261110T171021-0500`:
  2026-11-10T17:10:21-05:00 to 17:30:06-05:00,
  1,186 s.
  Startup 11 s.
  Image build 199 s,
  compile 136 s.
  Test run 967 s,
  nextest 966 s.
  No Clippy.
  Exit 1.
- `gpc-gate-wrapper-next-alpha-on-cd9b8132f8-20261110T163354-0500`:
  2026-11-10T16:33:54-05:00 to 16:54:07-05:00,
  1,214 s.
  Startup 17 s.
  Image build 185 s,
  compile 131 s.
  Test run 1,004 s,
  nextest 1,000.4 s.
  No Clippy.
  Exit 1.
- `gpc-gate-wrapper-next-alpha-on-483bbc6587-20261110T151454-0500`:
  2026-11-10T15:14:54-05:00 to 15:35:49-05:00,
  1,256 s.
  Startup 18 s.
  Image build 245 s,
  compile 148 s.
  Test run 990 s,
  nextest 987.8 s.
  No Clippy.
  Exit 1.
- `gpc-gate-wrapper-next-alpha-on-46e50d20f3-20261110T093258-0500`:
  2026-11-10T09:32:58-05:00,
  1,561 s.
  Startup 16 s.
  Image build 396 s,
  compile 318 s from a cold dependency layer.
  Test run 933 s,
  nextest 931.8 s.
  Clippy 213 s after it.
  Exit 0.
- `gpc-gate-wrapper-next-alpha-on-9d38e1c7b9-20261110T041640-0500`:
  2026-11-10T04:16:40-05:00 to 04:37:33-05:00,
  1,268 s.
  Startup 10 s.
  Image build 27 s,
  compile 1 s.
  Test run 954 s,
  nextest 945.4 s.
  Clippy 261 s after it.
  Exit 0.
- `gpc-gate-wrapper-tokio-drain-round-2-20261110T024918-0500`:
  2026-11-10T02:49:18-05:00 to 03:09:50-05:00,
  1,233 s.
  Startup 7 s.
  Image build 135 s,
  compile 108 s.
  Test run 890 s,
  nextest 883.2 s.
  Clippy 203 s after it.
  Exit 0.

Three facts about that list decide the rest of this section.

- The test run is 70 to 91 per cent of a gate,
  and it grew with the suite:
  the gate of 2026-11-03 ran about 4,000 tests in 270 to 520 s,
  and the gate of 2026-11-10 runs 7,192 in 890 to 1,105 s.
- Clippy is a flat 195 to 261 s and it stood after the suite,
  so it added its whole compile to the gate's wall time.
- The suite's wall time is its summed test time divided by two.
  Gate `cd9b8132f8` summed 2,000 s of test time inside a 1,000.4 s run,
  and gate `630cba63d4` summed 2,203 s inside 1,102.1 s.
  Those sums come from nextest's status lines,
  halved because `--success-output=final` prints each passing test's status line a second time at the end.
  Two test threads were busy for the whole run,
  so no part of the suite was idle and no lock or sleep was the binding constraint:
  the only way to shorten it without running fewer tests is to run more of them at once.

### The slowest tests

From gate `630cba63d4`,
with gate `cd9b8132f8` in brackets where the two differ:

- `service_landing_state::awkward_change_lists_land_as_native_git_commits_them`
  took 80.5 s,
  and 45.1 s in the earlier gate.
- `fault_trace::a_disk_that_fills_during_a_commit_leaves_a_recoverable_repository`
  took 77.2 s,
  and 76.7 s.
- `fault_trace::storage_that_fails_during_a_commit_leaves_a_recoverable_repository`
  took 65.8 s,
  and 70.0 s.
- `push::push_slow_server_tests::a_push_over_http_waits_past_the_old_low_speed_cutoff_and_lands`
  took 65.1 s in both gates.
- `merge_never_fails::a_merge_that_waits_out_a_live_holder_is_saved_and_not_failed`
  took 60.4 s.

Each of them waits out something the product ships and the test exists to prove:
the merge lock ceiling of one minute (`MERGE_LOCK_CEILING`),
the window a push must wait past a slow server,
a sweep over the fault list with `strace` under it,
a case sweep that starts six cases at a time.
Shortening any of them means shrinking a shipped bound or a sweep's width,
which would weaken the check,
so they stay.
The five cost about 350 s of thread time,
which is 175 s of the gate's wall time at two threads and 88 s at four.

### What waits on other work on the host

- The test run's CPU share was fixed at two by the container's `--cpus 2`,
  the repository's default for every container.
  Under a load average of 80 to 143 on an eight-CPU host,
  that share is what the run may use however busy the rest of the host is:
  the run neither starved nor grew,
  which is why gates of the same size measured 883 s to 1,105 s in the test run itself.
- The btrfs host controls a release also waits for run on the bare host,
  under no container bound at all.
  The package's own record of that run
  (`measuredFullRunSeconds` in `bin/native-release-host-control-requirements.ts`)
  is 3,286 s for 6,974 tests,
  which is 0.47 s per test against 0.28 s in the gate container.
  That is where a release waits on other work most,
  and on 2026-11-10 another agent's build replaced that run's test binary under it and 979 tests failed.
- The image pull of the CachyOS `latest` tag is part of every gate and needs the network,
  measured at 7 to 43 s of startup plus pull.
  The layered build after it is cached:
  the compile is 0 s when no source changed (`630cba63d4`),
  108 to 148 s for a wrapper-only change,
  and 318 s when a manifest or lock file moved the dependency layer.
- Most gates of the day ended red.
  Of the nineteen gates since 2026-11-09T13:32:02-05:00,
  twelve ended with a failed test,
  five were green,
  and two have no recorded status.
  A red gate makes a release wait for another whole gate,
  so this is the largest multiplier on the wait,
  and it is not what these changes address.

### The two changes, and the measurements behind them

Both are in the gate's own harness.

`bin/native-gate-checks.ts`
starts the suite and Clippy together,
one container each,
and the gate waits for both.
Both are started before either is awaited,
both are waited for even when one failed,
so no container is left behind a failure,
and either one still fails the gate on its own,
with Clippy's captured diagnostics quoted in the refusal.

`bin/native-gate-container-limits.ts`
gives the gate's containers four CPUs,
which nextest reads as four test threads and the image build as four compile jobs,
and a process bound of 256 for each of those threads.

The test run was measured on one image from the gate's own container,
alternating the CPU count,
each arm in its own `systemd-run --user --scope`
(`CPUQuota=900%`,
`MemoryMax=16G`,
`TasksMax=2000`,
`IOWeight=1`),
with the cgroup's own `pids.peak`,
`pids.current` and `memory.peak` read while it ran.
All arms ran 7,192 tests with 22 skipped.

- 2 CPUs and 512 processes,
  started 2026-11-10T18:08:50-05:00:
  1,005.0 s,
  nextest 1,001.9 s,
  7,192 passed,
  load average 143.27 at the start.
- 4 CPUs and 512 processes,
  started 2026-11-10T18:25:37-05:00:
  515.5 s,
  nextest 512.7 s,
  7,191 passed and 1 failed,
  load average 82.72.
- 2 CPUs and 512 processes,
  started 2026-11-10T18:34:15-05:00:
  992.4 s,
  nextest 990.3 s,
  7,192 passed,
  load average 144.23.
- 4 CPUs and 512 processes,
  started 2026-11-10T18:52:52-05:00:
  532.6 s,
  nextest 529.1 s,
  7,189 passed and 3 failed,
  process peak 512,
  which is the cap,
  memory peak 767 MiB.
- 4 CPUs and 1024 processes,
  started 2026-11-10T19:03:28-05:00:
  479.1 s,
  nextest 476.7 s,
  7,192 passed,
  process peak 401,
  memory peak 978 MiB.

Four test threads halve the suite:
479.1 s against 992.4 s and 1,005.0 s at two,
1.95 to 2.10 times,
with the same tests.
The summed thread time is 2,003.8 s and 1,980.5 s at two threads and 1,906.8 s at four,
so the work itself did not grow and the host had the CPUs to give.

Four threads need more processes,
because the fault sweeps of the binary suite start six cases at once inside one test,
and each case starts a service with its own threads and the Git programs it runs.
At 512 processes the demand reached the cap in both four-thread runs and the failures were `fork` refusals:
`the fixture's commit service starts: Os { code: 11, kind: WouldBlock }`.
The same four threads at 1024 processes peaked at 401 and passed everything,
so the process bound follows the thread count at 256 for each.
A two-CPU container is unchanged at the repository's 512.

Memory does not need to move.
The suite peaked at 978 MiB.
Clippy reached the 2 GiB bound itself,
2,048 MiB in the peak file of the run it passed in.
So the memory bound stays the repository default,
and a gate that ever fails Clippy on memory has `--memory 4g` to take.

The whole gate was then run twice through `bin/test-native-container.ts` with the changes in place,
tagged `gate-wall-time-after`,
and both times the Clippy container started in the same second as the suite's and finished long before it:

- Started 2026-11-10T19:19:50-05:00,
  wall 865 s.
  Startup and image build 369 s,
  because this worktree's snapshot rebuilt the path-crate layer and the subject layer,
  in three compiles of 79 s,
  37.8 s and 84 s at four jobs.
  Both containers started at 19:25:59-05:00.
  Clippy died at 19:28:16-05:00 after 137 s.
  The suite died at 19:34:09-05:00 after 490 s,
  nextest 488.9 s,
  7,191 passed and 1 failed.
- Started 2026-11-10T19:37:33-05:00,
  wall 512 s.
  Startup and image build 37 s,
  a full cache hit with 0.25 s of compile.
  Both containers started at 19:38:10-05:00.
  Clippy died at 19:40:28-05:00 after 138 s.
  The suite died at 19:46:05-05:00 after 475 s,
  nextest 473.2 s,
  7,190 passed and 2 failed.

So a gate whose build is cached,
which is the common case once the layers are warm,
takes 512 s.
The like-for-like gate before it is `9d38e1c7b9`,
the one cached-build gate of 2026-11-10 that ran Clippy,
at 1,268 s:
10 s of startup and 27 s of image build against 37 s of both,
954 s of suite against 475 s,
and a 261 s Clippy tail against none,
with 7,192 tests here against 6,974 there.
The three gates of the day whose build recompiled the subject layer took 1,186 s to 1,256 s,
and their suite stood at 967 s to 1,004 s,
so the build is not what the 512 s rests on.
The failures in those two runs are tests that were already failing on this base.
`repair::package_repair_prewarm_job::tests::the_filling_publishes_a_clean_copy_and_leaves_the_source_alone`
failed the same way in gate `cd9b8132f8` at two CPUs.
`fault_limits::a_descriptor_limit_close_to_exhaustion_on_the_service_leaves_a_recoverable_repository`
failed with "no descriptor limit let the commit land" in gates `prewarm-control-incremental` and `630cba63d4` at two
CPUs.

### What is left

- The host controls are the bigger wait (3,286 s against a 1,000 s suite),
  and nothing here shortens them:
  they run on the bare host,
  where every other agent's work is unbounded relative to them.
  Giving that run a cgroup with a CPU share,
  or running it in the gate's image the way the gate does,
  is a separate change with its own measurement.
- The four-thread curve had not flattened.
  479 s at four threads against 992 s at two is very close to linear,
  so six threads are the next point to measure,
  and the gate's systemd unit allows ten.
  The process bound would follow to 1,536.
- The gate image's dependency layer still costs 318 s when a manifest or a lock file moves,
  which a version bump does.
  The build's job count follows the gate's CPU count,
  so the cold layers now get the 1.6 to 1.9 times the package README measured at four CPUs,
  and the rest of that build is step overhead on a busy host.
- A mutation campaign that reuses a gate image keeps the gate's proof only when its own process bound is at least the
  gate's,
  so a campaign at the default 512 processes runs its baseline once.
  Passing `--pids-limit 1024` keeps it.
  Having the campaign take the gate's bound from the manifest is the change that would remove that cost.
- The two flaky tests named above are pre-existing and were left alone.
  Each one makes a whole gate red,
  and so costs a release another gate.
