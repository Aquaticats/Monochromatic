# Bazzite 44 cross-application stalls coincide with extreme Btrfs transaction and storage pressure

## Status

The original problem is an intermittent episode in which input to applications is delayed.
A matching episode recurred on 2026-08-28 around 00:40 local time.
The user reported that all applications were delayed and then recovered,
followed by a narrower Helium delay.
Three autonomous probes timed out Plasma while KWin answered normally.
The episode occurred inside Snapper's 00:39:20 to 00:43:15 snapshot-deletion interval.

A distinct most-application episode occurred around 05:43 during post-deletion synchronization.
All applications except Ghostty were delayed,
the pointer moved normally,
and Alt+Tab was not tested.
Plasma and KWin answered DBus probes in 5 and 4 ms.
Applications recovered around 05:45 while Snapper remained blocked until 05:52.
The Ghostty exception prevents classifying this as the original all-application symptom.
Its application-toolkit or display-protocol boundary was not measured.
These recurrences made cleanup and deletion the leading conditional trigger at that stage,
but did not identify the cross-application blocking mechanism.

On August 29,
a broader incident delayed the panel,
applications,
Alt+Tab,
pointer movement,
and network-dependent activity.
Cleanup had batched deletion of snapshots 966 through 983 and remained active for 2 hours 53 minutes.
During the report,
ten-second CPU pressure was 21.03%,
ten-second I/O pressure was 25.03%,
ten-second full I/O pressure was 19.68%,
and the last Btrfs commit took 230.563 seconds.
The maximum recorded commit was 246.996 seconds.

A shorter recurrence around 02:58 occurred after deletion cleanup was inactive.
It overlapped completion of a four-minute timeline snapshot creation and a qgroup-inherit warning.
This is a direct counterexample to active deletion being necessary.
It keeps Btrfs transaction work and the storage path in scope without proving that snapshots initiate the condition.
A bounded physical-NVMe latency histogram and Btrfs-worker monitor is now active.

A different episode was captured on 2026-08-27 around 04:25 local time:
all four Plasma panels stopped reacting to hover and click input,
while the pointer and ordinary windows remained responsive.
Queued panel clicks took effect after the episode ended.
Bazaar also became gray for about one second.
The user confirmed that this was not the usual global-input problem.

Panel Colorizer 6.5.0 was still installed once on every panel even though its visible behavior was disabled.
The user said it should already have been removed and authorized uninstalling it.
The four widget instances and package were removed,
its live settings were moved out of the active configuration,
and Plasma Shell was restarted.
This is a verified removal,
not proof that Panel Colorizer caused either incident class.

At about 05:25,
a later episode with overlapping symptoms occurred after that removal and Plasma restart.
The panel and Helium scrolling were delayed,
while other applications remained responsive.
A watchdog independently captured a Plasma event-loop timeout during the report.
This recurrence establishes that Panel Colorizer was not required for this later episode.
It does not establish that the 04:25 and 05:25 episodes had one cause,
or that Helium and Plasma shared one cause.

A third natural episode began around 18:38 after Panel Colorizer removal.
All Helium scrolling,
clicks,
and keyboard input were delayed,
and panel task clicks did not work.
Pointer movement,
Firefox,
and Alt+Tab remained responsive.
Three autonomous probes timed out Plasma while KWin answered normally.
The episode recovered without restarting Plasma.

## Symptom

### Original global-input episode

The defining symptom is delayed input across applications,
not merely an unresponsive panel.
Timestamped recurrences were captured around 00:40 and 05:43 on 2026-08-28.

The available negative evidence is:

- No AMDGPU reset,
  GPU fault,
  ring timeout,
  job timeout,
  or recovery occurred across sixteen retained boots.
- The current GPU temperatures were normal and no thermal throttling was measured.
- High VRAM use was observed,
  but no memory-pressure or GPU-failure event coincided with a user-visible episode.
- The user could not reproduce the global-input delay during the initial investigation.

### Separate Plasma-centered episodes

During the 2026-08-27 panel episode:

- All four panels stopped showing hover effects.
- Panel clicks were delayed until recovery.
- Ordinary windows remained interactive.
- Pointer movement remained responsive.
- The user did not test the panel clock or Meta-key launcher shortcut.
- The panels recovered without restarting Plasma Shell.

The passive observer recorded no system-wide stall around the report.
Samples from 04:25:19 through 04:26:49 local time showed:

```text
loopGapMs       1000.7 to 1001.2
kwinWaitMs      0.0 to 0.1
cpuPressureMs   2.7 to 10.3
ioPressureMs    0.0 to 2.3
memoryPressureMs 0.0
dm-0 ioMs       0 to 34
dm-1 ioMs       0
```

No matching Plasma,
KWin,
kernel,
AMDGPU,
or Btrfs diagnostic was emitted.
Both Plasma Shell and KWin answered a DBus peer ping immediately at 04:27:03,
near the end of the episode.
That probe was too late to prove whether Plasma's event loop had been blocked earlier.

During the later episode around 05:25:

- The panel and Helium scrolling were delayed.
- Other applications remained responsive.
- Plasma Shell failed a 1.5-second DBus probe.
- KWin answered its control probe in 5.8 ms.
- KWin scheduler wait stayed at 0.0 ms in the sampled interval.
- CPU pressure was 10.5 ms or less in the surrounding sampled intervals.
- I/O pressure was at or below 0.1 ms.
- Memory pressure remained 0.0 ms.
- No kernel,
  DRM,
  or AMDGPU fault was emitted.

The watchdog declared Plasma responsive after three later probes,
and the user placed visible recovery around 05:27.
Those recovery probes began only after all-thread stack collection finished,
so they do not measure the natural duration of the event.
The collector itself may account for part of the visible duration.
Plasma's 05:27:51 probe took 22.0 ms,
compared with the 4.9 to 6.2 ms startup range,
but remained below the 250 ms incident boundary.
The user continued to perceive some Helium scrolling delay after the panel recovered.
At 05:54,
the user confirmed that Helium and the panel were both working normally again.
The temporary continuation keeps Helium's client-local behavior separate from the measured Plasma event-loop stall.

During the later episode around 18:38:

- Helium scrolling,
  clicks,
  and keyboard input were delayed.
- Panel clicks did not work.
- Clicking Helium's panel task entry did not switch windows.
- Pointer movement and Firefox remained responsive.
- Alt+Tab still switched windows,
  including to Helium.
- Plasma missed three 1.5-second DBus probes.
- KWin answered the corresponding probes in 5.5 to 5.6 ms.
- The user reported recovery without restarting Plasma.

This boundary confirms a Plasma-plus-Helium event rather than an all-application input delay.

During the original-symptom recurrence around 00:40 on 2026-08-28:

- Snapper cleanup started at 00:36:17.
- A Btrfs qgroup scan completed at 00:39:20.
- Snapper queried quota and free space,
  then logged deletion of timeline snapshot 964 at 00:39:20.
- Plasma missed three 1.5-second probes reported at 00:40:19,
  00:41:18,
  and 00:42:17.
- KWin answered those probes in 5.3 to 5.8 ms.
- Plasma scheduler runtime and runnable wait deltas were 0.0 ms for each failed probe.
- The user reported that all applications were delayed and then recovered.
- The user then reported a narrower Helium delay.
- Snapper's next free-space query and cleanup completion occurred at 00:43:15.
- No AMDGPU fault,
  kernel lockup,
  memory-pressure interval,
  or contemporaneous I/O-pressure threshold crossing was captured.

### Distinct most-application recurrence around 05:43

During the 05:43 recurrence:

- Snapper cleanup started at 05:36:17.
- Timeline snapshot 965 deletion began at 05:38:26.
- A qgroup rescan completed at 05:38:26,
  and another completed at 05:43:05.
- Snapper's post-deletion free-space query then waited until 05:52:14.
- The user then reported that all applications except Ghostty were delayed.
- Pointer movement remained normal;
  Alt+Tab was not tested.
- Plasma and KWin answered DBus probes in 5 and 4 ms.
- Current CPU and memory pressure were negligible;
  ten-second I/O pressure was 0.0.
- Snapper's main thread was in `futex_do_wait`,
  its worker was in `hrtimer_nanosleep`,
  and the helper was in `poll`.
- Applications recovered around 05:45.
- The following free-space query returned at 05:52:14,
  showing that recovery preceded operation completion.
- No GPU fault,
  kernel lockup,
  or memory-pressure event was emitted.

The capture is preserved at
`/var/home/user/temp/agent/plasma-stall-captures/2026-08-28T09-43-user-reported-all-app-stall`.
The later Snapper daemon crash is excluded from natural attribution.
A diagnostic `snapper list` client was terminated after it remained blocked behind cleanup.
The later crash is most consistent with Snapper handling that disconnected sender,
but the exact queued sender was not recorded.
`server/snapperd.cc:116` calls `get_unix_userid` for a newly observed sender without a surrounding exception handler;
the journal recorded an uncaught `DBus::ErrorException` there.
The crash occurred after visible recovery and is observer-contaminated for stall attribution.
It nevertheless exposes a separate Snapper daemon robustness defect:
a disconnected queued client can terminate the daemon,
make the helper receive `NoReply`,
and fail cleanup.
Do not invoke or terminate Snapper CLI clients while cleanup is active;
use journal and direct Btrfs state until cleanup finishes.

The recurrences change the cleanup association from a Plasma-only correlation
to a leading trigger hypothesis for cross-application delay.
The 05:43 event also shows that Plasma and KWin can remain responsive while ordinary applications are delayed.
The evidence does not establish whether Btrfs accounting,
storage locks,
Snapper synchronization,
a desktop service,
or another shared dependency transmits the delay.

### August 29 systemwide-pressure and cleanup-inactive recurrences

At 02:49,
the user reported delayed panel input,
applications,
Alt+Tab,
pointer movement,
and internet access.
The pointer was delayed rather than fully frozen.
Plasma and KWin DBus replies after the report took 89 and 42 ms,
remaining below the prior timeout boundary.
Memory pressure was zero.

Snapper's midnight cleanup had begun at 00:00:12.
Number cleanup took about 12 minutes,
then timeline cleanup requested deletion of snapshots 966 through 983.
The operation was still active at 02:49.
Snapper's main thread waited on a futex,
its worker slept in `hrtimer_nanosleep`,
and the helper waited in `poll`.
The helper had idle I/O scheduling,
but Snapper daemon used best-effort scheduling and Btrfs transaction work ran in kernel threads.

A request to stop cleanup and Snapper daemon took 161 seconds to return.
The services stopped at 02:53:56,
current CPU and I/O pressure fell to 0.03% and 0.02%,
and the user reported recovery.
Because the stop request itself was delayed until settlement,
this timing does not prove that service termination caused recovery.

The network journal contains a user-initiated `systemctl restart NetworkManager` at 02:45:53.
That restart explains a contemporaneous disconnect and reconnect,
so network interruption is not independent evidence for the desktop-stall mechanism.

Around 02:58,
the user reported another short recurrence while deletion cleanup was inactive.
Timeline creation had run from 02:53:56 to 02:57:58,
when Btrfs logged `qgroup inherit needs a rescan`.
Ten-second full I/O pressure was 2.41%,
and the last completed Btrfs commit took 1.824 seconds.
Plasma and KWin then answered in 5 and 4 ms.
A concurrent Rust build discovered afterward began at 02:59:17,
after the recurrence,
so it cannot explain either August 29 incident.

The evidence is preserved at
`/var/home/user/temp/agent/plasma-stall-captures/2026-08-29T06-49-systemwide-pressure`.

## Root cause

### Original global-input episode: cleanup-associated; initiating mechanism unknown

The original symptom now has a timestamped recurrence.
It coincided with a blocking Snapper cleanup interval and three independent Plasma event-loop failures,
while KWin remained responsive.
The same interval had no matching GPU fault,
kernel lockup,
memory pressure,
or observer-wide scheduler delay.

The user retained hourly snapshot creation but moved cleanup to midnight after the 05:43 recurrence.
The persistent timer catches up after boot when a midnight run was missed.
This reduces deletion exposure during ordinary active hours without disabling automatic snapshots.
It is not enough to call Snapper or Btrfs the root cause:
many equivalent deletion cycles completed without a user-visible stall,
and no capture identifies the dependency that delayed applications.

The direct event-loop watchdog was stopped after capturing the target symptom.
Its first failure captured both process states and a Plasma stack.
A second stack was also captured,
but the stack collector pauses the target process and can prolong an existing stall.
Later captures therefore used DBus probes and procfs wait channels without attaching a debugger.

### Coupled Btrfs-accounting and SSD-latency hypothesis

The root device is a 2 TB `SPCC M.2 PCIe SSD` using Realtek's RTS5772DL controller,
which PCI identification explicitly describes as DRAM-less.
Firmware is `VF101C68`.
The kernel allocated the controller's full preferred 64 MiB host-memory buffer,
and the controller reports a volatile write cache.
SMART reports 1% endurance used,
47 °C,
no media errors,
and no NVMe error-log entries.
Healthy SMART data does not measure slow successful completions.

The physical device uses Kyber with 2 ms read and 10 ms synchronous-write targets.
Writeback throttling is enabled with a 2 ms target.
Snapper cleanup's helper uses idle I/O priority,
but Snapper daemon uses best-effort priority and has default cgroup I/O weight.
Btrfs cleaner,
transaction,
qgroup,
and writeback work executes in kernel threads,
so userspace priority alone cannot prove isolation of that work.

Traditional qgroups remain a filesystem-level candidate.
An upstream Btrfs maintainer explains that back-reference accounting cost grows with shared extents
and can block a transaction for minutes.
The local fixture's empty-file tree did not reproduce the production pathological regime;
its quota-mode timing null applies only to that fixture and does not exclude qgroup cost with real shared data extents.

The leading model is therefore coupled rather than exclusive:
full-qgroup transaction work can amplify metadata and lock duration,
a DRAM-less controller can amplify commit or flush latency,
and default block isolation can transmit device contention to applications.
No layer has been isolated as the initiating cause.
A kernel-side trace now records physical NVMe request latency independently of Btrfs commit duration.
Normal device latency during another long commit would favor filesystem accounting or locks;
slow physical completions would support the SSD or firmware side of the model.

## Diagnostic interpretation

### The later panel episode crossed Plasma's event-loop boundary while KWin remained responsive

The first failed probe began at approximately 05:25:23.8 and reached its 1.5-second limit around 05:25:25.3.
KWin answered in 5.8 ms in the same watchdog pass.
The user's `now` marker arrived around 05:25:52,
so autonomous detection preceded the marker and the later evidence commands.
The Plasma GUI thread stack was:

```text
QWaitCondition::wait
QSGThreadedRenderLoop::polishAndSync
QSGThreadedRenderLoop::handleExposure
QWindow::event
QtWaylandClient::QWaylandWindow::updateExposure
```

The exact Qt source is `qtdeclarative` tag `v6.11.1`.
Its threaded-render-loop comment states that the GUI thread initiates `polishAndSync`,
then blocks until the render thread finishes synchronization
([`qsgthreadedrenderloop.cpp:45-59`](https://code.qt.io/cgit/qt/qtdeclarative.git/tree/src/quick/scenegraph/qsgthreadedrenderloop.cpp?h=v6.11.1#n45)).
The implementation first polishes QML items,
then posts a synchronization event and waits on the render thread
([`qsgthreadedrenderloop.cpp:1546-1681`](https://code.qt.io/cgit/qt/qtdeclarative.git/tree/src/quick/scenegraph/qsgthreadedrenderloop.cpp?h=v6.11.1#n1546)):

```cpp
d->polishItems();

w->thread->postEvent(new WMSyncEvent(window, inExpose, w->forceRenderPass, scProxyData));

w->thread->waitCondition.wait(&w->thread->mutex);
```

A later failed probe at 05:31:43 again timed out Plasma while KWin answered in 5.5 ms.
That Plasma stack was earlier in the same render-loop path:

```text
surfaceForWindow
WindowEffects::installBlur
PlasmaQuick::PlasmaWindow::resizeEvent
QQuickGridLayoutBase::rearrange
QQuickLayout::updatePolish
QQuickWindowPrivate::polishItems
QSGThreadedRenderLoop::polishAndSync
QSGThreadedRenderLoop::handleExposure
```

The second event is observer-contaminated by at least:

- Manual `eu-stack` attachment to Helium's multi-threaded browser and GPU processes.
- OpenSnitch rule creation and audit activity for `eu-stack`.
- OpenSnitch BPF unload and load activity.
- A Plasma invalid-XML notification diagnostic at 05:31:01.

The `WindowEffects::installBlur` frame is one momentary sample during that contaminated window.
It is retained as raw evidence,
not used to identify an initiating component or counted as an independent natural reproduction.

The failed DBus probe establishes that Plasma's GUI event loop did not service the request within 1.5 seconds.
The first stack shows that the GUI thread was at Qt's normal render-synchronization wait point.
All sampled render threads were later idle in `processEventsAndWaitForMore`,
so the sequential stack snapshot does not identify what delayed the earlier synchronization.
The second stack likewise cannot establish that ordinary layout,
blur installation,
or dynamic-cast work was unusually slow.
The captures do not identify which Plasma window was exposed,
which QML item initiated a resize,
or why Helium scrolling remained delayed.
No kernel GPU fault appeared during either interval.

### The 18:38 recurrence repeatedly blocked sleeping Plasma while KWin remained responsive

The lightweight watchdog captured three natural failed Plasma probes starting at:

- 18:38:11,
  with a 1,500.9 ms timeout and 5.6 ms KWin response.
- 18:39:12,
  with a 1,502.8 ms timeout and 5.5 ms KWin response.
- 18:41:01,
  with a 1,502.4 ms timeout and 5.5 ms KWin response.

Plasma accumulated 0.0 ms of scheduler runtime and 0.0 ms of runnable wait during each failed probe.
The second wait-channel snapshot placed Plasma's main thread in `futex_do_wait`;
the first and third reported `0` for that thread.
Render threads were also in futex waits.
These procfs snapshots show a sleeping synchronization boundary,
not the initiating operation or owning component.
They do not establish that every futex wait was abnormal.

KWin scheduler wait remained between 0.0 and 0.2 ms,
and the one-second observer classified no system-wide incident.
A transient 394.2 ms/s I/O-pressure sample appeared near the first timeout,
but dm-0 accumulated only 2 ms of block I/O and dm-1 had none.
No AMDGPU,
DRM,
kernel lockup,
or memory-pressure diagnostic accompanied the episode.
A PipeWire link activation failed at 18:38:20,
but one coincident audio-link failure does not establish the cause of either application's input delay.

The captures are:

- `/var/home/user/temp/agent/plasma-stall-captures/2026-08-27T22-38-11.285Z`
- `/var/home/user/temp/agent/plasma-stall-captures/2026-08-27T22-39-12.336Z`
- `/var/home/user/temp/agent/plasma-stall-captures/2026-08-27T22-41-01.483Z`

Unlike the earlier all-thread stack capture,
these captures used only DBus probes and procfs wait channels.
They did not attach a debugger or stop Plasma.

### AMDGPU `REG_WAIT` messages are real timeouts but have not occurred at runtime here

The exact kernel source is OpenGamingCollective Linux tag `v7.2-ogc6`,
commit `53a8eea83bb37294a0fe3d5ceaf6b5f1d8d141b2`.

The register helper passes the polling interval,
try count,
function,
and source line into `generic_reg_wait`
(`drivers/gpu/drm/amd/display/dc/inc/reg_helper.h:216-221`):

```c
/* macro to poll and wait for a register field to read back given value */

#define REG_WAIT(reg_name, field, val, delay_between_poll_us, max_try) \
        generic_reg_wait(CTX, \
                REG(reg_name), FN(reg_name, field), val,\
                delay_between_poll_us, max_try, __func__, __LINE__)
```

`dcn32_program_compbuf_size` waits for four DET-size registers before increasing the compression-buffer size
(`drivers/gpu/drm/amd/display/dc/hubbub/dcn32/dcn32_hubbub.c:140-160`):

```c
if (compbuf_size_segments > hubbub2->compbuf_size_segments) {
    REG_WAIT(DCHUBBUB_DET0_CTRL, DET0_SIZE_CURRENT, hubbub2->det0_size, 1, 100);
    REG_WAIT(DCHUBBUB_DET1_CTRL, DET1_SIZE_CURRENT, hubbub2->det1_size, 1, 100);
    REG_WAIT(DCHUBBUB_DET2_CTRL, DET2_SIZE_CURRENT, hubbub2->det2_size, 1, 100);
    REG_WAIT(DCHUBBUB_DET3_CTRL, DET3_SIZE_CURRENT, hubbub2->det3_size, 1, 100);
}
```

`optc32_disable_crtc` disables the timing generator and then waits for its clock to stop being busy
(`drivers/gpu/drm/amd/display/dc/optc/dcn32/dcn32_optc.c:172-199`):

```c
REG_UPDATE(OTG_CONTROL,
        OTG_MASTER_EN, 0);

REG_UPDATE(CONTROL,
        VTG0_ENABLE, 0);

/* CRTC disabled, so disable  clock. */
REG_WAIT(OTG_CLOCK_CONTROL,
        OTG_BUSY, 0,
        1, 150000);
```

Per-boot monotonic journal timestamps place every retained local event within 39.705 seconds of boot.
Boot `-15` had no matching event;
boots `-14` through `0` did.
The two `optc32_disable_crtc` occurrences were in boots `-14` and `-13` around 39 seconds.
The `dcn32_program_compbuf_size` occurrences were between 11.343 and 39.705 seconds.

This timing supports an early display-configuration phase on this machine,
but the functions can also participate in later display mode transitions.
The messages must not be called harmless solely because they happened early.
The narrower verified statement is that no matching runtime timeout coincided with the reported desktop symptoms here.

### Timeline creation was clean, while cleanup aligned with every natural Plasma-centered event

The exact Snapper source is tag `v0.13.0`,
commit `3a3bd97083976d28538d402284ff947b4aab5b8f`.

The hourly helper creates a snapshot whose cleanup label is `timeline`
(`client/systemd-helper/systemd-helper.cc:98-104`):

```cpp
SCD scd;
scd.description = "timeline";
scd.cleanup = "timeline";
scd.userdata = userdata;

snapper->createSingleSnapshot(scd, report);
```

A non-empty cleanup label enables quota handling
(`snapper/Snapshot.cc:640-647`):

```cpp
snapper->getFilesystem()->createSnapshot(num, num_parent, read_only, !cleanup.empty(),
                                         empty);
```

The Btrfs backend passes the configured qgroup to snapshot creation
(`snapper/Btrfs.cc:332-336`):

```cpp
create_snapshot(subvolume_dir.fd(), info_dir.fd(), SNAPSHOT_NAME, read_only,
                quota ? qgroup : no_qgroup);
```

Snapper then requests qgroup inheritance
(`snapper/BtrfsUtils.cc:152-165`):

```cpp
if (qgroup != no_qgroup)
{
    struct btrfs_qgroup_inherit *inherit;

    inherit = (btrfs_qgroup_inherit*) &buffer[0];
    inherit->num_qgroups = 1;
    inherit->num_ref_copies = 0;
    inherit->num_excl_copies = 0;
    inherit->qgroups[0] = qgroup;
    util_inherit = (struct btrfs_util_qgroup_inherit *)inherit;
}
```

When the kernel cannot use its quick-inherit accounting path,
it marks the qgroup inconsistent
(`fs/btrfs/qgroup.c:3582-3587`):

```c
if (need_rescan)
    qgroup_mark_inconsistent(fs_info, "qgroup inherit needs a rescan");
```

The current system starts this path hourly.
At 04:00:12 local time,
`snapper-timeline.service` created the root snapshot and the kernel emitted:

```text
BTRFS warning (device dm-0): qgroup marked inconsistent, qgroup inherit needs a rescan
```

The snapshot produced dm-0 and NVMe activity,
including a measured 21,556 KiB dm-0 write interval at 04:00:13.
KWin ping time stayed between 5.4 and 6.3 ms around the warning,
KWin scheduler wait remained at or below 0.1 ms,
and the user reported no stutter while actively trying to trigger one.
The scans completed at 04:22:12 and 04:23:50.

A second qgroup warning occurred at 05:00:12.
The procfs observer recorded no scheduler or resource-pressure threshold crossing around it.
At 05:00:25,
Plasma Shell answered in 5.6 ms and KWin answered in 5.4 ms.
No user symptom marker accompanied this cycle,
so these measurements establish responsive sampled services rather than absence of a subjective symptom.

At 15:00:00,
another timeline snapshot emitted the same qgroup warning and triggered a measured root-filesystem I/O-pressure
interval.
The observer sampled 559.1 to 595.0 ms/s of I/O pressure,
with dm-0 reading up to 20.08 MiB and writing up to 11.65 MiB in one-second intervals.
KWin scheduler wait remained at zero.
Plasma answered in 6.0 ms and KWin in 5.4 ms at 15:00:06,
and neither observer classified a desktop incident.
This positive-I/O control shows that the qgroup timeline path
and measurable dm-0 pressure did not require a desktop stall.

The installed `snapper-cleanup.timer` runs hourly with `OnUnitActiveSec=1h`.
The 04:25 and 05:25 natural Plasma-centered events both occurred while that cleanup service was active:

- The 04:20:34 cleanup logged deletion of snapshot 969 at 04:23:50.
  The user reported the first panel episode around 04:25.
  Cleanup completed at 04:29:57.
- The 05:21:32 cleanup logged deletion of snapshot 970 at 05:24:49.
  Plasma's failed probe began about 34 seconds later.
  Cleanup completed at 05:34:56.
- The same cleanup logged deletion of snapshot 971 at 05:31:02.
  The observer-contaminated second probe began about 41 seconds later.
- The 18:34:32 cleanup logged deletion of snapshot 964 at 18:37:22.
  Natural failed Plasma probes began about 49 seconds,
  1 minute 50 seconds,
  and 3 minutes 39 seconds later.
  Cleanup completed at 18:41:21,
  and the user reported recovery shortly after the third probe recovered.

All three user-reported Plasma-centered episodes therefore have a cleanup association,
unlike the earlier assessment based only on timeline creation.
This is repeated temporal evidence,
not a demonstrated call path.

Snapper prints the deletion message immediately before a blocking DBus `DeleteSnapshots` request
(`client/proxy/commands.cc:277-296`).
Service stdout alone does not prove that the whole interval until the next cleanup-phase message belongs to that
request,
because quota and free-space DBus calls inside the cleanup loop do not emit helper-phase messages.

The 00:40 recurrence has a tighter boundary.
`/var/log/snapper.log` recorded the pre-deletion free-space query at 00:39:20,
the helper immediately logged deletion of snapshot 964,
and Snapper recorded the loop's next free-space query at 00:43:15.
The cleanup loop re-evaluates that condition only after `remove` returns
(`client/cleanup.cc:353-397`).

The interval is deletion completion and synchronization,
not necessarily one DBus `DeleteSnapshots` call.
The server deletes the subvolume and remembers its ID
(`snapper/Btrfs.cc:411-431`).
The following free-space query calls `filesystem->sync()`
(`snapper/Snapper.cc:886-906`).
For remembered deleted subvolumes,
`Btrfs::sync()` sleeps until Btrfs no longer reports each ID,
then syncs again
(`snapper/Btrfs.cc:1576-1596`).
A 00:41:18 procfs sample found `systemd-helper` blocked in `poll`,
Snapper's main thread in a futex wait,
and its second thread in `clock_nanosleep`,
matching that post-deletion wait boundary.
The kernel logged qgroup scan completion at 00:39:20,
before the interval.
The three Plasma failures and matching all-application delay occurred inside it.

This filesystem uses full qgroup accounting and the kernel's default subtree-drop threshold of 3.
Snapshot 963 has a level-3 root,
so its highest shared subtrees are level 2 and do not reach the default skip threshold.
The kernel's `drop_subtree_threshold` interface can skip accounting at or above its value,
marking qgroup data inconsistent for a later rescan.
The production qgroups were already inconsistent after the next hourly snapshot creation,
so cleanup was going to rescan them even without changing the threshold.

A disposable 2 GiB Btrfs fixture reproduced a level-3 tree with 180,000 empty files.
Threshold 2 activated the intended branch:
every threshold-2 deletion changed `inconsistent` from 0 to 1,
while every threshold-3 deletion left it at 0.
Independent first-after-mount controls did not show a timing benefit.
Threshold-2 deletions took 29.006 and 29.007 seconds;
threshold-3 deletions took 29.007 and 30.007 seconds.
Matched controls took 30.006 and 31.009 seconds with simple quotas,
and 31.007 seconds in both quota-disabled cases.
The fixture therefore validates threshold semantics but not threshold- or quota-mode-based latency reduction.
It cannot exclude a production-only qgroup contribution.

Simple quotas accepted parent qgroup creation,
explicit assignment,
and snapshot inheritance commands.
A data-bearing control exposed incompatible accounting semantics.
After a 64 MiB allocation in the live subvolume,
a read-only inherited snapshot's child qgroup and parent `1/0` each reported only 4 KiB.
Deleting the live reference left the retained allocation charged to the live subvolume,
not the snapshot parent.
Snapper queries that parent's exclusive usage for `SPACE_LIMIT`,
so simple quotas would undercount snapshot-retained data.
No quota mode or threshold was changed on the production filesystem.

No I/O-pressure interval,
KWin delay,
or system-wide input failure accompanied the failed Plasma probe.
The 04:00 and 05:00 clean timeline-creation measurements still oppose a simple snapshot-creation trigger.

The 06:22 cleanup ran during the encrypted-filesystem scrub and deleted no snapshot.
It completed at 06:25 without a Plasma or KWin probe delay.
The 07:00 timeline creation also left both probes responsive while the scrub was active.
These are scrub-contaminated cases and do not test the deletion-specific association.

Later post-scrub controls did exercise deletion without the encrypted scrub or loop-device tests:

- The 11:27 cleanup deleted snapshots 964 and 965 at 11:30:39 and 11:34:55.
- The 12:28 cleanup deleted snapshot 964 at 12:31:10.
- The 13:29 cleanup deleted snapshot 964 at 13:32:15.
- The 14:30 cleanup deleted snapshot 964 at 14:33:18.
- The 15:31 cleanup deleted snapshot 964 at 15:34:14.
- The 16:32 cleanup deleted snapshot 964 at 16:35:12.
- The 17:33 cleanup deleted snapshot 964 at 17:36:12.
- The 19:35 through 23:36 cleanups each deleted snapshot 964 without a captured desktop stall.

Neither desktop observer recorded an incident candidate or event-loop stall across those cycles.
Plasma and KWin DBus heartbeats remained successful,
and the one-second observer reported no KWin wait accumulation around the deletions.
These repeated deletion controls prove that deletion is not sufficient to trigger the symptom.

The 00:40 recurrence nevertheless strengthens evidence for a conditional cleanup interaction
because the original all-application symptom and every natural Plasma-centered episode occurred during deletion
while cleanup remained active.
At that time,
`FREE_LIMIT` was 200 GiB while Snapper measured about 182.6 GiB free.
The space-aware pass therefore deleted each new hourly timeline snapshot,
leaving snapshot 963 as the only retained snapshot after cleanup.
That create-delete cycle explains repeated snapshot number 964 on Snapper 0.13.0,
but does not identify the cross-application delay mechanism.
Changing the creation and cleanup timers should be treated as a controlled comparison,
not a confirmed fix.

## Environment findings

### Panel Colorizer remained active despite disabled visible styling

The installed package was Panel Colorizer 6.5.0,
source tag `v6.5.0`,
commit `157bf8105e92afd9d6fa82faef872099d05ee9c2`.
The latest upstream release during investigation was 8.0.0.

The Plasma layout contained four instances:

```text
panel 5  widget 41
panel 32 widget 42
panel 34 widget 43
panel 36 widget 39
```

Each instance had `hideWidget=true` and `isEnabled=false` in the layout.
That did not disable its DBus helpers because their separate default is true
(`package/contents/config/main.xml:75-79`):

```xml
<entry
    name="enableDBusService"
    type="Bool">
    <default>true</default>
</entry>
```

Each instance creates one service model and two signal monitors when that setting is enabled
(`package/contents/ui/main.qml:2036-2056`):

```qml
DBusServiceModel {
    id: serviceModel
    enabled: plasmoid.configuration.enableDBusService
}

DBusSignalMonitor {
    enabled: plasmoid.configuration.enableDBusService
    method: "property_changed"
}

DBusSignalMonitor {
    enabled: plasmoid.configuration.enableDBusService
    method: "preset_changed"
}
```

The service model launches `service.py`
(`package/contents/ui/DBusServiceModel.qml:8-18`):

```qml
property string serviceUtil: toolsDir + "service.py"
property string pythonExecutable: plasmoid.configuration.pythonExecutable
property string serviceCmd: pythonExecutable + " '" + serviceUtil + "' "
                            + Plasmoid.containment.id + " " + Plasmoid.id

function toggleService() {
    if (enabled)
        runCommand.run(serviceCmd);
    else
        (dbusQuit.call());
}
```

Each signal monitor launches `gdbus_get_signal.sh`
(`package/contents/ui/DBusSignalMonitor.qml:16-18`):

```qml
readonly property string dbusMessageTool: "'" + toolsDir + "gdbus_get_signal.sh'"
readonly property string monitorCmd:
    `${dbusMessageTool} ${busType} ${service} ${iface} ${path} ${method}`
```

That script starts both `gdbus monitor` and `tail -f`
(`package/contents/ui/tools/gdbus_get_signal.sh:7-11`):

```bash
TMPFILE=$(mktemp)
gdbus monitor --"${BUS_TYPE}" --dest "${SERVICE}" >"$TMPFILE" &
PID=$!
exit_code=130
tail -f "$TMPFILE" | while IFS= read -r line; do
```

The live Plasma service tree matched this source shape:
four Python services,
eight `gdbus monitor` processes,
eight `tail -f` processes,
and their shell wrappers were present before removal.

This proves that the apparently disabled widgets still ran helper infrastructure.
It does not prove that those helpers caused the 04:25 panel freeze or original global-input stalls.
No live Plasma stack was captured during the 04:25 episode,
and the removal made same-configuration recurrence unavailable.
The later Plasma-centered episode occurred while the package,
widget references,
helpers,
and live settings were absent.
Panel Colorizer was therefore not required for that later recurrence.

### AMDGPU VM-memory warnings are teardown warnings and remain a watch item

The runtime messages such as:

```text
VM memory stats for proc polypane(...) task polypane:cs0(...) is non-zero when fini
```

come from VM destruction when an accounting check remains non-zero
(`drivers/gpu/drm/amd/amdgpu/amdgpu_vm.c:2904-2912`):

```c
if (!amdgpu_vm_stats_is_zero(vm)) {
    struct amdgpu_task_info *ti = vm->task_info;

    dev_warn(adev->dev,
         "VM memory stats for proc %s(%d) task %s(%d) is non-zero when fini\n",
         ti->process_name, ti->task.pid, ti->task.comm, ti->tgid);
}
```

The local events occurred as Polypane,
KInfoCenter,
Helium,
Floorp,
and Firefox GPU VMs ended.
They were not GPU reset or ring-timeout diagnostics,
and KWin stayed responsive around the measured events.
The same warning has been associated upstream with an AMDGPU accounting leak path,
so recurring warnings remain worth tracking rather than dismissing.

### Encrypted SATA Btrfs corruption is a separate integrity risk

The encrypted SATA filesystem reported:

```text
bdev /dev/mapper/crypt_sda errs: wr 0, rd 0, flush 0, corrupt 405004, gen 0
```

Retained boot `-2` contained 434 visible checksum-failure lines from 2026-08-16 through 2026-08-18.
The kernel also emitted 41 `btrfs_print_data_csum_error` suppression notices
and 41 matching device-stat suppression notices.
The device-stat notices accounted for 404,483 suppressed callbacks,
so the 434 visible lines are only a sample of the persistent counter's increments.
That visible sample covered 59 inode numbers and 358 inode-offset pairs in top-level subvolume 5.

Current inode resolution mapped 52 of those inode numbers:
50 were Steam game or compatibility files,
and two were files in a clean Git checkout with a working public remote.
Seven inode numbers no longer existed,
so their former paths cannot be classified from the current filesystem.
Targeted 4 KiB buffered reads then exercised the recorded logical offsets in every mapped file.
The probe read 232 current ranges without `EIO`;
one additional old offset was beyond the current shorter file length.
The persistent corruption counter remained exactly 405,004 after that probe.

This narrow probe did not reproduce a checksum error.
It is not proof that the original physical extents are sound:
the page cache can satisfy buffered reads,
and CoW updates can place a current inode offset on a different extent.
It also does not verify suppressed offsets or the rest of the 1.42 TiB of allocated data.

The filesystem has one device,
uses `Data,single`,
and uses `Metadata,DUP`.
`btrfs scrub status` reported no prior scrub statistics.
Official [`btrfs-scrub(8)` documentation](https://btrfs.readthedocs.io/en/stable/btrfs-scrub.html)
says scrub can repair only from another known-good replica,
so this layout cannot reconstruct damaged single-profile file data.
SATA SMART reported PASSED with zero reallocated,
uncorrectable,
program/erase,
and CRC errors.
SMART health does not erase the historical Btrfs checksum failures
or rule out corruption outside the drive,
such as memory or data supplied before a write.
No initiating cause has been established.

A full scrub checked 1.42 TiB from 06:14:26 through 09:42:27 local time in 3:28:01.
The command and kernel both completed with status zero,
and the scrub reported `Error summary: no errors found`.
No `dm-1` checksum or I/O error appeared in the kernel journal during the scrub.
The cumulative device statistics remained
`wr 0, rd 0, flush 0, corrupt 405004, gen 0`,
so the scrub added no corruption event.
After recording that result and the historical count,
`btrfs device stats --reset` established a new all-zero baseline.

The clean scrub shows that currently allocated extents verified successfully.
It does not identify the historical failure's cause,
prove that deleted or CoW-replaced extents were sound,
or repair historical `Data,single` corruption from another Btrfs copy.
No captured desktop symptom was attributed to this filesystem.

## Verification

### Environment

```text
Operating System: Bazzite 44
KDE Plasma: 6.7.4
KDE Frameworks: 6.29.0
Qt: 6.11.1
Graphics platform: Wayland
Kernel: 7.2.0-ogc6.1.fc44.x86_64
Graphics processor: AMD Radeon RX 7600
Mesa: 26.2.1-4.fc44
Snapper: 0.13.0-3.fc44
Btrfs tools: 7.1-1.fc44
Panel Colorizer before removal: 6.5.0
```

### AMDGPU retained-boot catalog

The authoritative comparison uses per-boot monotonic time:

```sh
# doc/troubleshooting/bazzite-desktop-input-stalls.md
for boot in $(seq -15 0); do
  journalctl --dmesg --boot="$boot" --no-pager --output=short-monotonic \
    | rg 'REG_WAIT.*(dcn32_program_compbuf_size|optc32_disable_crtc)' -
done
```

Clean case:

- Boot `-15` contained no matching timeout.

Observed timeout cases:

- Boots `-14` through `0` contained `dcn32_program_compbuf_size` events within 39.705 seconds of boot.
- Boots `-14` and `-13` also contained `optc32_disable_crtc` events around 39 seconds.
- No retained runtime event matched a user-visible stall.

### Snapper cycle catalog

```sh
# doc/troubleshooting/bazzite-desktop-input-stalls.md
journalctl --dmesg --boot=0 --no-pager --output=short-iso \
  | rg --ignore-case 'qgroup|quota.*rescan|rescan.*quota' -

journalctl --since='2026-08-27 03:59:50' \
  --until='2026-08-27 04:24:00' \
  --no-pager \
  --output=short-iso \
  --unit=snapper-timeline.service \
  --unit=snapperd.service

journalctl --boot=0 \
  --unit=snapper-cleanup.service \
  --no-pager \
  --output=short-iso

systemctl cat snapper-cleanup.timer
```

Clean visible case:

- The 04:00 cycle triggered qgroup rescans and dm-0 activity without a reported stutter.

Clean instrumentation case:

- The 05:00 cycle crossed no observer threshold,
  and Plasma Shell and KWin answered normally at the sampled point.
- The user did not mark a symptom during this cycle,
  so it is not classified as a clean subjective case.

Cleanup-associated cases:

- The 04:25 panel report occurred during the 04:20:34 to 04:29:57 cleanup.
- The 05:25 Plasma and Helium report occurred during the 05:21:32 to 05:34:56 cleanup.
- Both reports followed a snapshot-deletion log.
- No failed Plasma probe,
  I/O-pressure interval,
  or user symptom marker was captured for retained cleanup runs before the watchdog existed.

Unverified case:

- No original global-input episode has a timestamp that can be compared with another rescan or cleanup.

### Plasma event-loop capture

The watchdog measured an ordinary startup range of 4.9 to 6.2 ms,
then used 250 ms as its incident boundary.
Its classifier self-test exercised both slow-success and failed-probe branches before observation.

The first natural failure record was:

```json
{
  "timestamp": "2026-08-27T09:25:25.295Z",
  "plasmaProbe": { "elapsedMs": 1501.350116, "succeeded": false },
  "kwinProbe": { "elapsedMs": 5.808578, "succeeded": true }
}
```

The raw state is in scratch storage:

```text
/var/home/user/temp/agent/plasma-stall-captures/2026-08-27T09-25-25.295Z
```

The second observer-contaminated state is:

```text
/var/home/user/temp/agent/plasma-stall-captures/2026-08-27T09-31-43.612Z
```

KWin's DBus probes succeeded,
but its stack collection failed with `dwfl_linux_proc_report pid 3768: Permission denied`.
The KWin control therefore has latency and procfs wait-channel evidence,
not a userspace stack.

The procfs wait-channel and userspace stack captures were sequential rather than atomic.
For example,
the first main-thread wait channel was `0`,
while the later userspace stack showed `QWaitCondition::wait`.
That difference prevents treating either snapshot as a full interval trace.

`eu-stack` 0.195 attaches to each live thread with `PTRACE_ATTACH`
([`libdwfl/linux-pid-attach.c:73-117`](https://sourceware.org/elfutils/ftp/0.195/elfutils-0.195.tar.bz2)),
which stops that thread while its state is read.
The all-thread collector was removed from the active watchdog after the two captures.
Current incident collection retains only DBus timings and procfs wait channels.

A Helium snapshot taken while scrolling still felt delayed found its browser main loop polling normally
and its GPU-process main thread waiting on a timed condition.
One contextual spot read reported 58% GPU busy and 6,959,525,888 of 8,573,157,376 VRAM bytes used,
or 81.2%.
Those single values have no unchanged-run band and are not attributed to the delay.
No symbols identified an active Helium call path,
so this is not a Helium root-cause capture.

### Panel Colorizer removal verification

Upstream recommends removing the widget and restarting Plasma Shell for troubleshooting
(`README.md:313-325`).
The executed removal followed that boundary:

```text
Removed live widgets:
[{"panelId":5,"widgetId":41},
 {"panelId":32,"widgetId":42},
 {"panelId":34,"widgetId":43},
 {"panelId":36,"widgetId":39}]

kpackagetool6:
Successfully uninstalled .../luisbocanegra.panel.colorizer/
```

Post-restart host verification reported four intact panels:

```text
panel 5:  kickoff, pager, margins separator, system tray, digital clock, task manager
panel 32: task manager
panel 34: task manager
panel 36: task manager
```

The package registry,
package directory,
active Plasma layout,
and process list contained no Panel Colorizer entry.
Plasma Shell remained active and answered its DBus peer ping.

The Plasma restart also cleared unrelated state and caches,
so its resource measurements provide no component attribution or remediation evidence.

A temporary rollback copy was retained at:

```text
/var/home/user/temp/agent/panel-colorizer-backup-before-uninstall-20260827
```

That is scratch storage,
not a durable backup.
The active Panel Colorizer settings directory was removed.

## Verified workarounds

No workaround for the original global-input stalls or the separate panel freeze has been verified.

The user declined a timer pause and wants automatic snapshots retained.
The production `FREE_LIMIT` was lowered to 100 GiB after the user selected the weaker reserve tradeoff.
No timer or Btrfs quota setting has been changed.
The disposable controls reject threshold 2,
simple quotas,
and disabled quotas as demonstrated latency fixes for the reproduced metadata-heavy deletion.

Capacity was the gating decision.
After removing and syncing the disposable fixture,
`/var/home` had 183.102 GiB available,
16.898 GiB below the former 200 GiB `FREE_LIMIT`.
The user selected a 100 GiB limit instead of data migration or storage expansion.
The applied limit was verified with 183.054 GiB available,
so the free-space condition was satisfied with 83.054 GiB of headroom.
The encrypted data filesystem had 2.265 TiB available,
but no data was moved.

Retained Snapper logs show free space falling from 451.4 GiB on July 5 to 182.6 GiB at the captured August 28 cleanup,
an average decline of 4.978 GiB per day over that interval.
Daily changes varied,
so this mean is evidence of sustained pressure rather than a precise exhaustion forecast.
Lowering `FREE_LIMIT` postpones rather than removes that capacity pressure.
Reclaiming or moving live data while identifying its growth source remains the safest response if free space approaches
100 GiB.

After capacity is addressed,
the leading reversible stutter mitigation is to keep hourly creation while moving cleanup to a chosen inactive time.
The user initially retained hourly cleanup,
then selected midnight cleanup after the 05:43 recurrence repeated the same deletion-interval association.
A persistent systemd timer catches up after boot when midnight was missed.
This retains hourly restore granularity and batches deletion exposure into one scheduled window.
It temporarily accumulates up to one cleanup interval of snapshots,
and the scheduled cleanup can still stall applications.
Changing both creation and cleanup to once daily further reduces transactions,
but provides coarser restore points and ranks second.

A structural mitigation is to move measured high-churn directories into nested subvolumes
outside the `/var/home` snapshot tree.
It can preserve hourly snapshots for remaining home data while reducing the tree Btrfs must remove.
It requires a reviewed data migration,
and excluded directories need a separate backup policy.
No directory has been selected or measured for exclusion,
so this remains behind the reversible timer change.

Disabling full qgroups remains an evidence-gathering fallback,
not a recommended production change.
Snapper's free-space condition does not require `QGROUP`;
only snapshot-specific `SPACE_LIMIT` accounting does.
The fixture measured no deletion improvement with quotas disabled,
so the known functionality loss has no demonstrated benefit.
Simple quotas rank lower because they also undercount snapshot-retained data in Snapper's parent qgroup.

Lowering `drop_subtree_threshold` to 2 activated Btrfs's intended skip path
but did not improve independent deletion timings.
It marks qgroups inconsistent;
Snapper's next quota query then requests the rescan.
Giving `snapperd` idle CPU or I/O priority is also not a demonstrated fix:
the helper already has idle scheduling,
and filesystem transaction work runs in kernel workers.

Mitigation ranking after the capacity gate:
hourly creation with daily cleanup,
then daily creation and cleanup,
then measured structural exclusion,
then full-qgroup removal,
then simple quotas,
then threshold 2 or daemon scheduling.
Hourly creation with daily cleanup ranks above daily creation because it preserves more restore points with the same
deletion window.
Daily creation ranks above structural work because it is reversible and requires no migration.
Measured structural exclusion ranks above quota changes because it targets snapshotted data
while preserving restore points.
Full-qgroup removal ranks above simple quotas because both lack a fixture latency benefit,
but simple quotas also invalidate parent usage accounting.
Threshold and daemon scheduling rank last because neither has evidence of reducing deletion impact.

## Verified local actions

### Lower the Snapper free-space limit to 100 GiB

The inactive cleanup service was confirmed before changing the configuration.
The previous `/etc/snapper/configs/root` was preserved at
`/var/home/user/temp/agent/snapper-root.before-free-limit-100GiB-20260828`.
`snapper --config root set-config FREE_LIMIT=100GiB` updated the live configuration.
Both `snapper get-config` and the configuration file reported `FREE_LIMIT=100GiB` afterward.
Automatic creation and cleanup remained enabled on their existing hourly timers,
and no cleanup was triggered manually.
The 01:36 scheduled cleanup had already removed timeline snapshot 964 before this change;
boot snapshot 963 remained.

The first post-change timeline snapshot was created at 02:00.
Cleanup ran from 02:36:17 to 02:39:04 without emitting a deletion message,
and timeline snapshot 964 remained afterward.
The filesystem had 182.881 GiB available,
82.881 GiB above the new limit,
and full qgroups were consistent.
This verifies that the new free-space condition prevents the immediate create-delete loop.
It does not prevent later deletion when fixed retention limits are exceeded,
nor prove that deletion impact is resolved.
The 05:43 recurrence occurred when fixed retention deleted snapshot 965,
confirming that limitation.

### Move cleanup to midnight with boot catch-up

Hourly timeline creation remains enabled.
The timer override at `/etc/systemd/system/snapper-cleanup.timer.d/schedule.conf`
clears the vendor unit's `OnBootSec=10m` and `OnUnitActiveSec=1h` triggers,
adds `OnCalendar=*-*-* 00:00:00`,
and sets `Persistent=true`.
The user selected boot catch-up despite its risk of running during an active session.

After daemon reload and timer restart,
`systemctl show` reported an active waiting timer,
`Persistent=yes`,
and its next trigger at 00:00 on August 29.
A persistent stamp was created at 05:53;
the next trigger remained midnight and no immediate run occurred.
Hourly timeline creation's next trigger remained unchanged.
The failed cleanup state from the observer-contaminated daemon crash was reset,
and the restarted Snapper daemon was healthy.

Daily cleanup can batch roughly one day of hourly snapshots,
so its deletion interval may be longer than an hourly cleanup.
Timeline creation and cleanup both become eligible at midnight;
Snapper serializes DBus work,
but their ordering is not guaranteed.
If the filesystem approaches the 100 GiB free-space limit,
free-space deletion pressure returns inside the midnight window.
`Persistent=true` catches up only after the timer was inactive across a missed calendar event;
an awake or suspended system still uses the midnight event normally.

The first midnight run confirmed the batching risk.
It requested deletion of 18 timeline snapshots and remained active for 2 hours 53 minutes.
The active helper and daemon were stopped after preserving the incident,
but the stop request itself took 161 seconds.
The user rejected snapshot deletion as the root-cause conclusion,
so no second timer-policy change was made.
Hourly creation and the midnight cleanup timer remain configured while physical-device latency is measured.

Removing the override,
reloading systemd,
and restarting `snapper-cleanup.timer` restores the vendor hourly schedule.
The restored monotonic timer may schedule cleanup within an hour of restart.

### Remove Panel Colorizer from every panel

This user-requested maintenance action was completed and verified through Plasma's scripting host,
`kpackagetool6`,
and a Plasma Shell restart.
It was not a root-cause finding or a verified incident workaround.

Tradeoffs:

- Panel Colorizer styling and automatic preset behavior are gone.
- Its saved presets are absent from the live configuration.
- No causal or remediation inference follows from removal,
  regardless of what happens during the remaining observation window.
- Removal prevents reproducing the panel freeze under the old four-instance configuration.

### Keep low-rate event-loop and resource monitoring active

The current observation uses one-second procfs and block-stat sampling plus five-second Plasma/KWin DBus probes.
The lower probe rate avoids the earlier four-pings-per-second pattern.
A user-visible original episode should be marked immediately with `now` so the surrounding evidence can be preserved.

Tradeoffs:

- Five-second probes can miss shorter event-loop stalls.
- DBus probes are not perfectly passive.
- Procfs scheduler counters do not detect every blocked or busy event-loop failure.

### Use the clean encrypted SATA scrub as a new baseline

Preserve irreplaceable files from `/var/mnt/encrypted` despite the clean scrub.
Single-profile file data still has no second Btrfs copy from which scrub can repair future corruption.
Monitor `btrfs device stats /var/mnt/encrypted` for any increment from the new all-zero baseline.

The user prioritized integrity verification over preserving the remaining clean desktop observation.
The foreground scrub ran with:

```sh
# doc/troubleshooting/bazzite-desktop-input-stalls.md
sudo btrfs scrub start -B --limit 100M /var/mnt/encrypted
```

The `--limit` option did not remain active;
the verified live-limit workaround is documented in the "Foreground scrub limit resets before completion" section.
The live workaround held the active limit at 100 MiB/s.
The scrub checked 1.42 TiB in 3:28:01 and reported no errors.
The starting `corruption_errs` value remained 405,004 until it was recorded and deliberately reset.
The live scrub limit was then restored from 100 MiB/s to its prior sysfs value of `0`.

Desktop resource measurements from 06:14:26 through 09:42:27 are scrub-contaminated
and cannot represent natural idle behavior.
The Plasma and KWin event-loop probes remained useful for detecting service boundaries,
but every overlap with scrub requires explicit classification.

## Retained targeted diagnostic plan

The original symptom was captured and passive observers were stopped.
This plan is dormant while mitigation is prioritized.
If the user later resumes causal investigation,
use staged instrumentation rather than another unchanged observation window.

First,
arm a conditional collector when `snapper-cleanup.service` becomes active.
Discover the transient `systemd-helper` and DBus-activated Snapper process instead of retaining fixed process IDs.
Keep a ring buffer in `/run/user/1000`,
which is a measured `tmpfs`,
and persist it only after a Plasma probe failure or another declared incident.
For Snapper,
Plasma,
KWin,
and Helium threads,
record `/proc` wait channels,
current syscalls,
kernel stacks,
scheduler counters,
and I/O counters alongside PSI and existing DBus timings.
The collector must run with the access needed for `/proc/<pid>/stack` and `/proc/<pid>/syscall`.
Those files are ptrace-gated and expose kernel stacks or current syscall state,
not a complete userspace call path.
Validate the collector on a responsive cleanup before interpreting an incident capture.

Second,
develop and validate any deeper tracing against a disposable loop-backed Btrfs filesystem.
Use matched cases with qgroups disabled,
qgroups enabled without deletion,
and qgroups enabled with snapshot deletion.
Use a separate canary with an intentional delay to prove that the detector can observe delay.
This fixture can validate tools and show that a mechanism is possible,
but a negative result cannot exclude behavior on the production root filesystem.
The fixture must have explicit storage,
CPU,
memory,
and duration limits and must never use either mounted production filesystem as its Btrfs target.

Third,
if procfs evidence remains inconclusive,
consider bounded scheduler and off-CPU tracing during a natural cleanup.
Upstream `perf sched` can report task runtime,
runnable delay,
and sleep intervals;
`perf record` can collect call graphs and off-CPU samples.
`bpftrace` can observe scheduler and block-I/O tracepoints.
None of `perf`,
`bpftrace`,
or `trace-cmd` is installed on this host as of 2026-08-27,
so this stage would first require a separately reviewed installation and smoke test.
Establish a responsive cleanup-duration control before treating a traced incident as natural evidence.

Do not attach GDB or `eu-stack` to Plasma or KWin during another natural incident.
The earlier all-thread collection paused Plasma and contaminated the recovery boundary.
If the preceding stages narrow the suspect operation,
a final fallback is a syscall-filtered `strace` of Snapper's helper or daemon only,
never an unrestricted trace of the desktop processes.

Ranking:
conditional procfs capture,
then disposable-fixture tool validation,
then bounded `perf` or BPF tracing,
then Snapper-only `strace`.
The ordering preserves the production incident first,
validates each more intrusive observer before use,
and leaves process-stopping attachment out of natural captures.

Sources:

- [`proc_pid_stack(5)`](https://man7.org/linux/man-pages/man5/proc_pid_stack.5.html)
- [`proc_pid_syscall(5)`](https://man7.org/linux/man-pages/man5/proc_pid_syscall.5.html)
- [`perf-sched(1)`](https://www.man7.org/linux/man-pages/man1/perf-sched.1.html)
-
[Upstream `perf-record(1)` source documentation](https://github.com/torvalds/linux/blob/master/tools/perf/Documentation/perf-record.txt)
- [bpftrace one-line tutorial](https://bpftrace.org/tutorial-one-liners)
- [Btrfs qgroup performance and simple-quota documentation](https://btrfs.readthedocs.io/en/stable/btrfs-quota.html)
- [Btrfs `drop_subtree_threshold` documentation](https://btrfs.readthedocs.io/en/stable/ch-sysfs.html#uuid-qgroups)
-
[2026 `linux-btrfs` report about qgroup-induced system hangs](https://www.spinics.net/lists/linux-btrfs/msg165086.html)
-
[`systemd.timer` persistent calendar semantics](https://www.freedesktop.org/software/systemd/man/latest/systemd.timer.html)
- [Linux Kyber scheduler documentation](https://docs.kernel.org/5.17/block/kyber-iosched.html)
- [Linux cgroup v2 I/O controller documentation](https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html#io)
- [Linux buffered-writeback throttling source](https://github.com/torvalds/linux/blob/master/block/blk-wbt.c)

## What does not work

### Foreground scrub limit resets before completion

The installed `btrfs-progs` is version 7.1,
source tag `v7.1` commit `4ab0e80be9e3bb1db2e6038e6d4316d35fb7ba8b`.
The requested `scrub start --limit 100M` did not persist:
a live status showed 531.07 MiB/s,
`btrfs scrub limit` displayed no limit,
and the sysfs value was `0`.

The source saves the old value and writes the requested value
(`cmds/scrub.c:1411-1418`):

```c
sp[i].old_limit = read_scrub_device_limit(fdmnt, devid);
ret = write_scrub_device_limit(fdmnt, devid, throughput_limit);
```

After creating the scrub and progress threads,
it restores the old limit before joining the scrub thread
(`cmds/scrub.c:1595-1608`):

```c
for (i = 0; i < fi_args.num_devices; ++i) {
    /* Revert to the older scrub limit. */
    ret = write_scrub_device_limit(fdmnt, di_args[i].devid, sp[i].old_limit);

    if (sp[i].skip)
        continue;
    devid = di_args[i].devid;
    ret = pthread_join(t_devs[i], NULL);
```

The same ordering remains in upstream commit `6797ce7600556138081382441bbc6104f35736e2`.
No matching open or closed upstream issue or pull request was found using
`scrub limit reset foreground`.

The separate live command applied the requested limit:

```sh
# doc/troubleshooting/bazzite-desktop-input-stalls.md
sudo btrfs scrub limit --all --limit 100M /var/mnt/encrypted
```

`btrfs scrub limit` then reported `100.00MiB`,
the sysfs value became `104857600`,
and subsequent block samples read 98.61 to 102.13 MiB per second.
This workaround leaves the sysfs limit set after scrub completion,
so it must be restored to its prior value of `0`.

### Treating the panel episode as the original stutter

The user explicitly distinguished them.
The panel episode affected all panels while ordinary windows and pointer motion remained normal.
The original episode delays input everywhere.
Combining them would produce the wrong diagnosis.

### Treating early `REG_WAIT` timestamps as proof of harmlessness

The source proves a register failed to reach its expected state within the requested poll count.
It does not prove that the same diagnostic can never matter during a runtime display transition.
The local evidence only shows that retained events occurred early and did not coincide with a captured stall.

### Tuning undocumented AMDGPU parameters first

No local reset,
fault,
ring timeout,
or runtime `REG_WAIT` supports an AMDGPU parameter change.
Changing display-core or power parameters now would add an uncontrolled variable without a matching failure signal.

### Attributing the panel freeze to Panel Colorizer from presence or removal

Panel Colorizer was present on every panel during the 04:25 episode,
and it was later removed at the user's request.
Neither fact establishes cause.
The broad Plasma-centered symptom later recurred while it was absent,
which establishes only that Panel Colorizer was not required for the later event.
That recurrence does not prove whether it contributed to the earlier event.

### Forcing Qt Quick's basic render loop as a presumed fix

The captured process used Qt Quick's threaded render loop without a `QSG_RENDER_LOOP` environment override.
KDE previously
[forced the basic loop on Wayland](https://invent.kde.org/plasma/plasma-workspace/-/commit/9c998d3083f622a1677782248d4c8e238c935dc2),
then
[reverted that change](https://invent.kde.org/plasma/plasma-workspace/-/commit/c97448ebe5987acd08da9475733f92931c074b95)
because the basic loop reused one OpenGL context across windows with incompatible EGL configurations.
A forced-basic session is therefore an untested diagnostic experiment with known tradeoffs,
not a verified fix.

### Calling Snapper the cause from qgroup warnings alone

The warning and rescans are real,
but many fully observed cleanup and deletion cycles had no visible symptom and no KWin delay.
The 00:40 original-symptom recurrence occurred during deletion after its qgroup scan completed.
This makes the cleanup path the leading trigger hypothesis,
not proof that qgroup warnings or Snapper itself caused the delay.

### Claiming the high-rate monitor prevented the bug

The original symptom did not occur while that monitor ran,
but no controlled comparison proved suppression.
The high-rate KWin and AMDGPU probes were removed as possible experimental confounders,
not as a demonstrated cause or cure.

### Filing the separate panel incident as a Panel Colorizer bug

Panel Colorizer issue
[luisbocanegra/plasma-panel-colorizer#100](https://github.com/luisbocanegra/plasma-panel-colorizer/issues/100)
describes taskbar items becoming unclickable and queued or misdirected click regions,
but it concerned version 1.0.0 and a reproducible panel-mask condition.
The workaround was merged in 2024.
The 04:25 episode on 6.5.0 did not capture a stack or panel-mask state.
The later Plasma-centered episode recurred after the package and helpers were absent,
so it adds no evidence for a Panel Colorizer defect.

Issue
[luisbocanegra/plasma-panel-colorizer#556](https://github.com/luisbocanegra/plasma-panel-colorizer/issues/556)
describes lag from repeated full `panelWidgets` configuration writes.
The 04:25 episode had no matching dm-0 or PSI stall,
and the later recurrence had no Panel Colorizer process that could write this setting.
There is no additive evidence for that thread.

## Upstream filing decision

No `.out-of-scope/` entry matches Bazzite,
KWin,
Plasma,
AMDGPU,
Snapper,
Btrfs,
or Panel Colorizer.

### Original global-input episode

1.  **Is it really upstream's fault?**
    Unknown.
    The captured episode is associated with Snapper cleanup and Btrfs deletion,
    but no cross-application blocking path is identified.
2.  **Can upstream fix it?**
    Possibly after timer-pause or targeted-tracing evidence separates Snapper,
    Btrfs,
    and desktop behavior.
3.  **Are they supporting this use case?**
    Snapper supports automatic Btrfs snapshot cleanup,
    and the desktop and input paths are supported.
4.  **Would the repository welcome a contribution?**
    Snapper issue 337 previously accepted cleanup-time performance reports,
    but the responsible project remains unsettled here.
5.  **Will they likely fix it?**
    Unknown without the blocking path or a repeatable trigger.
6.  **Has a minimal compatible fix been prototyped?**
    No.
    The proposed timer pause is a diagnostic workaround,
    not a software fix.

Decision:
do not file the stall yet.
Preserve the 00:40 and 05:43 incidents and evaluate the midnight-cleanup mitigation before assigning an upstream.

### Snapper daemon disconnected-client crash

1.  **Is it really upstream's fault?**
    The unhandled DBus exception is in Snapper's daemon,
    but this occurrence followed an observer-terminated client and is not a natural stall cause.
2.  **Can upstream fix it?**
    Yes.
    Snapper can handle sender disappearance around `get_unix_userid` without terminating the daemon.
3.  **Are they supporting this use case?**
    Yes.
    CLI clients can disconnect or enforce their own operation limits.
4.  **Would the repository welcome a contribution?**
    Not checked because a clean reproduction has not been built.
5.  **Will they likely fix it?**
    Unknown without a minimal reproduction and full DBus message identity.
6.  **Has a minimal compatible fix been prototyped?**
    No.

Decision:
do not file the contaminated occurrence.
Reproduce a disconnected queued client against a disposable or isolated daemon before proposing an upstream report.

### Panel Colorizer

1.  **Is it really upstream's fault?**
    No evidence establishes that.
    The later Plasma-centered episode recurred without Panel Colorizer.
2.  **Can upstream fix it?**
    No Panel Colorizer failure path has been identified.
3.  **Are they supporting this use case?**
    Yes.
    The project customizes Plasma panels and documents bug reporting.
4.  **Would the repository welcome a contribution?**
    Yes.
    `CONTRIBUTING.md` and the bug template accept focused reports after duplicate search.
    No AI-assistance prohibition was found.
5.  **Will they likely fix it?**
    Unknown because no defect in their current version is reproduced.
6.  **Has a minimal compatible fix been prototyped?**
    No.
    There is no diagnosed Panel Colorizer call path to change.

Decision:
do not file.
Constraints 1,
2,
5,
and 6 fail.
The upstream template also asks reporters to verify the latest release,
which cannot be done against the removed 6.5.0 configuration.
Existing issues #100 and #556 cover the nearest known behaviors,
but neither receives additive evidence from this investigation.

### Plasma or Qt Quick

KDE bug
[449163](https://bugs.kde.org/show_bug.cgi?id=449163)
describes superficially similar Wayland panel freezes,
but it was closed for an older Qt Wayland root cause.
KDE maintainers explicitly asked reporters to file focused new reports because similar freezes can have different
causes.
Qt bug
[QTBUG-37677](https://bugreports.qt.io/browse/QTBUG-37677)
documents input queuing when rendering is blocked,
but concerns Qt 5 on Mir and X11 rather than this Qt 6 Wayland path.
Neither is a demonstrated duplicate.

1.  **Is it really upstream's fault?**
    The failed boundary is inside Plasma and Qt Quick,
    but the initiating Plasma window and underlying trigger remain unknown.
2.  **Can upstream fix it?**
    Possibly after an uncontaminated reproducible trigger identifies the affected window or item.
3.  **Are they supporting this use case?**
    Yes.
    Plasma panels and Qt Quick's Wayland threaded render loop are supported paths.
4.  **Would the repository welcome a contribution?**
    KDE accepts focused plasmashell performance reports,
    and Qt accepts Qt Quick reports.
5.  **Will they likely fix it?**
    Unknown without a repeatable trigger or responsible project boundary.
6.  **Has a minimal compatible fix been prototyped?**
    No.
    Switching render loops was not treated as a fix because Plasma previously reverted a forced-basic workaround after
    it caused other Wayland rendering failures.

Decision:
do not file yet.
Constraints 1,
5,
and 6 remain incomplete.
The first failed probe and stack are additive evidence.
The second stack is excluded from component attribution because its window was observer-contaminated.
The responsible project and minimal reproduction are not established.

There is therefore no upstream issue or comment draft to preserve yet.
