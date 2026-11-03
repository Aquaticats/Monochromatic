# Firefox contributions to remove UI waits for storage

## Proposal evidence at 2026-11-03T09:07:48+00:00

This is a proposal,
not an implementation or filing-readiness claim.
No Firefox source,
profiles,
configuration,
or upstream issue was changed.
No prototype or runtime stall experiment was run.

The completed audit baseline is Firefox ESR `153.6.0esr`,
commit `a34c4160e3f75ab9918fbe4669ff7ae56f1977c8`.
Current-source checks use main commit `2b71df5bf2e07525d1129be46e0ee02ec70d34d8`,
whose commit timestamp is `2026-11-03T08:00:48Z`.
The current files still contain the audited StartupCache table-lock dependence,
the Glean memory-report global-lock acquisition,
and certificate-storage main-thread bookkeeping under its disk-worker lock.

## Recommended first Firefox contribution: help the existing StartupCache fix land safely

Mozilla already owns the structural work in
[bug 2048221, StartupCache::WriteToDisk optimization][startup-work].
The live Bugzilla API reported `ASSIGNED`,
with Emilio Cobos Álvarez as assignee,
and last update `2026-10-30T17:46:11Z`.
Do not open a duplicate issue or present snapshotting as an untried new idea.

The neighboring
[bug 2037923][startup-shutdown]
is `RESOLVED FIXED`.
Its landed change stops using low-priority I/O for the shutdown cache write;
the current source retains that mitigation at
[`StartupCache.cpp:679-690`][startup-current].
It does not remove the lock held across the ordinary write.
Current `GetBuffer` blocks on `mTableLock` at line `393`;
the background writer takes the same lock around `WriteToDisk` at lines `800-801`.

The structural patch was already reviewed and landed as
[`dc6ef27385dda3a44d7aa3d655663ced75ae565b`][startup-patch].
It introduced a separate `mIOLock`,
refcounted cache entries for snapshots,
and write-local header metadata.
The same-day revert is
[`eec2f8dc61cc9f851fe5303cc05fc015d3128e6f`][startup-revert].
[Bug 2048221 comment 5][startup-failures]
records `MOZ_CRASH() [@ LateWriteObserver::Observe]` in these Windows debug tests:

- `testing/marionette/harness/marionette_harness/tests/unit/test_cli_arguments.py`,
  `TestCommandLineArguments.test_startup_timeout`.
- `testing/firefox-ui/tests/functional/security/test_ssl_status_after_restart.py`.

The preceding missing-includes fix for `TestStartupCache` was also backed out with the structural patch.
These are recorded failures,
not a proven root-cause diagnosis by this investigation.

[Comment 11][startup-tooltip]
contains a reporter's fresh Windows Nightly tooltip hang,
with the main thread waiting in `GetBuffer` and a writer in `NtWriteFile`.
The reporter says it resolved after about `5` to `10` minutes.
That duration and dump analysis are the reporter's evidence,
not our measurement.
Comments `12` to `14` clarify that the low-priority change was not Linux-only.
Comment `11` already proposes try-lock-as-cache-miss and removing low I/O priority as interim mitigations.
Repeating those suggestions would add no new analysis.
Treat a cache miss as a fallback whose loader path must also be checked:
avoiding the cache lock does not by itself establish that fallback module loading is disk-independent.

The concrete useful deliverable is a reproducible concurrency test plus an explanation of the failed shutdown schedule,
offered to the existing assignee for the current patch:

- Pause a cache writer after it owns the snapshot but before its persistence finishes.
  Prove an unrelated main-thread cache lookup or UI event can finish while that worker stays paused.
- Pause that same writer during shutdown.
  Prove shutdown does not announce the write finished until actual write/close completion,
  and that no write reaches the late-write-forbidden phase.
- Exercise invalidation while a snapshot exists,
  new cache entries arriving during a write,
  failed writes,
  and the subsequent startup read.
  Preserve entry lifetime,
  stale-snapshot rejection,
  cache format,
  and corruption fallback.
- Extend `startupcache/test/TestStartupCache.cpp`,
  which already contains `StartupWriteRead`,
  `WriteInvalidateRead`,
  and `WriteObject` tests.
  Re-run the specifically named Windows late-write regressions before proposing a reland.

One schedule worth testing comes directly from the reverted diff:
`WriteToDisk` clears `mTableDirty` after snapshot capture,
before taking `mIOLock` and performing disk writes;
`EnsureShutdownWriteComplete` calls `WriteToDisk`,
which can return early when `mTableDirty` is false.
That is a test hypothesis about confusing snapshot capture with completed persistence,
not an assertion that it alone explains the recorded crashes.
An explicit in-flight writer completion boundary is required regardless of the chosen implementation.

The existing bug's component is `Core: Performance: Startup`.
The source ownership metadata routes `startupcache` to `Core: XPCOM`,
and the existing mitigation used `#xpcom-reviewers`.
Start with the assigned work and its reviewers instead of creating another competing design thread.
Any communication would be a later authorized step;
none was sent.

## Certificate storage: remove the UI dependency from bookkeeping before redesigning trust data

Current [`cert_storage/src/lib.rs:1193-1241`][cert-current]
takes `security_state.write()` in `SecurityStateTask::new`
to increment `remaining_ops`,
and again in `done` to decrement it.
The background `run` holds that same write lock through `open_db()` and `task_action`.
`HasPriorData` requires the main thread at lines `1319-1337` and constructs such a task.
`GetRemainingOperationCount`,
lines `1360-1369`,
also reads the counter under the shared storage lock on main.

Candidate contribution:
move pending-operation accounting into a separately owned nonblocking state,
so submitting work,
finishing callbacks,
and reading the count do not acquire the lock held during disk work.
Keep database mutation serialized on the existing queue.
This is narrower than replacing certificate storage or changing revocation persistence.

Required semantics and tests:

- Preserve callback thread affinity,
  callback order relative to counter decrement,
  and the public pending-operation count.
- Cover task construction/dispatch failure and callback failure;
  counts must not leak or underflow because of the refactor.
- With a storage-worker gate held,
  prove a second main-thread API call and a main-thread sentinel event finish before the gate is released.
  Then release the worker and assert the correct security result and persisted state.
- Retain existing tests in
  `security/manager/ssl/tests/unit/test_cert_storage_direct.js`,
  `test_cert_storage_preexisting.js`,
  and `test_cert_storage_broken_db.js`.
  Add ordering and contention coverage instead of replacing these persistence/recovery tests.

The memory reporter separately takes `security_state.read()` at lines `1693-1701`;
moving the counter does not fix that path.
Treat an asynchronous certificate-storage memory reporter as a separate follow-up with the reporter lifecycle tests
described for Glean.
Do not use stale or missing trust/revocation data as a performance fallback.
Route this work through `Core: Security: PSM` ownership.
Recent file history repeatedly names `keeler` as reviewer;
that is evidence for finding the right team,
not authorization to contact or assign anybody.

The complete duplicate/work-in-progress search for this new candidate was not run.
No claim is made that Mozilla lacks an existing tracking bug.
Current source and recent path history were checked to avoid proposing an already-landed fix.

## Glean memory reporting: use the asynchronous reporter protocol

Current [`FOG.cpp:870-878`][fog-current]
calls `fog_collect_reports` synchronously.
Current `toolkit/components/glean/api/src/lib.rs:79` calls `glean::alloc_size`;
current `third_party/rust/glean-core/src/lib.rs:654-656` calls `with_opt_glean`;
current `core/mod.rs:111-117` takes the global Glean mutex.
The completed audit traces persistence under the same lock.

Candidate contribution:
register FOG as an asynchronous memory reporter,
perform Glean-owned memory measurement on the appropriate owning/dispatcher sequence,
and return its report to main with exactly one completion notification.
Existing Firefox infrastructure supports this:
[`nsIMemoryReporter.idl:225-237,310-314`][async-reporter-interface]
has async registration and `endReport`,
and [`MediaRecorder.cpp:90-125`][async-reporter-example]
shows an implemented asynchronous `CollectReports` pattern ending with `manager->EndReport()`.

Prefer that exact asynchronous report over substituting an unlabeled cached byte count.
A cached snapshot is an alternative only if its age and approximation semantics are explicitly accepted by the owners.
Returning zero for a busy lock would misreport memory and conceal the issue.

Important implementation constraints:

- Do not call the existing locking `alloc_size` function from inside an already locked `with_glean` closure.
  The API needs an owning-sequence measurement path that does not recursively acquire the same mutex.
- Check allocation-measurement callback thread safety,
  retain callback/data lifetimes,
  dispatch completion on the correct thread,
  and finish exactly once on initialization failure,
  shutdown,
  cancellation,
  and overlapping report requests.
- Prove main-thread progress while a Glean persistence worker is gated.
  After releasing the worker,
  assert a valid report and completion,
  while ordinary counters and persistence still round-trip correctly.
- Use the existing `FOGFixture` C++/Rust integration-test pattern under
  `toolkit/components/glean/tests/gtest/`,
  plus xpcshell memory-report integration.
  Firefox's Glean developer testing documentation distinguishes pure Rust tests from Gecko-linked GTests.

Route Gecko integration through `Toolkit: Telemetry` and `#toolkit-telemetry-reviewers-rotation`.
If an API change is needed in the vendored Glean SDK,
coordinate it with Glean's upstream owner rather than carrying an unexplained local vendor edit.
Current-source checks establish the lock path;
the full duplicate search and upstream SDK API design remain open.

## Regression detection: extend existing checks to indirect waits

Firefox already has startup I/O checks:
[`browser_startup_mainthreadio.js:4-18`][startup-io-test]
and `browser_startup_content_mainthreadio.js` record unexpected main-thread file operations.
The first test has a phase-specific allowed-operation list.
Its source at lines `54-68` documents gaps:
Unix read coverage includes only NSPR reads,
Linux writes include only NSPR writes,
and fsync observation is Windows-only there.
The implementation at lines `662-665` expects I/O markers only when
`!AppConstants.RELEASE_OR_BETA`;
the nearby comment has a contradictory negation,
so rely on the condition.
An empty ESR profile cannot be treated as proof of no I/O.

The profiler feature declarations already include
`mainthreadio`,
`fileio`,
and `fileioall`.
`tools/profiler/tests/xpcshell/test_feature_mainthreadio.js`
has both instrumented file-I/O cases and a no-feature control.
Reuse those controls when adding a new measurement surface.

The missing regression contract is broader than direct syscalls:
an event handler must remain runnable while a relevant disk worker is paused.
Add bounded deterministic worker gates and a UI sentinel test to the component suites,
then correlate profiler stacks with worker I/O in an integration workload.
A gate delays the test's own operation;
it generates no competing host load.
The test harness must always release gates on failure and respect its deadline.
Include a positive control that deliberately retains the old lock dependence and is detected.

Retain first-use,
post-update cache rebuilding,
normal interaction,
and shutdown as separate scenarios.
Do not score success only by fewer `fsync` calls,
or by ordinary warm-start timing.
Direct file-I/O markers do not prove absence of mapped-file faults or lock-mediated waits.
Actual UI input/frame latency and durable-state correctness are the acceptance boundaries.

## Contribution policy and sequence

Current [Firefox AI coding policy][ai-policy]
permits AI-assisted work under normal correctness/security/maintainability standards,
requires the human submitter to understand and personally review every change,
and assigns responsibility to that human.
The checked policy does not prescribe a special AI disclosure footer.
That absence is limited to the policy reviewed here;
it is not a claim about every Mozilla project or intake form.

Current [submission guidance][submit-policy]
requires an associated Bugzilla bug,
coordination with an existing assignee,
module-owner acceptance of the proposed change,
tests,
and review through Phabricator.
GitHub hosts the code;
a GitHub pull request is not the documented Firefox review route.
The reviewer guide prefers review groups over an individual when suitable.

Recommended order:
StartupCache reproduction and shutdown-lifecycle tests,
then the certificate bookkeeping separation,
then asynchronous Glean/certificate memory reporting,
with indirect-wait regression tests accompanying every change.
StartupCache ranks first because there is current assigned work,
a recorded normal-use trigger,
and specific failed tests to unblock.
Certificate bookkeeping ranks next because it isolates an avoidable UI lock acquisition
without redesigning trust evaluation.
Memory reporting follows because it needs asynchronous lifecycle and callback safety work across interfaces.
This is a scope/risk ranking,
not a time or performance forecast.

[startup-work]: https://bugzilla.mozilla.org/show_bug.cgi?id=2048221#c0
[startup-shutdown]: https://bugzilla.mozilla.org/show_bug.cgi?id=2037923#c8
[startup-current]: https://github.com/mozilla-firefox/firefox/blob/2b71df5bf2e07525d1129be46e0ee02ec70d34d8/startupcache/StartupCache.cpp#L679-L690
[startup-patch]: https://github.com/mozilla-firefox/firefox/commit/dc6ef27385dda3a44d7aa3d655663ced75ae565b
[startup-revert]: https://github.com/mozilla-firefox/firefox/commit/eec2f8dc61cc9f851fe5303cc05fc015d3128e6f
[startup-failures]: https://bugzilla.mozilla.org/show_bug.cgi?id=2048221#c5
[startup-tooltip]: https://bugzilla.mozilla.org/show_bug.cgi?id=2048221#c11
[cert-current]: https://github.com/mozilla-firefox/firefox/blob/2b71df5bf2e07525d1129be46e0ee02ec70d34d8/security/manager/ssl/cert_storage/src/lib.rs#L1193-L1241
[fog-current]: https://github.com/mozilla-firefox/firefox/blob/2b71df5bf2e07525d1129be46e0ee02ec70d34d8/toolkit/components/glean/xpcom/FOG.cpp#L870-L878
[async-reporter-interface]: https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/base/nsIMemoryReporter.idl#L225-L237
[async-reporter-example]: https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/dom/media/MediaRecorder.cpp#L90-L125
[startup-io-test]: https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/base/content/test/performance/browser_startup_mainthreadio.js#L4-L18
[ai-policy]: https://github.com/mozilla-firefox/firefox/blob/2b71df5bf2e07525d1129be46e0ee02ec70d34d8/docs/contributing/ai-coding.md?plain=1#L13-L22
[submit-policy]: https://github.com/mozilla-firefox/firefox/blob/2b71df5bf2e07525d1129be46e0ee02ec70d34d8/docs/contributing/how_to_submit_a_patch.md?plain=1#L38-L71
