# Chromium live histogram mapping patch candidates

## Source assessment at 2026-11-03T11:16:59+00:00

The user requested a patch to attach to Chromium issue `581090722`.
This document supports that patch work;
it does not claim that a patch has been implemented or tested.
The main task owns implementation and validation.
This is same-vendor source research,
not an independent review.

The measured browser was Chromium `159.0.8141.0`,
snapshot `1733707`,
source `986947eb4afc18471e51b7cb473f7c2f779f62bf`.
Current main was fetched as `9cbb3a49a216136a4ee0337003fa53a3b3fe61ff`,
commit time `2026-11-03T10:56:54Z`.
The allocator,
startup,
consent,
file-provider,
shared-memory,
and existing-test files cited here were byte-compared between those revisions.
The comparison covers `49` selected files;
each matched between revisions.

The [retained reproduction](../../troubleshooting/chromium-disk-responsiveness/live-histogram-reproduction.md)
establishes browser-main writes through a live regular-file-backed `BrowserMetrics` mapping.
The faulting instruction is `lock xadd %r12d,(%rax)`
in `base::SampleVectorBase::Accumulate(int, int)`,
at ELF virtual address `0xce44e00` in that binary.
The candidate must remove that storage dependency from live recording,
not merely move explicit flush calls.

## Recommended first candidate

Prototype choosing the existing `LocalMemory` storage mode at initial browser allocator creation
when an unbranded Chromium process cannot report metrics.
Preserve the mapped mode for explicitly forced reporting and deliberately supported test overrides.
Keep the decision in Chrome's startup/metrics policy boundary,
with `components/metrics/persistent_histograms.cc` continuing to own storage construction.
This is a proposed bounded patch,
not a complete fix for reporting-enabled Chrome.

The evidence for this boundary is:

- [Chrome startup][startup] unconditionally chooses `MappedFile`
  before field trials and before the regular metrics service exists.
- [The metrics accessor][reporting-policy] disables reporting in non-Google-branded builds,
  except explicit force-reporting and a test-only override.
- [Disabled source registration][disabled-deletion] schedules recursive deletion of atomic metrics directories.
  [Chrome's provider registration][browser-source] registers `BrowserMetrics` in that category.
- [The existing local mode][local-mode] retains the global histogram allocator,
  system-profile registration,
  allocator tracking,
  and ordinary/sparse histogram machinery without a regular-file mapping.
  [Its POSIX backing][local-backing] uses anonymous shared `mmap`,
  with a heap fallback.

The practical benefit to prove is narrower than "no browser I/O":
ordinary and sparse samples continue recording,
while this process has no active regular-file-backed `BrowserMetrics` mapping.
The patch should not disable histogram recording,
alter UMA sample types,
or replace the global allocator after installation.
Anonymous memory can still encounter swap or memory-pressure stalls;
the proposed property concerns regular-filesystem-backed live recording.

The limitation must be explicit in the patch title and description:
this candidate addresses unbranded processes whose reporting policy discards the persisted files.
It does not yet solve live mapped writes for an opted-in Google Chrome user,
or a branded user who revokes consent after the allocator exists.

### Startup policy exceptions

`metrics::IsMetricsReportingForceEnabled()`
is an [early command-line check][reporting-switches],
so preserving that path does not require field-trial initialization.
Do not exercise forced reporting against real servers in a test.

`--metrics-recording-only` is a separate switch.
It enables recording for testing;
it is not itself evidence that the file provider will retain `BrowserMetrics` for upload.
The patch's test matrix should cover that path without silently equating recording with persistence.

`MetricsServiceAccessor::IsForceMetricsReportingEnabledPrefLookup()` exposes a base test override.
[Chrome's accessor][chrome-accessor] also has a separate consent override used before browser startup.
The early decision must account for the chosen test hook explicitly.
Calling the existing local-state accessor with a null `PrefService` is not a valid production consent query.

Branded Chrome consent requires more than substituting a cached crash-upload flag.
[The POSIX backup-file accessor][backup-consent] derives a cached/on-disk consent indication,
while regular UMA consent comes through local-state reporting policy.
First-run opt-in,
enterprise policy,
sampling,
and same-session consent changes can affect the later service state.
If an early RAM-only choice is extended to these users,
the design must say how later opt-in obtains crash persistence without replacing existing histogram pointers.
That is a genuine behavior decision,
not a mechanical reuse of the unbranded condition.

### Delayed spare creation

[Chrome's later cleanup call][cleanup-call] is currently unconditional.
[Persistent histogram cleanup][cleanup] uses a [desktop delay][cleanup-delay] of `90` seconds for spare creation.
Choosing `LocalMemory` alone therefore leaves that background file operation in place.

If the first patch suppresses spare creation for a deliberately RAM-only mode,
use the intended storage policy rather than `HasPersistentLocation()` as the condition.
A requested mapped allocation can fail and fall back to local memory;
that failure path may still need a spare for the next run.
Retain the existing handling of legacy artifacts and other platform callers.
Do not claim that no `.pma` file can exist merely because the active mapping is anonymous.

## What a deleted mapping establishes

Both reported captures showed `(deleted)` on the active mapping.
That establishes the observed mapping's pathname state,
not who unlinked it.
The disabled-provider cleanup is a source-supported candidate unlink path;
it has not been traced as the unlink actor in those captures.

[`DeletePersistentLocation()`][delete-location] also demonstrates why unlinking cannot be the remedy:
it marks the allocator deleted and opens the file with delete-on-close,
while explicitly retaining the behavior of already-open handles.
The method does not remap existing memory or redirect existing histogram objects.
The [global allocator contract][allocator-lifetime] forbids replacing or destroying the allocator
because histograms retain pointers into its memory.
Leaking the old allocator during a late swap would leave those old callers on the old mapping.

The proposed first patch chooses anonymous backing before any such pointers are published.
Trace the file deletion in a disposable baseline if the patch description attributes the observed unlink to policy cleanup.
Otherwise preserve the attribution limit.

## Existing RAM and persistence mechanisms

### Global local-memory and shared-memory allocators

[`CreateWithLocalMemory()`][allocator-creation] creates a `LocalPersistentMemoryAllocator`.
Its "persistent" layout supports the histogram machinery;
the name does not mean that anonymous memory survives process death.

[`CreateWithSharedMemoryRegion()`][shared-allocator] maps an existing shared region.
The API contract says the memory can outlive one process when another process retains access.
[`HistogramSharedMemory::Create()`][shared-launch] and the child launch helpers already construct
and transfer writable histogram regions between parent and child processes.
They do not provide an existing browser-global disk collector.
Using shared memory for the browser requires an explicit owner for the retained handle
and for persistence after browser termination.

### File snapshot helpers

[`WriteToPersistentLocation()`][snapshot-write] atomically writes an allocator's used bytes
to a previously assigned pathname.
It has no periodic scheduler,
consent-generation check,
external lifetime owner,
or browser-crash callback.
It passes a view of the live allocator bytes to the file writer;
an atomic file replacement alone does not prove a coherent concurrent snapshot of histogram data.

[`PersistentHistogramStorage`][storage-destructor] allocates local memory and writes it on destruction.
It is used by helpers such as the installer and notification helper.
The destructor performs the write synchronously on its caller.
It is precedent for explicit RAM-to-file persistence,
not a drop-in asynchronous browser-crash solution.

Any snapshot design must coordinate total and logged sample counts,
ordinary single-sample optimization,
sparse records,
system-profile data,
and publication order.
[Final-delta imports][delta-contract] can double count if consumed more than once.
An old snapshot can also contain a stale logged-sample boundary after a newer upload.
Concurrent file replacement therefore needs accounting tests,
not only an atomic-rename test.

Consent revocation is another boundary.
The current delete helper does not clear `persistent_location_`,
and the snapshot writer can write to that pathname later.
A new asynchronous writer must prevent queued or in-flight persistence from recreating data
after the consent-driven deletion has completed.
Keep deletion and publication ordering in the persistence owner without waiting for disk on the UI thread.

### Existing Crashpad shared-memory stream

[`SystemProfileUserStream`][system-profile-stream] already retains a serialized system profile in shared memory.
Chrome passes a duplicate read-only region to the Linux Crashpad handler
[before starting the handler][crashpad-handoff].
[The stream writer][stream-writer] publishes a completed slot atomically;
it explicitly does not support concurrent writers without external synchronization.
[The handler source][stream-consumer] copies that stream into a minidump.

This is useful precedent for cross-process retention and stable snapshot publication.
It is not the same contract as next-run `.pma` collection:
it carries a system-profile payload into crash reports,
not all live histogram records into `FileMetricsProvider`.
A histogram collector must also handle termination paths that produce no minidump,
handler failure,
consent revocation,
shutdown,
and resource exhaustion.
Those behaviors have not been implemented or verified by this source pass.

## Broader design choices

Keeping browser recording in shared memory and retaining it in an external collector
is the candidate that can preserve metrics after browser-only process death
without making the browser write through a regular-file mapping.
It requires metrics/crash-owner agreement on collector lifetime,
storage publication,
consent,
and failure behavior.
Do not describe RAM as durable across machine power loss or a failure of both processes.

An in-process periodic snapshot is a simpler implementation direction,
but loses samples after the most recent completed snapshot when the process dies.
That loss window needs an explicit metrics-owner decision.
The current mapped design is not perfectly durable either:
[system-profile code already documents torn-page risk][profile-flush],
and [shutdown marking][shutdown] is explicitly best effort.
Those limits do not authorize an arbitrary new loss window.

Ranking:
initial RAM backing for reporting-ineligible unbranded processes > shared memory with an external collector
> in-process periodic snapshots,
because the first confines change to data the reporting path already discards;
the external collector can retain data across browser-only death,
whereas an in-process snapshot necessarily loses its unpublished tail.
The first option's coverage is narrower,
so it must not be presented as the complete solution for reporting-enabled Chrome.

Preallocation,
prefaulting,
`mlock`,
moving explicit flushes,
or lowering worker I/O priority does not remove the regular-file backing of the faulting instruction.
The service already has an [off-main-thread system-profile flush][profile-flush].
None of these is established as a fix for the observed write-fault dependency.

## Tests and build targets

The narrow candidate needs a testable early policy boundary and real-browser coverage:

- In an unbranded process with ordinary reporting disabled,
  verify the intended local mode,
  a live allocator without a persistent location,
  correct ordinary and sparse sample counts,
  and absence of the regular-file-backed active `BrowserMetrics` map.
- Verify the reporting-enabled path still creates its mapped allocator,
  retains the embedded system profile,
  and preserves next-run unlogged samples after terminating the disposable browser.
  Cover explicit force-reporting and test overrides without sending telemetry to public servers.
- Cover recording-only mode,
  mapped-allocation failure followed by local fallback,
  and the delayed spare behavior selected by the patch.
- Keep the existing consent-disabled and sampled-out cleanup tests passing.
  Revoke consent after startup and ensure no new persistence path recreates deleted files.
- Replay the measured bounded ordinary/sparse recording fixture on baseline and patched binaries.
  Verify the positive control can show the mapped-write fault,
  inspect the patched mapping,
  and measure a browser-main operation separately from renderer evaluation.
  Do not manufacture host load or infer latency improvement solely from source changes.

[`MetricsServiceBrowserFilesTest`][browser-file-tests] already sets consent before browser startup.
Its upload,
no-upload,
and sampled-out cases test retention/removal of non-PMA files.
They explicitly ignore `.pma` files and do not prove the active-mapping property.
Add an assertion that exercises that missing boundary.

[`FileMetricsProviderTest`][provider-tests] includes
`MetricsDisabledRegisterAtomicFile`,
`MetricsDisabledRegisterAtomicDir`,
and `MetricsDisabledRegisterActiveFile`.
The active-file case deliberately keeps its file,
so do not broaden atomic-directory deletion policy to every metrics source.

For the narrow Chrome startup policy patch,
the relevant native executables are `chrome/test:unit_tests` and `chrome/test:browser_tests`.
Chrome metrics supplies dedicated unit/browser source sets.
If the implementation changes the allocator or persistence helper,
also use `base:base_unittests` and the metrics source set linked into `components_unittests`.
The [build definitions][base-test-target] contain
`PersistentHistogramAllocatorTest`,
`PersistentHistogramStorageTest`,
and `HistogramThreadsafeTest` sources.
Useful existing cases include `FileReadonlyRecovery`,
`CreateWithFile`,
`MovePersistentFile`,
`StatisticsRecorderMerge`,
and `SnapshotDeltaThreadsafe`.
These targets and cases were located in source;
they have not been built or executed in this research pass.

A new asynchronous persistence design additionally needs fault-injection coverage for
snapshot publication,
duplicate accounting,
ordinary/sparse concurrent updates,
allocation failure,
blocked disk writes,
queued-generation bounds,
consent revocation during publication,
collector loss,
browser termination,
and shutdown.
Do not attach a patch as validated merely because the downloaded unmodified snapshot ran successfully.

## Ownership and remaining decisions

[Metrics ownership][metrics-owners] prefers `chromium-metrics-reviews@google.com` for review assignment.
[Chrome startup ownership][startup-owners] applies to the initial storage choice.
The inherited metrics owners also cover `chrome/browser/metrics`.
Changing Crashpad stream handling introduces the crash component's ownership boundary.

The concrete first implementation can stay within Chrome startup storage selection
and its tests while reusing the existing allocator.
The author must settle how the existing early test overrides select storage,
and whether spare-file work belongs in the same patch.
Broader coverage needs agreement on crash-loss and collector behavior before treating a new persistence design as equivalent.

## Verification at 2026-11-03T11:35:27+00:00

All `31` pinned source selections passed the `agent-browser` check
at `2026-11-03T11:27:25.866Z`.
The selected GitHub lines matched freshly fetched source and were highlighted.
The owned browser closed successfully,
and its systemd scope is inactive.
Scoped `mise run lint:monochromatic-lint` passed.

GitHub's repository-contents HTML renderer was exercised on commit `5e0dc5e6a1`.
Its heading sequence and inline code spans matched the Markdown input,
with no stray backticks,
unexpected emphasis,
or tables.
The source research landed in `253ca18e57` and `5e0dc5e6a1` before this verification record.
No Chromium C++ file was changed by this research pass.

[startup]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/chrome/app/chrome_main_delegate.cc#L1803-L1825
[reporting-policy]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/components/metrics/metrics_service_accessor.cc#L33-L48
[disabled-deletion]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/components/metrics/file_metrics_provider.cc#L291-L322
[browser-source]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/chrome/browser/metrics/chrome_metrics_service_client.cc#L280-L303
[local-mode]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/components/metrics/persistent_histograms.cc#L175-L235
[local-backing]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/base/metrics/persistent_memory_allocator.cc#L1063-L1102
[reporting-switches]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/components/metrics/metrics_switches.cc#L12-L20
[chrome-accessor]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/chrome/browser/metrics/chrome_metrics_service_accessor.cc#L35-L56
[backup-consent]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/chrome/installer/util/google_update_settings_posix.cc#L30-L36
[cleanup-call]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/chrome/app/chrome_main_delegate.cc#L939-L946
[cleanup]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/components/metrics/persistent_histograms.cc#L274-L300
[cleanup-delay]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/components/metrics/persistent_histograms.cc#L24-L38
[delete-location]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/base/metrics/persistent_histogram_allocator.cc#L1067-L1080
[allocator-lifetime]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/base/metrics/persistent_histogram_allocator.cc#L967-L990
[allocator-creation]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/base/metrics/persistent_histogram_allocator.cc#L740-L756
[shared-allocator]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/base/metrics/persistent_histogram_allocator.h#L469-L494
[shared-launch]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/base/metrics/histogram_shared_memory.cc#L83-L104
[snapshot-write]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/base/metrics/persistent_histogram_allocator.cc#L1049-L1065
[storage-destructor]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/base/metrics/persistent_histogram_storage.cc#L93-L156
[delta-contract]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/base/metrics/persistent_histogram_allocator.h#L277-L299
[system-profile-stream]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/components/metrics/system_profile_user_stream.h#L20-L48
[crashpad-handoff]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/components/crash/core/app/crashpad_linux.cc#L260-L268
[stream-writer]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/components/crash/core/common/shared_memory_user_stream_writer.h#L20-L53
[stream-consumer]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/components/crash/core/app/shared_memory_user_stream_data_source.cc#L56-L75
[profile-flush]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/components/metrics/metrics_service.cc#L1517-L1553
[shutdown]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/components/metrics/metrics_shutdown.cc#L13-L25
[browser-file-tests]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/chrome/browser/metrics/metrics_service_browsertest.cc#L330-L505
[provider-tests]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/components/metrics/file_metrics_provider_unittest.cc#L1343-L1402
[base-test-target]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/base/BUILD.gn#L3973-L3979
[metrics-owners]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/base/metrics/OWNERS#L1-L8
[startup-owners]: https://github.com/chromium/chromium/blob/986947eb4afc18471e51b7cb473f7c2f779f62bf/chrome/app/OWNERS#L1-L7
