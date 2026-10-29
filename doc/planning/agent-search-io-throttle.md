# Throttling agent search I/O

Agents run broad `rg`, `grep`, and `find` searches that stall the desktop because the repository disk is slow.
This note records what was measured on 2026-10-29 and what is proposed.
Nothing here is accepted; the proposals need an explicit decision before any work starts.

## Measured

- `/proc/pressure/io` read `some avg60=56.89` and `full avg60=40.94` while other sessions were running (one sample).
- `rg --files --hidden --no-ignore .` from the repository root lists 464,482 files; `.git` is 1.3G.
- `/home` is btrfs on `/dev/dm-1`, a LUKS mapping of a partition on `nvme0n1` (SPCC M.2 PCIe SSD).
- Both block-device scheduler files printed `kyber` as the active scheduler.
- The `io` cgroup controller is enabled for `user@1000.service`.
- `package/dev-script/claude-cpu-cap/src/run-pass.ts` states that I/O pressure is deliberately not watched.

## Cgroup read caps do not throttle through LUKS here

A throwaway user unit read a 64 MiB file with `dd iflag=direct`, alternating uncapped, capped, and uncapped arms.

- `IOReadBandwidthMax=/dev/dm-1 8M`: systemd accepted it, but the unit's `io.max` held no `252:1` line.
  The kernel never recorded a limit for the device-mapper device. The capped arm ran at 519 MB/s, same as control.
- `IOReadBandwidthMax=/dev/nvme0n1 8M`: `io.max` showed `259:0 rbps=8000000`,
  yet the capped arm ran at 484 MB/s against 461 and 463 MB/s uncapped.
  An 8M/s cap would have needed about 8 seconds; the read took 0.14 seconds.
- `IOReadIOPSMax=/dev/nvme0n1 200`: 2,000 direct 4 KiB reads finished in 0.12 seconds,
  within the spread of the uncapped arms (0.09 to 0.10 seconds).

Why the physical-disk cap is bypassed is not established; the probe only shows that it is.
Kernel documentation says `io.max` limits are enforced by blk-throttle on all devices
and says nothing about stacked devices (docs.kernel.org/admin-guide/cgroup-v2.html).
`io.weight` needs iocost or BFQ for the device, and `ionice` priorities are honored only by BFQ and mq-deadline
(docs.kernel.org/block/ioprio.html), so neither helps under `kyber` without a scheduler change that needs root.

## Proposals

Ranking: B > C > A > D.

- B, a PreToolUse hook that denies broad searches: unaffected by the cgroup result, and the Bash hook machinery exists.
  Risk: shell-string parsing is bypassable by scripts and can false-positive.
- C, an `.rgignore` plus softening the `--hidden`/`--no-ignore` advice in `CLAUDE.md`:
  cheap, but `--no-ignore` bypasses the file, so it only works with the rule edit.
  The share of the 464,482 files that `node_modules`, `target`, `.cache`, and `.claude/worktrees` account for is unmeasured.
- A, I/O caps on the agent cgroup: demoted because the two probes above show no throttling.
  Revisit after identifying which layer drops the limit (for example, a test on a non-LUKS device),
  or after trying `IODeviceLatencyTargetSec=`, which was not tested.
- D, an `AGENTS.md` rule alone: weakest, since agents drift from rules under load.

B beats C because a hook can inspect the `--no-ignore` flag that defeats an ignore file.
C beats A because A is currently shown not to work.
A beats D because A could still be mechanical if the throttling layer is found, while D relies on obedience.
