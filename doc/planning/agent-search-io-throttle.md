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

- Baseline for the dm-crypt workqueue trial, before any change: single cold 4 KiB direct reads, 100 spread and 100 in a
  64 MiB window per device, raw `nvme0n1p6` and the LUKS mapping `dm-1` interleaved twice, other processes at 35 to 117
  reads per second. Medians, spread then window: raw 0.305 / 0.228 ms and 1.173 / 0.231 ms; `dm-1` 1.160 / 0.238 ms and
  0.936 / 0.253 ms. The raw spread median itself moved from 0.305 to 1.173 ms between runs, so a single-read probe
  at this load cannot resolve a dm-crypt effect; p99 spikes of 100 to 212 ms hit both devices alike.
  A fair test of the workqueue flags needs concurrent reads, which is synthetic load and needs an explicit waiver.

- dm-crypt workqueue trial, user ran `cryptsetup refresh luks-a968fc93-... --allow-discards --perf-no_read_workqueue
  --perf-no_write_workqueue` (no `--persistent`); `dmsetup table` then showed `3 allow_discards no_read_workqueue
  no_write_workqueue`. Same interleaved probe, other processes at 52 to 121 reads per second. Medians, spread then window:
  raw 1.406 / 0.658 ms and 1.145 / 0.232 ms; `dm-1` 1.174 / 0.265 ms and 1.152 / 0.245 ms. Against the baseline
  (`dm-1` 1.160 / 0.238 ms and 0.936 / 0.253 ms) there is no change beyond run-to-run spread. This is a null result from
  a single-read probe that could not resolve a difference to begin with, not evidence that the flags do nothing under
  concurrent reads.

- dm-crypt workqueue trial with a real job: `mise run //:lint:oxlint` over the whole repository (8,176 files, 488 rules,
  16 threads; it finds 18,203 warnings and 7,044 errors, so the task exits 1 by design), run as a capped transient unit
  (`MemoryMax=24G`, `CPUQuota=1600%`) after dropping the page cache. Counters are system-wide `/proc/diskstats` deltas,
  so they include other sessions. All three runs per arm are comparable; the first run of each arm was not systematically different.
  - Flags on (`no_read_workqueue`, `no_write_workqueue`): 78.6 s, 108.6 s, 107.3 s; `nvme0n1` read wait 2.96, 2.88, 7.76 ms;
    `dm-1` read wait 5.80, 7.10, 24.96 ms.
  - Flags off (defaults, after `cryptsetup refresh ... --allow-discards`; table back to `1 allow_discards`): 98.4 s, 80.6 s,
    102.5 s; `nvme0n1` read wait 2.94, 2.52, 5.52 ms; `dm-1` read wait 6.18, 5.97, 12.84 ms.
  - The wall-time ranges overlap fully (78.6 to 108.6 s against 80.6 to 102.5 s); means 98.2 s against 93.8 s. The
    workqueue bypass has no effect this measurement can resolve, so the live mapping was returned to its defaults.
  - One whole-repo lint issues about 53,000 to 116,000 reads on `nvme0n1` (1.4 to 4.0 GiB) in 80 to 110 seconds, roughly 800 to
    1,100 reads per second from a single job, which is the load level at which other readers saw 8 ms median cold-read waits.
  - `dm-1` read wait is about twice `nvme0n1`'s in calmer runs in both arms, so the extra wait above the drive is not the
    dm-crypt workqueue.

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

## Layer one, end-to-end check (inconclusive)

Both halves of layer one are live: the Ghostty terminal scopes carry `IOWeight=10`, and the watcher sets `claude-code-bash`
to `io.weight` `default 10` (commit `401b01cbe3`). The end-to-end check ran the real `//:lint:oxlint` job from
the weight-10 cgroup or from a weight-100 unit while a light reader (60 cold 4 KiB files, in a default-weight scope) was timed,
five reads per arm, in two pairs:

- light alone: median 0.04 s and 0.07 s.
- lint at weight 10 (this shell): light median 0.16 s (max 0.37 s), then 0.41 s (max 5.23 s).
- lint in a weight-100 unit: light median 0.16 s (max 1.95 s), then 0.08 s (max 9.76 s).

Maxima favor weight 10 in both pairs; medians do not (equal in the first pair, opposite in the second).
Other sessions made the lint itself take 130 to 365 s, so the disk was far from the calm case.
The weight is applied and did no harm, but the check does not show that it protects a light reader.

## The search-shadow package

`package/search-shadow/cli`, crate `monochromatic-search-shadow` (0.1.1 published; the source on `main` is the many-tool
version, not yet released), binary `search-shadow` (Rust, `clap` builder API).
Started under one of seventeen names (`rg`; `grep`, `egrep`, `fgrep`, `ugrep`, `ug`, `ag`, `ack`; `find`, `fd`, `fdfind`,
`bfs`, `tree`; `du`, `ncdu`, `tokei`, `cloc`) it lowers its own disk priority with `ioprio_set` (best-effort level 7, the only
`unsafe` code, in `src/priority.rs`) and replaces itself with the real tool, so it adds no process. Started as `search-shadow`
it offers `link DIRECTORY`, which links a name only when a real tool of that name is on `PATH`, and removes a link whose
real tool is gone. With exit code 2 and an explanation it refuses:

- for `rg`, the short `-r` flag anywhere (ripgrep's `--replace`; `rg -rl PATTERN` prints `l` in place of every match). No escape hatch.
- for `rg`, `--hidden`, `--no-ignore*`, `-u` or `-.` with no search path and no piped input at a git repository root.
- for `grep`, `egrep` and `fgrep`, a recursive search with no path or a wide path (repository root, home, filesystem root).
- for `find` and `bfs`, a wide start path without `-maxdepth` of 3 or less.

`SEARCH_SHADOW_ALLOW_WIDE=1` allows one deliberate wide walk (not the `rg -r` ban). The other tools are throttle-only, and `fd`
is never refused. The owner decided on 2026-10-29 that breaking scripts which rely on plain GNU behavior is acceptable.

Checks: `cargo clippy --deny warnings` clean; 45 nextest tests pass (unit tests for each policy plus process-level tests in
which a fake real tool reports `best-effort: prio 7` under the shadow and does not when run alone, grep and find refusals,
and a non-UTF-8 argument passed through byte for byte). `mise run //package/search-shadow/cli:test:container` builds a podman
image holding the real tools this host lacks and checks each shadow against its real tool; see "Where each layer stands" for
its state.

Publication: the dry run passed (20 files, 38.5 KiB compressed). The first real attempt on 2026-10-29 was rejected with
`403 Forbidden: authentication failed`, because the stored token in `~/.cargo/credentials.toml` no longer authorizes publishing
(the same finding as `doc/runbook/publish-crate-first-time.md`), and the registry still answered 404 for the crate.
After the owner created a new token and ran `cargo login`, the publish succeeded: the registry reports `max_stable_version`
0.1.0, license `LGPL-3.0-or-later`, 39,464 bytes. A disposable `cargo install --version 0.1.0` from the registry built and ran
(`search-shadow --version` printed `search-shadow 0.1.0`). A GitHub Actions trusted publisher was created through
`POST /api/v1/trusted_publishing/github_configs` with the token's `trusted-publishing` scope (config id 29471,
`cargo-publish.yml`, no environment), but with the wrong owner: the runbook said `Aquaticat` (id 66041952) and the repository's
owner is `Aquaticats` (id 343218767). The error surfaced when the workflow run URL showed `Aquaticats`; the first token had
meanwhile stopped working (`403` on every call), so the owner made a second token with only the `trusted-publishing` scope,
and with it the entry was replaced (deleted config 29471, created config 29473 with owner `Aquaticats`).
The runbook now names `Aquaticats`. One related line is unchecked:
`.github/workflows/forbidden-strings.yml` passes `--signer-workflow Aquaticat/Monochromatic/...` to `gh attestation verify`,
and a signer-workflow string may not follow a GitHub redirect the way a URL does.

CI: `.github/workflows/cargo-publish.yml` gained `ss-detect`, `ss-publish-crate`, `ss-release-state`, `ss-build-binary` and
`ss-create-release` (commit `0a53556811`), modeled on the nested-wayland-session jobs: a version bump on `main` publishes through
Trusted Publishing and, once the crate's `Cargo.toml` carries `[package.metadata.binstall]`, builds two Linux archives and
creates one immutable release. Version 0.1.1 (commit `7b3d373a07`) carries the metadata and went out through that route:
run `40863841089` succeeded in every job, crates.io lists 0.1.1, the release has both archives and the checksums, and
`cargo binstall --dry-run --disable-strategies compile,quick-install` finds the archive. The plan entry (commit
`b8f5581796`) installs it through `mpm`; see the next section.

## Activation, as verified

The toolchain plan entry `cargo:monochromatic-search-shadow` (`route: cargo-binstall`, Linux only) generated
an `mpm` override pinning `binstall --no-confirm --version 0.1.1`, one line in `mpm.packages.toml`, a lock entry,
and `.cache/search-shadow-bin` as the first `PATH` entry of the generated `mise.toml`.
`mise run toolchain:sync` then reported `1 installed, 61 unchanged, 0 failed` and created
`.cache/search-shadow-bin/rg`, a link to `~/.cargo/bin/search-shadow`.
Checked through `mise exec` in the repository root:

- `command -v rg` resolves to `.cache/search-shadow-bin/rg`; `rg --version` prints ripgrep 15.2.0, the locked tool behind it.
- A search with a named path returns results; `--replace TEXT` works.
- `rg -rl PATTERN FILE` and `rg --hidden PATTERN` (no path, repository root) exit 2 with the explanations;
  `rg --hidden PATTERN .github/...` runs. `SEARCH_SHADOW_ALLOW_WIDE=1` lifts only the wide-walk refusal, not the `-r` ban.
- A running `rg` reports `best-effort: prio 7` under `ionice -p`.

The agent's own Bash tool had inherited its `PATH` before the change, so only new shells (or `mise exec`) resolve the shadow.
The toolchain's `cargo-binstall` provider runs `cargo binstall --no-confirm --version X CRATE` with no compile ban, so a
source-only crate installs by compiling (about 9 seconds measured). The plan entry currently carries one `postinstall.link`
for `rg` (the schema allows one link per entry), and the link directory precedes `.cache/toolchain-bin` in the generated PATH.

For the many-tool version the owner chose a `postinstall.run` kind that runs `search-shadow link .cache/search-shadow-bin`,
so the crate owns the tool list. It is implemented (commit `4dc59d9057`): `PostinstallRun` and `PostinstallStep` in `types.ts`,
`parsePostinstallRun` in `plan.load.guards.ts`, `runPostinstallCommands` in `exposure.ts` (the command starts from the worktree
root with the exposure directory and the toolchain bin directory first on `PATH`; a failure lands in the sync failure
record), and a call in `sync.ts` after the exposure rebuild, because `sync.ts` runs postinstall links before `rebuildExposure`
and `search-shadow link` needs the exposure directory on `PATH` to find the locked `rg`. The plan entry is version 0.2.0 with
`run: { sourceBin: 'search-shadow', args: ['link', '.cache/search-shadow-bin'] }`; the lock and manifests were regenerated in
commit `920e76c1db` by the session that owned the other toolchain edits.

Verified on 2026-10-29 with `mise run toolchain:sync` (3 installed, 59 unchanged, 0 failed): `.cache/search-shadow-bin` holds
nine links (`du`, `egrep`, `fd`, `fgrep`, `find`, `grep`, `rg`, `tokei`, `tree`; only tools whose real binary was on `PATH`).
Through `mise exec` in the repository root, `rg`, `grep`, `find` and `fd` resolve to those links; `rg -rl foo package`,
`grep -rn foo` and `find . -name x` exit 2 with their explanations; `find . -maxdepth 1 -name package` runs; `find --version`
prints GNU findutils 4.10.0. The two new runner tests in `providers.unit.matrix.test.ts` first could not run, because the
container task `test:container:toolchain` failed on `chmod -R a+rX .../package/module`, a path the category rename removed;
commit `37ea4d8fec` fixed that, and the session that owns the toolchain files reported (second hand, quoting the result
lines) that both cells, ubuntu root and ubuntu user, passed the `postinstall command` suite ("2 cells passed").
The sync also installed `monochromatic-lint` 0.4.0 and `monochromatic-git-policy-cli` 0.2.13, because a third session's
version bumps were in the working tree when it ran.

## Git wrapper UX review

Probed on 2026-10-29 with read-only and dry-run commands; the owner is improving the wrapper in another session, so these
are inputs to that work and nothing here was changed.

- Latency: the wrapper adds about 2 to 6 ms per call over `/usr/bin/git` (`status` of one path 12 to 13 ms against 9 to 15 ms;
  `log` 5 to 7 ms against 3 ms; `commit --dry-run` with a pathspec 104 to 106 ms against 102 to 106 ms). A 5.4 s first `status`
  followed a page-cache drop and is git's own cold cost. Real commits took minutes under load: the process list showed
  the shadow-repository `git commit` in uninterruptible disk wait with a `git-lfs filter-process` child, one having read
  340 MB after 16 seconds.
- Rejections are one JSON object on stderr (688 bytes for `add -A`, 450 for a pathspec-less commit). The wording is good:
  cause, remedies, a bypass flag, a hint. Problems: it is machine format for a human or agent reader, with
  `schemaVersion`, `sequence`, `trigger` and `policyId` ahead of the sentence; the two shapes differ (`type: finding` carries
  `policyId`, `severity`, `hint` and `fix`, while `type: core-finding` carries `coreId` and none of those, so the commit-only
  rejection has no hint); the exit code is 1, the same as git's own generic failure; and the commit-only example
  `git commit -m <msg> <path>` uses short flags and omits `--`, against the repository's long-flag rule.
- Silence: a commit that takes minutes prints nothing, which looked like a hang and led to two identical `git commit` processes
  from this session while the first was pending; three shadow commits ran at once, and the wrapper gave no notice that
  an identical commit was already in progress.
- Not measured: `git commit --help` took 7.7 s through the wrapper, and was not compared with the real binary.

## Where each layer stands

The earlier ranking of four alternatives is superseded: the owner decided the layers are complementary, not exclusive, and
required a harness-neutral result.

Delivered and verified:

- BFQ on `nvme0n1`, persistent through `/etc/udev/rules.d/61-nvme-bfq.rules` (the boot-time event is untested).
- `IOWeight=10` on Ghostty terminal scopes (drop-in) and `io.weight` 10 on `claude-code-bash` (watcher). The end-to-end
  check did not show that the weight protects a light reader.
- The `rg` shadow, installed through `mpm` and first on `PATH`, running `rg` at best-effort level 7 and refusing `-r` and
  unbounded hidden walks. `AGENTS.md` rules `RT1` and `RGT` now describe it.

Ruled out: the dm-crypt workqueue flags (no effect on the whole-repo lint) and APST (no measurable latency effect; the
kernel refuses userspace Set Features on it).

Open:

- Read caps on `/dev/nvme0n1`: the bandwidth cap throttles cold buffered reads, the IOPS cap was never verified at a value
  below the delivered rate, and nothing applies either to the agents yet. A cap on `/dev/dm-1` is silently dropped.
- The many-tool shadows: released as 0.2.0 on 2026-10-29 (crates.io lists it; release `monochromatic-search-shadow-v0.2.0`
  holds the x86_64 and aarch64 archives and the checksums; CI run 40871385671). `mise run //package/search-shadow/cli:test:container`
  passes every check against the real ripgrep, grep, find, ugrep, ag, ack, fd, bfs, tree, du, ncdu, tokei and cloc in podman
  (121 s on a warm dnf cache; the first build took 814 s because the Fedora mirror delivered about 60 KiB/s, so the
  `Containerfile` disables the unused openh264 repository and keeps the dnf cache between builds). The test found one real
  defect, fixed before the release: `find --version` with a wide working directory was refused. The `postinstall.run`
  step and the plan entry at 0.2.0 are in place and verified (see "Activation, as verified"). In Claude Code shells `grep`
  and `find` are shell functions running Claude's embedded `ugrep` and `bfs`, so a `PATH` shadow does not reach them.
  The runner's matrix tests pass in the toolchain container matrix (reported by the owning session).
- `/tmp` is a 16 GiB tmpfs with a per-user quota of about 13 GiB that was 98 percent used on 2026-10-29 (hundreds of
  `native-binary-*` directories of about 95 MB and `monochromatic-lint-stub-cargo-*` of 5 MB left by test runs of other
  sessions), which made `git commit` fail with `Disk quota exceeded` while writing its signing buffer. Setting
  `TMPDIR=${HOME}/temp/agent/gittmp` for the commit avoids it; the leaking tests are not fixed.
- Queued, decided by the owner on 2026-10-29: `toolchain:sync` warns when it is about to replace an installed tool with a
  different version. Background: trees share one `~/.cargo/bin`, `isAtLockedVersion` in `sync.ts` observes the shared
  `cargo binstall --list` record and needs an exact match with the lock, so trees pinning different versions of one cargo tool
  (lint 0.3.0 and 0.4.0 that day) overwrite each other on every sync. The warning goes through the tagged logger and names the
  entry id, the observed version and the locked version; one test accompanies it. It waits for the session that owns
  `package/file-enforcer/core/src/toolchain/` to commit its cascade (`observed` may become a symbol sentinel instead of `null`),
  and that session migrates test imports afterwards. The owner declined per-tree cargo installs for now.
- A Claude Code PreToolUse hook for the commands a `PATH` shadow cannot see (not built).
- `.github/workflows/forbidden-strings.yml` passes `--signer-workflow Aquaticat/Monochromatic/...`; whether that matches after
  the move to `Aquaticats` is untested.
- Commit load: each commit runs a whole-worktree shadow `git commit`; the owner is improving the git wrapper separately.
