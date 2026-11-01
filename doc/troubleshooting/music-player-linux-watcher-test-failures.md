# Linux desktop tests fail at initial queue and burst-report boundaries

## Observed failures

Depot run `ssvlvk57k4`,
attempt `jqxgk2969q`,
executed unchanged desktop source `3434eaa455d569c18b52cf15100f77f9db251d21`
on the prepared Linux runner.
The native package task used default nextest concurrency.
These failures remain separate incidents because their asserted boundaries differ.

`engine::tests::watcher_drives_rescan_update_through_engine`
failed at `package/music-player/desktop-app/src/engine_tests.rs:242`:

```text
expected Queue(1) after opening a root with one audio file
```

The source waits up to ten seconds for its initial single-track queue.
It had not reached the later file creation and rescan assertion.
Calling this a demonstrated watcher-rescan failure would overstate the evidence.

`engine::tests::idle_root_emits_no_extra_queue_updates`
failed at `package/music-player/desktop-app/src/engine_tests.rs:143`:

```text
assertion `left == right` failed: expected one Queue(2) from the open scan
  left: None
 right: Some(2)
```

It did not receive the initial two-track queue within ten seconds.
It had not reached the subsequent no-extra-update observation window.
This result does not prove that an idle library emitted extra updates.

`watch::tests::a_burst_of_writes_is_one_report_naming_each_file_once`
failed at `package/music-player/desktop-app/src/watch_tests.rs:291`:

```text
and only one report
```

Its preceding first-report path and plain-write classification assertion passed.
The failure is an additional report within the follow-up window.
The macOS issue `#1064` fails the earlier first-report comparison instead;
the same test name does not make these the same incident.

## Verification scope

The run executed 1067 of 1460 eligible tests:
1064 passed,
three failed,
one separate ignored case remained skipped,
and fail-fast left 393 eligible cases unexecuted.
The independent accounting collector matched the executed identities and the manifest.
The source files named in Observed failures had no diff from that revision to main
when inspected on 2026-10-16.

The native source case `cb95cf6d78` passed these same test names in separate Linux runs.
That different revision is not an unchanged-input repeat or a proof of cause.
The dedicated full-suite recovery is Depot run `pbg4h8ww6k`,
using harness `af3ccd254d` with only an in-memory `--no-fail-fast` addition.
That attempt failed during `slint-viewer --version` discovery before reaching nextest.
Corrected recovery `xf4hhngdqg` uses harness `409e6ec106`
and the identical immutable source `3434eaa455d569c18b52cf15100f77f9db251d21`.
Its artifact retains complete stdout and stderr,
task body hashes,
and matching before/after source and input hashes.
Independent reconciliation verified all 1460 eligible identities passed,
every ordinal from 1 to 1460,
and the exact summary:

```text
Summary [  29.563s] 1460 tests run: 1460 passed, 1 skipped
```

The previously failing rescan-initial-queue,
idle-initial-queue,
and burst-report tests passed at ordinals 752,
1071,
and 1057 respectively.
This is a separate observed successful repeat.
It neither identifies the historical failure causes nor proves those incidents fixed;
issues `#1079`,
`#1080`,
and `#1081` remain open.
The full identity and provider evidence is retained in
[the native recovery accounting](../audit/depot-native-recovery-2026-10-17.evidence.json).

## Open diagnosis

The logs do not identify whether initial queue absence reflects scheduling,
backend setup,
worker failure,
or another cause.
They do not identify which extra burst event produced the additional report.
Neither resource contention nor a regression is established by these observations alone.

Further investigation should retain initial engine updates,
worker diagnostics,
raw watcher events,
and fixture registration/write ordering.
Measure unchanged-input variability before making a timing or flakiness claim.
Keep original full-suite outcomes alongside any targeted reproduction.
No source fix or relaxed assertion is part of this historical CI measurement.

The nextest coverage omission and verified full-run recovery are tracked separately in issue `#1073`
and [the nextest diagnosis](depot-native-nextest-completion.md).
No external-tool defect or upstream filing is established by these application-test results.
