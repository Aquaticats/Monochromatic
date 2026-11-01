# Translation repair history 2026-08-26 audit: the owner's instruction that opened the audit, the baseline, the method and the prior claims overturned

The owner's instruction that opened the audit and held back new work until it closed,
the baseline measured before anything was touched,
how the source was partitioned into ten slices and read under seven lenses,
the running tally of findings when the slice reports were verified,
the claims of earlier sessions overturned against a primary source,
and the two items found while auditing.
Its last item gives a line count against a cap for the single working handover file the audit read,
not for any file of this record.

Part of [Translation repair history: 2026-08-26, the package audit and the work after it](translation-repair-history-2026-08-26-audit-and-the-work-after-it.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## AUDIT IN PROGRESS: the whole package, before any new work (`#236`, 2026-08-26)

The owner's instruction,
sent mid-turn,
verbatim:
"Do an exhaustive audit of the package so far.
Focus on existing issues rather than branching into new ones for now,
until you're confident there are no big existing issues."
And,
separately:
"Update handover more aggressively."
Both are in force.
No new feature,
lever,
or measurement run starts until the audit closes;
fixing an existing defect is in scope,
and `#235` is finding one.

### Baseline, measured at `b20120fbb` before anything was touched

- `lint:types`:
  exit 0.
- `lint:oxlint`:
  0 warnings,
  0 errors,
  965 files.
- `buildAndTest`:
  exit 0,
  750 suites `] PASS `,
  0 `] FAIL `,
  log `~/temp/agent/audit-buildAndTest-20260826T024842Z.log`.
- Source:
  532 files,
  150304 lines.
  Tests:
  432 files,
  146083 lines.
- 138 source files have no sibling `<name>.unit.test.ts`.
  That heuristic overcounts,
  since a test elsewhere may cover them;
  it is refined per file during the audit rather than trusted.

### Method

The source is partitioned into ten slices,
every file in exactly one:
`provider`,
`translate`,
`repair`,
`probes`,
`calibrate`,
`consolidate`,
`document`,
`artifact`,
`slices`,
`rendering`.
File lists:
`~/temp/agent/audit-slices/<slice>.txt`.
Each slice is read in full by a forked reviewer under seven lenses:
silent failure (the `#235` class),
wrong output,
resilience under provider failure,
corpus or key leakage,
contract drift between prose and code,
repo-rule violations the linters cannot see,
and test-coverage gaps.
Reports land in `~/temp/agent/audit-<slice>.md`.
Every BLOCKER and MAJOR finding is re-verified at the cited line by the main session before it is recorded;
a reviewer's word is a lead,
not evidence.
The verified,
ranked result lands in `doc/audit/translation-repair-package-audit.md`,
one task per BLOCKER or MAJOR,
fixes in rank order after that.
That register is open as of `918d9a8b3` with the baseline,
the whole-package mechanical sweeps
(all clean:
no forbidden request fields,
zero debt markers,
every suppression justified,
no banned constructs,
38 of 38 CLIs wrapped),
the documentation layer,
and six verified findings `A-1` to `A-6`;
read it before this section.
Running tally,
2026-08-26 after all ten slice reports,
every BLOCKER and MAJOR re-verified at the cited line:
2 BLOCKER (A-1 fixed as `#235`;
probes-1 open as `#247`),
21 MAJOR (A-2 fixed with `#235`;
`#237` to `#246` and `#248` to `#251` from the first six slices;
`#252` corpus read failures drop entries silently;
`#253` a transient picture-reader failure is cached as permanent;
`#254` a CRLF original disables the line-structure family;
`#255` the rendering audit's provenance verdict is void on roster-paired artifacts;
`#256` valueless `--run` and `--against` read as absent;
`#257` the round-three instruments cannot read their own draw;
six more folded into `#237`,
`#238`,
`#246`,
`#249`),
and roughly seventy MINOR,
all in `doc/audit/translation-repair-package-audit.md`.
Fix order after `#235`:
`#247` (an instrument that lies),
then the MAJORs in task order,
then the MINORs.
Advisor's adjustment,
adopted:
`#235` lands as soon as the `provider` and `calibrate` slice
reports are verified,
not after the whole audit,
because those two slices hold every file the fix touches.

### Prior-session claims overturned so far, each against a primary source

1.  `#235` cause.
    Recorded as "in `budget-routing.ts`,
    not yet located";
    that file is correct.
    The cause is `createRunClient`'s unkeyed-Hyper fallback plus a launch without secrets.
    Details under the OPEN DEFECT section,
    which is corrected in place.
2.  Round count.
    Recorded as twenty-two;
    the log holds twenty-five,
    all `5/10`.
3.  Subagents.
    The compaction summary carried "no subagents unless the user asks".
    The transcript holds the owner's standing instruction,
    verbatim:
    "For speed:
    If you have work you can defer to a Sonnet 5 subagent in parallel,
    defer to it."
    Forks carry the audit's reading;
    verification stays with the main session.
4.  Fork secrets.
    Recorded as "fork worktrees have none";
    this worktree's `.env.local.json` is byte-identical to the main worktree's (`cmp`,
    2026-08-26).
    The half-dark run was launched with bare `node`,
    which is what left the key unset;
    `mise run` from this worktree injects both keys.
5.  Silence.
    Recorded as "nothing above warn";
    the calibration's closing coverage line named the five seats
    (`WROTE NOTHING AT ALL: ... covers 5 of 10 seats`),
    without counts or cause.
    Recorded as calibrate-1 in the register.

### Found while auditing, unfixed, owed

- FIXED in `8b289c3ab`:
  `RunConfigError` extends `StatedRefusalError`,
  so a missing key is repeated by name at exit 6 with no frames (negative control run,
  2026-08-26).
- This file is 3463 lines against its own 2000-line cap.
  The trim into `translation-repair-history.md` is owed in the audit's documentation pass.
