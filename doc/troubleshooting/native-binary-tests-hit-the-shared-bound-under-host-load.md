# Native binary tests hit the 5-second test hang guard under host load

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

The binary tests of `package/git-policy/cli` run every wrapped command under `timeout --signal=KILL 5`
(`TIME_BOUND_SECONDS` in `src/native/binary_support.rs`).
This is a hang guard of the test harness,
short so that a mutant that loops fails every affected control quickly.
It is not a requirement on the wrapper:
no wrapped command has to finish within 5 seconds.
The one latency requirement is that a commit made through the wrapper takes less than 5 seconds longer than the same
commit made with native Git (`package/git-policy/cli/SPEC.md`,
beside the forwarded-command latency requirement).
On a host whose load average was 50 or more,
three `discard::` controls failed with
`the command hit the 5-second bound` and passed alone in under a second.

Note 2026-11-09T16:55:17-05:00:
these citations in this record name paths that no longer exist.
`package/git-policy/cli/src/native/binary_support.rs` is now
`package/git-policy/cli/src/native/binary/binary_support.rs`,
moved 2026-11-03 in `3ec1e1201b`.
`package/git-policy/cli/src/native/binary_fixture_tests.rs` is now
`package/git-policy/cli/src/native/binary/binary_fixture_tests.rs`,
moved 2026-11-03 in `3ec1e1201b`.
`package/git-policy/cli/src/native/git_version_cache_tests.rs` is now
`package/git-policy/cli/src/native/invocation/git_version_cache_tests.rs`,
moved 2026-11-03 in `a0d60dc29a`.
`package/git-policy/cli/src/native/real_git_candidate.rs` is now
`package/git-policy/cli/src/native/invocation/real_git_candidate.rs`,
moved 2026-11-03 in `a0d60dc29a`.
`package/git-policy/cli/src/native/binary_resolution_tests.rs` is now
`package/git-policy/cli/src/native/binary/binary_resolution_tests.rs`,
moved 2026-11-03 in `3ec1e1201b`.
`package/git-policy/cli/src/native/binary_supervisor.rs` is now
`package/git-policy/cli/src/native/binary/binary_supervisor.rs`,
moved 2026-11-03 in `3ec1e1201b`.

## What was measured

Debug build,
Git 2.56.0,
fixtures on the `build.build-dir` volume (btrfs on dm-crypt).
Per-command wall times came from 25 reruns of each single test;
process starts from `strace --follow-forks`.

- A wrapped explicit-path `commit` starts 40 Git processes and makes 36 sync calls (the exact budget of
  `src/native/binary_budget_tests.rs`),
  uses 82 to 88 ms of CPU and took 0.15 to 0.45 s of wall time,
  against 4 ms for
  direct Git.
  Capped to a tenth of a core it took 0.84 s,
  so CPU contention alone needs a share under 2 percent to reach
  5 s;
  the rest of a slow commit is waiting for sync calls.
  A probe of write,
  fsync and directory fsync on that volume
  took a median of 12.5 ms,
  p99 60 ms and a maximum of 139 ms while one suite ran,
  and 12.6 ms median,
  16 ms maximum
  when idle.
- Every other wrapped command of the three tests took at most 0.12 s.
  The wrapper adds one `rev-parse` (and,
  for
  `stash`,
  one `version`) to a forwarded command.
- `git restore --help` makes Git run `man`,
  which starts `preconv`,
  `tbl`,
  `nroff`,
  `col`,
  `groff`,
  `troff` and `grotty`:
  11 process starts for one command.
  It took 18 ms median,
  0.7 s on a cold start and 3.2 s once in 12 runs while the load
  average was 57.
- Not reproduced:
  a 5 s stall of an ordinary command.
  Three whole-suite runs under 100,
  200 and 800 percent CPU
  caps,
  with a probe beside one,
  never showed a non-commit wrapped command of the `discard::`,
  `dependent_version::`
  and `controls::` tests above 0.07 s,
  so the bound stays at 5 s for single Git commands (about 70 times that).
- `dependent_version` and `controls` tests:
  at most 0.07 s and 0.05 s per command (173 process starts for the 50
  commands of `every_control_before_the_subcommand_is_removed`,
  3 or 4 per command).
  No mechanism was found for them
  at that time;
  the section "The two task 66 tests:
  the cold first start explains them,
  and they no longer stall"
  later in this document found it.

## Cause and fix

The bound was applied to two things it does not fit:
a commit transaction,
which the repository already bounds with
`COMMIT_BOUND_SECONDS` (120 s) in `binary_commit_tests.rs`,
and a manual-page render that is Git's work and not the
wrapper's.
`observe_raw` in `binary_support.rs` now panics,
on every host and without reading a clock,
when a command
under the 5 s bound is a commit the wrapper lands through its transaction or a `--help` with no `man` stub in the
fixture's `bin` directory.
The discard controls use `run_commit` and `stub_manual_viewer`.

## A forwarding test that hit the bound with no commit and no manual page

`forwarding::exit_codes_and_stderr_match_native_git` failed once in a mutation baseline (`native:mutation:scoped`,
`--file src/native/command_push.rs`;
log `~/temp/agent/mutpar-w4-command_push.log.2`,
run of 2568 tests in the
2 CPU,
2 GiB container on a loaded host whose load average that log does not record)
with `the command hit the 5-second bound` at `binary_support.rs:233` after 5.628 s.
The same log shows the two sibling tests of that module started just before it taking 1.10 s
(`argument_bytes_reach_git_unchanged`) and 2.25 s (`environment_overlay_reaches_git`),
against 0.05 s each on the host
when alone:
the whole container was 20 to 50 times slower at that moment,
not one command.

### What was measured

- The test wraps 12 commands (two `rev-parse`,
  two `diff`,
  `show`,
  `log`,
  three usage errors,
  `status --porcelain`,
  `branch --list`) and runs each once more through direct Git.
  None is a commit and none renders a manual page,
  so the
  guard in `shared_bound_misuse` does not apply to it.
- `strace --follow-forks` of each wrapped command:
  at most one Git process started by the wrapper (the `rev-parse` it
  adds) beside the process it becomes,
  and no `fsync`,
  `fdatasync`,
  `flock`,
  `nanosleep` or `clock_nanosleep` call.
  The
  usage errors start no `rev-parse` at all.
  Unlike a commit,
  there is no sync wait to be slowed by a busy volume.
- Wall time per wrapped command through `timeout --signal=KILL 5`,
  300 runs of each of the 12 commands (3600 starts)
  in one fixture while the load average was 28 to 39:
  minimum 2.7 ms,
  median 3.7 to 5.9 ms,
  99th percentile at most
  9.3 ms,
  and a maximum of 63 ms (one `rev-parse --verify`),
  13 ms for the next largest.
  The direct runs took 0.6 to 1.3 ms.
- The same bound was hit in other baselines by unrelated tests
  (`discard::every_named_alternative_runs_in_the_main_worktree` and
  `discard::the_hatch_after_the_subcommand_runs_each_form` in `mutpar-w5-markdown_linter.log.3`;
  `worktree_copy::an_interrupted_main_checkout_copy_is_recovered_by_the_next_creation` in
  `mutpar-w3-doctor.log.2`),
  so no command is common to the stalls.

### What was not found

No mechanism that makes one of the 12 commands slow was found,
and none is claimed.
A command that normally takes 4 ms needs a stall of more than 1000 times that to reach 5 s;
the measurements above show
no work in these commands that can wait on its own (no sync call,
sleep or lock in the traced calls;
the test sends
empty,
closed standard input),
so the stall most likely came from the container or host under the command
(CPU quota,
memory pressure or disk reads of an evicted executable are candidates,
none verified).
The test is left on the shared bound:
moving it to the commit bound would hide a real forwarding loop,
which is what
the short bound is for,
and making the wrapper faster has nothing to act on,
since the wrapper adds one `rev-parse`
and none of the traced waiting calls.

### What changed

The failure message of `observe_raw` now names the stalled command (its arguments and working directory,
built only
when the bound was hit,
so a passing run costs nothing).
The next occurrence then says which command stalled;
the unnamed `assertion left != right failed` message above is why
this one cannot be attributed.
If a later occurrence names a command with no sync call or sleep again,
the next measurement to take is the process
start and file read times inside the container during the stall,
not the command.

## Mechanism found: the first start of a fixture's cloned wrapper copy reads the executable from disk

The section "What was not found" is superseded for the forwarding stall:
the mechanism was measured afterwards.
Each binary test made a private copy of the wrapper with `cp` (`copy_executable` in
`package/git-policy/cli/src/native/binary_support.rs`).
The storage of this host is btrfs (two LUKS devices,
`dm-0` and `dm-1`,
under `/var/home`),
so the default
`cp --reflink=auto` makes a clone that shares the original's extents.
A clone has no pages in the page cache,
so the first command of a test that starts the copy faults the executable in
from disk,
inside the 5-second bound.
The 3600-start measurement in "What was measured" used one fixture,
so it paid that once in 3600 starts
and showed it only as its 63 ms maximum.

### Method

The measurements ran in `podman run` containers from the gate's own test image,
with the container's own limits
(`--memory`,
`--cpus=2`,
and for disks `--device-read-bps`,
`--device-write-bps`,
`--device-write-iops`),
and no stress
tool:
two workers built a fixture like `exit_codes_and_stderr_match_native_git` (copy of the debug wrapper,
a repository
with two commits),
ran its 12 commands wrapped (under `timeout --signal=KILL 5`,
as the test does) and direct,
and
removed it,
40 times each.
Per command:
wall time,
`ru_majflt`,
`ru_minflt` and `ru_inblock` from `wait4`.
The scripts are scratch (`~/temp/agent/reclaim/`),
not committed.
The debug wrapper is 21.9 MB (`CARGO_PROFILE_DEV_DEBUG=0`),
the release wrapper 9.9 MB.

### Memory caps do not change it

Wrapped commands,
960 samples per cap,
host load as it was (other mutation containers running),
default `cp`
(wall milliseconds,
median,
99th percentile,
maximum):

- 2g:
  4.26,
  1739, 5153.
- 1g:
  4.34,
  124, 1058.
- 512m:
  3.93,
  1973, 3719.
- 256m:
  4.20,
  204, 764.
- 128m:
  4.06,
  559, 2683.
- 96m:
  3.99,
  1216, 1910.

Direct Git in the same runs:
median 1.0 to 1.1 ms,
maximum at most 9.5 ms,
1 to 8 major faults in 960 starts.
A 5.15 s sample (killed by the bound,
with 0.55 ms of CPU) occurred at the 2g cap with no disk cap,
where the
container's own page cache is nowhere near full,
so the container reclaim is not the mechanism;
the cap neither
causes nor removes the stall.
The slow samples are the first wrapped command of a fixture:
in the 2g and 512m runs,
106 of 160 first commands took
over 100 ms,
against 1 or 0 of 160 for every other command index.
The first command took a median of 287 ms (maximum 5153 ms) with 4 major faults and 33368 blocks of 512 bytes read
(16.29 MiB);
the same command in a started fixture took 3 to 4 ms.
Its CPU time was 9 ms,
so the rest was waiting for the read.

### A slow disk turns it into a failure

With the read bandwidth of both btrfs devices capped at 5 MB/s (`--device-read-bps` on `/dev/dm-0` and `/dev/dm-1`;
capping `dm-1` alone left the release build's data,
which sat on `dm-0`,
unthrottled and misled one run),
first wrapped
command of each of 12 fixtures:

- Debug wrapper,
  default `cp` (clone):
  5000.6 to 6211.7 ms,
  every one killed by the bound.
- Debug wrapper,
  `cp --reflink=never`:
  3.5 to 4.2 ms.
- Release wrapper,
  clone:
  2522 to 4807 ms,
  median 3713 ms,
  under the bound only because 5 MB/s reads 9.45 MiB in time.
- Release wrapper,
  `cp --reflink=never`:
  3.0 to 4.0 ms.

The other 11 commands of the test took at most 5.5 ms (debug,
data copy) in the same runs.
Three tests run with `cargo nextest` in the container at the same 5 MB/s cap (three runs each):

- Before:
  `forwarding::exit_codes_and_stderr_match_native_git` and
  `management::namespace_word_elsewhere_is_an_ordinary_argument` failed in all three runs with
  `the command hit the 5-second bound` after 5.5 to 6.9 s;
  `markdown::a_failing_linter_stops_the_command` passed after 3.3 to 4.0 s.
- After:
  all three passed in every run,
  0.055 to 0.097 s each.

### An evicted executable behaves the same

Evicting a plain copy's pages (`fsync`,
then `posix_fadvise` with `DONTNEED` after a short wait) and starting it again
read the same amount as a clone,
so eviction under host memory pressure costs what a clone costs:
the kernel reads most of the file in the first fault,
not only the pages the start touches.
Debug:
4 major faults,
16.29 MiB,
median wall time 46 ms;
release:
1 major fault,
9.45 MiB,
26 ms (a disk that was not
capped).
A 5 MB/s disk therefore needs 3.3 s for the debug wrapper and 1.9 s for the release wrapper,
and the cost scales with
the file size and the disk,
not with anything the wrapper does at start.

### What changed first

- `copy_executable` passes `--reflink=never`:
  the copy is written through the page cache and stays cached,
  and a slow
  read of the original happens in `cp`,
  outside the bound.
  Guard:
  `a_byte_copy_of_the_wrapper_shares_no_storage_with_the_built_executable` in
  `package/git-policy/cli/src/native/binary_fixture_tests.rs` asks the filesystem for the copy's extents
  (`FS_IOC_FIEMAP`) and requires none to be shared.
  With the argument removed it failed (`shares 1 extents with .../target/debug/cli-git-native`) and with it it passes;
  a filesystem that cannot report extents skips the check.
  The bound is unchanged at 5 s.
- The release profile strips symbols (the symbol tables sit after the loaded segments and were read ahead anyway).
  First-start read in MiB (release wrapper,
  evicted pages):
  9.45 default,
  8.09 stripped,
  7.43 stripped plus fat
  link-time optimization plus one code generation unit (7.79 MB file,
  from 9.91 MB),
  6.50 with `opt-level = "s"` added.
  Only stripping is adopted:
  the other two cost build time (46 s to 68 s at 8 jobs,
  50 s to 92 s at 4 jobs for the
  first) or have unmeasured scanner throughput (`opt-level = "s"`).
  A 300-file,
  7 MB commit through the wrapper (8 alternating runs per build) took a median of about 2.95 s for the
  default,
  2.95 s stripped and 2.79 s for the `opt-level = "s"` build,
  so none of those was slower.
  Warm start medians stayed at 3.4 to 3.6 ms and the wrapped explicit-path commit at 86 to 95 ms (CPU 69 to 73 ms)
  for all four builds.
  Wall time of the cold first start was dominated by host disk load and varied from 20 ms to 147 ms between builds
  with nearly equal reads,
  so only the byte counts,
  which are deterministic,
  are claimed.
  The profile adds no syscall.
  The stripped build loses function names in panic backtraces.

`package/git-policy/cli/src/native/binary_fixture_tests.rs` moved on 2026-11-03 to
`package/git-policy/cli/src/native/binary/binary_fixture_tests.rs` in `3ec1e1201b`
(refactor(git-policy-cli):
move the binary-level controls into a binary folder).
`package/git-policy/cli/src/native/git_version_cache_tests.rs` moved on 2026-11-03 to
`package/git-policy/cli/src/native/invocation/git_version_cache_tests.rs` in `a0d60dc29a`
(refactor(git-policy-cli):
move the invocation and forwarding modules into an invocation folder).

### Disk write and fsync

Write caps (`--device-write-bps` 5 MB/s and `--device-write-iops` 50 on both devices) slowed only the bulk write of the
22 MB fixture copy (one setup of 1.3 s in 12) and did not slow any fsync-bearing command:
the direct commit took
4.8 ms and the wrapped explicit-path commit a median of 92 ms (maximum 182 ms),
as uncapped (95 ms).
Dirty-page write-back and btrfs fsync are done by kernel workers that the per-cgroup write limit did not reach (an
unverified reading),
so slow fsync is not modelled by this method and the sensitivity of commands to it is unverified.
The traces in "What was measured" show no sync call in the 12 forwarding commands;
a wrapped commit makes 36.
The commit tests run under `COMMIT_BOUND_SECONDS` (120 s),
so a slow fsync does not reach the 5-second bound.

### Evidence about the bound

Under a 5 MB/s read cap the stall of the unfixed test was 5.5 to 6.9 s,
and it grows with executable size over disk
speed (a 19 MiB read at 1 MB/s took 21 s),
so a bound of 10 s would have hidden this case at 5 MB/s and not at 1 MB/s.
With the data copy no wrapped command of the three tests exceeded 0.1 s at 5 MB/s,
so these measurements support no
raise of the bound.
Not measured:
reads of Git's own executable and libraries when they are evicted (the wrapped and direct runs both pay
it),
and the unit tests in `package/git-policy/cli/src/native/git_version_cache_tests.rs`,
which copy Git with
`std::fs::copy` (a clone on btrfs too) but run under no 5-second bound.

## Why a fixture needs no copy of the wrapper, and what replaced it

Each fixture used to copy the 21.9 MB debug wrapper (`copy_executable`),
so a suite of about 240 fixtures wrote about
5 GB,
and every test started a new inode whose pages were not cached.
Nothing in the wrapper needs a private file:
`current_exe` is used only to recognise itself by file identity (`same_file` in
`package/git-policy/cli/src/native/real_git_candidate.rs`,
used by `entry.rs` and the resolver),
hook entries are
recognised by the name the wrapper was started under (`main.rs`),
and no control writes to the wrapper path.
The only controls that need a second file with the wrapper's bytes are the byte-copy forms in
`package/git-policy/cli/src/native/binary_resolution_tests.rs`,
which still call `copy_executable`.

`fixture` now hard-links the built executable (`link_built_wrapper` in
`package/git-policy/cli/src/native/binary_support.rs`) and falls back to a data copy where a hard link is impossible
(a fixture root moved to another filesystem with `CLI_GIT_TEST_ROOT`).
That fallback was superseded on 2026-11-01:
it also applied to every fixture on a host that relocates
`build.build-dir`,
and the link failure now leads to one stored copy of the build instead of a copy per fixture (section
"The hard link failed on the development host,
so every fixture copied the wrapper").
All fixtures of all test processes,
which `cargo nextest` runs one per process,
then share one inode and one set of
cached pages,
so a cold read of the executable is paid once per machine state and not once per test.
A per-process copy (a `OnceLock`) would not have shared anything,
because each test is its own process.
A symbolic link as the fixture's wrapper would break `std::fs::hard_link` in the resolution controls,
which link
the wrapper's path and would link the link itself;
`/proc/self/exe` resolves a symbolic link to its target and a hard
link to the path used to start it,
and both were exercised by the full-suite runs in the measurements of this section.

One read per process,
after the link,
warms the file (`warm_wrapper`):
the wait for a slow disk then happens
there,
outside the bound.
The read has to come after the link:
on overlay storage (the test containers) a hard link of an image-layer file copies
it up to a new inode,
and a read of the original before the link warmed the wrong inode.
The first attempt read before the link,
and at a 2 MB/s read cap the first wrapped command of the test was killed at
5001 ms with no major fault,
which a probe isolated to the copy-up.

Measurements (debug wrapper 21.9 MB,
containers from the gate image,
2 GiB,
2 CPUs):

- Placing the wrapper in a fixture,
  100 repetitions:
  `cp --reflink=never` median 4.6 ms (maximum 256 ms),
  `cp` clone 1.2 ms,
  hard link 0.03 ms (maximum 0.16 ms);
  a warm read of the cached file 1.2 ms per process.
- Bytes written for the wrapper over the suite:
  about 5 GB (240 fixtures of 21.9 MB) before,
  none after.
  The cgroup `io.stat` underreports this (132 MB for the copy design against 0.8 MB for the link design) because
  write-back runs outside the container's attribution,
  so the estimate comes from the fixture count.
  The link design read the executable once (21.86 MB `rbytes`,
  the cold copy-up).
- Full `native_binary` suite:
  278 passed with the copy design in 57.7 s and 279 passed (one test added) with the link
  design in 54.1 s and 61.5 s in two runs;
  the host was loaded,
  so no difference in wall time is claimed.
- Evicted wrapper (`posix_fadvise` DONTNEED before the run),
  three tests of the brief,
  5 MB/s read cap,
  two runs each:
  the link design with the warm read passed (4.2 s for the two first-command tests,
  all of it in the warm read);
  without the warm read it passed in 3.2 s,
  the 16 MiB first start being 3.3 s at 5 MB/s,
  only just under the bound.
- Same state at a 2 MB/s read cap:
  the link design with the warm read passed both runs (10.8 and 11.1 s,
  in the warm
  read);
  the data copy passed both runs (10.5 and 11.4 s,
  in `cp` reading the original);
  the link design without the
  warm read failed both runs with `the command hit the 5-second bound` at 5.02 s.
  So the link design is not more robust than the data copy against a slow disk (both move the read outside the bound);
  it removes the writes,
  the per-fixture setup cost and the cold start of every new inode.
- Guards,
  each shown failing once:
  `a_fixture_wrapper_is_the_built_executable_itself` (same device and inode as the
  built executable,
  skipped where a hard link of the build cannot be made in the fixture) failed with the copy restored
  (`not the same file as .../target/debug/cli-git-native`);
  the byte-copy guard failed with `--reflink=never` removed.

## The two task 66 tests: the cold first start explains them, and they no longer stall

Two baselines of the mutation campaign of 2026-10-29 failed with `the command hit the 5-second bound` in
`dependent_version::literal_pathspecs_check_a_named_manifest_like_plain_mode` and
`controls::a_path_named_like_a_hatch_is_forwarded_and_a_hatch_is_removed` while the host load average was about 28.
The failure message did not yet name the command,
and the commands measure 3 to 40 ms when run alone.
Question:
do they still stall now that fixtures hard-link the wrapper?

### Method

Two gate images built by `native:test:container` with a stand-in `podman` that skipped the final `run` (scratch,
`~/temp/agent/t66/`),
one per revision:
`c3dddf91ac` (the parent of `e76a99bddc`,
where each fixture clones the wrapper
with `cp`) and `1091639e0b` (current main,
with the hard link and `warm_wrapper`).
Both images carried one temporary line in `observe_raw` that printed the wall time and arguments of every bounded
command (not committed).
Each run was one container with `--memory=2g --cpus=2 --pids-limit=128` and `--device-read-bps` on both `/dev/dm-0`
and `/dev/dm-1` (the container storage is `/var/home`,
on `dm-1`),
running `cargo nextest run --test native_binary`
for the two tests,
or for the whole `dependent_version::` and `controls::` modules.
Before the test,
some runs evicted pages with `dd iflag=nocache count=0`:
`wrapper` evicts the built executable,
`all` also evicts `/usr/bin/git`,
`/usr/bin/timeout`,
`/usr/bin/sh`,
`/usr/bin/env` and the shared libraries.
No stress tool and no extra load;
the host load average was 5.8 when the runs started.

### What each test runs under the bound first

Both tests build their repository with `git` calls that go straight to real Git (`direct`,
no `timeout`),
so Git and its
libraries are read before any bounded command.
The first bounded command of each test is therefore also the first start of the wrapper's file:
`cli-git check -- package/module/a/package.json` in the `dependent_version` test and
`reset --no-enforce-worktree --quiet -- --no-enforce-worktree` in the `controls` test.

### Results at the revision before the hard link (positive control)

Per-test outcomes,
two tests per run,
no eviction:
- 5 MB/s read cap,
  3 runs:
  all 6 test executions killed at 5000.7 to 5000.8 ms;
  the killed command was in every case the
  first bounded command listed above (named by `observe_raw`),
  nextest times 5.7 to 6.5 s.
- 2 MB/s read cap,
  3 runs:
  all 6 killed the same way.
- No cap,
  3 runs:
  all passed;
  the first bounded command of the 6 fixtures took 153 to 421 ms (median 353 ms),
  against
  3.4 to 25.8 ms for every later command.
  This is the cold first start of a clone,
  and its size on an uncapped disk is
  why a loaded host reaches 5 s while a quiet one never shows it.

### Results at current main

Test executions that hit the bound:
0 of 66 in the two-test runs and 0 of 30 in the module runs.
- 5 MB/s:
  5 runs without eviction,
  5 with `wrapper`,
  5 with `all`;
  nextest wall time 4.0 to 4.2 s (`none`,
  `wrapper`)
  and 5.3 to 5.5 s (`all`),
  all of it before the first bounded command (the copy-up and read of the 22 MB wrapper,
  then
  Git's pages in the unbounded setup).
- 2 MB/s:
  5 runs with `wrapper` (10.3 to 10.5 s) and 5 with `all` (13.3 to 13.8 s);
  then 3 runs of the two whole
  modules with `all` (10 tests each,
  13.7 to 13.9 s).
- 1 MB/s with `all`:
  5 runs,
  26.7 to 28.8 s.
- No cap:
  3 runs,
  0.21 to 0.27 s.
- First bounded command of each fixture,
  current main (milliseconds):
  no cap,
  median 12.6,
  maximum 13.5;
  5 MB/s,
  median
  13.1,
  maximum 26.5;
  2 MB/s with `wrapper`,
  maximum 69;
  2 MB/s with `all`,
  maximum 268;
  1 MB/s with `all`,
  median 144,
  maximum 281;
  module runs at 2 MB/s with `all`,
  maximum 129.
  The worst is 17 times under the bound.
- Later commands of the two tests took at most 14.5 ms in every run.
  In the module runs (214 commands) one later command
  took 101.8 ms (a `status` under `--no-enforce-final-newline`) and the others at most 16.8 ms.

### Reading

- The two tests are the first bounded start of the wrapper's file in each of their processes,
  and both stall at the old
  revision under a read cap at exactly that command,
  for exactly the reason in "Mechanism found":
  the clone's pages are
  not cached.
  That the two 2026-10-29 failures were this mechanism is consistent with the evidence,
  not proven:
  no
  host trace of those two runs exists,
  and the cap models slow storage,
  not the load average of 28.
- The hard link plus `warm_wrapper` removes it:
  the wait for a slow disk moves into the fixture setup,
  which has no
  bound,
  and the tests passed at caps down to 1 MB/s with the wrapper evicted.
- A residual cold read remains at the first bounded command (up to about 280 ms at 1 MB/s,
  which is roughly 280 KB of
  pages that the earlier `git init` and `commit` did not touch,
  probably in Git's own binary or libraries).
  It was not
  attributed to a file,
  because it is 17 times below the bound at 1 MB/s.
  A stall of 5 s from such reads would need
  about 5 MB of cold pages at 1 MB/s.
- Not covered:
  fsync waits (the cap model does not slow them,
  see "Disk write and fsync"),
  memory pressure that evicts
  pages after the warm read,
  and the real load of 28.
  The tests have no sync call,
  and a stall of the kind that remains
  possible would name its command in the `observe_raw` failure message.
- The bound stays at 5 s.
  Nothing here supports changing it.

## The hard link failed on the development host, so every fixture copied the wrapper

The section "Why a fixture needs no copy of the wrapper" says the hard link applies everywhere except a root moved with
`CLI_GIT_TEST_ROOT`.
That was false on the development host:
`~/.cargo/config.toml` relocates `build.build-dir`,
so `CARGO_TARGET_TMPDIR`
(where fixtures live) is on `/var/mnt/encrypted` (device 81 here) while the final `target/debug/cli-git-native`
is on `/var/home` (device 56).
`std::fs::hard_link` failed with `Invalid cross-device link` for every fixture,
and `link_built_wrapper` fell back to
`copy_executable`:
a 21.9 MB data copy per fixture,
the design the section above meant to remove.
The guard `a_fixture_wrapper_is_the_built_executable_itself` skipped itself exactly there ("cannot hold a hard link of
the build;
nothing to check"),
so nothing reported it.

### What was found

- `~/temp/agent/bin4.log` (a host run of `native:test:binary:host`):
  five `builtin::` tests failed with
  `the command hit the 5-second bound` after 6.2 to 6.6 s,
  on `add .`,
  `stash list`,
  `reset --hard`,
  `switch` and
  `checkout --force`,
  all among the first 17 tests of the run,
  and a `commit::` test that passed took 6.2 s in the same
  wave.
  The failure logs of `bin2.log`,
  `bin4.log` and `bin6.log` print the
  `fixture wrapper: no hard link ... (Invalid cross-device link ...)` line (7,
  6 and 2 times).
  The failing commands differ;
  the first wave of parallel tests and the cross-device copy are common.
- With the copy,
  a wave of 169 tests wrote about 3.7 GB (169 times 21.9 MB) in 6 to 7 s.
  Host-wide `Dirty` in `/proc/meminfo` reached 1.3 to 1.5 GB during such a wave,
  and an `fsync` probe on the fixture
  volume took 615 ms at the worst (median 4 ms);
  `/proc/meminfo` is host-wide,
  so these numbers do not attribute the
  dirty pages to the suite.
- Building two fixtures took 2.07 s with the copy and 0.14 s with the link (the guard test with the fix disabled and
  enabled).

### What is not proven

The stall itself,
a command waiting 5 s behind queued write-back,
was not reproduced before and after the fix.
The host was at a load average of 20 to 25,
where 142 to 172 tests passed in 4 to 8 s in both states.
That a loaded volume delays the `fsync` calls of other tests' commands behind the queued copies is an inference from
the symptoms above and from the earlier measurements of this document,
not a measured cause.
What is certain:
the copy was never intended,
it was about 3.7 GB of avoidable writes per wave,
and the failing runs
carried the cross-device message.
A reproduction would cap the write bandwidth of the fixture volume in a container (`--device-write-bps`) at the
revision before and after the fix;
it was not run.

### The fix

`link_built_wrapper` links the built executable;
when that link fails with `EXDEV` (the link's own failure,
not a
comparison of device numbers,
which bind mounts defeat) it links one stored copy of the build on the fixture's
filesystem (`link_stored` in `binary_wrapper_store.rs`),
and it copies per fixture only where neither link works.
The stored copy is named by a hash and the length of the build's bytes (`content_name`),
read once per process,
and a
copy is read back and must have that name before it is published.
Length and modification time were used first and rejected after the cross-vendor review:
equal-length builds with one
timestamp share them.
The hash is SipHash from the standard library,
which protects against accidental equality only.

A caller that finds no stored copy writes its own partial file,
created exclusively (`create_new`,
see
"2026-11-01T09:50:10-05:00:
cross-vendor review of the wrapper store and its controls"),
in `<fixture root>/wrapper-store/` and links it to the final name with `std::fs::hard_link`,
which fails with `AlreadyExists` if another caller published first;
it then removes its partial and uses the winner's file.
There is no lock:
the final name is only ever a complete file,
a reader only opens the final name,
and the first
publisher wins.
Callers that start together each write a partial copy once per build,
so the first wave after a rebuild writes up to
one copy per concurrent test process (derived,
not measured:
with one process per CPU on a 28-CPU host,
up to about
600 MB,
against about 3.7 GB before and about 22 MB with a lock);
every later caller finds the final file.

Pruning is by idleness:
a copy of another build,
or a partial file,
is removed only when its status-change time is
more than a day old.
A fixture's link changes that time,
so a copy a run is linking is never idle,
and the pruning cannot remove a copy
between a caller's finding it and linking it except for a copy a day idle that a second run starts using in that
instant.
That remaining case is handled too:
`link_stored` asks for the stored copy again when the link finds it gone (three
attempts),
so the caller still ends with a link to the stored file and not a per-fixture copy.
Placing the fixtures beside the executable instead was tried first and rejected:
`target/` is inside the repository,
so a fixture meant to be outside any repository (the `require-root` and resolution controls) ran inside the
worktree's own.

Guards (`binary_wrapper_store_tests.rs`),
shown failing together with six changes made at once,
each failure naming
the control it should (one build,
six controls failed,
six passed):
- `a_late_publisher_leaves_the_winners_file_in_place`:
  `rename` in place of the link.
- `the_name_of_a_build_is_its_content`:
  a name that ignores the bytes.
- `a_copy_that_does_not_have_the_builds_name_is_not_published`:
  the check of the copy against its name removed.
- `a_stored_copy_that_vanishes_before_the_link_is_made_again`:
  a single link attempt.
- `concurrent_callers_end_with_links_to_one_final_file_of_the_build`:
  pruning every copy at once (the control keeps a
  recent copy of another build and checks that no partial file or second copy stays).
- `fixtures_are_the_built_executable_or_its_one_stored_copy` and
  `fixtures_on_another_filesystem_share_the_stored_copy_of_the_build` (named
  `fixtures_share_one_wrapper_file_with_the_built_executables_bytes` in the first version):
  the `EXDEV` branch
  disabled,
  so each fixture copies.
  The second covers a root in `/dev/shm`,
  which fails the link of the build with
  `EXDEV` where that is another filesystem,
  and reports when it is not.
Not covered:
  a copy of another build removed by a different process between its being found and linked is shown for
one removal at one moment (the control removes the stored file in `link_stored`'s hook),
  not under a real second run.
An earlier version used an advisory lock;
  the owner's stance against locks replaced it with the lock-free publish.

### 2026-11-01T09:50:10-05:00: cross-vendor review of the wrapper store and its controls

A later cross-vendor review of the store and its controls (OpenAI model,
output kept at
`/var/home/user/temp/agent/ct/pi-review2.out` on the development host) made eight claims;
each was checked against the
code,
and all eight held.
What changed,
finding by finding:
- Partial names (`.partial-<pid>-<counter>`) repeat across process-identifier reuse and PID namespaces that share the
  store,
  and a caller that died after linking its partial to the final name leaves the published file under a second
  name;
  `cp` into that name truncates the published executable.
  `reserve_partial` now creates the name with
  `create_new` and moves to the next counter value when it exists (`RESERVE_ATTEMPTS` tries),
  then `cp` writes into the
  empty file and the build's mode is applied afterwards (`cp` into an existing file keeps that file's mode).
- Pruning decided on a status-change time read earlier than the unlink,
  so a link made in between was not noticed.
  Only
  a copy whose link count is one (no fixture holds it) is a candidate now,
  and the name is renamed to a reserved
  partial name and its link count read again before it is deleted;
  a count that grew means a fixture linked in the
  window,
  and the name is linked back.
  A caller that asks while the name is away publishes another copy,
  so the
  remaining window costs one extra copy and never a fixture without its wrapper.
  A lock-free removal cannot make
  the check and the unlink one step,
  so that residual case is accepted and stated.
- `fixtures_share_one_wrapper_file_with_the_built_executables_bytes` required shared inodes where the factory allows
  private copies,
  and compared two fixtures with each other,
  so an implementation that always used the stored copy
  passed.
  `fixtures_are_the_built_executable_or_its_one_stored_copy` probes what the root's filesystem links and then
  requires the built executable itself,
  the one stored copy,
  or (only for a non-`EXDEV` failure) copies with the built
  bytes.
  On this development host the default fixture root (Cargo's `build-dir`,
  relocated to another volume) is on a
  different filesystem than the final artifacts,
  so the stored-copy branch is the one the default run exercises;
  the
  built-executable branch runs with `CLI_GIT_TEST_ROOT` on the artifacts' filesystem.
- The relocated-root control assumed `/dev/shm`,
  removed its directory only on success,
  and ignored capacity.
  It
  now picks the first of `/dev/shm` and `$XDG_RUNTIME_DIR` that is a writable directory,
  fails the probe link with
  `EXDEV` and has room for two copies,
  and otherwise prints why it skipped;
  a `Scratch` guard removes the directory
  when the control panics.
- `a_slow_commit_tree_prints_a_wait_notice_that_the_comparison_leaves_out` slept past the grace;
  the watcher thread can
  resume after the signer has ended (`watch_loop` in `child_watch.rs` finds no wait marked),
  so the notice was lost
  under load.
  The signer now waits for a release file that the control creates when it reads the notice.
  The 5-second
  bound of the shared binary tests is unchanged.
- `build_name` hashed the executable before `OnceLock::get_or_init`,
  so callers that arrived together each read the
  22 MB file.
  `name_once` runs the read inside `get_or_init`.
  The outcome of a failed read is kept for the process (the
  callers then fall back to private copies),
  because `OnceLock` cannot keep a failed initialisation open on stable
  Rust;
  a read that failed once rarely succeeds later in a test run.
- The concurrency control used one worker per CPU and the real executable,
  with nothing forcing overlap.
  It now runs a
  named number of callers on a 64 KiB stand-in build,
  and `shared_copy_named` gives each caller a gate after its
  checked partial copy and before the link,
  so every caller publishes against all the others.
- The pruning control never aged an entry or refreshed one through a link.
  `remove_idle_builds_at` takes the clock as
  an argument,
  so aging is tested at the exact boundary without sleeping;
  a copy with an ancient modification time
  and a fresh status-change time must stay,
  which an implementation that reads the modification time fails.

Each new control was shown failing against a deliberate change (the production code was restored afterwards):
- `a_partial_name_that_exists_is_never_reused` and `a_copy_never_writes_through_a_leftover_partial_name`:
  `create(true)`
  in place of `create_new(true)`.
- `callers_that_ask_together_for_the_name_of_a_build_read_it_once`:
  the read moved outside `get_or_init`.
- `a_copy_with_an_old_modification_time_and_a_recent_link_stays`:
  `mtime()` in place of `ctime()` (the previous pruning
  control still passed).
- `a_copy_leaves_the_store_exactly_when_it_has_been_idle_for_the_idle_age`:
  `<=` in place of `<` in the age test.
- `a_link_made_during_the_removal_keeps_the_copy`:
  a plain removal in place of the set-aside and recheck.
- `concurrent_callers_end_with_links_to_one_final_file_of_the_build`:
  copies serialized behind a mutex,
  which keeps the
  callers from meeting at the gate.
  Replacing the link by a `rename` is not caught by this control in every schedule
  (it passed once),
  and stays the job of `a_late_publisher_leaves_the_winners_file_in_place`,
  which orders the two
  publishers.
- `fixtures_are_the_built_executable_or_its_one_stored_copy`:
  a factory that always uses the stored copy,
  with
  `CLI_GIT_TEST_ROOT` on the artifacts' filesystem.
- `a_scratch_directory_goes_with_its_control_and_a_missing_filesystem_is_a_reason`:
  an empty `Drop`.
- `a_slow_commit_tree_prints_a_wait_notice_that_the_comparison_leaves_out`:
  a 3-second sleep at the start of the
  watcher thread,
  which simulates a watcher resuming late.
  The sleeping version of the control failed (no notice) and
  the released version passed.

## A wait notice in a diagnostics comparison

A `merge_isolation` control that compares the wrapper's diagnostics with native Git's failed once on a
`cli-git: still waiting for git commit-tree` line.
That line is correct behaviour:
a step that outlasts its notice grace (`PLUMBING_NOTICE_GRACE` 5 s and
`OBJECT_NOTICE_GRACE` 10 s in `child_class.rs`;
`commit-tree` is a bulk step) prints it on standard error however
healthy the step is,
and a loaded host stretches any step.
The failing run was reported on 2026-11-01 around 02:14 and the class graces landed in `ac97d3b4f3` at 01:58 that day,
so the failing branch may still have had the one-second grace;
this was not verified,
and the run was not kept.
The controls now compare the diagnostics without wait-notice lines (`without_wait_notices` in `binary_support.rs`,
used by `run` in `binary_merge_isolation_tests.rs`),
and
`a_slow_commit_tree_prints_a_wait_notice_that_the_comparison_leaves_out` holds a signer stand-in until the control has
read the notice from the wrapper's standard error,
and checks that it is printed and removed.
Its first version only slept past the grace,
which did not make the notice certain:
see "2026-11-01T09:50:10-05:00:
cross-vendor review of the wrapper store and its controls".
Controls that assert a notice or its absence (`binary_wait_notice_tests.rs`,
`binary_lfs_hang_tests.rs`) read raw
bytes and are unchanged.

### The service's note that a commit is inside a write is a second such line (2026-11-10)

`event_path_commit::an_untracked_name_that_is_not_utf8_does_not_stop_a_commit` failed in the whole binary suite on
`main` at `b4ed7deb73` with

```text
thread 'event_path_commit::an_untracked_name_that_is_not_utf8_does_not_stop_a_commit' (2076808) panicked at src/native/binary/binary_event_path_commit_tests.rs:136:5:
assertion `left == right` failed
  left: (Some(0), "monochromatic-git-policy-cli: this commit is inside a write of the commit service, `git update-ref --stdin in .../repo` (it moves refs/heads/main to 3963eee51e143f603a9a31f314473488f583d480; the commit is also at refs/worktree/cli-git-service/1794345924936614 until the update ends), begun 0 seconds ago. ...")
 right: (Some(0), "")
```

The control required standard error to be empty,
and the client printed the service's note that its own request was inside a write.
That note is intended:
the owner asked that the client learn its commit may be on the branch when the write starts,
not a minute later (`service_busy.rs`,
`inside_notice`),
and the landing's `git update-ref --stdin` held the write for 2,069 ms in that run,
long enough for a beat to fall inside it (the same service line shows `lock_hold_ms=2089` and
`git update-ref@489+2069`).
So the line is the host being slow,
which this control does not claim.

`without_advisory_notices` (`binary_support_observation.rs`) is the reading that leaves out every line the wrapper prints
about a wait:
the wait notice,
that note in its short words,
and the longer words of a write that lasted whole minutes.
`binary_event_path_commit_tests.rs` reads every command of the module through it,
because those controls are about event bytes,
an exit status and a repository,
and `only_the_lines_about_a_wait_are_left_out_of_a_read_command` requires a finding line and another sentence of the
wrapper's to stay.
Verified:
`event_path_commit::`
13 of 13 pass (`Summary [54.673s] 13 tests run: 13 passed, 909 skipped`).
Not measured:
how often a beat falls inside a landing's write on this host;
the control passed in the same run for every other commit it makes.

## Tests that looked slow in one gate and are not

`candidate_prediction::tests::failures_report_git_and_leave_the_index_alone` and
`intent_to_add_reverts_dry_runs_and_uncommitted_removals` took 14.114 s and 14.171 s,
and
`command_add::tests::table_and_bulk_spellings_match_git` 9.475 s,
in `rel-gate2.log` at a host load near 100.
The same tests take 0.03 to 0.1 s in the gates `ct-gate2.log` and `ct-gate3.log`,
and over 10 runs on the development
host the two candidate tests had maxima of 0.055 s and 0.145 s and the command test 0.141 s.
The two candidate tests ended within 0.06 s of each other while their siblings in the module took 0.07 to 1.1 s,
so
they were stalled together,
not slow in themselves;
no test code was changed for them.

## What would make the commit itself faster

The commit's 40 starts and 36 syncs are the subject of `doc/planning/cli-git-spawn-and-sync-reduction.md`;
the budget
tests ratchet them down as slices land.

## The binary suite failed on fork at the 128-process container limit

On 2026-11-01 the mutation campaign baseline of the native wrapper exited 4 (cargo-mutants:
the unmutated tree fails its
tests),
because the binary suite's `fault_enospc` and `forwarding` tests failed on `fork` with `EAGAIN`.
The gate and mutation containers were then bounded by `--pids-limit=128`,
the default of
`package/mutation-test/cli-cargo-mutants/src/container-limits.ts`.
The default is now 512 (`defaultPidsLimit`),
which feeds the git-policy gate and mutation containers and the linter and
forbidden-strings mutation campaigns.
The linter's own test containers hard-code 128 in their own scripts and are unchanged.
Not measured:
how many processes those tests need.
512 was chosen by the owner's instruction,
not by a measurement.

## The first wave on a fresh store: five binary tests hit the bound on btrfs, 2026-11-02

Five binary tests hit `the command hit the 5-second bound` at 5.6 s in a run on `/var/home` with a fresh
`CLI_GIT_TEST_ROOT` (`subvolume::the_command_migrates_a_checkout_previews_repeats_and_deletes_the_old_tree`,
`subvolume::the_rollback_restores_the_ordinary_directory`,
and three `worktree_copy::` controls that run
`git worktree add`),
while two gates and three mutation campaigns used the host.
The library tests passed.

### What was measured

- Not a product wait.
  `strace -f -ttt -T` of every wrapped command in three cold runs (the bound raised to 119 s in a
  scratch copy,
  the harness otherwise unchanged) showed one long call in the slowest command:
  a single `fsync(3)` of
  4.74 s,
  0.19 s after the command started,
  with no `futex`,
  `flock` or `poll` wait beyond the normal child waits and
  no lock or claim of the wrapper involved (`merge --no-edit` in a linked worktree,
  load average 14).
  All other
  syscalls of that command took at most 0.15 s.
- Cold and warm store,
  same 48 tests (`worktree_` and `subvolume` filters),
  same host,
  raised bound,
  load 30 to 33:
  with a fresh `wrapper-store` the slowest wrapped command took 20.6 s and 13 of 116 commands took 4 s or more
  (`worktree add` and `migrate-to-subvolume` ones),
  the run 58.6 s;
  with the store already populated by the run before,
  the slowest took 0.70 s and the run 7.2 s.
  The tests,
  the binary and the load were the same,
  so the product's own
  waits are not what moved.
- The fresh store is where the harness writes.
  The fixtures live on `/var/home`,
  the build on another volume,
  so the
  hard link fails with `EXDEV` and `link_stored` copies the 190 MB debug executable.
  Callers that start together each
  wrote their own copy:
  sampling `/proc/meminfo` and the store every 250 ms,
  six partial copies of 190 MB were being
  written at once,
  host `Writeback` reached 720 MB,
  and the fixtures took up to 2 s each to build.
  The fixtures link
  the result and start their bounded commands next,
  so the commands that sync (the ignored-state copy,
  the journal,
  the migration) run while that write-back is in flight.
- The probe that copies 190 MB and times a wrapped `worktree add` straight after,
  with and without `fsync`,
  did not
  show it (14 to 46 ms in all four arms,
  load 13):
  one copy at a time on a calm volume is harmless.
  The stall needs the
  wave,
  not any dirty data.
  This is the weakest link of the explanation;
  the strace and the cold-against-warm
  comparison are the evidence for it.

### What changed

- `binary_wrapper_store.rs`:
  a caller that finds a partial copy being written (modified within `PROGRESS_WINDOW`,
  10 s,
  with no second name) waits for the stored copy instead of writing its own (`wait_for_publisher`;
  it polls for
  the file,
  holds nothing,
  ends at the window without a write or at `WAIT_CAP`,
  120 s,
  and then copies as before),
  and
  the publisher fsyncs its copy before it links it,
  so the write-back is over before any waiting fixture starts a
  command.
  Guards in `binary_wrapper_store_tests.rs` run the wait on a clock that moves only when the wait pauses
  (`WaitClock`) and read how it ended (`WaitEnd`:
  published,
  nobody copying,
  cap reached) and how many pauses it took,
  so
  no control measures elapsed wall time (a first version did,
  and failed once in a gate under load:
  3.2 s against a
  bound written for 0.8 s).
  Each is shown failing against a mutant of the wait:
  a live copy is
  waited for and the published one used,
  an old copy,
  an aliasing leftover and a copy that stopped growing are not,
  the cap ends the wait,
  and a caller with nothing in progress copies at once.
  The fsync has no control (nothing in the
  process observes it);
  it is covered by the measurement below only.
- With both,
  three cold runs under strace had the slowest wrapped command at 1.30,
  1.08 and 1.79 s and no call of
  0.5 s or more other than child waits (before the fsync,
  with only the wait:
  0.93,
  2.94 and 5.24 s),
  and an
  unmodified cold run passed all 48 tests (6.5 s,
  load 14).
- `a_filter_that_creates_a_worktree_runs_under_the_inherited_lease` also failed in one of those runs,
  not at the bound
  but with `the filter saw the lease: NotFound`.
  A clean source whose tracked files are older than its index is cloned
  or snapshotted by the worktree ladder,
  and then Git runs no checkout and no smudge filter;
  whether the files were
  older depended on whether a second boundary fell between writing them and committing.
  The control now leaves an
  untracked file in the source,
  so the checkout is Git's on every host.
  Red:
  the same control with the files aged and
  the index refreshed,
  without the untracked file,
  fails with that message;
  green:
  with the untracked file and the
  aging both.

### What is not claimed

A fixed 5-second wall bound can still be exceeded by a slow `fsync` that other sessions cause:
the wave was the
harness's own contribution and is gone,
the volume's other writers are not.
A bound that does not count time spent in
disk wait (state `D`) and still kills a command that loops or deadlocks would separate the two;
it needs a supervisor
in `observe_raw` in place of `timeout` and is not built.
The 5-second bound stays,
as task 66 decided.

## The hang guard now judges progress, not wall time

The sections above keep a 5-second wall bound as the hang guard and show that other sessions' write-back and load can
break it with no hang present:
a wall bound cannot tell a command that waits for the disk or is starved of processor
time from a loop or a deadlock.
The owner's rule is that a test another session's load can fail is flaky,
so the bound was replaced.

### What it does

`package/git-policy/cli/src/native/binary_supervisor.rs` runs one monitor thread per test process.
The shared bound still starts a command as `timeout --signal=KILL <seconds> <program>`,
but the seconds are now
`SHARED_BACKSTOP_SECONDS` (600),
and the monitor finds those commands among the test's own children.
Every 100 ms it looks at the command's process tree in `/proc`:
the processor ticks (own and waited-for children),
the
bytes counters of `io`,
the page faults of the children each process waited for,
the set of processes with their start
times,
and the state of every task of every process (not the thread-group leader's alone).
A tree counts as moving if any of those changed since the last look,
or if any task is runnable (`R`,
so starved of
processor,
not idle),
in disk wait (`D`),
or stopped or traced (`T`,
`t`:
held from outside).
A test that holds a command for its own reasons (it will drain a pipe,
open a fifo or send a signal later) suspends the idle
rule with `hold_idle_judgment`;
the processor budget and the backstop go on.
The monitor ends the tree through pinned processes only
(see "2026-11-02T16:27:37-05:00:
cross-vendor review of the supervisor"),
and the test fails loudly,
in these cases:
- nothing moved and nothing was runnable or in disk wait for 5 seconds (a deadlock,
  a sleep that never ends);
- the tree used 10 seconds of processor time (a loop);
- ten minutes of wall time passed (the backstop).

The failure message names the command line,
its directory,
the reason,
and each task's process ID,
name,
state,
wait
channel and command line.
The kill is recorded before the first signal goes out.
`observe_raw` claims it right after the wait,
before any cleanup,
and
panics with the message;
a kill nobody claims within 3 seconds aborts the test process with the same message,
and a process
that exits sooner exits with status 101 from an exit handler,
so no call site that runs a shared-bound command itself can lose it.
A loop that burns processor time is judged by the processor budget,
which load cannot lengthen:
starvation shows as
runnable tasks and a small tick count,
so a starved command is never mistaken for a loop.
The idle window never ends what the old 5-second wall bound would have let finish.
The processor budget does not have that property:
a tree that runs in parallel can use 10 processor seconds in less than 5 wall
seconds,
which the old bound let finish,
and the budget ends it.

### Controls

The tests in `binary_supervisor_tests.rs` (`supervisor_guard`) run real commands under the monitor with short limits:
- a deadlock of two shell processes on fifos is ended and reported with its tasks;
- a shell loop is ended by a 1-second processor budget and reported;
- a command that sleeps and works in turns for longer than the window is not ended;
- a command doing `dd conv=fsync` is in disk wait for longer than the window and is not ended,
  and the control prints a
  loud note,
  not a failure,
  if the volume was so fast that no disk wait was observed;
- a shared-bound command that hangs fails the test that ran it with the reason and the command.

Fourteen hand-applied mutants of the monitor were run against these;
two stayed green (a processor peak replaced by the
latest figure,
and the missing kill of the process group),
and two controls were added that turn them red.
The review of the supervisor replaced the group kill
(see "2026-11-02T16:27:37-05:00:
cross-vendor review of the supervisor");
its controls are listed there.

### What was measured

The full binary suite (`native:test:binary:host`) took 113.360 s before the change (load average 17.3 at start,
12.7 at
end) and 115.274 s after it (load average 34.5 at start,
20.4 at end),
each once,
in a scope capped at 600 percent of
processor,
on the default fixture root.
The difference is inside what the changed load explains,
and no speed claim is made.

### What is not claimed

- Commits under `COMMIT_BOUND_SECONDS` (120 s) are not supervised:
  their work happens in the commit service,
  outside the
  command's process tree,
  so tree progress says nothing about them.
  They keep the plain wall bound.
- Explicit-seconds `bounded_for` callers (the loop control that expects `TIMED_OUT_SIGNAL`) keep plain `timeout`.
- A command whose `timeout` parent exits while stray children hold its pipes is not judged,
  and a process that double-forks
  into a new session leaves the tree and is neither watched nor ended (see "Limits that no predicate removes").
- Throttling the disk was not used to simulate a slow volume:
  cgroup `io.max` has no effect on btrfs writeback on this
  host,
  and `strace --inject=...:delay_exit` leaves the tracee in ptrace stop,
  not disk wait.
  The disk-wait control
  relies on the real wait of `dd conv=fsync`.

## The other wall-clock assertions of the crate, and what each became

`rg --line-number 'elapsed\(\) <' package/git-policy/cli/src` listed assertions of three kinds.
The per-test limit in `.config/nextest.toml` (a test still running after five minutes is stopped and counted as failed)
turns a wait that never ends into a failure that names the test,
so a test whose only failure mode was "it takes
too long" no longer needs its own clock.

### Upper bounds that the outcome already proves

The returned outcome names the reason the wait ended,
so the elapsed assertion was removed:
- `service_index_wait_tests.rs` (release with the quick cadence) and `service_index_lock_tests.rs` (refused removal,
  ended or
  moved watch,
  watch set again,
  limit reached):
  limits of 30 s or more with outcomes `Released`,
  `IndexInUse`,
  `Unavailable` or a taken lock.
- Not among them,
  and restored by the review of the supervisor
  (see "2026-11-02T16:27:37-05:00:
  cross-vendor review of the supervisor"):
  the dead instance and the holder gone in `service_index_lock_tests.rs`,
  and the stale
  hold,
  the gone holder and the other boot in `service_index_wait_tests.rs`.
  A landing that deferred the removal of a dead
  owner's lock to the end of the wait returns the same outcome after the wait,
  so the outcome does not prove that it did not
  wait.
  The lock controls now give the platform no watch,
  so a wait is impossible and a deferred removal is refused at once;
  the wait controls give a limit of a day,
  so a wait that is taken for a leftover's hold hangs into the per-test limit.
- `worktree_copy_lock_tests.rs`,
  `a_wait_ends_when_the_lock_passes_to_another_owner`:
  the limit is now 60 s and the
  `expect` on the wait reports a give-up.
- `auto_push_single_flight_tests.rs` and `bounded_query_tests.rs`:
  the `TimedOut` variant with its arguments is only produced
  by the deadline.
- `child_watch_tests.rs`,
  `signal_relay_tests.rs`,
  `standards_review_run_tests.rs` (the silent gateway):
  a command or a job that was not stopped ends with a different outcome.
  The runner itself,
  `the_runner_is_bounded`,
  is not
  among them:
  a runner that waited for the whole job and then returned `None` after the deadline passes a test that only
  checks the outcome,
  so the job blocks on a channel that is released after the runner returned and the control checks
  that it has not finished
  (changed by the review of the supervisor,
  see "2026-11-02T16:27:37-05:00:
  cross-vendor review of the supervisor").
- `binary_client_service_tests.rs`:
  the wait limit is 60 s,
  and the absence of the wait notice proves that nothing waited.
- `binary_service_provision_tests.rs`,
  `start_retry_tests.rs`:
  the remedy text and the error kind (only `WouldBlock` is retried).
- `binary_service_tests.rs`,
  `binary_service_scratch_tests.rs`,
  `binary_push_environment_tests.rs`,
  `candidate_reader_failure_tests.rs`,
  `systemd_dbus_tests.rs`:
  the thing that must end the wait is now the only thing that can
  (a start wait of an hour,
  a stand-in that never exits,
  a peer that holds until released),
  so a missing bound hangs and the
  per-test limit reports it.
- `binary_service_lane_tests.rs`:
  the redundant assertion went,
  and the receive limit is 60 s.

### Upper bounds where wall time is the property

- `service_index_wait_tests.rs`,
  `a_watch_that_ended_is_set_again...` and `the_wait_drops_an_ended_watch...`:
  the bound is the
  poll interval of 30 s.
  A release seen long before the next poll was seen as an event.
- `index_writer_forward_tests.rs`,
  `a_foreign_index_lock_is_gits_refusal_and_is_not_waited_for`:
  an absent wait shows only as
  time.
  The outcome,
  Git's status,
  is the same after a wait;
  the bound of 10 s sits below the 20 s lock wait of the task
  environment.
  Kept.
- `lfs_output_mask_tests.rs`:
  a complexity guard;
  a quadratic scan of the line takes minutes,
  the bound is 30 s (it was 2 s).

### Lower bounds and patience deadlines

Lower bounds cannot fail from load and stay.
Polling deadlines that assert an event eventually happens are 30 s or more
now (`signal_relay_tests.rs`,
`systemd_dbus_tests.rs` and `transaction_owner_tests.rs` were raised from 20 s or 10 s to 60 s).
The 2 s wait in `kill_with_everything_it_started` (`binary_support.rs`) is a grace before a kill,
not an assertion.

### Not found by that search

Assertions that keep the elapsed time in a variable first,
or compare with `<=` or `>`,
do not match the pattern.
The review of the supervisor (see "2026-11-02T16:27:37-05:00:
cross-vendor review of the supervisor")
audited the ones with `elapsed < `:
`service_review_tests.rs` (`the_budget_bounds_the_whole_review`,
4 times a budget of
500 ms) and `standards_review_client_tests.rs` (`a_silent_server_ends_at_the_deadline`,
3 times a deadline of 400 ms) lost
their upper bounds,
because the outcome proves the stop (a review the budget did not end runs to the silent gateway's close
and is done;
a call that missed its deadline ends with a different error text),
each with a planted bug that turns it red.
`bounded_push_tests.rs` (20 s against an 8 s helper) and `auto_push_tests.rs` (30 s against an 8 s helper) keep their
bounds:
they are several times the stand-in's life,
so they fail a stop that waited for the helper and not a slow host.

## 2026-11-02T16:27:37-05:00: cross-vendor review of the supervisor

A cross-vendor review of the supervisor found four P1,
eight P2 and one P3 defects;
each was verified against the code and
fixed on `bound-supervisor-r2` with a control and a hand-applied mutant that turns it red.

- Process IDs are reused after a process is reaped,
  so a signal to a number read a moment ago,
  or to a process group,
  can reach
  an unrelated process.
  Every signal now goes through a pin (`binary_supervisor_pin.rs`):
  a pidfd opened only after the start
  time read before equalled the one read after,
  signalled with `pidfd_send_signal`,
  which reaches that process or fails.
  A tree is ended by stopping every member,
  searching again for children the stopped members had started,
  and killing
  them,
  so no member escapes between the look and the kill and no group is signalled.
  Where the kernel has no pidfd,
  the
  pin reads the start time again right before the signal;
  that window is what a pidfd closes.
  Controls:
  `a_pin_names_one_process_and_signals_nothing_else`,
  `ending_a_tree_ends_every_member_at_every_depth`.
- A kill nobody claimed could still pass a test:
  the monitor is a detached thread and the process may exit before its grace.
  The record is now published before the first signal,
  and an exit handler ends the process with status 101 when it finds an
  unclaimed record,
  whatever the test body concluded.
  Controls:
  `the_record_of_a_kill_is_published_before_the_first_signal`,
  `an_unclaimed_kill_fails_the_process_at_its_exit` (a probe in a process of its own).
- `observe_raw` claimed the kill after the service cleanup,
  which can outlast the grace:
  `supervised_wait` claims it right after
  the wait (`a_kill_is_claimed_before_the_cleanup_runs`) and writes the input from a thread of its own while the pipes drain from
  the spawn (`a_command_that_writes_more_than_a_pipe_holds_while_its_input_is_sent_finishes`).
- A child between `fork` and `exec` was remembered as "not shared" for good,
  and `exec` keeps the process ID and start time;
  nothing is remembered now (`a_command_is_found_after_it_execs_into_the_shared_bound`).
  A registered command enters the registry
  directly with its pin,
  and an entry leaves it when its pin says the process is gone,
  never by a stale snapshot
  (`a_registered_command_stays_watched`).
- The state of a worker thread in disk wait was invisible behind a sleeping leader:
  every task is read
  (`the_states_of_every_task_are_read`).
- A zombie root counted as a live command:
  it is finished (`a_command_that_has_exited_is_finished_and_not_judged_idle`).
- Legitimate external waits:
  a pipe that fills is drained from the spawn;
  a wait on the test is held with
  `hold_idle_judgment` (`a_wait_on_the_test_is_judged_idle_only_when_the_test_does_not_hold_it`);
  a stopped or traced task is held
  from outside (`a_tree_stopped_from_outside_is_not_judged_idle`).
- Calls are no sign of life:
  `syscr` and `syscw` left the figure,
  so a loop that polls an empty descriptor is no longer
  restarting the window by its calls (`the_readers_take_the_fields_whatever_the_command_is_called`).
- Children that came and went between two looks leave their page faults in the counters of the process that waited for them,
  which is a sign of life (`the_work_of_children_that_came_and_went_is_seen`).
- Three tests whose wall-clock bound the audit had removed lost the property with it,
  and got it back without a clock:
  `the_runner_is_bounded`,
  `a_dead_service_instances_lock_does_not_block_the_next_landing` (and the live holder's),
  and the stale-hold
  and gone-holder controls of `service_index_wait_tests.rs`,
  each shown red with the bug planted.

### Limits that no predicate removes

- A process that double-forks and starts a session leaves the tree it was found in and is neither watched nor ended.
  The
  commit service and the wrapper's other background programs outlive their commands on purpose,
  so ending everything that was
  once a descendant would break the tests;
  a per-command cgroup would contain them but is not available to an unprivileged test
  process in the gate container.
  A hang in such a process is ended by the per-test limit.
- The idle rule cannot tell an endless wait from a finite wait on something outside the tree;
  the explicit hold,
  the held states
  and the drained pipes are the answer,
  and a command that waits on an external service that is making progress needs a hold.
- A loop that polls without moving bytes,
  forking or using processor time stays below every sign:
  it ends at the backstop or the
  per-test limit.
  Process churn counts as a sign of life,
  so a loop that forks cheap children is bounded by the processor budget
  only.
- A race in which a process ID is reused between two reads of its start time is closed by the pidfd;
  the fallback without pidfds
  leaves a window of two reads.
- The exhausted-resource and PID-reuse failures themselves were not produced:
  no deterministic way to force a reuse exists
  without privileges,
  so the pin is tested by what it refuses.

## A merge that waited for the commit service's answer, 2026-11-11

Six controls of the binary suite's merge groups failed in one whole run,
each with the same words:

```text
the harness supervisor ended the command `/usr/bin/timeout --signal=KILL 600 …/bin/git merge --no-edit topic` in …/repo:
no progress for 5.0 s: no task was runnable, waiting for the disk or held from outside, and processor time, bytes moved,
the work of waited-for children and the set of processes stood still (a deadlock or a wait on something that never happens)
```

The failing controls were
`word_merge::a_part_line_names_each_input_as_a_blob_that_survives_the_resolution`,
`word_merge::the_journal_receives_each_merged_part`,
`word_merge_finish::a_merge_autostash_is_applied_by_the_ending_commit`,
`word_merge_finish::cherry_pick_and_revert_edit_run_the_editor`,
`word_merge_finish::merge_edit_runs_the_editor_and_merge_message_is_kept`
and `word_merge_finish::each_operation_is_finished_like_a_clean_run`,
in a run of 949 tests that passed 941
(`package/git-policy/cli`,
worktree `pi-wrapper-suite-failures-on-main-tip`;
the summary and each failure's own output are kept at
`~/temp/agent/wrapper-suite-5/binary-suite-failures-2026-11-11.log`,
which the session's own process log of that run supplied).

### What the killed trees were doing

Every message named the same task:
the wrapper,
waiting in `unix_stream_read_generic`,
the kernel's read of the Unix socket it sends its commit request on.
The same run's service log names the other half of that wait:

```text
monochromatic-git-policy-cli-service: the background sync of 6 records of a landing took 48597 ms
```

So the wrapper was not deadlocked:
it had asked the commit service for an answer,
and the service's own records were being flushed through a volume that was stalling for tens of seconds
under the other sessions' writes.
No look at the *command's* tree can see that work:
the service is not a descendant of the command,
and it outlives it on purpose (`binary_supervisor_tests.rs`,
`a_command_that_has_exited_is_finished_and_not_judged_idle`).

The idle rule as it stood counted a task asleep in a socket read as a hang,
which is right for a deadlock between two programs of one command
and wrong for a wait whose answer depends on another program's disk work.
The window is also shorter than the wrapper's own telling:
`WAITING_NOTICE_AFTER` is 10 s (`src/native/service/wait_ceilings.rs`),
so a healthy command is ended before it can say where it stands,
and the notice is written once per place the service names,
not repeatedly.

### What changed

The look at a tree reads each task's kernel wait channel (`/proc/<pid>/wchan`,
the file the kill message already printed)
and marks a process asleep in `unix_stream_read_generic` or `unix_stream_data_wait` as waiting for an answer
(`binary_supervisor_proc.rs`,
`waits_for_an_answer`).
A tree with such a task restarts the idle window as one waiting for the disk does
(`binary_supervisor.rs`,
`Judge::observe`),
so a command whose answer is being prepared is not called hung,
and a deadlock on a pipe,
a futex or a lock is still ended at the window
(`a_deadlock_is_ended_and_reported_with_its_tasks`,
`a_tree_that_stands_still_is_hung_when_the_window_has_passed`).

The control is `a_wait_for_an_answer_over_a_socket_is_not_a_hang`
(`binary_supervisor_tests.rs`):
a child that reads a Unix socket pair this test holds the other end of,
which must not be ended over several windows and must finish when the answer is written.

### Limits

- A deadlock *between the wrapper and its commit service* now looks like any answer wait,
  so it ends at the processor budget or the ten-minute backstop instead of the idle window.
  The wrapper's own answer ceiling is the product's limit for that wait (`ANSWER_CEILING`),
  and a service that stops answering is what its own controls are for.
- A task asleep in a pipe or a futex read is still judged idle,
  which is what the fifo deadlock control shows.
- The wait channel is read once per process per look,
  one small file per poll of 100 ms.
