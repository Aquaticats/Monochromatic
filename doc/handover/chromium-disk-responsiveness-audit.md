# Chromium disk responsiveness audit

## Delivered audit at 2026-11-03T03:45:50-05:00

The user requested:
"Look through the entire Chromium source and flag anything blocking on disk and could impact UI responsiveness."

The audit is read-only with respect to Chromium and the user's browser configuration.
The source baseline is the installed release `156.0.8078.73`,
commit `2f9a11097447f226133bc1a31c787291a5eeab98`.
The GitHub tag API confirmed that association.

The source checkout is `${HOME}/temp/agent/chromium-disk-audit-20261103`.
The [audit overview](../troubleshooting/chromium-disk-responsiveness.md)
links the detailed source traces,
machine-readable candidates,
coverage manifests,
and validation records.

## Completed work

- Inventoried the Chromium checkout and scanned `505,073` files,
  including hidden and ignored source paths.
  Accounted for the skipped symlinks and separate dependency gitlinks.
- Retained `84,855` primary candidate lines,
  plus separate object-file-operation and native-loader passes.
  Verified compressed-inventory counts and source positive controls.
- Acquired the exact V8,
  Skia,
  ANGLE,
  Dawn,
  PDFium,
  ICU,
  LevelDB,
  SQLite,
  and WebRTC pins.
  Scanned their `144,427` regular tracked files.
- Traced desktop UI,
  renderer,
  graphics,
  mobile,
  startup,
  and shutdown dependencies on disk.
- Verified all `131` cited source selections in disposable browsers against pinned local source.
- Passed the scoped Markdown lint task and browser inspection of the rendered overview and evidence documents.
  Internal document targets and rendered anchors checked successfully.

The delegated source passes used the same model vendor and are not independent reviews.
No Chromium source,
runtime feature flags,
or user profile state was changed.
No upstream filing was made.

## Coverage limits

The corpus totals `649,500` files.
The other `264` top-level gitlinks,
the selected dependencies' `148` nested gitlinks,
private code,
generated output,
binary assets,
OS internals,
and proprietary drivers were not exhaustively inspected.
The scans are lexical discovery plus selected call-chain tracing,
not a proof that every reachable blocking operation has been found.

The new findings are static synchronous paths with conditional responsiveness impact.
No new latency experiment or freeze-rate measurement was performed.

## Evidence already available

[`helium-window-freezes.md`](../troubleshooting/helium-window-freezes.md)
records a correlated Helium main-thread write fault in the disk-backed `BrowserMetrics` mapping.
It also records ordinary Chromium main-thread filesystem waits without identifying their mapped file.
These are separate observations.
The report keeps these observations distinct from the new static candidates.

## Validation incident and cleanup

The first citation browsers hit the chosen `128`-task cgroup cap.
Recorded `pids.events` limit hits and browser navigation failures were retained.
Controlled relaunches with `512` tasks succeeded;
successful sessions had zero task-limit events.
Blank-line representation and browser frame readiness were checked before accepting source matches.
The overview documents the evidence and distinguishes separate failures.

Owned browser sessions were closed.
Source acquisition and scan processes finished;
the dependency acquisition's Git fsmonitor daemons were stopped.
A final procfs check found no Git processes with working directories under the audit checkout.
The previous session's authorized Helium recorder was not changed.

## Commits and remaining work

The audit and inventories landed in `7467f5f7c7`,
`ffa2daa798`,
`f776290a3b`,
`4c47b9ba0d`,
and `c8dce0da31`.
Commit `e8c225b2fa` records the successful citation coverage summary.
The closing commit adds the rendered-verification record and this handover.

No browser changes or fixes are queued.
The unexamined dependency surfaces and unmeasured runtime incidence are stated limits of this delivered audit.
