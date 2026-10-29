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
- `package/claude-cpu-cap/daemon/src/run-pass.ts` states that I/O pressure is deliberately not watched.

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

### Persistence

The user approved persisting BFQ. `/etc/udev/rules.d/61-nvme-bfq.rules` sets `queue/scheduler` to `bfq` for `nvme0n1`
on `add|change`; it sorts after Fedora's `/usr/lib/udev/rules.d/60-schedulers.rules`, which sets `kyber` for NVMe.
Positive control: after resetting the scheduler to `kyber`, `udevadm trigger --action=change --name-match=/dev/nvme0n1`
restored `bfq`, and `sda` stayed on `kyber`. The boot-time `add` event was not exercised (no reboot).
To revert, delete the rule file and run `echo kyber | sudo tee /sys/block/nvme0n1/queue/scheduler`.

## Scope requirement: not Claude only

The user ruled out a fix that applies only to Claude Code.
`package/claude-cpu-cap/daemon/src/paths.ts` targets exactly one cgroup,
`app.slice/claude-code-bash`, and re-applies the limit on every pass because Claude Code recreates it each boot.
Each running `claude` TUI sits in its own `app.slice/app-ghostty-surface-transient-<n>.scope`,
so any terminal-launched tool (other agent harnesses under `package/pi-plugin` and `package/dsh-plugin`, builds, the user's
own commands) lands in the same kind of scope. A cgroup name cannot tell an agent from the user's own terminal work.
Any mechanism must therefore key on something harness-neutral: a dedicated slice that launchers opt into,
the terminal scopes as a class, or protecting the interactive desktop instead of limiting the agents.
No `pi` process was running when this was checked, so its cgroup placement is unobserved.

### Harness-neutral candidates checked

- Name-prefix drop-in: a drop-in at `~/.config/systemd/user/zzprobe-.scope.d/io.conf` with `IOWeight=7` gave
  `zzprobe-test.scope` an `io.weight` of 7, while the control `zzother-test.scope` kept 100.
  This was verified with `systemd-run --scope`; scopes that Ghostty creates (`app-ghostty-surface-transient-<n>.scope`)
  were not exercised, so a prefix drop-in for them is untested. The drop-in directory was removed afterwards.
- Shadowing `rg`, `grep`, `find`: `mise.toml` puts `.cache/toolchain-bin` and `.cache/cli-git-bin` first on `PATH`
  (generated from `file-enforcer.config.ts`), and `.cache/cli-git-bin/git` is already a shadow of `git`, so there is precedent.
  The shadow reaches only shells where the mise environment is active. In Claude Code shells, `grep` and `find`
  are shell functions that run Claude's embedded `ugrep` and `bfs` (with `--hidden`), and functions win over `PATH`,
  so a `PATH` shadow never sees them; `rg` is not a function, so a shadow does apply to it.
  Repository rules forbid bash wrappers, so a shadow would be a Node or Rust executable, whose start-up cost per call is unmeasured.
  A shadow can throttle (run the real binary at a low I/O class or weight) or reject; throttling changes no results.

## The NVMe itself

Read-only probe of `/dev/nvme0` on 2026-10-29 (`nvme id-ctrl`, `nvme get-feature`, `smartctl --all`, `lspci -vv`, `dmsetup table`).

- Controller: Realtek RTS5772DL, DRAM-less (`lspci` PCI ID `10ec:5772`), model string `SPCC M.2 PCIe SSD`,
  firmware `VF101C68`, NVMe 1.4, 2.04 TB, one 512-byte-LBA namespace.
  It asks the host for a Host Memory Buffer: `hmpre` 16384 and `hmmin` 8192 (the libnvme manual defines both as the
  preferred and minimum size the host is requested to allocate).
- `maxcmd` is 4. libnvme defines it as "the maximum number of commands that the controller processes at one time"
  ([mankier.com/2/nvme_id_ctrl](https://www.mankier.com/2/nvme_id_ctrl)). Read literally, the controller works on
  four commands at once whatever queue depth the host submits; this is not yet measured, and an independent review of this chip was not found
  (the roadmap listing at [anandtech.com/show/21441](https://anandtech.com/show/21441) leaves 4K random IOPS blank).
- Not the cause: the PCIe link is Gen4 x4 (16GT/s, width x4), temperature 46 C with every thermal counter zero,
  3% of endurance used, no media errors, 287 TB read and 93.6 TB written over 10,626 power-on hours, 75 unsafe shutdowns.
- Power states: APST is enabled. After 100 ms idle the drive enters state 3 (exit latency 10 ms),
  and after 2000 ms idle state 4 (exit latency 60 ms). `nvme_core.default_ps_max_latency_us` is 100000, which permits both.
  Bursty access therefore pays 10 to 60 ms on the first command after a pause; the effect is unmeasured.
- Queue settings: `queue_count` 17, `nr_requests` 256, `max_sectors_kb` 128, `read_ahead_kb` 128, write cache on,
  `wbt_lat_usec` 0, `io_poll` 0.
- dm-crypt above it (`aes-xts-plain64`, `allow_discards` only): none of `no_read_workqueue`, `no_write_workqueue`,
  `same_cpu_crypt` or `submit_from_crypt_cpus` is set, so reads and writes go through dm-crypt's internal workqueues.
  The kernel documentation only describes these flags and gives no performance guidance
  ([docs.kernel.org dm-crypt](https://docs.kernel.org/admin-guide/device-mapper/dm-crypt.html)),
  so their effect on this drive needs a measurement.

### Measurements on the drive

- APST on versus off could not be run. The probe's `nvme set-feature` call returned "Permission denied", and the auto-mode
  classifier refused a direct `set-feature` write ("Modify Shared Resources"). The drive's APST table was never changed
  (the restore check compared identical to the original).
- First-read latency after an idle gap, APST left as is: single 4 KiB direct reads from a 64 MiB file on `/var/home`,
  8 rounds per gap, medians 0.03 ms (gap 0), 0.04 ms (50 ms), 0.04 ms (300 ms), 0.05 ms (1500 ms), 0.04 ms (4000 ms),
  worst single read 0.08 ms.
  The drive never idled: other sessions completed 1,313 reads during the 50 ms gaps, 7,854 during 300 ms, 32,771 during 1500 ms
  and 90,430 during 4000 ms (about 2,800 reads per second), so this says nothing about APST wake-up cost.
- Retraction: an earlier revision of this note read the 0.04 ms figure as showing that the drive's raw small-read latency
  is not the problem. That was wrong. The probe file had been written seconds before it was read, so those reads were
  most likely served from the drive's or the host's cache; they say nothing about cold data.
- Cold single 4 KiB direct reads on the raw partition `nvme0n1p6` (881 GiB, `sudo dd iflag=direct`, 200 reads per
  category, alternating). Two runs under different measured load:
  - Heavy load, other sessions at about 2,800 reads per second: whole-partition spread median 8.84 ms, p90 25.5 ms,
    p99 211 ms; one 64 MiB window median 8.22 ms, p90 24.6 ms, p99 118 ms. Queueing behind the other sessions dominates,
    and the two categories are indistinguishable.
  - Light load, other processes at 70 reads per second: whole-partition spread median 1.13 ms, p90 1.34 ms, p99 7.9 ms,
    max 9.0 ms; one 64 MiB window median 0.23 ms, p90 0.82 ms, p99 17.2 ms, max 79.7 ms.
    Spreading reads across the device costs about five times the median of reading one small region.
    This matches the expected behavior of a DRAM-less drive whose address map does not fit its 64 MiB host buffer,
    but the mechanism is not proven; other explanations are first-touch reads of cold flash pages.
- Kernel counters from `/proc/diskstats` in six 5-second windows while load varied: at 3 reads per second `nvme0n1`
  showed 0.50 ms average read wait; at 26 reads per second 1.05 ms; at 174 reads per second 1.31 ms, while the LUKS
  mapping `dm-1` above it showed 0.85 ms, 1.82 ms and 3.15 ms. dm-crypt and the layers above add roughly 1.5 to 2 ms
  at moderate load, so the encryption layer is a measurable second cost.
- Why APST cannot be switched with `nvme set-feature`: `nvme_admin_cmd_allowed()` in the kernel's
  `drivers/nvme/host/ioctl.c` returns false for a userspace Set Features on `NVME_FEAT_AUTO_PST`
  (and on host behavior, host memory buffer and queue count), even for root, so the ioctl returns `EACCES`
  (confirmed with `strace`: `ioctl(NVME_IOCTL_ADMIN_CMD) = -1 EACCES`). Lockdown mode is `none` and Secure Boot is disabled.
  The driver's own control is `nvme_set_latency_tolerance()` behind the PM QoS file `pm_qos_latency_tolerance_us`;
  that route was not exercised. APST also could not matter during agent bursts here, because the other sessions kept
  the drive busy far below the 100 ms idle that starts a power-state transition.

- APST toggled through the driver's own PM QoS file (`/sys/class/nvme/nvme0/power/pm_qos_latency_tolerance_us`, initial
  value 100000, equal to the module default). Writing `0` made the drive report `APSTE=Disabled` with no idle-timer entries
  (`nvme get-feature` 0x0c); writing `100000` back restored a report identical to the original, verified in a `finally` block.
  Latency effect, raw 4 KiB direct reads on `nvme0n1p6` (72 of 72 reads proved by `/proc/diskstats` to reach the drive),
  four arms (default, disabled, default, disabled), six reads per gap: medians 0.22 to 0.31 ms in every arm and gap,
  maximum 2.18 ms. The 500 ms gaps were nearly idle (0 to 33 other reads) and showed no 10 ms penalty with APST enabled,
  so the 100 ms transition to the 10 ms-exit state costs nothing measurable here. The 3000 ms gaps were busy (174 to 14,320
  other reads), so the 2000 ms transition to the 60 ms-exit state was not exercised and remains untested.
  An earlier run of the same comparison through a file on `/var/home` read 0.02 to 0.04 ms and is discarded: it had no
  evidence the reads reached the drive and ran about seven times faster than the raw reads, so they were likely cache hits.
- `cryptsetup refresh` on an active mapping updates it without unmounting; flags set without `--persistent` are not stored in
  the LUKS2 header (`cryptsetup-refresh(8)`), and refreshing with no optional flags restores defaults, so a refresh must repeat
  `--allow-discards`, which the live table currently has, or TRIM passthrough would be dropped.

Implication for the I/O limit: a broad search by an agent produces exactly the spread-out cold-read pattern that is
slowest on this drive, and concurrent readers multiply the wait for everyone, so bounding the agents' read rate
(the bandwidth cap on `/dev/nvme0n1` is shown to work) addresses the actual bottleneck.

Candidate measurements, none run yet:

- Queue-depth sweep of 4 KiB random reads on the raw device read-only, against the same sweep through the LUKS mapping
  (synthetic load, which the repository rules ban unless the user waives them).
- APST on versus off, which needs the user to permit `nvme set-feature` on `/dev/nvme0`.

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
