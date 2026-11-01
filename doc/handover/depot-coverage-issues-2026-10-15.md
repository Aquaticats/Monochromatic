# Depot coverage issue follow-up

The user requested a GitHub issue for every observed problem.
This follow-up owns the coverage and cost gaps identified during the authorized Depot trial.
It does not authorize production migration.

## Completed work

The [desktop cost overlay](../audit/depot-desktop-cost-2026-10-15.md)
records the complete 441-second desktop attempt and its coverage limits.
Its independent calculation check,
Markdown lint,
and rendered inspection passed.

Created these issues with source evidence and independently verifiable acceptance criteria:

- [Native verification routes](https://github.com/Aquaticats/Monochromatic/issues/1023),
  covering existing native checks beyond the Windows and Alpine music-player suites.
- [Android host and native checks](https://github.com/Aquaticats/Monochromatic/issues/1024),
  covering JVM tests,
  lint,
  native ABIs,
  and the distinction between instrumentation assembly and device execution.
- [Browser and end-to-end CI execution](https://github.com/Aquaticats/Monochromatic/issues/1025),
  covering existing Playwright suites and their reports.
- [`done` browser user-flow coverage](https://github.com/Aquaticats/Monochromatic/issues/1026),
  covering suite authoring against implemented application behavior.
- [Affected consumers](https://github.com/Aquaticats/Monochromatic/issues/1027),
  covering changed-package and transitive-consumer task selection.
- [Obsolete ownership package filter](https://github.com/Aquaticats/Monochromatic/issues/1028),
  covering the semantic-bridge workflow's stale plural path.
- [Shared-input trigger omissions](https://github.com/Aquaticats/Monochromatic/issues/1029),
  covering dependency and toolchain inputs consumed by existing jobs.
- [Remaining workload costs and allocation](https://github.com/Aquaticats/Monochromatic/issues/1030),
  covering unmeasured checks and an explicit proposed execution allocation.

## Existing issues reused

The all-state issue search returned 974 issues before the remaining parallel reports were filed.
A subsequent latest-issue search checked for concurrently created duplicates.
Existing bodies and comments were inspected for the closest matches.

- [Root package-task discovery](https://github.com/Aquaticats/Monochromatic/issues/305)
  and its overlapping [root-test report](https://github.com/Aquaticats/Monochromatic/issues/555)
  already track omissions in the aggregate test task.
- [Playwright discovery](https://github.com/Aquaticats/Monochromatic/issues/310)
  and the [paused-package reproduction](https://github.com/Aquaticats/Monochromatic/issues/603)
  already track the active-package boundary.
- [General CI direction](https://github.com/Aquaticats/Monochromatic/issues/122)
  remains the broader setup issue.
  The new workload issues specify separate execution evidence.
- [Logger post-release verification](https://github.com/Aquaticats/Monochromatic/issues/479)
  records local browser coverage pending a CI route.
- [Historical hook consolidation](https://github.com/Aquaticats/Monochromatic/issues/160)
  was superseded by the Git policy platform;
  it does not establish the proposed consumer selector.

No duplicate issue was created for those discovery or historical-hook reports.

## Ownership boundaries

Other active tasks own the desktop chooser,
fixture selection,
Android context,
snapshot and service behavior,
cache behavior,
release and quota handling,
scheduling incidents,
and GitHub log retrieval.
This follow-up creates current-repository issues only.
It does not post upstream reports.

## Verification and handoff

All eight new issue bodies passed CommonMark rendering before publication.
Their fetched GitHub body bytes match the reviewed local bodies.
GitHub reports each issue open with the `needs-triage` label.
The issues preserve production provider selection as an unresolved trial decision.

The original coverage issue queue is complete.
The main trial ledger can use the linked issues to track implementation and verification.

## Final completeness audit

The follow-up audit reads the full trial report,
runner-sizing report,
and interview handover against all-state issue history.
Its refreshed search returned 999 issues.
The original source,
run records,
fix commits,
and troubleshooting documents distinguish these additional incidents:

- [Action-policy startup rejection](https://github.com/Aquaticats/Monochromatic/issues/1033):
  closed after the user-approved policy admitted both runner jobs.
- [Mise release-age HTTP 403](https://github.com/Aquaticats/Monochromatic/issues/1034):
  open;
  the published-version workaround passed,
  but the original response cause remains unknown.
  Existing [issue 506](https://github.com/Aquaticats/Monochromatic/issues/506)
  concerns a different release-asset HTTP 404.
- [Local task-capacity exhaustion](https://github.com/Aquaticats/Monochromatic/issues/1035):
  closed after the authorized increase and installed shell verification.
- [Kernel/helper update mismatch](https://github.com/Aquaticats/Monochromatic/issues/1036):
  closed after coordinated recovery;
  distinct from exhausting a correctly enforced limit.
- [Trial task discovery](https://github.com/Aquaticats/Monochromatic/issues/1037):
  closed after moving the task into an existing package task file and running it successfully.
- [Omitted compositor path](https://github.com/Aquaticats/Monochromatic/issues/1038):
  closed after passing the freshly built executable to the real E2E runner.
  The subsequent fixture failure remains separate.
- [Standalone command lookup](https://github.com/Aquaticats/Monochromatic/issues/1039):
  open;
  the absolute-path workaround passed,
  but the server-side bare-name lookup cause remains unknown.
- [Unavailable browser discovery](https://github.com/Aquaticats/Monochromatic/issues/1040):
  open;
  distinct from the [usage API credential boundary](https://github.com/Aquaticats/Monochromatic/issues/999).

All eight additional bodies passed rendered inspection and byte-for-byte comparison with GitHub.
All carry `needs-triage`.
The five closed issues contain the actual recovery evidence;
the three open issues retain their unresolved boundaries.
The audit reread both live worker task bounds as `2048` without changing them.

The coverage and final-audit passes together created 16 issues:
11 open and five closed at verification.
No production code or account setting changed during issue filing.

## Findings that do not need another issue

The trial's application,
snapshot,
cache,
quota,
scheduling,
and budget findings are linked in the [main trial issue inventory](depot-trial-issues.md).
The published scanner's binstall metadata has
[issue 1031](https://github.com/Aquaticats/Monochromatic/issues/1031).
The unrelated cause of intermittent GitHub protected-branch rejection remains
[issue 1032](https://github.com/Aquaticats/Monochromatic/issues/1032).

The interview's process corrections have related existing tracking:
[measurable facts versus interview questions](https://github.com/Aquaticats/Monochromatic/issues/425),
[under-scoped verification](https://github.com/Aquaticats/Monochromatic/issues/572),
and [completion and authorization boundaries](https://github.com/Aquaticats/Monochromatic/issues/677).
The proposed policy edits in the interview handover remain unaccepted;
issue filing does not adopt them.

Sizing tradeoffs,
the successful build's CPU-utilization warning,
unselected check frequencies,
and unchosen provider allocations are not filed as bugs.
The documented modifier-repeat limitation remains explicit in desktop coverage results;
the pure detector has repeat-event tests,
and the compositor does not inject repeat events for modifier keys.
No full passing interactive claim is inferred from that separate unit coverage.

The final completeness audit is complete for the trial reports and related setup incidents.
