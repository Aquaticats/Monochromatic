# snapper 0.13.0 `SPACE_LIMIT` and `FREE_LIMIT` do not stop `/var/home` filling

## Symptom

A 1.86 TiB btrfs filesystem holding `/var/home` reached 96 percent used with 86 GiB free,
even though the snapper config for that subvolume set both space-aware limits:

```text
# /etc/snapper/configs/root
QGROUP="1/0"
SPACE_LIMIT="100GiB"
FREE_LIMIT="100GiB"
TIMELINE_CLEANUP="yes"
TIMELINE_LIMIT_HOURLY="0-2"
TIMELINE_LIMIT_DAILY="0-3"
TIMELINE_LIMIT_WEEKLY="0-1"
```

Free space was below the configured `FREE_LIMIT="100GiB"` floor,
and no snapshot was removed.

Three related observations appeared at the same time.

`snapper list` reported `0 B` used space for every snapshot:

```text
# snapper list output, trimmed to the Used Space column
72 │ single │ │ Wed Oct 14 08:00:00 2026 │ root │   0 B │ timeline │ timeline │
73 │ single │ │ Wed Oct 14 09:00:01 2026 │ root │   0 B │ timeline │ timeline │
```

`btrfs` reported the quota group as inconsistent,
and gave the `home` subvolume a referenced
size three orders of magnitude too small:

```text
# btrfs qgroup show -reF /var/home
WARNING: qgroup data inconsistent, rescan recommended
Qgroupid    Referenced    Exclusive  Max referenced  Max exclusive   Path
--------    ----------    ---------  --------------  -------------   ----
0/257        336.00KiB        0.00B            none           none   home
```

Twelve hourly snapshots existed although `TIMELINE_LIMIT_HOURLY="0-2"` allows two.

Deleting data did not reduce usage.
Removing six directories totalling 182 GiB from the live
tree left `btrfs filesystem usage` unchanged,
and `Data,single` used rose from 93.98 to
94.96 percent across the same window.

## Root cause

Four independent mechanisms combine.
Each is named with its deciding source.

### Cleanup runs once a day while snapshots are created every hour

`systemctl list-timers` shows the two snapper timers on different schedules:

```text
# systemctl list-timers --all | grep snapper
Wed 2026-10-14 15:00:00 EDT  Wed 2026-10-14 14:00:03 EDT  snapper-timeline.timer  snapper-timeline.service
Thu 2026-10-15 00:00:00 EDT  Wed 2026-10-14 00:00:00 EDT  snapper-cleanup.timer   snapper-cleanup.service
```

`snapper-timeline.service` only creates snapshots.
`client/systemd-helper/systemd-helper.cc:86`
is the whole of its per-config work:

```cpp
// client/systemd-helper/systemd-helper.cc:86
cout << "Running timeline for '" << value.first << "'." << endl;
```

The cleanup entry point is a separate function,
`client/systemd-helper/systemd-helper.cc:132`:

```cpp
// client/systemd-helper/systemd-helper.cc:132
cout << "Running cleanup for '" << value.first << "'." << endl;
```

The journal confirms the split.
`snapper-timeline.service` logs only `Running timeline for
'root'.` each hour.
`Running cleanup for 'root'.` appears only under `snapper-cleanup.service`.

Both limits are therefore evaluated once per day.
Any threshold crossed between two cleanup
runs stays uncrossed from snapper's point of view until the next midnight.

### `FREE_LIMIT` reads statvfs and was healthy, but saw a value above the floor

`Cleaner::cleanup` runs three passes in order.
`client/cleanup.cc:408` starts with the
count-based pass unconditionally,
then adds the two conditional passes:

```cpp
// client/cleanup.cc:408
Cleaner::cleanup(Plugins::Report& report)
{
    ProxySnapshots& snapshots = cleanable.get_snapshots();
    cleanup(snapshots, report);
    if (is_quota_aware())
        cleanup(snapshots, std::bind(&Cleaner::is_quota_satisfied, this), report);
    if (is_free_aware())
        cleanup(snapshots, std::bind(&Cleaner::is_free_satisfied, this), report);
}
```

The free-space predicate does not touch quota data.
`client/cleanup.cc:322`:

```cpp
// client/cleanup.cc:322
Cleaner::is_free_satisfied() const
{
    FreeSpaceData free_space_data = snapper->queryFreeSpaceData();
    if (free_space_data.size == 0)
        return true;
    bool satisfied = parameters.free_limit.is_satisfied(free_space_data.size, free_space_data.free);
    return satisfied;
}
```

`Snapper::queryFreeSpaceData` is a plain `statvfs` call.
`snapper/Snapper.cc:881`:

```cpp
// snapper/Snapper.cc:881
FreeSpaceData
Snapper::queryFreeSpaceData() const
{
    const Btrfs* btrfs = dynamic_cast<const Btrfs*>(getFilesystem());
    if (!btrfs)
        SN_THROW(FreeSpaceException("free space only supported with btrfs"));
    SDir general_dir = btrfs->openGeneralDir();
    filesystem->sync();
    FreeSpaceData free_space_data;
    std::tie(free_space_data.size, free_space_data.free) = general_dir.statvfs();
    return free_space_data;
}
```

So `FREE_LIMIT` was never affected by the quota problem.
It simply ran at 00:00,
when free
space was still above 100 GiB,
and returned satisfied.
The filesystem crossed below the floor
later in the day and had no second evaluation.

### `SPACE_LIMIT` reads qgroup exclusive bytes and was blinded

The quota predicate does depend on quota data.
`client/cleanup.cc:280`:

```cpp
// client/cleanup.cc:280
Cleaner::is_quota_satisfied() const
{
    QuotaData quota_data = snapper->queryQuotaData();
    if (quota_data.size == 0)
        return true;
    bool satisfied = parameters.space_limit.is_satisfied(quota_data.size, quota_data.used);
    return satisfied;
}
```

Per-snapshot size is the qgroup exclusive value.
`snapper/Snapshot.cc:187`:

```cpp
// snapper/Snapshot.cc:187
subvolid_t subvolid = get_id(openSnapshotDir().fd());
qgroup_t qgroup = calc_qgroup(0, subvolid);
QGroupUsage qgroup_usage = qgroup_query_usage(general_dir.fd(), qgroup);
return qgroup_usage.exclusive;
```

With the qgroup inconsistent,
`exclusive` is meaningless.
That produces both the `0 B` column
in `snapper list` and a `quota_data.used` small enough that `SPACE_LIMIT="100GiB"` is always
satisfied.
`is_quota_aware` at `client/cleanup.cc:258` did not detect this,
because
`prepareQuota()` succeeded:
the journal contains no `quota not working` line,
which is the
message that catch block prints.

### The qgroup is invalidated hourly and the rescan rarely completes

`QGROUP="1/0"` makes snapper create snapshots with qgroup inherit,
which the kernel treats as
invalidating.
The kernel log shows one successful rescan followed by re-invalidation and a
series of paused rescans:

```text
# journalctl -k | grep -i qgroup
Oct 14 00:44:41 bazzite kernel: BTRFS info (device dm-1): qgroup scan completed (inconsistency flag cleared)
Oct 14 01:09:00 bazzite kernel: BTRFS warning (device dm-1): qgroup marked inconsistent, qgroup inherit needs a rescan
Oct 14 01:09:00 bazzite kernel: BTRFS info (device dm-1): qgroup scan paused
Oct 14 13:57:40 bazzite kernel: BTRFS info (device dm-1): qgroup scan paused
Oct 14 14:04:24 bazzite kernel: BTRFS info (device dm-1): qgroup scan paused
```

Snapper's own compensating step is `btrfs qgroup clear-stale`,
and its source predicts the
outcome.
`client/systemd-helper/systemd-helper.cc:191`:

```cpp
// client/systemd-helper/systemd-helper.cc:191
cout << "Running 'btrfs qgroup clear-stale " << general_dir << "'." << endl;
SystemCmd cmd2({ BTRFS_BIN, "qgroup", "clear-stale", general_dir });
if (cmd2.retcode() != 0)
{
    // This fails more often than not since qgroups of just deleted
    // subvolume are busy. So do not set an error code here. Still log the
    // failure to help people understand this stuff.
    cerr << "'btrfs qgroup clear-stale " << general_dir << "' failed." << endl;
}
```

On this system the journal shows the `Running 'btrfs qgroup clear-stale /var/home/.snapshots'.`
line with no following failure line,
so the command returned success and the inconsistency
still persisted.

### Snapshot pinning makes deletion alone ineffective

btrfs frees a block only when no subvolume references it.
Snapshot 73,
created at 09:00:01,
still contained every directory deleted from the live tree afterwards:

```sh
# doc/troubleshooting/snapper-space-limits-not-enforced.md
sudo find /var/home/.snapshots/73/snapshot/user/worktrees \
  -maxdepth 1 -name '.cli-git-worktree-copy-*' | wc -l
```

That returned 6,
the full set.
The same snapshot contained 172 entries under
`user/.cargo/build`.
Deleting the live copies therefore reclaimed nothing until the snapshots
were removed.

### Timeline limits are honored, at the daily run

`TimelineCleaner::calculate_candidates` at `client/cleanup.cc:727` keeps the youngest
`limit_hourly`,
`limit_daily` and `limit_weekly` snapshots and marks the rest as candidates.
It ran,
and it worked.
The journal for `snapper-cleanup.service` records the deletions from
the 00:00 run,
which started at midnight and worked until 01:23:21:

```text
# journalctl -u snapper-cleanup.service
Oct 14 01:11:09 bazzite systemd-helper[221185]: Deleting snapshot from root:
Oct 14 01:11:09 bazzite systemd-helper[221185]: 51
...
Oct 14 01:19:53 bazzite systemd-helper[221185]: Deleting snapshot from root:
Oct 14 01:19:53 bazzite systemd-helper[221185]: 1
Oct 14 01:23:21 bazzite systemd-helper[221185]: Running empty-pre-post cleanup for 'root'.
Oct 14 01:23:21 bazzite systemd-helper[221185]: Running 'btrfs qgroup clear-stale /var/home/.snapshots'.
```

The gaps in snapshot numbering observed in `snapper list` are those deletions.
Twelve hourly
snapshots accumulated because twelve hours elapsed since the previous cleanup,
not because the
limit was ignored.

### Hypotheses this doc disproves

Three readings were held during investigation and are wrong.
They are recorded so the next
investigator does not re-derive them.

`SPACE_LIMIT="100GiB"` and `FREE_LIMIT="100GiB"` are not invalid values.
`snapper-configs(5)`
documents both as `float or size` and states `absolute value new in version 0.9.0`,
and
`snapper get-config` echoes `100GiB` without error.

Cleanup is not never invoked.
`systemctl list-unit-files` shows `snapper-cleanup.timer`
enabled,
and its journal records snapshot deletions.

`FREE_LIMIT` is not blinded by qgroup inconsistency.
`Cleaner::is_free_satisfied` at
`client/cleanup.cc:322` calls `queryFreeSpaceData`,
which is `statvfs` per
`snapper/Snapper.cc:881`.
Only `SPACE_LIMIT` reads qgroup data.

## Verification

Versions under test:

- `snapper-0.13.0-3.fc44.x86_64`,
  reporting `snapper 0.13.0`,
  `libsnapper 8.0.0`,
  flags `btrfs,no-bcachefs,lvm,no-ext4,xattrs,rollback,btrfs-quota,selinux`
- `btrfs-progs-7.1-1.fc44.x86_64`,
  reporting `btrfs-progs v7.1`
- kernel `7.2.9-ogc3.1.fc44.x86_64`
- snapper source tag `v0.13.0`,
  commit `3a3bd97083976d28538d402284ff947b4aab5b8f`,
  matching `VERSION` 0.13.0 and `LIBVERSION` 8.0.0 in the checked-out tree

Reproduce the quota half of the failure:

```sh
# doc/troubleshooting/snapper-space-limits-not-enforced.md
sudo btrfs quota status /var/home
sudo btrfs qgroup show -reF /var/home
sudo snapper --config root list
```

Reproduce the schedule half:

```sh
# doc/troubleshooting/snapper-space-limits-not-enforced.md
systemctl list-timers --all | grep snapper
sudo journalctl -u snapper-cleanup.service --no-pager | tail -25
sudo journalctl -u snapper-timeline.service --no-pager | tail -10
```

Reproduce the pinning half on any snapshot taken before a large deletion:

```sh
# doc/troubleshooting/snapper-space-limits-not-enforced.md
sudo find /var/home/.snapshots/<N>/snapshot/user -maxdepth 2 -name '<deleted-name>'
```

Catalog of readings that are trustworthy:

- `btrfs filesystem usage <path>` reports `Used` and per-profile chunk usage independent of
  qgroup state,
  and tracked the reclaim correctly.
- `df` on the subvolume mount agrees with `btrfs filesystem usage`,
  to its whole-gigabyte
  rounding.
- `journalctl -k` records `qgroup scan completed (inconsistency flag cleared)` and
  `qgroup marked inconsistent`,
  which is the authoritative account of quota state changes.
- `find` under a snapshot path lists that snapshot's real contents.

Catalog of readings that are not:

- The `Used Space` column of `snapper list` reads 0 B for every snapshot while the qgroup is
  inconsistent,
  because it is `qgroup_usage.exclusive` per `snapper/Snapshot.cc:187`.
- `btrfs qgroup show` reports `WARNING: qgroup data inconsistent` and returns numbers that
  contradict `btrfs filesystem usage`.
  It reported `home` at 336.00 KiB,
  then 49.85 GiB,
  then 101.01 GiB across successive reads,
  while the subvolume held roughly 600 GiB.
- `btrfs quota rescan --wait <path>` prints `quota rescan started` and returns while the scan
  is only paused.
  `btrfs quota rescan --status <path>` then reports `no rescan operation in
  progress` with `Inconsistent: yes` still set,
  so a zero exit status does not mean the
  rescan completed.

## Verified workarounds

Deleting the pinning snapshots reclaimed the space.
Measured on this system:
available space
went from 86 GiB at 96 percent used to 1057 GiB at 39 percent used,
and `btrfs filesystem
usage` went from `Used: 1.64TiB` to `Used: 659.18GiB`.
Every snapshot referencing the deleted
data had to go;
removing a subset reclaimed nothing,
because a block stays allocated while any
subvolume references it.
Tradeoff:
snapshot history for the covered window is lost,
and there
is no restore point between the deletion and the next snapshot.

```sh
# doc/troubleshooting/snapper-space-limits-not-enforced.md
sudo snapper --config root delete --sync <number>
sudo snapper --config root create --type single --cleanup-algorithm timeline \
  --description "baseline"
```

Running cleanup more often than daily makes `FREE_LIMIT` able to react within the hour instead
of within the day.
Add a systemd timer override or an equivalent unit invoking
`/usr/lib/snapper/systemd-helper cleanup`.
Tradeoff:
each run performs `btrfs qgroup
clear-stale` and subvolume deletions,
which re-mark the qgroup inconsistent and add I/O.

Setting `QGROUP=""` in the snapper config stops snapshot creation from using qgroup inherit,
which removes the hourly re-invalidation.
Tradeoff:
`SPACE_LIMIT` becomes permanently
inoperative,
since `is_quota_aware` at `client/cleanup.cc:258` returns false without a qgroup,
so only `FREE_LIMIT` and the count limits remain.
On this system that is a net gain,
because
`SPACE_LIMIT` was already inoperative through bad data.

Moving unbounded caches off the snapshotted subvolume removes the growth that the limits were
failing to contain.
Applied here to Cargo intermediate artifacts,
whose configured
`build.build-dir` had accumulated 438 GiB:

```toml
# ~/.cargo/config.toml
[build]
build-dir = "/var/mnt/encrypted/cargo-build/{workspace-path-hash}"
```

Tradeoff:
the target filesystem must be mounted whenever a build runs,
and generated dep-info
embeds the absolute build directory path,
so relocating invalidates the previous cache rather
than migrating it.

## What does not work

`btrfs quota rescan --wait /var/home` did not clear the flag.
Four attempts returned
`quota rescan started` with exit status 0,
and `btrfs quota status` still reported
`Inconsistent: yes (rescan needed)` after each.
The kernel logged `qgroup scan paused` at the
attempt timestamps.
The only observed completion was at 00:44:41,
which fell inside the
00:00 to 01:23 run of `snapper-cleanup.service`,
so quiet system activity does not explain
why that one completed and these did not.
All four ran while container and build activity was ongoing on the same filesystem:
a `cargo-mutants` container,
`podman` pulls,
and rootless container storage churn.
An idle window is the untried variable.
The difference is not established.

Raising `FREE_LIMIT` alone does not help while cleanup stays daily,
because the value is only
read at cleanup time.

Editing `/etc/snapper/configs/root` has no effect while `snapperd.service` is running.
The daemon serves the config it loaded at startup,
so the file and the effective value diverge silently.
Setting `FREE_LIMIT="250GiB"` in the file left `snapper --config root get-config`
reporting `100GiB`,
and it reported `250GiB` only after `systemctl restart snapperd.service`.
The file held exactly one `FREE_LIMIT` line throughout,
so this was caching and not a duplicate key.
Confirm any config change with `snapper --config root get-config` rather than by reading the
file.

Trusting `snapper list` `Used Space` to decide whether snapshots are worth deleting is
unsound while the qgroup is inconsistent.
Every snapshot reads 0 B regardless of what it pins.

Deleting files from the live tree without also removing the snapshots that reference them
reclaims nothing and still costs metadata churn.
`Metadata,DUP` used rose from 34.90 GiB to
38.12 GiB during a window in which 182 GiB of live files were removed and no space returned.

## Upstream filing decision

`.out-of-scope/` was checked.
The directory holds `bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
and `typescript-project-references.md`.
None exempts
snapper,
btrfs-progs,
or quota behaviour.
`cargo-workspace.md` concerns the absence of a
repository-root Cargo workspace and is unrelated to `build.build-dir`.

The tracker was searched with `gh search issues --repo openSUSE/snapper "FREE_LIMIT"`,
which
returned nine results,
and `gh search issues --repo openSUSE/snapper "qgroup inconsistent"`,
which returned none.
The closest match is openSUSE/snapper#967,
`The man page of SPACE_LIMIT
and FREE_LIMIT is confusing`,
closed.
That issue concerns the wording of the manual page.
It
does not cover the evaluation frequency of the two limits,
and it does not cover `SPACE_LIMIT`
being silently inoperative when the qgroup is inconsistent,
so it is related rather than a
duplicate.

The six constraints,
walked:

1. **Is it really upstream's fault?**
   No. The daily cleanup schedule comes from the
   distribution's `snapper-cleanup.timer`,
   not from snapper's code.
   Reading `FREE_LIMIT` only
   at cleanup time is the documented consequence of that design,
   and the fix is local
   scheduling or local configuration.
2. **Can upstream fix it?**
   Partially,
   and only for a different complaint.
   snapper cannot make
   a distribution timer fire more often.
   It could surface that `SPACE_LIMIT` is inoperative
   while the qgroup is inconsistent,
   instead of printing `0 B` and proceeding.
3. **Are they supporting this use case?**
   Yes.
   Both limits are documented features in
   `snapper-configs(5)`,
   including absolute sizes since 0.9.0.
4. **Would the repo welcome our contribution?**
   Not established.
   No `CONTRIBUTING.md` policy
   check was run for this doc,
   and no filing is proposed,
   so this constraint is unevaluated
   rather than failed.
5. **Will they likely fix it?**
   Unknown,
   and not gated here because constraint 1 already
   fails.
   openSUSE/snapper#967 shows maintainers do engage on these two settings.
6. **Have we prototyped a minimal fix?**
   No. Constraint 1 fails,
   so the auto-prototype step
   does not trigger.
   The effective fix on this system is configuration and scheduling,
   both
   recorded under "Verified workarounds".

Decision:
do not file.
Constraint 1 fails,
which is sufficient under the default policy.
If a
future session wants to pursue the narrower complaint that `SPACE_LIMIT` silently reports `0 B`
instead of warning when `btrfs quota status` says `Inconsistent: yes`,
re-evaluate constraints
2,
4,
5 and 6 for that specific framing,
and check openSUSE/snapper#564,
`quota not working (preparing quota failed)`,
for overlap first.

Nothing to add to openSUSE/snapper#967.
That thread is about manual-page wording and is
closed.
The findings recorded here concern evaluation frequency and quota blindness,
which are
different claims,
and posting them there would not advance it.
