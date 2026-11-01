# btrfs-progs `scrub start --limit` resets the limit before scrub completion

## Symptom

This command accepted and printed a 100 MiB/s limit:

```sh
# doc/troubleshooting/btrfs-scrub-start-limit-reset.md
sudo btrfs scrub start -B --limit 100M /var/mnt/encrypted
```

While the scrub remained active,
`btrfs scrub limit /var/mnt/encrypted` reported no limit
and `/sys/fs/btrfs/01c308e7-06fa-4737-8a7b-3bb5fcba871d/devinfo/1/scrub_speed_max` contained `0`.
The scrub read about 531 MiB/s rather than the requested rate.

The separate live command worked:

```sh
# doc/troubleshooting/btrfs-scrub-start-limit-reset.md
sudo btrfs scrub limit --all --limit 100M /var/mnt/encrypted
```

The sysfs value then became `104857600`.
Subsequent block samples read 98.61 to 102.13 MiB per second.

## Root cause

The installed `btrfs-progs` is version 7.1.
Its exact source is tag `v7.1`,
commit `4ab0e80be9e3bb1db2e6038e6d4316d35fb7ba8b`.
The same ordering remains in upstream commit `6797ce7600556138081382441bbc6104f35736e2`.

`btrfs scrub start` initializes `throughput_limit` to zero.
It saves every previous per-device value and writes `throughput_limit`
even when the command has no `--limit` option.
It then starts one blocking scrub thread per device.

The join loop restores each device before joining that device's thread:

```c
for (i = 0; i < fi_args.num_devices; ++i) {
        ret = write_scrub_device_limit(fdmnt, di_args[i].devid, sp[i].old_limit);

        if (sp[i].skip)
                continue;
        ret = pthread_join(t_devs[i], NULL);
}
```

On a single-device filesystem,
the saved value returns immediately while the kernel scrub continues.
On a multi-device filesystem,
the first value returns before the first join
while later devices retain the temporary value until preceding joins finish.
Without `--limit`,
that temporary value is zero.

This ordering explains both observed forms:

- A single-device `scrub start --limit 100M` becomes unlimited during scrub.
- A multi-device `scrub start` can leave later devices unlimited while an earlier device retains its configured limit.

## Refined fix

The [prototype patch](btrfs-scrub-start-limit-reset.patch) implements explicit ownership of temporary limits:

- Do not read or write scrub limits unless `--limit` was supplied.
- Change a device only after its previous value was read successfully.
- Mark ownership only after writing the temporary value successfully.
- Keep owned values active until all scrub threads have joined.
- Transfer restoration responsibility from the daemonizing parent to its child.
- Restore owned values on normal completion,
  setup errors,
  and interrupt cleanup.
- Before restoring,
  confirm the live value still equals the temporary value.
  A different value is treated as an operator's live override and preserved.

The final comparison and restore use separate sysfs operations.
The kernel does not expose compare-and-swap for this value,
so an operator change landing between those operations could still be overwritten.
The patch narrows that race but cannot make restoration atomic.

## Verification

### Versions and source

```text
btrfs-progs v7.1
kernel 7.2.0-ogc6.1.fc44.x86_64
installed source 4ab0e80be9e3bb1db2e6038e6d4316d35fb7ba8b
prototype base 6797ce7600556138081382441bbc6104f35736e2
validated prototype head 1fa218e893c6ec71db95199535111454413b2818
```

The prototype built successfully in a container limited to 2 GiB of memory and 2 CPUs.
A rootless container could not allocate loop devices,
so the functional CLI test ran on disposable host loop devices.

The final `tests/cli-tests/024-scrub-limit/test.sh` fixture used:

- Four 2 GiB sparse loop images.
- Btrfs `Data,RAID1` and `Metadata,RAID1` profiles.
- A committed 64 MiB file.
- Preconfigured values of 16 or 30 MiB/s.
- Temporary values of 4 or 16 MiB/s.

`sync FILE` alone allowed an immediate scrub to discover only 320 KiB.
The test therefore runs `btrfs filesystem sync` to commit the fixture transaction before scrub discovery.

The fixed implementation passed the complete upstream CLI case in 28 seconds:

```text
make TEST=024-scrub-limit test-cli
exit status: 0
```

The identical test with only `cmds/scrub.c` restored to unpatched upstream failed in two seconds:

```text
scrub limit is 0, expected 16777216
test failed for case 024-scrub-limit
exit status: 2
```

The fixed control verifies these paths:

- A foreground scrub without `--limit` preserves configured values during and after scrub.
- A foreground scrub with `--limit` holds the temporary value and restores prior values after completion.
- A daemon scrub holds its temporary value after parent exit and restores prior values when the child finishes.
- A live `btrfs scrub limit` override remains after the originating scrub finishes.
- An interrupted foreground scrub restores prior values.

An earlier manual four-device control also sampled only while kernel status was `running`:

```text
unpatched: scrubStatus=0 runningSamples=35 mismatch=1
fixed:     scrubStatus=0 runningSamples=265 mismatch=0
```

The disposable filesystems unmounted,
all loop devices detached,
and retained image files were removed after every control.

### Code-reviewed paths

Per-device flags make partial setup deterministic:
failed old-value reads and failed temporary writes are not treated as owned values.
The shared `out` path retries restoration after later setup failures.
These injected failure branches were inspected but were not forced in the loop-device test.

## Verified workarounds

Apply the limit after starting scrub:

```sh
# doc/troubleshooting/btrfs-scrub-start-limit-reset.md
sudo btrfs scrub start -B /mountpoint
sudo btrfs scrub limit --all --limit 100M /mountpoint
```

For a foreground scrub,
run the start command in one process and the limit command from another process.

Tradeoffs:

- The live command changes persistent-in-mount sysfs state rather than tying restoration to one scrub invocation.
- Record every previous device value and restore it explicitly after scrub completion.
- A process that exits before restoration can leave the live limit active
  until it is changed or the filesystem unmounts.

The production scrub used this workaround.
It checked 1.42 TiB in 3:28:01 and reported no errors.
The live limit was then restored from 100 MiB/s to its prior sysfs value of zero.

## What does not work

### Trusting the startup line

`Starting scrub on devid 1 (limit 100.00MiB/s)` reports the value read immediately after it was written.
It does not prove the value remains active while the kernel scrub runs.

### Sampling the whole userspace process lifetime

Process lifetime includes valid setup and teardown transitions.
Foreground assertions must begin after kernel status becomes `running`.
A short daemon scrub may finish before `btrfs scrub status` exposes that state,
so the daemon test directly checks its temporary sysfs value and waits for restoration.

### Using `btrfs check --repair`

This bug controls scrub throughput.
It is unrelated to filesystem structural repair,
and `btrfs check --repair` cannot correct userspace limit ownership.

## Existing upstream report

Torstein Eide reported the same multi-device symptom to `linux-btrfs` on 2026-04-20:

- Device 1 remained at 100 MiB/s.
- Devices 2 through 4 became unlimited during scrub.
- All values returned to 100 MiB/s after scrub.
- The behavior reproduced with `btrfs-progs` 7.0 and 6.12.

The [mailing-list thread][existing-report] still has no reply.
This is stronger prior art than issue 943 because it reports the defect rather than the feature request.

The defect originated in commit
[`1b28dd73de42fae29e2fc18dde1e70d6daedecbd`][introducing-commit],
which added `scrub start --limit` for [issue 943][issue-943] through [pull request 947][pr-947].

## Upstream filing assessment

A new report without a patch would duplicate the existing mailing-list report.
The refined fix and regression test now provide the missing actionable evidence.

### Patch email

Benefits include following `Documentation/dev/Developer-s-FAQ.rst`'s default code-review path
and linking the existing report while including the complete fix and test in one logical change.

Drawbacks include required author attribution,
a Developer Certificate of Origin sign-off,
email patch formatting,
and possible review revisions.

### GitHub pull request

Benefits include explicit support in `README.md`,
GitHub pull-request checks,
and code review attached to the diff.

Drawbacks include required attribution,
likely review revisions,
and context split from the existing mailing-list report unless linked carefully.

### Report-only follow-up

The benefit is adding the confirmed root cause and workaround with little submission ceremony.

The drawbacks are leaving maintainers to reconstruct an already validated patch
and duplicating evidence without delivering the fix.

My ranking is patch email > GitHub pull request > report-only follow-up.
Patch email ranks over a pull request because the developer FAQ makes mailing-list review the default
and an existing report already anchors the discussion.
A pull request ranks over a report-only follow-up because it delivers tested code.

No external message or patch has been posted.
Before submission,
the human author must review the patch,
choose the author identity,
and add their own `Signed-off-by` line.
The existing report should be linked and its author copied;
no `Reported-by` tag should be asserted without confirming that attribution.

### Patch changelog draft

```text
Subject: [PATCH] btrfs-progs: scrub: keep temporary limit until completion

scrub_start() restores each device's previous throughput limit before
joining that device's scrub thread. A single-device scrub therefore runs
without the requested limit. On multi-device filesystems, later devices
can retain the temporary zero value while an earlier thread is joined.
The function also writes zero when --limit was not supplied.

Track whether --limit was supplied and whether each temporary value was
successfully installed. Restore owned values after all scrub threads join
and from error cleanup. Leave a value unchanged when an operator replaced
the temporary value while scrub was active. Transfer restoration ownership
to the daemon child after fork.

Extend 024-scrub-limit to cover no-option preservation, foreground and
daemon restoration, a live operator override, and interrupt cleanup.

Fixes: 1b28dd73de42 ("btrfs-progs: scrub: add the new --limit option to set the throughput limit at runtime")
Link: https://lore.kernel.org/linux-btrfs/CAL5DHTHA8a=BemS5mVLdvt5CT+DVgegMFftdMqyk+=B60KFAsA@mail.gmail.com/
```

[existing-report]: https://lore.kernel.org/linux-btrfs/CAL5DHTHA8a=BemS5mVLdvt5CT+DVgegMFftdMqyk+=B60KFAsA@mail.gmail.com/T/
[introducing-commit]: https://github.com/kdave/btrfs-progs/commit/1b28dd73de42fae29e2fc18dde1e70d6daedecbd
[issue-943]: https://github.com/kdave/btrfs-progs/issues/943
[pr-947]: https://github.com/kdave/btrfs-progs/pull/947
