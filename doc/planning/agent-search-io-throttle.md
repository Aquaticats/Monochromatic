# Throttling agent search I/O

Agents run broad `rg`, `grep`, and `find` searches that stall the desktop because the repository disk is slow.
This note records what was measured on 2026-10-29 and what is proposed.
Nothing here is accepted; the proposals need an explicit decision before any work starts.

## Correction

The first revision of this note (commit `43786bb026`) concluded that cgroup read caps do not throttle reads.
That conclusion was wrong: the probe file lived under `~/temp`, a symlink to `/var/mnt/encrypted/temp`,
which is the SATA drive (`/dev/mapper/crypt_sda`), while the repository lives on `/var/home` (`/dev/dm-1`, on `nvme0n1`).
Every probe now places its file on `/var/home`, confirmed with `findmnt --target`.

## Measured

- `/proc/pressure/io` read `some avg60=56.89` and `full avg60=40.94` while other sessions were running (one sample).
- `rg --files --hidden --no-ignore .` from the repository root lists 464,482 files; `.git` is 1.3G.
- `/home` is btrfs on `/dev/dm-1`, a LUKS mapping of a partition on `nvme0n1` (SPCC M.2 PCIe SSD).
- `nvme0n1` and `sda` both list `none mq-deadline [kyber] bfq` with `kyber` active; the `dm-*` devices have no scheduler file.
- The `io` cgroup controller is enabled for `user@1000.service`.
  `io.cost.qos` and `io.cost.model` read empty, so iocost is not configured.
- `package/dev-script/claude-cpu-cap/src/run-pass.ts` states that I/O pressure is deliberately not watched.

## Cgroup read caps on the repository disk

A throwaway user unit read a 64 MiB file placed on `/var/home`.

- `IOReadBandwidthMax=/dev/dm-1 8M`: systemd accepted it, but the unit's `io.max` had no `252:1` line,
  so the kernel recorded no limit on the device-mapper device. Cap the physical disk instead.
- `IOReadBandwidthMax=/dev/nvme0n1 8M`, buffered reads after dropping the file's cached pages:
  8.35 seconds capped against 0.10 and 0.14 seconds uncapped. The cap works for buffered reads, the pattern `rg` uses.
- The same cap on a direct read (`dd iflag=direct`) did not slow the read (154 MB/s), and the unit's `io.stat`
  charged only about 2 MB to `259:0`. One run, unexplained; direct I/O is not the `rg` access pattern.

Cold reads of many small files (the `rg` pattern) on `/var/home` while other sessions were running builds:
2,000 files of 4 KiB took 19.31 seconds uncapped (about 100 files per second), and a later run of 400 files took
20.44 seconds (about 20 files per second). An IOPS cap above that delivered rate shows nothing, so the cap value must
sit below the rate the disk delivers under load; the capped arm of this probe was never completed.

Kernel documentation says `io.max` limits are enforced by blk-throttle on all devices (docs.kernel.org/admin-guide/cgroup-v2.html),
`io.weight` needs iocost or BFQ, and `ionice` priorities are honored only by BFQ and mq-deadline
(docs.kernel.org/block/ioprio.html), so neither helps under `kyber` without a scheduler change that needs root.

## BFQ on nvme0n1

The user authorized switching `nvme0n1` from `kyber` to `bfq`; the switch was made for the running boot only
(`echo bfq | sudo tee /sys/block/nvme0n1/queue/scheduler`), with no udev rule. Revert with `echo kyber` to the same file.

A heavy reader (600 cold 4 KiB files) ran against a light reader (60 cold files) on `/var/home`, two runs per arm:

- light alone: 0.61 and 0.12 seconds (the run-to-run spread is as large as most of the effects below).
- heavy at `IOWeight=100`: light took 0.25 and 0.55 seconds.
- heavy at `IOWeight=1`: light took 0.06 and 0.07 seconds.
- heavy in the idle scheduling class: light took 0.85 seconds (one run), heavy took 7.99 seconds.

The weight arms moved in the expected direction (weight 1 below weight 100 in both pairs), so the probe has a working
positive control, but the disk was not saturated: the heavy reader finished 600 files in under 2 seconds,
unlike the 20 to 100 files per second measured earlier under load. The test therefore shows `io.weight` is honored under
BFQ and says nothing yet about relief during a real stall. The idle class did not protect the light reader in the one run.

## Commits are a second I/O consumer

While this note was committed, `ps` showed three concurrent shadow-repository `git commit` processes
(two of them duplicates started by this session while the first was slow, one from another session).
Each was in uninterruptible disk wait (`folio_wait_bit_common`) with a `git-lfs filter-process` child;
one had read 340 MB (`rchar`) and 31 MB from disk (`read_bytes`) after 16 seconds.
`package/git-policy/cli/SPEC.md` ("Private preparation") says the native commit runs on a complete private index
with the whole worktree as `--work-tree`, and the repository tracks 19,252 files.
The inference is that each commit stats most tracked files on a disk that delivers 20 to 100 cold reads per second;
the number of files git actually stats during the commit is unmeasured (`GIT_TRACE2_PERF` on a shadow commit would show it).
Throttling agent search alone leaves this load; the cgroup cap would need to cover the commit gate too.

## Proposals

Ranking: A > B > C > D.

- A, I/O caps on the agent cgroup targeting `/dev/nvme0n1`: bounds every command, and the bandwidth cap is shown to work.
  Risks: it slows legitimate builds, the numbers need measuring, and the many-small-files IOPS cap is not yet verified.
- B, a PreToolUse hook that denies broad searches: stops the load at the source and the Bash hook machinery exists.
  Risk: shell-string parsing is bypassable by scripts and can false-positive.
- C, an `.rgignore` plus softening the `--hidden`/`--no-ignore` advice in `CLAUDE.md`:
  cheap, but `--no-ignore` bypasses the file. The share of the 464,482 files that `node_modules`, `target`, `.cache`,
  and `.claude/worktrees` account for is unmeasured.
- D, an `AGENTS.md` rule alone: weakest, since agents drift from rules under load.

A beats B because a cap bounds every command an agent runs, while a hook only catches commands it recognizes.
B beats C because a hook can inspect the `--no-ignore` flag that defeats an ignore file.
C beats D because C is mechanical, while D relies on obedience.
