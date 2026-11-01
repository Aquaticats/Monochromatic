# Android Emulator 37.1.11 rejects a private Fold AVD after a forced container stop

## Symptom

The design-only `Fold_No_Hardware_Probe` AVD had been running in a
6 GiB/2 CPU Podman container.
`podman stop --time 20` warned that `SIGTERM` did not stop the emulator
within 20 seconds and resorted to `SIGKILL`.
A later boot attempt of that **same disposable AVD** exited with code `1`:

```text
FATAL | Running multiple emulators with the same AVD is an experimental feature.Please use -read-only flag to enable this feature.
```

This was emitted by Android Emulator 37.1.11.0 before guest boot,
not by Gboard or the music-player prototype.
It is distinct from the host `qemu_thread_create` failure recorded in
[fedora-44-fold-emulator-user-task-ceiling.md](fedora-44-fold-emulator-user-task-ceiling.md).
The `-read-only` suggestion in the fatal message does not meet a writable
disposable-guest test's needs.

## Root cause boundary

The local Android 37.1.11 build is not mapped to a public source commit.
The nearby public `emu-master-dev` source in
[`android-qemu2-glue/main.cpp`][emulator-main] at lines `1844` to `1873`
shows a writable-instance check after the snapshot lock:

```cpp
const char* coreHwIniPath = avdInfo_getCoreHwIniPath(avd);
// The read-only branch uses a temporary INI instead.
if (opts->read_only) {
    TempFile* tempIni = tempfile_create();
    coreHwIniPath = tempfile_path(tempIni);
} else if (!opts->check_snapshot_loadable &&
           filelock_create_timeout(coreHwIniPath, 2000) == NULL) {
    derror("Running multiple emulators with the same AVD ");
    derror("is an experimental feature.");
    derror("Please use -read-only flag to enable this feature.");
    return 1;
}
```

That path identifies an unsuccessful lock acquisition as the branch emitting
the observed fatal message in the available source.
It does **not** prove which condition made the installed binary's lock
acquisition fail.
A [separate Android automation report][comparable-issue] describes stale
`hardware-qemu.ini.lock` after a killed architecture probe on other emulator
versions and AVDs;
that is comparable precedent,
not a reproduction of this container failure.

The private AVD directory contained an empty `multiinstance.lock` and a
three-byte `hardware-qemu.ini.lock`,
both with modification times predating this failed restart.
Before moving them,
`adb devices -l` found no device,
`pgrep` found no process for this AVD,
`podman ps` showed no emulator container,
and `lsof`,
`fuser` and `lslocks` reported no owner of either path.
`lslocks` did report unrelated active locks,
so its empty AVD match was not caused by an entirely empty lock listing.
These checks bound the risk of relocating **this private fixture's** files;
they do not establish how the packaged emulator interprets every lock or
rule out a different hidden owner in another environment.

[emulator-main]: https://android.googlesource.com/platform/external/qemu/+/refs/heads/emu-master-dev/android-qemu2-glue/main.cpp
[comparable-issue]: https://github.com/kaeawc/auto-mobile/issues/5202

## Verification

- **Failed case:**
  Starting `Fold_No_Hardware_Probe` under the same
  `podman run --memory=6g --cpus=2` fixture with both lock files present
  emitted the quoted FATAL and returned code `1` before ADB connected.
- **Passing case:**
  Both exact files were moved to private scratch rather than deleted:
  `/home/user/temp/agent/fold-avd-lock-backup.zQiURuZu/`.
  Repeating the same bounded container command then reported
  `ADB_BOOT_READY emulator-5580`.
  Its guest reported AVD name `Fold_No_Hardware_Probe`,
  unfolded device state `2`,
  font scale `2.0`,
  and selected Gboard.
  Launching the existing debug Search activity returned `Status: ok`;
  UI Automator saw the real `Folders` browser,
  Open,
  `cam` field and complete keyboard-closed playback mode.
- The paired before/after run proves **joint relocation** recovered this
  AVD's launch.
  It does not isolate which lock file was decisive or establish that
  `SIGKILL` alone created an invalid lock.

No original `Pixel_9_Pro_Fold` AVD path was modified or launched for this
recovery.
The failing container was removed on exit;
the passing container retained the 6 GiB/2 CPU bounds.

## Later bounded shutdown without a forced stop

On a later design-only visit,
the same disposable AVD booted within an inspected 6 GiB/2 CPU Podman cap.
Android Emulator 37.1.11.0 then rejected the host's
`adb -s emulator-5580 emu kill` console command:

```text
KO: authentication token does not match ~/.emulator_console_auth_token
KO: unknown command, try 'help'
```

This diagnostic names an authentication mismatch;
it does not establish which container or host token was responsible.
No token was copied,
modified or requested.
`podman stop --time 60 fold-e2-native-variants` then completed in 3 seconds
without the earlier observed forced-kill warning.
`adb devices -l` no longer listed the emulator,
and `podman ps` no longer listed this container.
The original AVD was not touched.
The longer graceful stop is a measured outcome of that visit,
not proof that every future emulator exit leaves no stale locks.

A later static Search-status capture on the same disposable AVD completed
without opening an IME.
`podman stop --time 60 fold-search-empty-review-avd` then warned:

```text
StopSignal SIGTERM failed to stop container fold-search-empty-review-avd in 60 seconds, resorting to SIGKILL
```

The Podman process exited `137`;
`podman ps` and `adb devices -l` then showed no live emulator container or
device.
No subsequent reboot or AVD-lock ownership check was attempted during
this capture session,
so it does not establish whether writable restart is affected.
Do not infer that the earlier three-second graceful exit is reliable,
or relocate a lock merely because this later stop was forced;
the owner checks described in this document are still required before a
writable retry.

## TalkBack-study restart recurrence and console shutdown

A later 6 GiB/2 CPU `Fold_No_Hardware_Probe` TalkBack study reproduced
the writable-startup FATAL before guest boot.
The container named `fold-search-talkback-review-avd` returned code `1`.
ADB,
Podman,
process listing,
`lsof`,
`fuser` and `lslocks` found no owner of the disposable AVD's exact lock
paths before they were jointly moved to private backup
`/home/user/temp/agent/fold-search-talkback-lock-backup.ugdNRZd1/`.
The same bounded launch in `fold-search-talkback-review-avd-retry`
then reached `ADB_BOOT_READY emulator-5580`.
Inspection reported memory `6442450944` bytes and NanoCPUs
`2000000000`.
This is a further bounded failing/passing pair for joint lock relocation,
not identification of which lock failed or proof of a forced-stop cause.

At study end,
the disposable guest's speech overlay was verified disabled,
Speak usage hints restored enabled,
and `accessibility_enabled=0` with
`enabled_accessibility_services=null`.
Shutdown used the owning container's ADB context:

```text
podman exec fold-search-talkback-review-avd-retry /home/user/Android/Sdk/platform-tools/adb -s emulator-5580 emu kill
OK: killing emulator, bye bye
OK
```

Immediately after that accepted command,
the container and ADB device were still present.
A later owner check found no matching container,
ADB device or `Fold_No_Hardware_Probe` emulator process.
No `podman stop` or forced kill was needed in this visit.
The automatically removed container's final exit status/log was not
retained,
so console acceptance plus eventual disappearance is the measured
shutdown evidence,
not a stronger assertion about exit code or every saved guest setting.
No subsequent restart was attempted and no further locks were moved.
The original AVD remained untouched.
See [the console-token boundary](android-emulator-console-token-container-home.md)
for why the owning container's ADB avoids the previously observed host
authentication mismatch.

## Recurrence after an emulator crash in an owned Fold container (2026-10-05)

Android Emulator 37.2.12.0 (build_id 16428233) ran the owned AVD copy
`Pixel9ProFold_Fresh_5FXdOc` in the 6 GiB/2 CPU container
`music-player-settings-pane-fold`.
The emulator received `SIGSEGV` mid-session,
so no shutdown ran;
see [the separate crash record](android-emulator-37-software-renderer-sigsegv.md).
The next writable boot of that same AVD copy returned the quoted FATAL and
exited `1` before guest boot.

The AVD directory held an empty `multiinstance.lock` and a three-byte
`hardware-qemu.ini.lock`,
both last modified when the crashed boot started.
The three bytes were ASCII `31` and a NUL.
The emulator logged itself as `pid 31` inside the container in every owner
log of that session that reached a graceful shutdown.
That is consistent with a process-ID liveness check passing on a reused ID
in the new container,
but the installed binary is not mapped to source,
so the failed lock condition is still not established.

This recovery **departed from the verified workaround** in two ways.
The owner check used only `podman ps --all` and `pgrep`,
which found no container of that name and no emulator process for that AVD;
`adb devices`,
`lsof`,
`fuser` and `lslocks` were not run.
Both lock files were then deleted,
not moved to a private backup.
Their names,
sizes,
times and the content quoted here are the only record of them.
Follow the workaround section,
not this departure.

After the deletion the same bounded container command booted.
The guest still carried the crashed visit's font scale `2.0`,
so its user data survived.
That visit's recorded baseline was restored and read back,
the owning namespace's `adb emu kill` was accepted,
the owner exited `0` and no container or emulator process remained.

After that clean shutdown the AVD directory again held an empty
`multiinstance.lock` and no `hardware-qemu.ini.lock`,
and the next writable boot succeeded with it present.
An empty `multiinstance.lock` alone therefore did not block a writable boot
here.
A boot with only `hardware-qemu.ini.lock` present was not tried,
so that file is implicated,
not isolated.

### Second recurrence the same day, workaround followed

After the emulator crash described as the second occurrence in
[the segmentation-fault record](android-emulator-37-software-renderer-sigsegv.md),
the owned AVD copy again held `hardware-qemu.ini.lock` (content `31`
and a NUL byte) and an empty `multiinstance.lock`.
`podman ps` for both owned container names,
`pgrep` for the AVD name and for an ADB client on the study port,
and `fuser` on both lock files and the user data image found no owner.
Both files were then moved to a backup folder outside the AVD directory,
and the next boot of the same bounded container started normally.

### A restored setting can be lost at the console kill

A related finding from the same day,
recorded here because it concerns what a stop leaves behind.
The restore step set `font_scale` back,
read it back as `1.0`,
and then stopped the guest with the console's `kill`.
Two later boots first read `2.0`.
After the restore step was changed to run `sync` in the guest,
wait eight seconds and run `sync` again before the console kill,
the next boot first read `1.0`.
A second boot after another restore also first read `1.0`.
That is two confirming boots,
not a proof,
and a read-back before shutdown says nothing about what the next boot reads:
the next boot's first reading is the check.

### A shutdown cut short by the emulator's own wait leaves a snapshot lock

On 2026-10-05 a visit ended normally as far as its script could tell:
the guest's settings were restored and read back,
and the console answered `OK: killing emulator`.
Android Emulator 37.2.12.0 then printed:

```text
# emulator output at the end of that visit
INFO         | Wait for emulator (pid 31) 20 seconds to shutdown gracefully before kill;you can set environment variable ANDROID_EMULATOR_WAIT_TIME_BEFORE_KILL(in seconds) to change the default value (20 seconds)
USER_INFO    | Snapshots have been disabled by the user, save request is ignored.
INFO         | Saving snapshot 'default_boot' took 2 ms
Killed
```

The command inside the container returned `137`.
So the emulator killed its own guest process when the shutdown outlasted its wait;
neither the visit's script nor `podman` forced it.
The host was busy with other work that evening;
a load average near 27 was read fourteen minutes before this shutdown,
and none at the shutdown itself.

The AVD directory then held three lock files:
`hardware-qemu.ini.lock` and `snapshot.lock.lock`,
each with the content `31`,
and an empty `multiinstance.lock`.
The snapshot lock's modification time was the time of that shutdown.

The next boot exited `1` before the guest started,
18 seconds after its container started,
with a diagnostic this record had not seen before:

```text
# emulator output at the next start
FATAL        | A snapshot operation for 'Pixel9ProFold_Fresh_5FXdOc' is pending and timeout has expired. Exiting...
```

The visit's bootstrap did not notice that the emulator had gone
and waited for the device for its whole bound of ten minutes.

What was done,
and what each step showed:

- `podman ps` for both owned container names,
  `pgrep` for the emulator on the study's ports and for an ADB client on its port,
  and `fuser` on the lock files and the user data image found no owner.
- Moving only `hardware-qemu.ini.lock` and `multiinstance.lock` to a backup was the first step,
  taken by a script that did not yet know the third file.
  No boot was tried in that state.
- After `snapshot.lock.lock` was also moved to the backup,
  the next boot of the same bounded container reached the guest.
  That shows the joint relocation of the three files recovers the boot.
  It does not isolate the snapshot lock,
  although the FATAL names a snapshot operation and the earlier FATAL of this record does not.
- The container's start command now sets `ANDROID_EMULATOR_WAIT_TIME_BEFORE_KILL=90`,
  the variable the emulator's own message names.
  The next shutdown printed `Wait for emulator (pid 31) 90 seconds`,
  no `Killed`,
  and the owning process exited `0`.
  That is one shutdown,
  on a host still under the same kind of load.
- The bootstrap now stops as soon as the emulator's owning process has exited,
  instead of waiting out its bound.
- The script that repeats visits now moves any of the two process-naming locks it finds before a visit,
  after the same owner checks,
  whether the visit before it crashed or was cut short at shutdown.

## Verified workaround and tradeoffs

For this **disposable AVD only**,
first confirm no live emulator,
ADB device,
Podman emulator container,
or lock owner uses its directory.
Then preserve its exact stale lock files outside the AVD directory,
and boot the same bounded writable fixture again.
The successful run retained its installed debug app and Gboard settings;
no guest wipe was needed.
The backup is retained in private scratch for diagnosis,
not a public review asset.
If an owner is found,
do **not** move the lock;
stop or reconnect the actual owner instead.
Lock relocation is not a generic fix for every multiple-instance warning.

## What does not work

- The initial writable reboot with those two files still in place:
  it returned the FATAL before the guest appeared in ADB.
- Using `-read-only` solely to suppress the warning:
  not attempted because this design fixture needs writable guest state and
  the message itself identifies read-only as a different multi-instance mode.
- Removing locks from the original AVD or deleting arbitrary `*.lock` files:
  not attempted or authorized.
- Treating the GPU Vulkan warning in the failed boot log as this incident's
  cause:
  the deciding diagnostic was the later lock-related FATAL,
  and the same host-rendering setup booted after the exact lock relocation.

## Upstream filing decision

No upstream report is filed or drafted as ready.
The `.out-of-scope/` entries contain no Android Emulator lock exemption,
but the observed cleanup followed a forced stop of a private writable fixture:

1. An emulator fault is not isolated from the local forced-stop lifecycle.
2. An upstream cleanup change may be possible,
   but the installed binary's source identity and failed lock condition are
   not established.
3. Normal writable AVD startup is supported;
   this failed post-`SIGKILL` state is not proven to be a supported
   recovery guarantee.
4. Contribution policy for the responsible source revision was not audited.
5. No Android Emulator maintainer response to this exact case was
   established;
   the separate automation issue is not the emulator's upstream tracker.
6. No upstream patch or source-level positive control was built.

### Draft, do not file as-is

~~~md
Android Emulator 37.1.11: private Fold AVD reports multiple-instance fatal after forced stop

A writable disposable AVD stopped by a Podman SIGKILL fallback later returned
"Running multiple emulators with the same AVD" while ADB and process checks
found no other running emulator. Moving its two lock files to a private
backup allowed the same bounded boot command to start. The exact lock
failure and source revision have not been isolated, so this is not yet an
upstream issue with a verified fix.
~~~
