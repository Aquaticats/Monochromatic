# Improving browser responsiveness under storage stalls

## Proposal at 2026-11-03T04:04:06-05:00

The user asked:
"What can we do to improve the situation for everyone?"

The proposed contribution is an upstream sequence of reproducible failures,
regression tests,
and focused fixes,
followed by broader regression coverage.
The evidence base is the
[Chromium audit](../troubleshooting/chromium-disk-responsiveness.md),
[Firefox ESR audit](../audit/firefox-esr-disk-responsiveness.md),
and [captured BrowserMetrics incident](../troubleshooting/helium-window-freezes.md).
The [Chromium upstream check](browser-disk-responsiveness/chromium-upstream.md)
and [Firefox upstream check](browser-disk-responsiveness/firefox-upstream.md)
identify current work and contribution routes.
This is a proposal;
no browser source change or external posting is part of this planning task.

## Recommended order

Start with a shareable reproduction and regression test for Chromium's live histogram mapping,
because that mechanism has incident-level evidence.
In parallel,
contribute failure reproduction and shutdown tests to Firefox's existing startup-cache work.
Then address synchronous shader-cache reads and initialization waits,
whose alternatives need explicit rendering-cost measurements.
Build regression coverage alongside each fix so its benefit survives later changes.

This order prioritizes evidence and reusable fixes,
not an unmeasured estimate of affected-user counts or engineering duration.

## Establish a reproducible user-visible failure

Each contribution should identify the affected interaction,
thread,
file or mapping,
and wait mechanism.
A benchmark should measure UI input handling and frame presentation separately,
with startup and shutdown recorded as separate phases.
File-operation counts alone are not the result to optimize.

The proposed evidence package contains:

- An exact browser revision,
  build configuration,
  relevant features,
  and disposable profile.
- A named interaction such as opening a menu,
  recording a metric,
  reporting memory,
  or rendering a previously unused shader.
- A trace tying its delay to direct I/O,
  a file-page fault,
  or a lock/queue wait behind disk work.
- A deterministic test that deliberately holds the relevant worker or file operation pending
  while checking whether the UI can continue.
- A failing baseline,
  a passing proposed change,
  and checks of persistence,
  crash recovery,
  cancellation,
  and shutdown as applicable.

Injected delays belong inside disposable test fixtures or a bounded test environment.
The plan does not require synthetic host load or globally dropping filesystem caches.
Runtime comparisons should establish unchanged-build variation before interpreting a difference.
Traces intended for sharing should use disposable data and exclude private paths,
URLs,
accounts,
and profile contents.

## Chromium live metrics

The [confirmed mechanism](../troubleshooting/helium-window-freezes.md#metrics-write-faults-block-the-main-thread)
is a browser-main-thread write fault in a regular disk-backed histogram mapping.
The existing consumer workaround demonstrates that local-memory fallback removes that mapping,
but loses its persistence semantics.

The proposed upstream design direction keeps live recording in memory
and makes persistence independent of the recording thread.
It belongs in the existing histogram allocation and persistence implementation,
with Chrome startup selecting the supported storage mode.
It does not require every histogram caller to invent its own workaround.

The design must resolve:

- Crash-time metrics retention and what survives a process failure between persistence operations.
- Multiprocess access,
  allocator lifetime,
  concurrent counter updates,
  and histogram-delta correctness.
- Bounded memory and queued work when storage stops completing requests.
- Shutdown completion without reintroducing a UI wait on the stalled persistence path.

Moving the initial map operation to a worker does not remove later faults on writes through that mapping.
Anonymous/shared memory also does not guarantee freedom from every possible swap or memory-pressure stall.
The intended property is removal of the regular-file write-fault dependency from recording.
The persistence contract needs review before a replacement can be called a fix.

## Firefox locks shared with disk workers

The [audit](../audit/firefox-esr-disk-responsiveness.md#main-thread-startup-cache-lock-held-by-disk-writer)
shows startup-cache readers waiting on a mutex held across disk writes.
It also identifies analogous dependencies in certificate-state operations and Glean memory reporting.

The proposed approach uses stable in-memory state for readers
and worker-owned snapshots or generations for persistence.
Any publication lock covers exchanging completed state,
not opening,
serializing,
writing,
flushing,
or closing a file.
Firefox already supports asynchronous memory reporters.
For Glean,
the proposed integration measures on the owning sequence
and sends its result back through that existing protocol,
with exactly one completion notification.
An independently maintained approximate accounting snapshot is an alternative only if its semantics are accepted.
Returning zero when a lock is busy would produce an incorrect report.

These changes require lifetime and shutdown proofs:
the writer must not retain dangling references,
publish an older generation over a newer one,
write after shutdown forbids writes,
or invalidate data still used by a reader.
Security decisions must not silently use stale or incomplete state merely to avoid waiting;
where current data is required,
the operation needs asynchronous completion with its correctness checks preserved.

Existing upstream attempts and their failures determine the starting point.
The next contribution should extend the active work's regression coverage and resolve its failure cases,
not reopen the same design as an unrelated rewrite.

The next distinct Firefox candidate is certificate-operation bookkeeping.
The current implementation acquires the disk worker's state lock merely to maintain a pending-operation counter.
Separating that accounting can remove the submission-side dependency while keeping trust-data mutation serialized.
It needs count,
callback-order,
dispatch-failure,
and shutdown tests;
it does not remove the separate memory-reporting lock path.
The [Firefox evidence](browser-disk-responsiveness/firefox-upstream.md)
traces both candidates and their existing tests.

## Graphics caches and other optional disk data

The [Chromium GPU path](../troubleshooting/chromium-disk-responsiveness.md#gpu-persistent-cache-misses-synchronously-read-sqlite-backed-files)
and [Firefox shader path](../audit/firefox-esr-disk-responsiveness.md#webrender-shader-cache-reads-block-the-render-thread)
can synchronously read disk while producing frames.

The proposed contract makes already-resident cache lookup independent of initialization or persistence completion.
When an optional cache is unavailable,
the implementation can treat it as a miss,
schedule preparation earlier,
or continue through an existing asynchronous completion path.
The precise fallback is part of each graphics implementation's design.

Treating a cache miss as permission to compile synchronously can replace storage jank with compilation jank.
Likewise,
a failed cache try-lock can send a resource loader into a synchronous file-read fallback.
The entire caller path needs checking;
returning promptly from the cache alone is insufficient.
Verification therefore includes frame presentation,
compilation cost,
memory use,
cache-hit behavior,
and initialization/shutdown races.
A timeout checked after a blocking operation is not a deadline on that operation.

Required data needs a different treatment.
Profiles,
permissions,
certificates,
and user documents cannot be discarded like a shader-cache entry.
Their UI operations should remain pending with cancellation or retry where appropriate,
while preserving their data and security contracts.
Synchronous web-platform semantics also constrain changes such as localStorage loading.

## Prevent the same defects from returning

Extend each project's existing test and profiling infrastructure.
The regression inventory should distinguish direct I/O,
mapped-file faults,
shared-lock waits,
and synchronous completion waits.
Checking only syscall names misses the latter mechanisms.

A proposed concurrency test should prove this ordering:

- The storage worker has entered the deliberately blocked operation.
- The tested UI operation or an unrelated UI task completes while storage remains blocked.
- Releasing the worker completes persistence with valid contents and no lifetime violation.

The baseline must fail the intended assertion;
a passing run where storage work never started proves nothing.
Test error paths,
failed writes,
cache invalidation,
concurrent updates,
and shutdown ordering through the existing owning module's tests.
Bound the harness externally so a failure cannot strand its processes.

Performance tests should cover cold and warm state,
slow storage,
and constrained environments without relying on a quiet host.
Review exception lists for critical-thread blocking with an owner,
a reproducible justification,
and regression coverage.
The diagnostic and test machinery must itself avoid synchronous persistence on the UI path.

## Upstream contribution and rollout

Group work by failure mechanism and owning module.
A reproducible regression test and focused patch are more reviewable than a combined browser-wide rewrite.
Use existing matching issues and review threads when the new evidence adds something.
Source-search counts and unmeasured candidates do not justify a batch of bug reports.

Validate against current development code as well as the affected release.
Land through the projects' normal review and performance testing,
then consider release or ESR backports according to regression risk and project policy.
Track crashes,
data correctness,
memory growth,
and responsiveness during rollout.

### Existing work changes the contribution

The [Chromium check](browser-disk-responsiveness/chromium-upstream.md#existing-upstream-work-to-coordinate-with)
found open work on mapped BrowserMetrics reads,
sparse-histogram lock contention,
and GPU preloading.
Those are relevant collaborators and precedent;
none establishes a fix for the captured mapped-write fault.
Current development code also has an optional asynchronous GPU-cache initialization path
whose consumers can still wait before checking the memory cache.
Its initialization and metadata-preload features were disabled by default in the checked source.

The [Firefox check](browser-disk-responsiveness/firefox-upstream.md#recommended-first-firefox-contribution-help-the-existing-startupcache-fix-land-safely)
found an active assigned startup-cache issue,
`2048221`.
A structural lock/snapshot patch landed and was reverted after Windows late-write shutdown test crashes.
The nearby fixed issue `2037923` changed shutdown I/O priority;
it did not eliminate the shared lock.
The concrete next contribution is to reproduce the failed shutdown schedule,
add lifecycle/concurrency tests,
and help the assigned work land safely.
The proposed explanation involving an in-flight write is a hypothesis to test,
not an established root cause of the reverted patch's failures.

### Review and release requirements

The checked Chromium and Firefox policies allow AI assistance while making the human submitter responsible
for understanding and reviewing the changes.
Chromium additionally requires human responses to human review feedback.
Their documented review routes are Gerrit and Phabricator/Bugzilla,
respectively.
The evidence documents link the precise policies and ownership information.

No upstream report,
review comment,
or patch has been sent.
The proposal does not certify a patch or issue draft as ready to submit.
Current-source checks and the completed audits supply a starting point;
reproduction,
implementation,
correctness testing,
performance measurements,
and review remain the proposed work.
