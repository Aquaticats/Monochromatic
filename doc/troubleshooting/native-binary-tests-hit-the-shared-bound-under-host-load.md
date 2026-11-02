# Native binary tests hit the 5-second test hang guard under host load

The binary tests of `package/git-policy/cli` run every wrapped command under `timeout --signal=KILL 5`
(`TIME_BOUND_SECONDS` in `src/native/binary_support.rs`).
This is a hang guard of the test harness, short so that a mutant that loops fails every affected control quickly.
It is not a requirement on the wrapper: no wrapped command has to finish within 5 seconds.
The one latency requirement is that a commit made through the wrapper takes less than 5 seconds longer than the same
commit made with native Git (`package/git-policy/cli/SPEC.md`, beside the forwarded-command latency requirement).
On a host whose load average was 50 or more, three `discard::` controls failed with
`the command hit the 5-second bound` and passed alone in under a second.

## What was measured

Debug build, Git 2.56.0, fixtures on the `build.build-dir` volume (btrfs on dm-crypt).
Per-command wall times came from 25 reruns of each single test; process starts from `strace --follow-forks`.

- A wrapped explicit-path `commit` starts 40 Git processes and makes 36 sync calls (the exact budget of
  `src/native/binary_budget_tests.rs`), uses 82 to 88 ms of CPU and took 0.15 to 0.45 s of wall time, against 4 ms for
  direct Git. Capped to a tenth of a core it took 0.84 s, so CPU contention alone needs a share under 2 percent to reach
  5 s; the rest of a slow commit is waiting for sync calls. A probe of write, fsync and directory fsync on that volume
  took a median of 12.5 ms, p99 60 ms and a maximum of 139 ms while one suite ran, and 12.6 ms median, 16 ms maximum
  when idle.
- Every other wrapped command of the three tests took at most 0.12 s. The wrapper adds one `rev-parse` (and, for
  `stash`, one `version`) to a forwarded command.
- `git restore --help` makes Git run `man`, which starts `preconv`, `tbl`, `nroff`, `col`, `groff`, `troff` and `grotty`:
  11 process starts for one command. It took 18 ms median, 0.7 s on a cold start and 3.2 s once in 12 runs while the load
  average was 57.
- Not reproduced: a 5 s stall of an ordinary command. Three whole-suite runs under 100, 200 and 800 percent CPU
  caps, with a probe beside one, never showed a non-commit wrapped command of the `discard::`, `dependent_version::`
  and `controls::` tests above 0.07 s, so the bound stays at 5 s for single Git commands (about 70 times that).
- `dependent_version` and `controls` tests: at most 0.07 s and 0.05 s per command (173 process starts for the 50
  commands of `every_control_before_the_subcommand_is_removed`, 3 or 4 per command). No mechanism was found for them
  at that time; the section "The two task 66 tests: the cold first start explains them, and they no longer stall"
  later in this document found it.

## Cause and fix

The bound was applied to two things it does not fit: a commit transaction, which the repository already bounds with
`COMMIT_BOUND_SECONDS` (120 s) in `binary_commit_tests.rs`, and a manual-page render that is Git's work and not the
wrapper's. `observe_raw` in `binary_support.rs` now panics, on every host and without reading a clock, when a command
under the 5 s bound is a commit the wrapper lands through its transaction or a `--help` with no `man` stub in the
fixture's `bin` directory. The discard controls use `run_commit` and `stub_manual_viewer`.

## A forwarding test that hit the bound with no commit and no manual page

`forwarding::exit_codes_and_stderr_match_native_git` failed once in a mutation baseline (`native:mutation:scoped`,
`--file src/native/command_push.rs`; log `~/temp/agent/mutpar-w4-command_push.log.2`, run of 2568 tests in the
2 CPU, 2 GiB container on a loaded host whose load average that log does not record)
with `the command hit the 5-second bound` at `binary_support.rs:233` after 5.628 s.
The same log shows the two sibling tests of that module started just before it taking 1.10 s
(`argument_bytes_reach_git_unchanged`) and 2.25 s (`environment_overlay_reaches_git`), against 0.05 s each on the host
when alone: the whole container was 20 to 50 times slower at that moment, not one command.

### What was measured

- The test wraps 12 commands (two `rev-parse`, two `diff`, `show`, `log`, three usage errors, `status --porcelain`,
  `branch --list`) and runs each once more through direct Git. None is a commit and none renders a manual page, so the
  guard in `shared_bound_misuse` does not apply to it.
- `strace --follow-forks` of each wrapped command: at most one Git process started by the wrapper (the `rev-parse` it
  adds) beside the process it becomes, and no `fsync`, `fdatasync`, `flock`, `nanosleep` or `clock_nanosleep` call. The
  usage errors start no `rev-parse` at all. Unlike a commit, there is no sync wait to be slowed by a busy volume.
- Wall time per wrapped command through `timeout --signal=KILL 5`, 300 runs of each of the 12 commands (3600 starts)
  in one fixture while the load average was 28 to 39: minimum 2.7 ms, median 3.7 to 5.9 ms, 99th percentile at most
  9.3 ms, and a maximum of 63 ms (one `rev-parse --verify`), 13 ms for the next largest.
  The direct runs took 0.6 to 1.3 ms.
- The same bound was hit in other baselines by unrelated tests
  (`discard::every_named_alternative_runs_in_the_main_worktree` and
  `discard::the_hatch_after_the_subcommand_runs_each_form` in `mutpar-w5-markdown_linter.log.3`;
  `worktree_copy::an_interrupted_main_checkout_copy_is_recovered_by_the_next_creation` in
  `mutpar-w3-doctor.log.2`), so no command is common to the stalls.

### What was not found

No mechanism that makes one of the 12 commands slow was found, and none is claimed.
A command that normally takes 4 ms needs a stall of more than 1000 times that to reach 5 s; the measurements above show
no work in these commands that can wait on its own (no sync call, sleep or lock in the traced calls; the test sends
empty, closed standard input), so the stall most likely came from the container or host under the command
(CPU quota, memory pressure or disk reads of an evicted executable are candidates, none verified).
The test is left on the shared bound: moving it to the commit bound would hide a real forwarding loop, which is what
the short bound is for, and making the wrapper faster has nothing to act on, since the wrapper adds one `rev-parse`
and none of the traced waiting calls.

### What changed

The failure message of `observe_raw` now names the stalled command (its arguments and working directory, built only
when the bound was hit, so a passing run costs nothing).
The next occurrence then says which command stalled; the unnamed `assertion left != right failed` message above is why
this one cannot be attributed.
If a later occurrence names a command with no sync call or sleep again, the next measurement to take is the process
start and file read times inside the container during the stall, not the command.

## Mechanism found: the first start of a fixture's cloned wrapper copy reads the executable from disk

The section "What was not found" is superseded for the forwarding stall: the mechanism was measured afterwards.
Each binary test made a private copy of the wrapper with `cp` (`copy_executable` in
`package/git-policy/cli/src/native/binary_support.rs`).
The storage of this host is btrfs (two LUKS devices, `dm-0` and `dm-1`, under `/var/home`), so the default
`cp --reflink=auto` makes a clone that shares the original's extents.
A clone has no pages in the page cache, so the first command of a test that starts the copy faults the executable in
from disk, inside the 5-second bound.
The 3600-start measurement in "What was measured" used one fixture, so it paid that once in 3600 starts
and showed it only as its 63 ms maximum.

### Method

The measurements ran in `podman run` containers from the gate's own test image, with the container's own limits
(`--memory`, `--cpus=2`, and for disks `--device-read-bps`, `--device-write-bps`, `--device-write-iops`), and no stress
tool: two workers built a fixture like `exit_codes_and_stderr_match_native_git` (copy of the debug wrapper, a repository
with two commits), ran its 12 commands wrapped (under `timeout --signal=KILL 5`, as the test does) and direct, and
removed it, 40 times each.
Per command: wall time, `ru_majflt`, `ru_minflt` and `ru_inblock` from `wait4`.
The scripts are scratch (`~/temp/agent/reclaim/`), not committed.
The debug wrapper is 21.9 MB (`CARGO_PROFILE_DEV_DEBUG=0`), the release wrapper 9.9 MB.

### Memory caps do not change it

Wrapped commands, 960 samples per cap, host load as it was (other mutation containers running), default `cp`
(wall milliseconds, median, 99th percentile, maximum):

- 2g: 4.26, 1739, 5153.
- 1g: 4.34, 124, 1058.
- 512m: 3.93, 1973, 3719.
- 256m: 4.20, 204, 764.
- 128m: 4.06, 559, 2683.
- 96m: 3.99, 1216, 1910.

Direct Git in the same runs: median 1.0 to 1.1 ms, maximum at most 9.5 ms, 1 to 8 major faults in 960 starts.
A 5.15 s sample (killed by the bound, with 0.55 ms of CPU) occurred at the 2g cap with no disk cap, where the
container's own page cache is nowhere near full, so the container reclaim is not the mechanism; the cap neither
causes nor removes the stall.
The slow samples are the first wrapped command of a fixture: in the 2g and 512m runs, 106 of 160 first commands took
over 100 ms, against 1 or 0 of 160 for every other command index.
The first command took a median of 287 ms (maximum 5153 ms) with 4 major faults and 33368 blocks of 512 bytes read
(16.29 MiB); the same command in a started fixture took 3 to 4 ms.
Its CPU time was 9 ms, so the rest was waiting for the read.

### A slow disk turns it into a failure

With the read bandwidth of both btrfs devices capped at 5 MB/s (`--device-read-bps` on `/dev/dm-0` and `/dev/dm-1`;
capping `dm-1` alone left the release build's data, which sat on `dm-0`, unthrottled and misled one run), first wrapped
command of each of 12 fixtures:

- Debug wrapper, default `cp` (clone): 5000.6 to 6211.7 ms, every one killed by the bound.
- Debug wrapper, `cp --reflink=never`: 3.5 to 4.2 ms.
- Release wrapper, clone: 2522 to 4807 ms, median 3713 ms, under the bound only because 5 MB/s reads 9.45 MiB in time.
- Release wrapper, `cp --reflink=never`: 3.0 to 4.0 ms.

The other 11 commands of the test took at most 5.5 ms (debug, data copy) in the same runs.
Three tests run with `cargo nextest` in the container at the same 5 MB/s cap (three runs each):

- Before: `forwarding::exit_codes_and_stderr_match_native_git` and
  `management::namespace_word_elsewhere_is_an_ordinary_argument` failed in all three runs with
  `the command hit the 5-second bound` after 5.5 to 6.9 s;
  `markdown::a_failing_linter_stops_the_command` passed after 3.3 to 4.0 s.
- After: all three passed in every run, 0.055 to 0.097 s each.

### An evicted executable behaves the same

Evicting a plain copy's pages (`fsync`, then `posix_fadvise` with `DONTNEED` after a short wait) and starting it again
read the same amount as a clone, so eviction under host memory pressure costs what a clone costs:
the kernel reads most of the file in the first fault, not only the pages the start touches.
Debug: 4 major faults, 16.29 MiB, median wall time 46 ms; release: 1 major fault, 9.45 MiB, 26 ms (a disk that was not
capped).
A 5 MB/s disk therefore needs 3.3 s for the debug wrapper and 1.9 s for the release wrapper, and the cost scales with
the file size and the disk, not with anything the wrapper does at start.

### What changed first

- `copy_executable` passes `--reflink=never`: the copy is written through the page cache and stays cached, and a slow
  read of the original happens in `cp`, outside the bound.
  Guard: `a_byte_copy_of_the_wrapper_shares_no_storage_with_the_built_executable` in
  `package/git-policy/cli/src/native/binary_fixture_tests.rs` asks the filesystem for the copy's extents
  (`FS_IOC_FIEMAP`) and requires none to be shared.
  With the argument removed it failed (`shares 1 extents with .../target/debug/cli-git-native`) and with it it passes;
  a filesystem that cannot report extents skips the check.
  The bound is unchanged at 5 s.
- The release profile strips symbols (the symbol tables sit after the loaded segments and were read ahead anyway).
  First-start read in MiB (release wrapper, evicted pages): 9.45 default, 8.09 stripped, 7.43 stripped plus fat
  link-time optimization plus one code generation unit (7.79 MB file, from 9.91 MB), 6.50 with `opt-level = "s"` added.
  Only stripping is adopted: the other two cost build time (46 s to 68 s at 8 jobs, 50 s to 92 s at 4 jobs for the
  first) or have unmeasured scanner throughput (`opt-level = "s"`).
  A 300-file, 7 MB commit through the wrapper (8 alternating runs per build) took a median of about 2.95 s for the
  default, 2.95 s stripped and 2.79 s for the `opt-level = "s"` build, so none of those was slower.
  Warm start medians stayed at 3.4 to 3.6 ms and the wrapped explicit-path commit at 86 to 95 ms (CPU 69 to 73 ms)
  for all four builds.
  Wall time of the cold first start was dominated by host disk load and varied from 20 ms to 147 ms between builds
  with nearly equal reads, so only the byte counts, which are deterministic, are claimed.
  The profile adds no syscall.
  The stripped build loses function names in panic backtraces.

### Disk write and fsync

Write caps (`--device-write-bps` 5 MB/s and `--device-write-iops` 50 on both devices) slowed only the bulk write of the
22 MB fixture copy (one setup of 1.3 s in 12) and did not slow any fsync-bearing command: the direct commit took
4.8 ms and the wrapped explicit-path commit a median of 92 ms (maximum 182 ms), as uncapped (95 ms).
Dirty-page write-back and btrfs fsync are done by kernel workers that the per-cgroup write limit did not reach (an
unverified reading), so slow fsync is not modelled by this method and the sensitivity of commands to it is unverified.
The traces in "What was measured" show no sync call in the 12 forwarding commands; a wrapped commit makes 36.
The commit tests run under `COMMIT_BOUND_SECONDS` (120 s), so a slow fsync does not reach the 5-second bound.

### Evidence about the bound

Under a 5 MB/s read cap the stall of the unfixed test was 5.5 to 6.9 s, and it grows with executable size over disk
speed (a 19 MiB read at 1 MB/s took 21 s), so a bound of 10 s would have hidden this case at 5 MB/s and not at 1 MB/s.
With the data copy no wrapped command of the three tests exceeded 0.1 s at 5 MB/s, so these measurements support no
raise of the bound.
Not measured: reads of Git's own executable and libraries when they are evicted (the wrapped and direct runs both pay
it), and the unit tests in `package/git-policy/cli/src/native/git_version_cache_tests.rs`, which copy Git with
`std::fs::copy` (a clone on btrfs too) but run under no 5-second bound.

## Why a fixture needs no copy of the wrapper, and what replaced it

Each fixture used to copy the 21.9 MB debug wrapper (`copy_executable`), so a suite of about 240 fixtures wrote about
5 GB, and every test started a new inode whose pages were not cached.
Nothing in the wrapper needs a private file:
`current_exe` is used only to recognise itself by file identity (`same_file` in
`package/git-policy/cli/src/native/real_git_candidate.rs`, used by `entry.rs` and the resolver), hook entries are
recognised by the name the wrapper was started under (`main.rs`), and no control writes to the wrapper path.
The only controls that need a second file with the wrapper's bytes are the byte-copy forms in
`package/git-policy/cli/src/native/binary_resolution_tests.rs`, which still call `copy_executable`.

`fixture` now hard-links the built executable (`link_built_wrapper` in
`package/git-policy/cli/src/native/binary_support.rs`) and falls back to a data copy where a hard link is impossible
(a fixture root moved to another filesystem with `CLI_GIT_TEST_ROOT`).
That fallback was superseded on 2026-11-01: it also applied to every fixture on a host that relocates
`build.build-dir`, and the link failure now leads to one stored copy of the build instead of a copy per fixture (section
"The hard link failed on the development host, so every fixture copied the wrapper").
All fixtures of all test processes, which `cargo nextest` runs one per process, then share one inode and one set of
cached pages, so a cold read of the executable is paid once per machine state and not once per test.
A per-process copy (a `OnceLock`) would not have shared anything, because each test is its own process.
A symbolic link as the fixture's wrapper would break `std::fs::hard_link` in the resolution controls, which link
the wrapper's path and would link the link itself; `/proc/self/exe` resolves a symbolic link to its target and a hard
link to the path used to start it, and both were exercised by the full-suite runs in the measurements of this section.

One read per process, after the link, warms the file (`warm_wrapper`): the wait for a slow disk then happens
there, outside the bound.
The read has to come after the link: on overlay storage (the test containers) a hard link of an image-layer file copies
it up to a new inode, and a read of the original before the link warmed the wrong inode.
The first attempt read before the link, and at a 2 MB/s read cap the first wrapped command of the test was killed at
5001 ms with no major fault, which a probe isolated to the copy-up.

Measurements (debug wrapper 21.9 MB, containers from the gate image, 2 GiB, 2 CPUs):

- Placing the wrapper in a fixture, 100 repetitions: `cp --reflink=never` median 4.6 ms (maximum 256 ms),
  `cp` clone 1.2 ms, hard link 0.03 ms (maximum 0.16 ms); a warm read of the cached file 1.2 ms per process.
- Bytes written for the wrapper over the suite: about 5 GB (240 fixtures of 21.9 MB) before, none after.
  The cgroup `io.stat` underreports this (132 MB for the copy design against 0.8 MB for the link design) because
  write-back runs outside the container's attribution, so the estimate comes from the fixture count.
  The link design read the executable once (21.86 MB `rbytes`, the cold copy-up).
- Full `native_binary` suite: 278 passed with the copy design in 57.7 s and 279 passed (one test added) with the link
  design in 54.1 s and 61.5 s in two runs; the host was loaded, so no difference in wall time is claimed.
- Evicted wrapper (`posix_fadvise` DONTNEED before the run), three tests of the brief, 5 MB/s read cap, two runs each:
  the link design with the warm read passed (4.2 s for the two first-command tests, all of it in the warm read);
  without the warm read it passed in 3.2 s, the 16 MiB first start being 3.3 s at 5 MB/s, only just under the bound.
- Same state at a 2 MB/s read cap: the link design with the warm read passed both runs (10.8 and 11.1 s, in the warm
  read); the data copy passed both runs (10.5 and 11.4 s, in `cp` reading the original); the link design without the
  warm read failed both runs with `the command hit the 5-second bound` at 5.02 s.
  So the link design is not more robust than the data copy against a slow disk (both move the read outside the bound);
  it removes the writes, the per-fixture setup cost and the cold start of every new inode.
- Guards, each shown failing once: `a_fixture_wrapper_is_the_built_executable_itself` (same device and inode as the
  built executable, skipped where a hard link of the build cannot be made in the fixture) failed with the copy restored
  (`not the same file as .../target/debug/cli-git-native`); the byte-copy guard failed with `--reflink=never` removed.

## The two task 66 tests: the cold first start explains them, and they no longer stall

Two baselines of the mutation campaign of 2026-10-29 failed with `the command hit the 5-second bound` in
`dependent_version::literal_pathspecs_check_a_named_manifest_like_plain_mode` and
`controls::a_path_named_like_a_hatch_is_forwarded_and_a_hatch_is_removed` while the host load average was about 28.
The failure message did not yet name the command, and the commands measure 3 to 40 ms when run alone.
Question: do they still stall now that fixtures hard-link the wrapper?

### Method

Two gate images built by `native:test:container` with a stand-in `podman` that skipped the final `run` (scratch,
`~/temp/agent/t66/`), one per revision: `c3dddf91ac` (the parent of `e76a99bddc`, where each fixture clones the wrapper
with `cp`) and `1091639e0b` (current main, with the hard link and `warm_wrapper`).
Both images carried one temporary line in `observe_raw` that printed the wall time and arguments of every bounded
command (not committed).
Each run was one container with `--memory=2g --cpus=2 --pids-limit=128` and `--device-read-bps` on both `/dev/dm-0`
and `/dev/dm-1` (the container storage is `/var/home`, on `dm-1`), running `cargo nextest run --test native_binary`
for the two tests, or for the whole `dependent_version::` and `controls::` modules.
Before the test, some runs evicted pages with `dd iflag=nocache count=0`: `wrapper` evicts the built executable,
`all` also evicts `/usr/bin/git`, `/usr/bin/timeout`, `/usr/bin/sh`, `/usr/bin/env` and the shared libraries.
No stress tool and no extra load; the host load average was 5.8 when the runs started.

### What each test runs under the bound first

Both tests build their repository with `git` calls that go straight to real Git (`direct`, no `timeout`), so Git and its
libraries are read before any bounded command.
The first bounded command of each test is therefore also the first start of the wrapper's file:
`cli-git check -- package/module/a/package.json` in the `dependent_version` test and
`reset --no-enforce-worktree --quiet -- --no-enforce-worktree` in the `controls` test.

### Results at the revision before the hard link (positive control)

Per-test outcomes, two tests per run, no eviction:
- 5 MB/s read cap, 3 runs: all 6 test executions killed at 5000.7 to 5000.8 ms; the killed command was in every case the
  first bounded command listed above (named by `observe_raw`), nextest times 5.7 to 6.5 s.
- 2 MB/s read cap, 3 runs: all 6 killed the same way.
- No cap, 3 runs: all passed; the first bounded command of the 6 fixtures took 153 to 421 ms (median 353 ms), against
  3.4 to 25.8 ms for every later command. This is the cold first start of a clone, and its size on an uncapped disk is
  why a loaded host reaches 5 s while a quiet one never shows it.

### Results at current main

Test executions that hit the bound: 0 of 66 in the two-test runs and 0 of 30 in the module runs.
- 5 MB/s: 5 runs without eviction, 5 with `wrapper`, 5 with `all`; nextest wall time 4.0 to 4.2 s (`none`, `wrapper`)
  and 5.3 to 5.5 s (`all`), all of it before the first bounded command (the copy-up and read of the 22 MB wrapper, then
  Git's pages in the unbounded setup).
- 2 MB/s: 5 runs with `wrapper` (10.3 to 10.5 s) and 5 with `all` (13.3 to 13.8 s); then 3 runs of the two whole
  modules with `all` (10 tests each, 13.7 to 13.9 s).
- 1 MB/s with `all`: 5 runs, 26.7 to 28.8 s.
- No cap: 3 runs, 0.21 to 0.27 s.
- First bounded command of each fixture, current main (milliseconds): no cap, median 12.6, maximum 13.5; 5 MB/s, median
  13.1, maximum 26.5; 2 MB/s with `wrapper`, maximum 69; 2 MB/s with `all`, maximum 268; 1 MB/s with `all`, median 144,
  maximum 281; module runs at 2 MB/s with `all`, maximum 129. The worst is 17 times under the bound.
- Later commands of the two tests took at most 14.5 ms in every run. In the module runs (214 commands) one later command
  took 101.8 ms (a `status` under `--no-enforce-final-newline`) and the others at most 16.8 ms.

### Reading

- The two tests are the first bounded start of the wrapper's file in each of their processes, and both stall at the old
  revision under a read cap at exactly that command, for exactly the reason in "Mechanism found": the clone's pages are
  not cached. That the two 2026-10-29 failures were this mechanism is consistent with the evidence, not proven: no
  host trace of those two runs exists, and the cap models slow storage, not the load average of 28.
- The hard link plus `warm_wrapper` removes it: the wait for a slow disk moves into the fixture setup, which has no
  bound, and the tests passed at caps down to 1 MB/s with the wrapper evicted.
- A residual cold read remains at the first bounded command (up to about 280 ms at 1 MB/s, which is roughly 280 KB of
  pages that the earlier `git init` and `commit` did not touch, probably in Git's own binary or libraries). It was not
  attributed to a file, because it is 17 times below the bound at 1 MB/s. A stall of 5 s from such reads would need
  about 5 MB of cold pages at 1 MB/s.
- Not covered: fsync waits (the cap model does not slow them, see "Disk write and fsync"), memory pressure that evicts
  pages after the warm read, and the real load of 28. The tests have no sync call, and a stall of the kind that remains
  possible would name its command in the `observe_raw` failure message.
- The bound stays at 5 s. Nothing here supports changing it.

## The hard link failed on the development host, so every fixture copied the wrapper

The section "Why a fixture needs no copy of the wrapper" says the hard link applies everywhere except a root moved with
`CLI_GIT_TEST_ROOT`.
That was false on the development host: `~/.cargo/config.toml` relocates `build.build-dir`, so `CARGO_TARGET_TMPDIR`
(where fixtures live) is on `/var/mnt/encrypted` (device 81 here) while the final `target/debug/cli-git-native`
is on `/var/home` (device 56).
`std::fs::hard_link` failed with `Invalid cross-device link` for every fixture, and `link_built_wrapper` fell back to
`copy_executable`: a 21.9 MB data copy per fixture, the design the section above meant to remove.
The guard `a_fixture_wrapper_is_the_built_executable_itself` skipped itself exactly there ("cannot hold a hard link of
the build; nothing to check"), so nothing reported it.

### What was found

- `~/temp/agent/bin4.log` (a host run of `native:test:binary:host`): five `builtin::` tests failed with
  `the command hit the 5-second bound` after 6.2 to 6.6 s, on `add .`, `stash list`, `reset --hard`, `switch` and
  `checkout --force`, all among the first 17 tests of the run, and a `commit::` test that passed took 6.2 s in the same
  wave.
  The failure logs of `bin2.log`, `bin4.log` and `bin6.log` print the
  `fixture wrapper: no hard link ... (Invalid cross-device link ...)` line (7, 6 and 2 times).
  The failing commands differ; the first wave of parallel tests and the cross-device copy are common.
- With the copy, a wave of 169 tests wrote about 3.7 GB (169 times 21.9 MB) in 6 to 7 s.
  Host-wide `Dirty` in `/proc/meminfo` reached 1.3 to 1.5 GB during such a wave, and an `fsync` probe on the fixture
  volume took 615 ms at the worst (median 4 ms); `/proc/meminfo` is host-wide, so these numbers do not attribute the
  dirty pages to the suite.
- Building two fixtures took 2.07 s with the copy and 0.14 s with the link (the guard test with the fix disabled and
  enabled).

### What is not proven

The stall itself, a command waiting 5 s behind queued write-back, was not reproduced before and after the fix.
The host was at a load average of 20 to 25, where 142 to 172 tests passed in 4 to 8 s in both states.
That a loaded volume delays the `fsync` calls of other tests' commands behind the queued copies is an inference from
the symptoms above and from the earlier measurements of this document, not a measured cause.
What is certain: the copy was never intended, it was about 3.7 GB of avoidable writes per wave, and the failing runs
carried the cross-device message.
A reproduction would cap the write bandwidth of the fixture volume in a container (`--device-write-bps`) at the
revision before and after the fix; it was not run.

### The fix

`link_built_wrapper` links the built executable; when that link fails with `EXDEV` (the link's own failure, not a
comparison of device numbers, which bind mounts defeat) it links one stored copy of the build on the fixture's
filesystem (`link_stored` in `binary_wrapper_store.rs`), and it copies per fixture only where neither link works.
The stored copy is named by a hash and the length of the build's bytes (`content_name`), read once per process, and a
copy is read back and must have that name before it is published.
Length and modification time were used first and rejected after the cross-vendor review: equal-length builds with one
timestamp share them.
The hash is SipHash from the standard library, which protects against accidental equality only.

A caller that finds no stored copy writes its own partial file, created exclusively (`create_new`, see "The second
review"), in `<fixture root>/wrapper-store/` and links it to the final name with `std::fs::hard_link`, which fails with `AlreadyExists` if another caller published
first; it then removes its partial and uses the winner's file.
There is no lock: the final name is only ever a complete file, a reader only opens the final name, and the first
publisher wins.
Callers that start together each write a partial copy once per build, so the first wave after a rebuild writes up to
one copy per concurrent test process (derived, not measured: with one process per CPU on a 28-CPU host, up to about
600 MB, against about 3.7 GB before and about 22 MB with a lock); every later caller finds the final file.

Pruning is by idleness: a copy of another build, or a partial file, is removed only when its status-change time is
more than a day old.
A fixture's link changes that time, so a copy a run is linking is never idle, and the pruning cannot remove a copy
between a caller's finding it and linking it except for a copy a day idle that a second run starts using in that
instant.
That remaining case is handled too: `link_stored` asks for the stored copy again when the link finds it gone (three
attempts), so the caller still ends with a link to the stored file and not a per-fixture copy.
Placing the fixtures beside the executable instead was tried first and rejected: `target/` is inside the repository,
so a fixture meant to be outside any repository (the `require-root` and resolution controls) ran inside the
worktree's own.

Guards (`binary_wrapper_store_tests.rs`), shown failing together with six changes made at once, each failure naming
the control it should (one build, six controls failed, six passed):
- `a_late_publisher_leaves_the_winners_file_in_place`: `rename` in place of the link.
- `the_name_of_a_build_is_its_content`: a name that ignores the bytes.
- `a_copy_that_does_not_have_the_builds_name_is_not_published`: the check of the copy against its name removed.
- `a_stored_copy_that_vanishes_before_the_link_is_made_again`: a single link attempt.
- `concurrent_callers_end_with_links_to_one_final_file_of_the_build`: pruning every copy at once (the control keeps a
  recent copy of another build and checks that no partial file or second copy stays).
- `fixtures_are_the_built_executable_or_its_one_stored_copy` and
  `fixtures_on_another_filesystem_share_the_stored_copy_of_the_build` (named
  `fixtures_share_one_wrapper_file_with_the_built_executables_bytes` in the first version): the `EXDEV` branch
  disabled, so each fixture copies. The second covers a root in `/dev/shm`, which fails the link of the build with
  `EXDEV` where that is another filesystem, and reports when it is not.
Not covered: a copy of another build removed by a different process between its being found and linked is shown for
one removal at one moment (the control removes the stored file in `link_stored`'s hook), not under a real second run.
An earlier version used an advisory lock; the owner's stance against locks replaced it with the lock-free publish.

### The second review

A second cross-vendor review of the store and its controls (OpenAI model, output kept at
`/var/home/user/temp/agent/ct/pi-review2.out` on the development host) made eight claims; each was checked against the
code, and all eight held.
What changed, finding by finding:
- Partial names (`.partial-<pid>-<counter>`) repeat across process-identifier reuse and PID namespaces that share the
  store, and a caller that died after linking its partial to the final name leaves the published file under a second
  name; `cp` into that name truncates the published executable. `reserve_partial` now creates the name with
  `create_new` and moves to the next counter value when it exists (`RESERVE_ATTEMPTS` tries), then `cp` writes into the
  empty file and the build's mode is applied afterwards (`cp` into an existing file keeps that file's mode).
- Pruning decided on a status-change time read earlier than the unlink, so a link made in between was not noticed. Only
  a copy whose link count is one (no fixture holds it) is a candidate now, and the name is renamed to a reserved
  partial name and its link count read again before it is deleted; a count that grew means a fixture linked in the
  window, and the name is linked back. A caller that asks while the name is away publishes another copy, so the
  remaining window costs one extra copy and never a fixture without its wrapper. A lock-free removal cannot make
  the check and the unlink one step, so that residual case is accepted and stated.
- `fixtures_share_one_wrapper_file_with_the_built_executables_bytes` required shared inodes where the factory allows
  private copies, and compared two fixtures with each other, so an implementation that always used the stored copy
  passed. `fixtures_are_the_built_executable_or_its_one_stored_copy` probes what the root's filesystem links and then
  requires the built executable itself, the one stored copy, or (only for a non-`EXDEV` failure) copies with the built
  bytes. On this development host the default fixture root (Cargo's `build-dir`, relocated to another volume) is on a
  different filesystem than the final artifacts, so the stored-copy branch is the one the default run exercises; the
  built-executable branch runs with `CLI_GIT_TEST_ROOT` on the artifacts' filesystem.
- The relocated-root control assumed `/dev/shm`, removed its directory only on success, and ignored capacity. It
  now picks the first of `/dev/shm` and `$XDG_RUNTIME_DIR` that is a writable directory, fails the probe link with
  `EXDEV` and has room for two copies, and otherwise prints why it skipped; a `Scratch` guard removes the directory
  when the control panics.
- `a_slow_commit_tree_prints_a_wait_notice_that_the_comparison_leaves_out` slept past the grace; the watcher thread can
  resume after the signer has ended (`watch_loop` in `child_watch.rs` finds no wait marked), so the notice was lost
  under load. The signer now waits for a release file that the control creates when it reads the notice. The 5-second
  bound of the shared binary tests is unchanged.
- `build_name` hashed the executable before `OnceLock::get_or_init`, so callers that arrived together each read the
  22 MB file. `name_once` runs the read inside `get_or_init`. The outcome of a failed read is kept for the process (the
  callers then fall back to private copies), because `OnceLock` cannot keep a failed initialisation open on stable
  Rust; a read that failed once rarely succeeds later in a test run.
- The concurrency control used one worker per CPU and the real executable, with nothing forcing overlap. It now runs a
  named number of callers on a 64 KiB stand-in build, and `shared_copy_named` gives each caller a gate after its
  checked partial copy and before the link, so every caller publishes against all the others.
- The pruning control never aged an entry or refreshed one through a link. `remove_idle_builds_at` takes the clock as
  an argument, so aging is tested at the exact boundary without sleeping; a copy with an ancient modification time
  and a fresh status-change time must stay, which an implementation that reads the modification time fails.

Each new control was shown failing against a deliberate change (the production code was restored afterwards):
- `a_partial_name_that_exists_is_never_reused` and `a_copy_never_writes_through_a_leftover_partial_name`: `create(true)`
  in place of `create_new(true)`.
- `callers_that_ask_together_for_the_name_of_a_build_read_it_once`: the read moved outside `get_or_init`.
- `a_copy_with_an_old_modification_time_and_a_recent_link_stays`: `mtime()` in place of `ctime()` (the previous pruning
  control still passed).
- `a_copy_leaves_the_store_exactly_when_it_has_been_idle_for_the_idle_age`: `<=` in place of `<` in the age test.
- `a_link_made_during_the_removal_keeps_the_copy`: a plain removal in place of the set-aside and recheck.
- `concurrent_callers_end_with_links_to_one_final_file_of_the_build`: copies serialized behind a mutex, which keeps the
  callers from meeting at the gate. Replacing the link by a `rename` is not caught by this control in every schedule
  (it passed once), and stays the job of `a_late_publisher_leaves_the_winners_file_in_place`, which orders the two
  publishers.
- `fixtures_are_the_built_executable_or_its_one_stored_copy`: a factory that always uses the stored copy, with
  `CLI_GIT_TEST_ROOT` on the artifacts' filesystem.
- `a_scratch_directory_goes_with_its_control_and_a_missing_filesystem_is_a_reason`: an empty `Drop`.
- `a_slow_commit_tree_prints_a_wait_notice_that_the_comparison_leaves_out`: a 3-second sleep at the start of the
  watcher thread, which simulates a watcher resuming late. The sleeping version of the control failed (no notice) and
  the released version passed.

## A wait notice in a diagnostics comparison

A `merge_isolation` control that compares the wrapper's diagnostics with native Git's failed once on a
`cli-git: still waiting for git commit-tree` line.
That line is correct behaviour: a step that outlasts its notice grace (`PLUMBING_NOTICE_GRACE` 5 s and
`OBJECT_NOTICE_GRACE` 10 s in `child_class.rs`; `commit-tree` is a bulk step) prints it on standard error however
healthy the step is, and a loaded host stretches any step.
The failing run was reported on 2026-11-01 around 02:14 and the class graces landed in `ac97d3b4f3` at 01:58 that day,
so the failing branch may still have had the one-second grace; this was not verified, and the run was not kept.
The controls now compare the diagnostics without wait-notice lines (`without_wait_notices` in `binary_support.rs`,
used by `run` in `binary_merge_isolation_tests.rs`), and
`a_slow_commit_tree_prints_a_wait_notice_that_the_comparison_leaves_out` holds a signer stand-in until the control has
read the notice from the wrapper's standard error, and checks that it is printed and removed.
Its first version only slept past the grace, which did not make the notice certain: see "The second review".
Controls that assert a notice or its absence (`binary_wait_notice_tests.rs`, `binary_lfs_hang_tests.rs`) read raw
bytes and are unchanged.

## Tests that looked slow in one gate and are not

`candidate_prediction::tests::failures_report_git_and_leave_the_index_alone` and
`intent_to_add_reverts_dry_runs_and_uncommitted_removals` took 14.114 s and 14.171 s, and
`command_add::tests::table_and_bulk_spellings_match_git` 9.475 s, in `rel-gate2.log` at a host load near 100.
The same tests take 0.03 to 0.1 s in the gates `ct-gate2.log` and `ct-gate3.log`, and over 10 runs on the development
host the two candidate tests had maxima of 0.055 s and 0.145 s and the command test 0.141 s.
The two candidate tests ended within 0.06 s of each other while their siblings in the module took 0.07 to 1.1 s, so
they were stalled together, not slow in themselves; no test code was changed for them.

## What would make the commit itself faster

The commit's 40 starts and 36 syncs are the subject of `doc/planning/cli-git-spawn-and-sync-reduction.md`; the budget
tests ratchet them down as slices land.

## The binary suite failed on fork at the 128-process container limit

On 2026-11-01 the mutation campaign baseline of the native wrapper exited 4 (cargo-mutants: the unmutated tree fails its
tests), because the binary suite's `fault_enospc` and `forwarding` tests failed on `fork` with `EAGAIN`.
The gate and mutation containers were then bounded by `--pids-limit=128`, the default of
`package/mutation-test/cli-cargo-mutants/src/container-limits.ts`.
The default is now 512 (`defaultPidsLimit`), which feeds the git-policy gate and mutation containers and the linter and
forbidden-strings mutation campaigns.
The linter's own test containers hard-code 128 in their own scripts and are unchanged.
Not measured: how many processes those tests need.
512 was chosen by the owner's instruction, not by a measurement.
