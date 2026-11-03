# systemd 262.1 scheduler assignments hide BFQ from later attribute matches

## Host configuration recorded at 2026-11-03T01:00:37-05:00

The user requested permanent BFQ selection for all I/O devices.
On Bazzite `testing-45.20261102.4`,
kernel `7.2.15-ogc1.1.fc45.x86_64`,
`lsblk --nodeps --output NAME,TYPE,SCHED` initially reported `sda` using `kyber`
and `nvme0n1` using `bfq`.
`modinfo bfq` reported `filename: (builtin)`.
No module installation or bootloader change was needed.

The installed policy is `/etc/udev/rules.d/99-bfq.rules`:

```udev
# /etc/udev/rules.d/99-bfq.rules
# Prefer BFQ for every disk that offers it, including future hotplug devices.
# Read sysfs directly: earlier scheduler assignments replace udev's cached value.
# Preserve the distribution's scheduler requirement for host-managed zoned disks.
ACTION=="add|change", SUBSYSTEM=="block", ENV{DEVTYPE}=="disk", \
  TEST=="queue/scheduler", ATTR{queue/zoned}!="host-managed", \
  PROGRAM=="/usr/bin/grep --quiet --word-regexp bfq /sys%p/queue/scheduler", \
  ATTR{queue/scheduler}="bfq"
```

The file is owned by `root:root`,
mode `0644`,
with SELinux type `udev_rules_t`.
The previous `/etc/udev/rules.d/61-nvme-bfq.rules` was backed up to
`/var/backups/bfq-20261103/61-nvme-bfq.rules`
and removed from the active rules directory.
That previous rule matched only `nvme0n1`.

The policy applies on device addition and change,
including the boot coldplug performed by the installed `systemd-udev-trigger.service`.
It covers scheduler-capable drives regardless of their kernel names or transports.
The current `dm-0`,
`dm-1`,
`dm-2`,
and `zram0` expose no `queue/scheduler` file.
Their absence from scheduler selection is intentional;
the encrypted devices' underlying physical disks use BFQ.
Host-managed zoned disks are excluded to preserve the requirement expressed in
`/usr/lib/udev/rules.d/64-btrfs-zoned.rules`.
Neither currently attached physical disk is zoned.

## Symptom

An initial candidate rule guarded assignment with
`ATTR{queue/scheduler}=="*bfq*"`.
`udevadm verify` accepted it,
but `udevadm test --action=add` left the simulated `sda` scheduler as `kyber`.
Reading `/sys/block/sda/queue/scheduler` directly still showed BFQ as available.

The installed `/usr/lib/udev/rules.d/60-schedulers.rules` selects Kyber for SATA SSDs and NVMe disks.
The initial candidate's availability check ran after those assignments.

## Root cause

Source was inspected in a read-only sparse clone of `systemd/systemd`
at commit `a8ed5bc8e53bb665bcb3b1e5878960f984ee8c6a`.
This is an upstream source reference;
the runtime behavior was measured using the installed systemd `262.1` build.

[`src/libsystemd/sd-device/sd-device.c:2752`][write-cache]
writes an attribute and caches the supplied string.
The relevant write and cache calls are:

```c
// systemd/src/libsystemd/sd-device/sd-device.c:2781
r = write_string_file_fd(fd, copied, WRITE_STRING_FILE_DISABLE_BUFFER | WRITE_STRING_FILE_AVOID_NEWLINE);
```

```c
// systemd/src/libsystemd/sd-device/sd-device.c:2788
r = device_cache_sysattr_value(device, resolved, copied, /* error= */ 0);
```

[`src/libsystemd/sd-device/sd-device.c:2546`][read-cache]
returns a cached attribute before reading sysfs:

```c
// systemd/src/libsystemd/sd-device/sd-device.c:2546
/* Look for possibly already cached result. */
r = device_get_cached_sysattr_value(device, sysattr, ret_value, ret_size);
if (r != -ENOANO)
        return r;
```

Consequently,
an assignment of `kyber` can make the subsequent attribute match see `kyber`,
instead of the kernel's list of supported schedulers.

[`src/udev/udev-rules.c:3080`][test-cache]
also maintains this cache during simulation,
while skipping the actual sysfs write:

```c
// systemd/src/udev/udev-rules.c:3083
/* We assume the attribute is writable if the path points to a regular file, and cache
 * the value to make it shown by OPTIONS="dump" or obtained by later ATTR match token. */
```

The initial assumption that an `ATTR` match always rereads the kernel's scheduler list was incorrect.
The direct `grep` invocation in the installed rule reads that list independently of udev's attribute cache.
The source selections were opened and their quoted text confirmed with `agent-browser`.

## Verification

`udevadm verify` reported success for the final rule.
The following read-only checks were run against the installed configuration:

```sh
# Read-only rule and kernel checks
udevadm verify /etc/udev/rules.d/99-bfq.rules
/usr/bin/grep --quiet --word-regexp bfq /sys/block/sda/queue/scheduler
/usr/bin/grep --quiet --word-regexp bfq /sys/block/nvme0n1/queue/scheduler
lsblk --nodeps --output NAME,TYPE,SCHED
cat /sys/block/sda/queue/scheduler /sys/block/nvme0n1/queue/scheduler
```

The working catalog:

- The candidate rule's `sda` add simulation ended with `queue/scheduler : bfq`.
- Its `nvme0n1` change simulation ended with `queue/scheduler : bfq`.
- Both direct `grep` checks succeeded.
- The policy was applied through the real udev daemon:
  rules were reloaded,
  then a settled change event targeted `sda` and `nvme0n1`.
- Live readback showed `none mq-deadline kyber [bfq]` for both physical disks.
- Add simulation for `zram0` and change simulation for `dm-0` did not assign a scheduler.

The rejected catalog:

- The original late `ATTR{queue/scheduler}=="*bfq*"` guard failed for `sda`
  after the vendor Kyber assignment.
- `udevadm trigger --attr-match=queue/scheduler` exited with
  `queue/scheduler is not a valid key name`.
  Explicit `--sysname-match=sda --sysname-match=nvme0n1` filters succeeded.
- An unprivileged simulation logged permission errors from device-identification helpers.
  Privileged simulations removed those unrelated access failures.

Simulation has a material limit:
the installed `udevadm test` logged that it skipped execution of the rule's `grep` program.
Its simulated assignment alone does not validate that program's result.
The direct program invocations and the real daemon application supplied that evidence.

The final policy was activated with:

```sh
# Authorized host configuration application, already performed
sudo --non-interactive udevadm control --reload
sudo --non-interactive udevadm trigger --action=change --subsystem-match=block \
  --sysname-match=sda --sysname-match=nvme0n1 --settle --verbose
```

No reboot,
physical hotplug,
host-managed zoned disk,
or future kernel was exercised.
Persistence is configured through the installed local udev rule and verified boot coldplug configuration;
reboot survival was not directly tested.
The active TuneD profile was `balanced-bazzite`,
which includes `balanced`;
neither configured an elevator override.

## Verified workaround and limits

The installed rule reads sysfs through `/usr/bin/grep`
and assigns BFQ only when the live scheduler list contains the word `bfq`.
The tradeoff is a short process invocation per eligible add or change event.
The rule depends on `/usr/bin/grep` and a kernel offering BFQ.
It preserves the installed zoned-disk restriction.
Devices without a selectable block scheduler remain outside BFQ selection.

The previous NVMe-specific rule is retained in the backup directory for recovery.
No background watcher or custom service was installed.

## What does not work

Syntax validation alone does not establish that a late attribute condition will match.
Using the cached scheduler attribute as a capability list after another rule has assigned it is unreliable.
A simulation that skips `PROGRAM` execution does not establish its exit status.
Using a slash-containing attribute name with this `udevadm trigger --attr-match` failed.

## Upstream filing decision

- Fault attribution:
  no upstream defect is established.
  The rejected candidate depended on an incorrect assumption about attribute caching.
- Fixability:
  no upstream patch is needed for the requested host policy.
- Supported use:
  installed `udev(7)` documents `PROGRAM`,
  `ATTR`,
  and local rules;
  the applied policy succeeded.
- Contribution policy:
  not assessed because no contribution is proposed.
- Maintainer intent:
  not assessed;
  no claim about future upstream behavior is made.
- Prototype:
  the local configuration was verified,
  and no upstream source was modified.

No matching exemption was found in `.out-of-scope/`.
GitHub issue and pull-request searches scoped to `systemd/systemd`
with `udev scheduler cache` returned no results on `2026-11-03`.
Those narrow searches do not establish the absence of related reports.
There is nothing to file from this configuration change,
and no external issue or comment was sent.

[write-cache]: https://github.com/systemd/systemd/blob/a8ed5bc8e53bb665bcb3b1e5878960f984ee8c6a/src/libsystemd/sd-device/sd-device.c#L2752-L2797
[read-cache]: https://github.com/systemd/systemd/blob/a8ed5bc8e53bb665bcb3b1e5878960f984ee8c6a/src/libsystemd/sd-device/sd-device.c#L2546-L2549
[test-cache]: https://github.com/systemd/systemd/blob/a8ed5bc8e53bb665bcb3b1e5878960f984ee8c6a/src/udev/udev-rules.c#L3080-L3095
