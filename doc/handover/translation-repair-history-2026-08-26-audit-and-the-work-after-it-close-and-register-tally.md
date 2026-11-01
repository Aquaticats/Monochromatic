# Translation repair history 2026-08-26 audit: closing the audit

The register's closing tally measured by scanning every entry for a marker,
the note that the audit's number is a task-list item and not the unrelated issue,
the verified close with the entry-module sweep,
and the queue of work that followed.

Part of [Translation repair history: 2026-08-26, the package audit and the work after it](translation-repair-history-2026-08-26-audit-and-the-work-after-it.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Closing the audit: the register tally and the verified close

CLOSING `#236` (2026-08-26).
The register's closing tally was measured rather than assumed:
a scan of every entry
under "Findings register" for a FIXED,
CLOSED,
folded or tracked marker found three without one.
`provider-14` was
CLOSED already (the scan had not looked for that word);
`provider-1` was fixed with A-1 and lacked only its line;
`calibrate-1`,
a MAJOR tied to `#235`,
still owed the pointer from the calibrations' `WROTE NOTHING AT ALL` sentence
to the `SEAT` report,
which landed in `8bffaba9b` with its guard (`POINTS AT THE SEAT LINES`;
the pointer removed
fails it,
restored passes).
With that,
every entry carries a marker:
3 BLOCKER,
28 MAJOR,
35 MINOR by heading.
NOTE ON THE NUMBER:
`#236` here and throughout this file is the session's task-list item,
"Exhaustive audit of the
translation-repair package";
the GitHub issue numbered 236 is an unrelated,
already closed file-enforcer issue,
so
closing the audit mutates no GitHub issue.
The task-list items `#258` and `#236` are marked completed.

CLOSE VERIFIED (2026-08-26).
Whole-suite `buildAndTest` after `8bffaba9b`:
814 PASS,
0 FAIL,
exit 0 (solo run).
The
advisor checkpoint owed after the rendering group was made at the close;
it judged the close sound pending two
measurements the tally had asserted rather than measured,
and both came back clean.
The marker scan re-run on the
after-state,
with CLOSED in the marker set,
finds every entry marked:
3 of 3 BLOCKER,
28 of 28 MAJOR,
35 of 35 MINOR,
zero unmarked.
The entry-module rule (nothing an entry module declares may be exported through a barrel) was checked
across every built command a `mise.toml` task invokes,
not only the two that regressed:
38 of 38
`dist/final/node/*.mjs` entries carry `import.meta.main`,
with `index.mjs` (the barrel,
not an entry) at 0 as the
control.
No standing test guards this,
by the checkpoint's call:
the sweep is the audit act.
The next checkpoint is
due before `#219`'s AskUserQuestion;
none between `#213`,
`#230` and `#229`.

Queue:
`#213` (arm A and arm B under `mise run` from a worktree carrying the secrets),
`#230`'s recovery rate on the
next run,
`#229` lever 1,
and `#219` (the readiness signal via AskUserQuestion).
