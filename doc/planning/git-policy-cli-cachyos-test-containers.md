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

From lines 46775 to 46824 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- CachyOS test containers:
  "Make as many pkgs as possible come from podman.",
  then,
  asked which was meant:
  "Sorry,
  typo.
  Yes I meant from pacman.
  And I believe pacman can give a specific Git version.
  If not possible,
  bring me the evidence and re-ask."
  So Git,
  Node and strace move to pacman inside the CachyOS image wherever pacman can supply what the tests need,
  Git at the pinned version through pacman if pacman can install it;
  if it cannot,
  bring the evidence and ask again.
  Work on `feat/git-policy-cli-test-images-from-pacman`.
  The owner then added:
  "I just realized something.
  We don't need to pin git because my current system is already using git latest."
  So the test image takes pacman's current `git`,
  with no version pin.
  Then:
  "We don't need to pin Node too.
  And what else?"
  Asked through the question tool about the tools `toolchain.lock.json` locks (Node,
  cargo-nextest,
  Rust,
  pnpm) against CM7's "in containers and CI use its row's archive and sha256",
  the owner chose "Pacman for everything it has"
  (the option named the CM7 exception);
  so these test images take every tool pacman carries from pacman,
  and pnpm from the lock row only if pacman lacks it.
  Open:
  propose to the owner the AGENTS.md wording of that exception (CM7),
  not yet asked.
  Asked about the end-to-end image's Git 2.55.0 (the oldest Git the wrapper accepts),
  the owner answered:
  "Raise minimum,
  drop the test,
  merely print a warning on older git and not refuse."
  So the 2.55.0 build and its scenarios go,
  the minimum rises to the tests' Git,
  and an older Git gets a warning and the command runs.
  Both sent to the agent on `feat/git-policy-cli-test-images-from-pacman`.
  CM7's wording:
  the owner chose "Leave CM7 unchanged" with the note "Record this carve-out for whatever the git wrapper needs.";
  so `AGENTS.md` stays as it is,
  and the package docs of `package/git-policy/cli` record the carve-out (its test images take tools from pacman).
  The other container picks (resolve `latest` once per run,
  run on the local copy with a loud warning when the registry is unreachable) were not addressed and stand.

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
  Test run 1,104 s,
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
  and the gate of 2026-11-10 runs 7,192 in 883 to 1,104 s.
- Clippy is a flat 195 to 261 s and it stood after the suite,
  so it added its whole compile to the gate's wall time.
  The range is the five green gates of 2026-11-09 and 2026-11-10:
  195 s in `212fc5a4e2`,
  203 s in `tokio-drain-round-2`,
  213 s in `46e50d20f3`,
  219 s in `9724cb6270`,
  and 261 s in `9d38e1c7b9`.
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
  which is why gates of the same size measured 883 s to 1,104 s in the test run itself.
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
  fourteen recorded a failed test in their nextest summary,
  between one and sixty-nine of the 6,382 to 7,192 tests each ran,
  and five were green.
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
At 512 processes one of the two four-thread runs sampled its process peak at exactly 512,
which is the cap,
and both failed with `fork` refusals:
`the fixture's commit service starts: Os { code: 11, kind: WouldBlock }`.
The failure count there is not one per run,
because both runs also carried the two pre-existing flakes named below:
the one sampled run failed three tests,
two of them the sequencer ones that were refused a process,
and the other failed one test,
again a refusal.
At 1024 processes the run whose peak was sampled reached 401 and passed all 7,192,
so the process bound follows the thread count at 256 for each.
A two-CPU container is unchanged at the repository's 512.
A count of `Resource temporarily unavailable` lines is not a failure signal:
the passing 1024-process run printed 41 of them,
from tests that provoke that refusal on purpose
(`subvolume::subvolume_index_files::parallel_tests::a_report_that_says_an_index_could_not_be_started_leaves_no_job_that_can_rewrite_it`
is one),
and the failing 512-process run printed 43.

The memory bound was left where it is,
and the evidence for that is weaker than for the other two.
The suite peaked at 978 MiB of the 2 GiB.
Clippy reached the bound itself:
2,048 MiB in the peak file of the run it passed in,
which is no headroom at all.
Two passing runs say it has not been killed for memory yet,
and nothing more.
The first gate that fails Clippy on memory is when to move it,
and `--memory 4g` is the option that does it.

That happened on 2026-11-27 (issue 6562),
and the 4 GiB this passage named would not have held it either.
The gate on main at `d9ec2443bd` passed all 12,089 tests of its suite and then refused,
because the Clippy container's `clippy-driver` was killed while it compiled the crate's test build at the 2 GiB it
shared with that suite.
The measurement in "The Clippy container's own memory bound,
measured 2026-11-27" below put that container's peak at
4,574.4 MiB with a bound no step of the run met,
so the Clippy container now has a bound of its own:
8 GiB (`gateClippyMemory` in `bin/native-gate-container-limits.ts`).
The suite keeps the 2 GiB its own measurement supports:
978 MiB in the 2026-11-10 run above,
1,123.8 MiB in the integration pass of 2026-11-19 on a ZFS host
(`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`),
and 1,149.3 MiB in the arm of 2026-11-27.
The image build shared that 2 GiB until 2026-11-24 and now has one of its own,
measured in "The image build's own memory bound,
measured 2026-11-24" below.

The whole gate was then run twice through `bin/test-native-container.ts` with the changes in place,
tagged `gate-wall-time-after`,
and both times the Clippy container started in the same second as the suite's and finished long before it.
That is also a change in what the gate draws from the host at once:
the suite's four CPUs plus Clippy's four,
which is 8 CPUs inside the 1,000% its systemd unit allows,
against 4 before,
so the peak draw is the reading to compare when asking whether a gate crowds the other agents.

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
- Started 2026-11-10T19:57:14-05:00,
  wall 490 s.
  Startup and image build 24 s,
  every layer cached again.
  Both containers started at 19:57:38-05:00.
  Clippy died at 19:59:49-05:00 after 131 s.
  The suite died at 20:05:24-05:00 after 466 s,
  nextest 464.8 s,
  7,191 passed and 1 failed.

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
Every failure in those three runs is a test that was already failing before this change,
and the first of them is already fixed on main.
`repair::package_repair_prewarm_job::tests::the_filling_publishes_a_clean_copy_and_leaves_the_source_alone`
failed in all three,
and its control is the one commit `f18a7a764a` fixed at 2026-11-10T17:09:50-05:00,
whose message quotes the same line from gate `cd9b8132f8`:
the control read `debug/incremental` alone,
and the gate's image sets `CARGO_INCREMENTAL=0`.
This branch's base is `e498914475`,
which predates that fix,
so this gate cannot be green until main is merged in.
The image of gate `630cba63d4` was built after the fix,
and it passes all six tests of that module in one container,
and this branch's image fails one of them,
with the same `CARGO_INCREMENTAL=1` making it pass.
`fault_limits::a_descriptor_limit_close_to_exhaustion_on_the_service_leaves_a_recoverable_repository`
failed with "no descriptor limit let the commit land" in gates `prewarm-control-incremental` and `630cba63d4` at two
CPUs,
so it is not this change either.

### The image build's own memory bound, measured 2026-11-24

The gate's image build ran under the repository's shared 2 GiB container bound until this measurement,
because that bound was the default and no reading of the build existed.
At 2026-11-24T19:20-05:00 its compile sat at 2,047 MiB of it,
its own `memory.pressure` at some avg10 6.48,
while the host had 27,529 to 30,132 MiB available
(`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
"Catalog:
a rising stall from one capped container,
at 2026-11-24T19:20:00-05:00").
A compile held at its own cap runs in reclaim and swap instead of RAM,
and that lifts the machine-wide stall figure that decides whether new work may start (issue 6455).

Three arms were built,
one build each,
and they are the round's whole measurement rather than repeated runs:
each is a `podman build --no-cache` of the gate's own test image
from the same snapshot,
stand-ins,
vendored sources and Containerfile text that `bin/test-native-container.ts` writes,
at the gate's own CPU count (`--cpu-period=100000 --cpu-quota=400000`,
so `CARGO_BUILD_JOBS=4`),
in its own `systemd-run --user --scope` of six CPUs,
12 GiB and 2,000 tasks.
Every build container's own cgroup was read every 250 ms:
`memory.peak`,
`memory.current`,
`memory.max`,
`memory.pressure`,
`memory.events`,
`cpu.stat` and `pids.peak`.
Podman gives each build container a scope of its own beside the harness's,
so a step's peak and its own stall are
readable per step;
the harness's copy of the readings is in
`~/temp/agent/size-the-gate-image-build-memory-2026-11-24/keep/arms/`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here),
and the harness and the summariser that produced them are in that folder's `keep/harness/`.

The arms ran in this order,
host load at each start being what the day had:

- Arm a,
  `--memory=2g` (today's bound,
  the control),
  started 2026-11-24T19:51:18-05:00,
  build 574.1 s,
  load 19.16.
  Its build containers were not sampled:
  the harness then read only a scope's own `cgroup.procs`,
  and a build container's processes live in the scope's `container` child,
  so the peaks below are the two later arms' and this arm's own five-second samples of the compile it was held at.
- Arm b,
  `--memory=10g` (a cap no step of it met),
  started 2026-11-24T20:04:12-05:00,
  build 454.2 s,
  load 12.34.
- Arm c,
  `--memory=2g` (the control again,
  which bounds the round's run-to-run spread),
  started 2026-11-24T20:14:18-05:00,
  build 549.1 s,
  load 11.45.

At four jobs the image build compiles three layers,
and each is its own container with its own bound:

- The dependency layer (build step 8) peaked at 1,247.3 MiB under 10 GiB and at 1,302.4 MiB under the 2 GiB,
  so the shared default never bound it;
  it took 111.9 s in arm b and 110.6 s in arm c.
- The path-crate layer (step 15) peaked at 2,876.2 MiB under 10 GiB.
  Under the 2 GiB it reached 2,048.0 MiB and stayed there:
  `memory.events` `max` 9,437,
  some avg10 6.07,
  1.1 s of stall,
  52.2 s against arm b's 54.3 s.
- The wrapper's own sources (step 17),
  which recompile the crate and every test target,
  peaked at 5,363.4 MiB under 10 GiB with 0.1 s of stall in 87.2 s.
  Under the 2 GiB it reached 2,048.0 MiB and stayed there too:
  `memory.events` `max` 149,888,
  some avg10 22.33 at its highest,
  29.8 s of stall over 196 s of life,
  and 197.1 s against 87.2 s,
  which is 2.26 times as long.

Read as ratios to the controls of the same round rather than as absolute times:
arm b's step 17 was 0.41 of arm a's and 0.44 of arm c's,
while the two control arms differ from each other by 1.07,
so the compile is between 2.26 and 2.42 times faster without the cap,
which is larger than the spread the round itself showed.
The whole build was 0.79 of arm a and 0.83 of arm c,
against a control spread of 1.05,
so a whole gate build is between 1.21 and 1.26 times faster.
The other two steps' ratios (0.86 to 1.10) sit inside the control spread of their own steps (1.08 to 1.21),
so they are the host's noise and not the bound's doing.

So the build is bounded at 8 GiB (`gateBuildMemory` in `bin/native-gate-container-limits.ts`):
8,192 MiB,
1.53 times the 5,363.4 MiB the wrapper's own compile peaked at,
which leaves room for a source change that adds work and stays well inside the 20 GiB the gate's own systemd unit allows.
The suite's and Clippy's 2 GiB was untouched then;
Clippy got a bound of its own on 2026-11-27 ("The Clippy container's own memory bound,
measured 2026-11-27"),
and the suite keeps its 2 GiB.
The bound is per container,
so it is the largest single step that sizes it,
not the sum of a build's steps.

What it costs and how a person lowers it:

- The build may now take 8 GiB of the host's RAM at once instead of being held at 2 GiB.
  On this host,
  with 31 to 44 GiB available across the arms,
  that is memory that would otherwise have been spent on
  reclaiming and swapping,
  and the compile of the wrapper's sources is 2.26 to 2.42 times faster for it.
- A host that cannot spare 8 GiB lowers it with `--memory <size>` after the task name,
  which bounds the build and the suite together,
  and the gate prints which bound each container got before it builds.
  Below what the compile needs,
  that host pays in reclaim and time exactly as these capped arms did;
  it does not stall the rest of the host,
  because the bound is the container's own.
- A container bounded below the compile can also be killed rather than slowed:
  another round measured the crate's lib tests in the gate image under `podman run --memory=2g`,
  where the test `rustc` died with `signal: 9, SIGKILL` at `--cpus=4` and at `--cpus=2` and the same command compiled
  and ran at `--memory=4g`
  (that round's comment on issue 6455,
  2026-11-24).
  A build container carries the repository's swap allowance,
  so the capped arms above spent their time in reclaim instead,
  which is the slower and non-fatal version of the same bound.
- The bound is sized at four compile jobs.
  A gate given more CPUs compiles more jobs at once and wants its `--memory` raised with them;
  nothing here measured more than four.
- The base image's own build keeps the shared 2 GiB default:
  it compiles nothing,
  and no measurement of it exists.

### The Clippy container's own memory bound, measured 2026-11-27

The Clippy container ran under the repository's shared 2 GiB until this measurement,
and on 2026-11-27 that bound ended a gate whose other check had passed:
the gate on main at `d9ec2443bd` passed all 12,089 tests of its suite
(`Summary [1279.786s] 12089 tests run: 12089 passed (9 slow), 51 skipped`)
and then refused,
because `clippy-driver` was killed while it compiled `monochromatic-git-policy-cli`'s lib test
(`signal: 9, SIGKILL`),
with no diagnostic,
and the kernel log carried the kill in the container's own cgroup
(`Memory cgroup out of memory: Killed process (clippy-driver) total-vm:6042920kB, anon-rss:2074084kB`,
issue 6562).
The same command on the host at two jobs and with no swap finished clean at a memory peak of 4.4 GiB,
which said the step needs more than twice its bound but not what the gate's own container needs.

One arm was run,
because the gate's own run is the measurement:
`native:test:container` started 2026-11-27T03:26:20-05:00 in its own `systemd-run --user --scope` of twenty gigabytes,
ten CPUs,
4,000 tasks and a ninety-minute bound,
on the tree of `2939b47565` (main's tip,
with no change to the crate's Rust source),
with `--memory 8g`,
a bound no step of it met,
and the image tag `issue-6562-clippy-measurement`.
It built the gate's own image first,
which took 66 minutes at the gate's four compile jobs under a host load average of 96,
and then started both containers together the way every gate does.
Each container's own cgroup was read every 100 ms:
`memory.peak`,
`memory.current`,
`memory.max`,
`memory.swap.current`,
`memory.swap.max`,
`memory.events` and `pids.peak`.
The harness and its samples are in that round's scratch folder
(`keep/poller/gate-container-peaks.ts`,
`keep/measurement-peaks.jsonl`),
and what this record uses from them is stated here.

- Clippy's container peaked at 4,574.4 MiB of the 8,192 MiB bound,
  with `memory.events` `max` 0 and `oom_kill` 0,
  so that bound never bound it.
  Its `pids.peak` was 75,
  it ran 15 minutes 12 seconds,
  and up to 647.1 MiB of its charge sat in swap,
  which the host's own pressure put there rather than the container's bound,
  so its pages were at most 5,221.5 MiB.
- The suite's container,
  in the same run and under the same 8 GiB,
  peaked at 1,149.3 MiB with `memory.events` `max` 0 and `oom_kill` 0,
  which agrees with the 1,123.8 MiB the integration pass of 2026-11-19 read on a ZFS host.
  The arm was stopped by its scope's ninety-minute bound at 5,566 of the 12,089 tests,
  so it says nothing about the whole suite except through that peak;
  the whole gate at this bound is recorded below.

So the Clippy container is bounded at 8 GiB (`gateClippyMemory` in `bin/native-gate-container-limits.ts`):
8,192 MiB,
1.55 times the highest peak any run of it read (5,278.7 MiB) and 2,913 MiB above that peak,
which leaves room for a source change that adds work.
The suite keeps the 2 GiB its own measurement supports,
the image build keeps its 8 GiB,
and the line the gate prints before it builds names all three bounds;
a `--memory` after the task name still bounds every container.

What it costs and how a person lowers it:

- The gate may now take 8 GiB for Clippy where it took 2 GiB,
  beside the suite's own 2 GiB,
  and the two containers start together,
  so a gate at its bounds can hold 10 GiB of the host at once.
  The host held 19,522 MiB available when the arm started (`free -m`),
  and the capped behavior measured on 2026-11-27 was the kill itself rather than a slower compile.
- A host that cannot spare 8 GiB for Clippy lowers it with `--memory <size>` after the task name,
  which bounds the build,
  Clippy and the suite together;
  below what the compile needs,
  that host gets the 2026-11-27 failure rather than a slower compile.
- The bound is sized at the gate's own four compile jobs.
  A gate given more CPUs compiles more targets at once and wants `--memory` raised with them;
  nothing here measured more than four.
- One arm was measured and it has no control arm of its own.
  The peak is the kernel's exact high-water mark for that container,
  but a quieter host keeps more of a compile's pages in RAM and could read higher,
  and this arm is the only figure of the gate's own Clippy container that exists.

The whole gate at this bound is "The gate at the Clippy container's own bound,
run 2026-11-27" below.

### The gate at the Clippy container's own bound, run 2026-11-27

The bound was then tried by whole gates on the branch that sets it,
merged with main three times as main moved (`2a3bf031f8`,
`cb5ad03763`,
`ec4cb1ce37`),
each started by `native:gate` with `--unit-io-weight 1`,
its own image tag and the gate's own unit caps of ten CPUs,
20 GiB and 4,000 tasks.
Each gate's two containers were read the same way the arm was.

The readings of the Clippy container,
all against the 8,192 MiB bound:

- 4,574.4 MiB in the arm above,
  with 647.1 MiB of its charge in swap because the host was under its own pressure.
- 4,790.0 MiB in the first gate,
  in 6 minutes 54 seconds,
  with up to 414.0 MiB of swap and `memory.events` `max` 0.
- 4,835.4 MiB in the second gate,
  in 4 minutes 29 seconds,
  with 36.0 MiB of swap and `memory.events` `max` 0.
- 5,062.8 MiB in the third gate,
  in 3 minutes 8 seconds,
  with 18.9 MiB of swap and `memory.events` `max` 0.
- 5,278.7 MiB in the fourth gate,
  in 2 minutes 19 seconds,
  with no swap and `memory.events` `max` 0.
- 5,273.4 MiB in the fifth gate,
  in 1 minute 55 seconds,
  with no swap and `memory.events` `max` 0.

A compile the host is reclaiming reads lower than one that keeps its pages,
so the highest of those,
5,278.7 MiB,
is the figure the bound is sized from.
The three gates that read it ran with little or no swap,
and their readings agree within 0.3 percent of each other,
while the arm's 4,574.4 MiB is 704.3 MiB lower,
which is the room a host under pressure hides.
The suite's container read 1,149.3 MiB in the arm,
1,160.0 MiB in the first gate,
1,459.9 MiB in the second,
1,354.6 MiB in the third,
1,462.8 MiB in the fourth,
and 1,516.5 MiB in the fifth,
all of its own 2,048 MiB bound,
with `memory.events` `max` 0 and `oom_kill` 0 in every one,
and its `pids.peak` reached 504 and 576 of the 1,024 bound.
The suite's highest reading leaves 531.5 MiB of its bound,
which is worth a measurement of its own in a later round,
and the bound held it.
Clippy passed in every gate:
three refusals named their suites alone,
and two gates passed both checks.

- The first gate,
  `gpc-gate-issue-6562-clippy-bound-20261127T045928-0500.service`,
  refused with `Summary [1922.733s] 12096 tests run: 12093 passed (20 slow), 3 failed, 51 skipped`
  while the host's load average stood at 96 to 48,
  and took 49 minutes 48 seconds.
  Each of the three is a control that a loaded host moves under its feet:
  `a_scratch_commit_the_kernel_never_releases_is_ended_with_a_message` was held 31.9 s by a `gpg` in uninterruptible
  sleep (`the bound did not end the step`),
  `a_step_that_finishes_inside_its_ceiling_is_answered` read `the system refused a thread`,
  and `a_transfer_that_meets_the_floor_is_work_however_slow` ended a slow transfer at its 2-second window.
  The gate's refusal named the test run alone,
  so its Clippy check had passed.
- The second gate,
  `gpc-gate-issue-6562-clippy-bound-2-20261127T055625-0500.service`,
  refused with `Summary [1476.182s] 12096 tests run: 12095 passed (9 slow), 1 failed, 51 skipped`:
  `commit::native_commit_waiting_notice::tests::the_notice_reports_the_programs_that_ran_below_the_step`
  saw a `sleep` end between two of its looks,
  which `doc/troubleshooting/git-policy-cli-commit-index-span-control-counts-another-tests-spans.md` already records as
  a control whose own spans another test's spans can disturb.
  It took 25 minutes 35 seconds.
- The third gate,
  `gpc-gate-issue-6562-clippy-bound-3-20261127T062548-0500.service`,
  passed both checks:
  `Summary [1259.092s] 12096 tests run: 12096 passed (8 slow), 51 skipped`
  and `Clippy passed beside the test run.`,
  with `gate-exit-status=0`.
  It ran from 06:25:48-05:00 to 06:47:31-05:00,
  21 minutes 43 seconds,
  and its unit's own memory peak was 1,017.1 MiB of the 20 GiB the unit allows,
  which is the clients and the image build rather than the containers,
  since those sit in their own scopes.
- The fourth gate,
  `gpc-gate-issue-6562-clippy-bound-4-20261127T065353-0500.service`,
  ran on the merge at `cb5ad03763` and refused with
  `Summary [ 940.253s] 12114 tests run: 12113 passed (6 slow), 1 failed, 51 skipped`:
  `process::captured_child_run_to_its_end::tests::the_one_shot_git_questions_no_longer_block_a_thread_on_command_output`
  read `repo_read/git_system_config.rs does not use the captured run`.
  That control came into this branch with main's `9df756cd37`,
  and main's next merge (`88a63ef565`) no longer holds it,
  so the merge this round made after the gate took it away and the fifth gate ran without it.
  It took 23 minutes 53 seconds.
- The fifth gate,
  `gpc-gate-issue-6562-clippy-bound-5-20261127T072220-0500.service`,
  ran on `ec4cb1ce37`,
  the last merge of main this round made,
  and passed both checks:
  `Summary [ 889.107s] 12115 tests run: 12115 passed (5 slow), 51 skipped`
  and `Clippy passed beside the test run.`,
  with `gate-exit-status=0`.
  It ran from 07:22:20-05:00 to 07:42:55-05:00,
  20 minutes 36 seconds.

So the bound is what the gate needs at four compile jobs,
and every refusal before the green gates was the loaded host's own control or main's own moving code,
not the memory bound.

### The bound on each check

Starting the two checks together added a wait the gate did not have:
on 2026-11-11 a review of that change found that a failed test run could be known while Clippy's client never
answered,
and the gate waited for both forever,
so the failure a person needs was never reported.

Each check is now given one hour (`gateCheckDeadlineMilliseconds` in `bin/native-gate-checks.ts`).
Three times the slowest measured phase of the gate,
1,104 s of test run at two threads on 2026-11-10,
is 3,312 s,
so the bound is one hour,
which is the same arithmetic the btrfs host controls' three hours use against their 3,286-second run
(`bin/native-release-host-control-requirements.ts`).
A check that reaches the bound stops its client and everything that client started,
removes its container by the ID file the client wrote while it ran
(`bin/native-gate-container.ts`),
and reports the bound it missed.
The gate waits three minutes past that bound before it answers for a check which ignores its own bound,
which is longer than a removal may take,
so a stopped check always removes its container before the gate's refusal lets the run exit.

Measured that day against a stand-in `podman` whose `run` client never exits,
with both checks on a 1,500 ms bound:
both clients started with `--cidfile`,
both were stopped at the bound,
both containers were removed by their ID files,
and the refusal quoted the bound each client missed,
after 1,681 ms.

### What is left

- The host controls are the bigger wait (3,286 s against a 1,000 s suite),
  and nothing here shortens them:
  they run on the bare host,
  where every other agent's work is unbounded relative to them.
  Giving that run a cgroup with a CPU share,
  or running it in the gate's image the way the gate does,
  is a separate change with its own measurement.
- Two other ways to four threads were rejected on arithmetic,
  not on measurement,
  and either could be measured instead:
  declaring nextest `threads-required` on the fault sweeps would serialize about 409 s of their thread time,
  `fault_limits::` 113.1 s and `fault_trace::` 295.9 s in one two-thread log,
  which is most of the 513 s the fourth thread bought;
  and two half-partitioned containers of 512 processes each would put the same 1,024 processes on the host,
  add a partition whose completeness the gate would have to prove,
  and leave the proof manifest with one limit set to record.
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

### The suite container's own memory bound, measured 2026-11-27

The suite's container took the repository default of 2 GiB until issue 6583,
while the image build and Clippy took 8 GiB each.
The gates of the Clippy round read it at 1,149.3,
1,160.0,
1,459.9,
1,354.6,
1,462.8 and 1,516.5 MiB of that bound,
with `memory.events` `max` 0 and `oom_kill` 0 every time,
and the reading rose across one morning as the suite grew,
which is what the issue asked about:
what the peak is made of,
what bound to give it,
and what the gate says when the container is killed at that bound.

One arm was run,
because the gate's own run is the measurement:
`native:gate` started 2026-11-27T08:21:45-05:00 as
`gpc-gate-issue-6583-suite-measurement-20261127T082145-0500.service`
with `--unit-io-weight 1` and `--container-memory 8g`,
so every container of it ran under a bound no step of the run met,
in the worktree of this round (`83826b055e`).
Each of the run's two containers was read every 100 ms through
`bin/native-gate-container-peaks.ts`,
the reader this round added to the package beside the gate,
which writes one JSON line per sample:
`memory.current`,
`memory.peak`,
`memory.max`,
the whole of `memory.stat` and `memory.events`,
the process figures,
and the heaviest processes of the container named by what they are
(a nextest process is named by the test it was started for).
Its samples are `suite-measurement-peaks.jsonl` in that round's scratch folder,
and what this record uses from them is stated here.
The run passed both checks:
`Summary [1011.311s] 12115 tests run: 12115 passed (7 slow), 51 skipped`
and `Clippy passed beside the test run.`,
with `gate-exit-status=0`.

- The suite's container peaked at 1,486.0 MiB of the 8,192 MiB its arm gave it
  (`memory.peak` 1,486.0 MiB,
  the highest `memory.current` sample 1,483.5 MiB),
  with `memory.events` `max` 0,
  `oom` 0 and `oom_kill` 0,
  and `pids.peak` 455.
  It ran 1,011.311 s.
- What that peak is made of,
  read from `memory.stat` at the highest sample:
  976.7 MiB of anonymous memory,
  469.3 MiB of file (the page cache of what the suite read),
  25.0 MiB of kernel memory and 21.7 MiB of slab,
  and 0 MiB of shared memory:
  the fixture root's tmpfs held nothing at the peak,
  so the fixture file system is not what holds it.
- Of the 976.7 MiB anonymous,
  892.2 MiB was one process:
  `service::service_main::tests::the_service_setup_and_its_words_are_used_only_by_the_service_mode`,
  which held 891.8 to 892.3 MiB of resident memory for 25.4 s,
  from 13:28:41.379 to 13:29:06.845 in the samples that named it.
  Beside it,
  `cargo-nextest` itself held 82.2 MiB,
  and the three `source_guard::` controls sharing the other test threads held 28.2 to 29.1 MiB each.
- The peak is therefore not nextest's concurrency:
  four test threads of ordinary controls hold about 170 MiB between them,
  and one control of this suite holds five times that on its own.
- Page cache has a peak of its own later in the run:
  930.3 MiB at 13:38:36.759,
  during the binary controls,
  when the container's charge was 1,097.1 MiB.
  Page cache is reclaimable,
  so it adds to `memory.peak` without being memory the run needs.
- The container's charge was above 1,000 MiB in 6.0 percent of its 9,822 samples,
  above 1,200 MiB in 4.1 percent,
  and its median was 596.5 MiB.
  The bound is sized from the peak,
  not the median:
  a bound that binds at the peak costs a whole gate.
- Clippy's container,
  in the same run and under the same 8 GiB,
  peaked at 5,219.4 MiB with `memory.events` `max` 0 and `oom_kill` 0,
  which agrees with the five gates of 2026-11-27 (4,790.0 to 5,278.7 MiB)
  and sits inside the bound they sized.

So the suite's container is bounded at 4 GiB
(`gateSuiteMemory` in `bin/native-gate-container-limits.ts`):
4,096 MiB,
2.76 times the highest peak any run of it read and 2,610 MiB above that peak.
The room is sized for the shape of this suite's peak rather than for its test threads:
two more controls holding what the largest one holds (892 MiB each) still fit,
at 3,270 MiB,
inside 4,096 MiB.
The bound is further above its measured peak than the build's 1.53 and Clippy's 1.55 are above theirs,
and that is the point:
a suite grows by controls,
and one control of this suite is most of the suite's peak.

What it costs and how a person lowers it:

- A gate at its bounds can now hold 12 GiB of the host at once
  (4 GiB for the suite,
  8 GiB for Clippy,
  both started together),
  where it held 10 GiB while the suite shared the repository default.
  The image build's 8 GiB runs before either container.
- A host that cannot spare 4 GiB for the suite lowers it with
  `--memory <size>` after `native:test:container`'s task name,
  or with `--container-memory <size>` on `native:gate`,
  which bounds the suite,
  the build and Clippy together.
  Below what the suite's largest control needs,
  that host gets the kill the next section describes rather than a slower
  suite.
- The bound is sized at the gate's own four test threads.
  A gate given more CPUs runs more tests at once and wants `--memory` raised with them;
  nothing here measured more than four.

### What the gate says when its suite's container is killed for memory

A container killed inside its bound reads like a failing test:
the kernel ends whichever process the bound picked,
the check reports a signal or a crash,
and nothing in that account names the bound.
Issues 6455 and 6562 each cost a gate that way.

So each container's own `memory.events` is read every 100 ms while it runs
(`bin/native-gate-container-memory.ts`),
because a `--rm` container's cgroup goes with it and a figure asked for after the container ended is a figure asked of a
directory that is gone.
When the client ends,
the gate prints one line of its own,
beside whatever the check said,
and only when the events
count a kill:

```text
The suite's container was killed for memory: its cgroup counted 1 out-of-memory kill at the 4.0 GiB memory.max bound it
was given, with a peak of 4,096.0 MiB. That is the bound and not a failing test: raise it with --memory after the task
name, or with --container-memory when native:gate starts the run.
```

The bound in that line is the container's own `memory.max` as the kernel states it,
so it is the bound the container ran under and not a number a reader has to trust the log for,
and the peak is the highest charge the same cgroup held.
Both containers get the line,
each named as its own,
and a run that met no kill prints nothing.
What the line cannot see is a kill the kernel made and the container's own teardown removed within one 100 ms interval;
that window is why the reader polls rather than reading once at the end.

### What is left for the suite's container

- The 892 MiB control is the largest single figure in the suite and the first thing to look at when a gate is short of
  memory.
  Nothing here explains why that control holds 892 MiB;
  a control that holds a fifth of its bound on its own is worth its own issue.
- One arm was measured and it has no control arm of its own.
  The peak is the kernel's exact high-water mark for that container,
  but a suite that adds a control between two gates reads higher without anything else changing,
  so a bound is a reading of one day rather than a ceiling.
- The reader names the processes of a sample once per second,
  so the sample at the very peak may not be the one that names what held it.
  Reading the processes again whenever a sample sets a new high-water mark is the change that would close that gap.
- The fixture root's tmpfs holds nothing at the peak,
  at 768 MiB of its own ceiling,
  which is worth re-reading only if a
  control that writes a large fixture is added.
