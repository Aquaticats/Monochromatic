# Native binary tests hit the shared 5-second bound under host load

The binary tests of `package/git-policy/cli` run every wrapped command under `timeout --signal=KILL 5`
(`TIME_BOUND_SECONDS` in `src/native/binary_support.rs`).
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
  commands of `every_control_before_the_subcommand_is_removed`, 3 or 4 per command). No mechanism was found for them.

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
All fixtures of all test processes, which `cargo nextest` runs one per process, then share one inode and one set of
cached pages, so a cold read of the executable is paid once per machine state and not once per test.
A per-process copy (a `OnceLock`) would not have shared anything, because each test is its own process.
A symbolic link as the fixture's wrapper would break `std::fs::hard_link` in the resolution controls, which link
the wrapper's path and would link the link itself; `/proc/self/exe` resolves a symbolic link to its target and a hard
link to the path used to start it, and both were exercised by the full suite below.

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
  built executable, skipped when the fixture is on another filesystem) failed with the copy restored
  (`not the same file as .../target/debug/cli-git-native`); the byte-copy guard failed with `--reflink=never` removed.

## What would make the commit itself faster

The commit's 40 starts and 36 syncs are the subject of `doc/planning/cli-git-spawn-and-sync-reduction.md`; the budget
tests ratchet them down as slices land.
