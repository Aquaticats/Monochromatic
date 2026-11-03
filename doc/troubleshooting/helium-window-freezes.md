# Helium 0.20.2.1 launcher sessions stall on disk-backed diagnostic metrics writes

## Status at 2026-11-03T02:27:03-05:00

Captured main-thread stalls include writes to Chromium's `BrowserMetrics` memory mapping.
A launcher workaround now forces the verified memory-only fallback.
The user's subsequent manual launcher invocation has no `.pma` mapping.
This removes an observed blocking path;
it does not establish that every reported freeze has the same cause or fix the underlying filesystem latency.
The user reports whole-window freezes despite the existing I/O-weight mitigations,
and says Firefox does not show the problem.
The user specifically suspects excessive `fsync` calls
and requested a comparison with ordinary Chromium.
Browser restarts are authorized.

## Symptom

The entire Helium window becomes unresponsive,
including browser controls.
This is distinct from the historical Plasma and Helium incidents in
[`bazzite-desktop-input-stalls.md`](bazzite-desktop-input-stalls.md).
The user confirmed a normal-profile Helium freeze around `2026-11-03T01:59:00-05:00`.
During the same interval,
the normal browser main thread appeared in `wait_current_trans` across repeated samples.

## Root cause

### Metrics write faults block the main thread

At `2026-11-03T02:05:04-05:00`,
the original main thread faulted while writing address `0x7f453a1b61cc`.
The event's instruction pointer `0x55a645c36a65` matched the blocked thread's current instruction pointer.
The address fell inside its mapped
`BrowserMetrics/BrowserMetrics-6AE8D6B5-2176.pma` file.
The kernel stack showed:

```text
# /proc/8566/stack during the matching write fault
folio_wait_bit_common
folio_wait_writeback
btrfs_page_mkwrite
do_page_mkwrite
do_wp_page
__handle_mm_fault
handle_mm_fault
do_user_addr_fault
exc_page_fault
asm_exc_page_fault
```

Later matching faults in the same mapping showed
`wait_current_trans`,
`start_transaction`,
`btrfs_dirty_inode`,
`file_update_time_flags`,
and `btrfs_page_mkwrite`.
A captured metrics-write wait occupied `6,622 ms` of observed blocked time
from `02:13:58` to `02:14:04` local time.
These intervals were sampled at `100 ms`,
not measured as exact fault latency.

The observer recorded write faults only.
A later read fault can retain a stale preceding write event.
Attribute a mapping only when the instruction pointer matches
and the kernel stack shows the corresponding write-fault path.
The unmatched `filemap_fault` samples are not attributed to metrics.
Raw evidence is in
`${HOME}/temp/agent/helium-fsync-capture-20261103/pagefaults.jsonl`.

### Scope beyond Helium checked at 2026-11-03T02:42:31-05:00

The captured storage-wait behavior is not exclusive to Helium.
The retained main-thread observer also recorded ordinary Chromium PID `2972354`
in `folio_wait_writeback` through `btrfs_page_mkwrite`,
including a sample at `2026-11-03T02:08:41.068-05:00`.
Its process name was `chromium-browse`,
and the observer selected browser main processes while excluding `--type=` subprocesses.
Evidence is in
`${HOME}/temp/agent/helium-fsync-capture-20261103/main-thread.jsonl`.

This establishes the same class of main-thread filesystem wait in ordinary Chromium.
It does not establish a matching user-visible freeze or identify Chromium's faulting file:
only Helium had the fault address correlated with its `BrowserMetrics` mapping.
The metrics initialization itself is shared upstream Chromium code,
as shown in "Chromium creates mapped metrics before feature flags".
Other Chromium-derived browsers and operating systems were not tested for this failure.

### Chromium creates mapped metrics before feature flags

The installed Chromium revision is `2f9a11097447f226133bc1a31c787291a5eeab98`.
Its [`chrome/app/chrome_main_delegate.cc:1745-1765`][early-init]
explicitly initializes mapped histograms before field trials:

```cpp
// chromium/chrome/app/chrome_main_delegate.cc
InstantiatePersistentHistograms(
    metrics_dir,
    /*persistent_histograms_enabled=*/true,
    /*storage=*/kPersistentHistogramStorageMappedFile);
```

The source comment explicitly says the `kPersistentHistogramsFeature`
and `kPersistentHistogramsStorage` values will not be used at this point.

[`components/metrics/persistent_histograms.cc:124-164`][mapped-init]
constructs an active file inside `BrowserMetrics`,
moves the spare file into place,
and attempts a mapped allocator:

```cpp
// chromium/components/metrics/persistent_histograms.cc
base::ReplaceFile(spare_file, active_file, nullptr);
```

The verified fallback is in
[`components/metrics/persistent_histograms.cc:212-222`][memory-fallback]:

```cpp
// chromium/components/metrics/persistent_histograms.cc
base::GlobalHistogramAllocator* allocator =
    base::GlobalHistogramAllocator::Get();
if (!allocator) {
  // If no allocator was created above, try to create a LocalMemory one here.
  // This avoids repeating the call many times above. In the case where
  // persistence is disabled, an early return is done above.
  base::GlobalHistogramAllocator::CreateWithLocalMemory(kAllocSize, kAllocId,
                                                      kBrowserMetricsName);
}
```

### Filesystem latency and flush counts are separate observations

The confirmed incident includes a Btrfs transaction wait on the normal Helium main thread.
The initial syscall tracer covered flush calls only;
it did not identify the main thread's blocking syscall.
The normal browser did not have a flush lasting over a second in the preserved incident interval.
Do not replace this evidence with an unproven claim that its main thread called `fsync`.

Meanwhile,
a disposable Helium profile on the same filesystem had a `fdatasync` lasting `40,900.204 ms`.
The blocked worker's kernel stack repeatedly showed:

```text
# Captured disposable Helium worker stack
wait_log_commit
btrfs_sync_log
btrfs_sync_file
do_fsync
__x64_sys_fdatasync
```

Other disposable-profile flushes completed after `37,851.815 ms` and `12,620.263 ms`.
Those waits overlapped;
their sum is not elapsed stall time.
These observations support investigating shared filesystem transaction latency,
but do not establish excessive flush frequency or identify the transaction's initiating workload.

The launcher is `${HOME}/.local/share/applications/helium.desktop`.
Its original main `Exec` started `${HOME}/AppImages/helium.appimage` directly.
The live service had no explicit `IOWeight`,
CPU quota,
or memory cap.
Helium reports `0.20.2.1`,
using Chromium `156.0.8078.73`.
The distribution Chromium reports the same Chromium version.

The persisted Helium flags include `enable-vulkan@1`,
`default-angle-vulkan@1`,
`skia-graphite@1`,
and `force-enable-webgpu-interop@1`.
The GPU command line contained `--gpu-recent-crash-count=3`.
The session journal records repeated GPU initialization warnings:

```text
# Original Helium session journal
ui/ozone/platform/wayland/gpu/wayland_surface_factory.cc:249
'--ozone-platform=wayland' is not compatible with Vulkan. Consider switching to '--ozone-platform=x11' or disabling Vulkan
```

These are leads,
not an explanation of the reported window freezes.
Graphics configuration remains unchanged while storage is measured.

## Verification

The kernel is `7.2.15-ogc1.1.fc45.x86_64`.
The NVMe scheduler is BFQ.
The first host I/O PSI snapshot had `some avg10=34.32` and `full avg10=25.91`.
That snapshot was not tied to a user-marked freeze.

A private tracefs instance records `fsync` and `fdatasync` entry and exit events
for Helium and comparison-browser threads.
The collector uses kernel timestamps,
resolves file descriptors where they remain available,
and records procfs state and host I/O pressure.
It does not attach with `ptrace`.

The diagnostic source is retained at
`${HOME}/.local/state/helium-freeze-workaround/diagnostics/fsync-trace.ts`.
The capture directory is
`${HOME}/temp/agent/helium-fsync-capture-20261103`.
The system service is `helium-fsync-trace.service`,
bounded to a day,
one CPU,
512 MiB of memory,
64 tasks,
and `IOWeight=10`.
Each log retains its current file and bounded rotations at a `16 MiB` threshold.
The kernel buffer size is `8,256 KiB` across this host's CPUs.
The recorder expires at `2026-11-04T02:24:45-05:00`.
The kernel instance is removed on normal service shutdown.
The updated process sampler also captures kernel stacks and current syscalls of blocked main threads.
File-descriptor names are best-effort observations after event delivery;
descriptors may close or be reused.
The updated collector marks whether resolution preceded the syscall return.
Do not treat delayed names as definitive call-site attribution.
Private paths occur in the local evidence;
no trace was uploaded.

The collector emits a single real disk flush as a positive control.
The initial parser rejected the kernel's rendered `sys_fsync(...)` syntax;
this was caught before trusting the empty count and corrected.
The first root service launch also failed with `status=203/EXEC`.
The verified `/usr/bin/env` workaround from
[`selinux-denies-system-service-exec-of-home-files.md`](selinux-denies-system-service-exec-of-home-files.md)
allowed it to start.

### Confirmed incident capture at 2026-11-03T01:59:00-05:00

Preserved evidence is in
`${HOME}/temp/agent/helium-fsync-capture-20261103/incident-20261103T065900Z`.
It includes syscall events,
blocked-worker stacks,
process state,
and host pressure for `06:59:00Z` to `07:00:10Z`.
The normal browser main PID was `8566`.
Its `wait_current_trans` state appeared in `34` samples,
interspersed with normal `poll` waits.
Do not interpret that sample count as a continuous measured duration.

Temporary main-thread and write-fault observers captured the metrics mapping.
Those observers and the comparison browsers have stopped.
The current browser and the requested bounded recorder remain active.

### Comparison scope

Firefox ESR is the user's comparison browser;
its installed version is `153.6.0esr`.
The matched browser profiles live in
`${HOME}/.config/helium-browser-comparison-20261103`,
on the same Btrfs filesystem as the normal Helium profile.
The first disposable probe under `${HOME}/temp/agent` used a different encrypted SATA filesystem
and is excluded from the matched comparison.

The comparison separates startup,
idle,
default-durability IndexedDB transactions,
strict-durability IndexedDB transactions,
local-storage writes,
and settling.
Fresh profiles do not reproduce the user's extensions,
tabs,
history,
or experimental flags.
The browser order was reversed for the repeated pass.

Each completed run reported `20` IndexedDB transactions and `10` local-storage writes.
Completed-run flush counts were:

- Helium:
  `300` and `291`,
  all `fdatasync`.
- Chromium:
  `298` and `297`,
  all `fdatasync`.
- Firefox ESR:
  `3,824` and `4,359`,
  all `fsync`.

These counts include fresh-profile initialization and browser background activity.
They are not steady-state browsing rates,
and `fsync` and `fdatasync` have different contracts.
They do not support Helium uniquely issuing more flushes than Chromium for this workload.
They do not establish that Firefox is less efficient in the user's normal use.

Latency varied within unchanged configurations:
the longest Helium flush was `40,900.204 ms` in one completed run and `5,712.352 ms` in the other.
Do not rank browser storage latency from those maxima.
Existing host work continued throughout;
no synthetic host load was generated.

A Firefox attempt exceeded the original `120 s` harness deadline during settling.
It is excluded from completed-run counts.
The replacement used a `300 s` deadline and completed.
The phase markers and computed results are `phases.jsonl` and `analysis.json`
inside the comparison directory.
The comparison capture's final kernel statistics reported no overruns or dropped events.

### Workaround checks

The failing catalog includes the original live mapped-metrics stalls
and a disposable launch where `--disable-features=PersistentHistograms` still created the mapping.

The working catalog includes a non-writable metrics directory in a disposable profile,
a subsequent launch through the committed helper,
and the user's manual launch through the modified desktop entry.
Each had no `.pma` mapping after initialization.
The manually launched browser PID was `3074508` and remained running in repeated checks.

The helper's TypeScript check passed:

```sh
# Run from doc/troubleshooting/helium-window-freezes
mise run lint:types
```

Root task discovery only includes `package/*/*`.
The owning-directory invocation works;
the attempted fully qualified invocation from the root did not discover this diagnostic task.
See [`mise-excluded-research-task-discovery.md`](mise-excluded-research-task-discovery.md).

## Verified workarounds

The committed [launcher helper](helium-window-freezes/launcher.ts)
prepares the profile's `BrowserMetrics` directory with mode `0500`,
then replaces itself with the Helium AppImage and the supplied arguments.
The failed disk-mapping initialization takes Chromium's built-in local-memory fallback.
Helium cleanup removes that directory,
so the helper prepares it on every invocation.
A one-time permission change is insufficient across launches.

Installed locations are `${HOME}/.local/lib/helium/launcher.ts`
and `${HOME}/.local/share/applications/helium.desktop`.
The main,
new-window,
and private-window desktop commands use the helper through the stable installed Node exposure path.
`desktop-file-validate` passed.

The tradeoff is loss of on-disk persistence for diagnostic histograms.
Database flush behavior is unchanged.
Other filesystem waits remain possible,
including writes to the remaining GPU-cache mappings.
An AppImage integration update may replace the desktop entry;
verify the launch command after such an update.

The original desktop entry is retained at
`${HOME}/.local/state/helium-freeze-workaround/20261103T071500Z/helium.desktop`.
Restoring its original commands removes the helper from the launch path.

### Session restoration failure

The automated restart did not restore the user's session.
The user launched Helium themselves and accepted continuing with a new session.
Do not claim that session restoration passed.
Additional automated launches exited;
a service log includes a startup core dump whose cause was not established here.

Original preferences and session files were backed up before termination in
`${HOME}/.local/state/helium-freeze-workaround/20261103T071500Z`.
The session files total `2,065,088` bytes and have `SNSS` version `3` headers.
They remain available for possible recovery;
they have not been copied over the user's new session.

## What does not work

- A normal `poll` wait in a single browser-main-thread snapshot cannot establish UI responsiveness.
- Host I/O pressure alone cannot identify the operation blocking Helium.
- GPU initialization errors alone cannot identify the cause of a later window freeze.
- Comparing a populated Helium profile with a fresh Chromium profile cannot isolate browser implementation differences.
- `--disable-features=PersistentHistograms` does not prevent the installed startup path from creating mapped metrics.
- An early process-start mapping check is insufficient;
  initialization and continued execution must also be checked.

## Upstream filing decision

No upstream report or generated issue draft was sent.

- Responsibility:
  the synchronous mapped-metrics write is observed;
  the initiating cause of the filesystem latency remains unknown.
- Fixability:
  the tested local-memory fallback removes the observed mapping.
- Supported use:
  ordinary desktop browsing on the installed Linux build.
- Contribution policy:
  Helium's [inspected policy][helium-policy] forbids AI-generated issue and pull-request descriptions.
- No maintainer decision about this incident has been established.
- The consumer-side workaround was implemented and exercised;
  no upstream source patch is claimed.

## Next action

The next useful evidence is a recurrence in the current launcher session,
correlated with the retained syscall and blocked-main-thread capture.
The lost old session means ordinary pre-change and post-change browsing workloads are not identical.
Do not claim an end-to-end freeze-rate reduction from the mapping check alone.

[early-init]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/app/chrome_main_delegate.cc#L1745-L1765
[mapped-init]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/metrics/persistent_histograms.cc#L124-L164
[memory-fallback]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/metrics/persistent_histograms.cc#L212-L222
[helium-policy]: https://github.com/imputnet/helium/blob/2f81b9cafa38a95071e20de3f9cd6b0aca6d3c65/CONTRIBUTING.md#L17-L21
