# Translation repair history

## Current disposition

The finite redesign stopped after Candidate M on 2026-09-01.
Read the [timestamped failure report](../audit/translation-repair-redesign-failure-2026-09-01.md),
the [current handover](translation-repair.md),
and the [document map](translation-repair-document-map.md) before the historical record.
No historical next-action list authorizes a corpus run,
model call,
spent-prompt retry,
or successor implementation.

This index replaces one append-only historical file,
written from 2026-07-16 to 2026-09-01,
with a record in chronological groups.
Each group has its own index,
which carries the stop condition and lists the group's files;
each file holds one subject or one span of days and is named for it.
Superseded claims remain historical evidence,
not current instruction.

The split of 2026-09-01 cut the file into groups and then,
by size,
into segments named by number,
and the audits of that day linked from the [document map](translation-repair-document-map.md) describe that layout.
On 2026-10-13 the segments of each group were joined again,
with the titles and continuation headings that split had added removed,
and filed by what they hold.
Headings were added where a long stretch had none;
no other wording changed.

Dates in the record are the working day as written at the time,
with no zone,
and the record is kept as written rather than redated.
An owner answer given between 20:00 and 24:00 EDT
(00:00 to 04:00 UTC)
can carry the day before its UTC date:
the voting rulings recorded under 2026-08-14,
for one,
were given between 00:47 and 01:44 UTC on 2026-08-15.
The decision records,
the current handover
and the package's own docs and source
date owner answers by their UTC day
(ledger D12 in `package/module/translation-repair/doc/audit-ledger.md` (moved on 2026-10-18 to
`package/translation-repair/doc/audit-ledger.md`)).

Task numbers in the record (`#N`,
"task N") are the writing session's task-tracker ids,
not GitHub issues,
and are kept as written;
the document map's "Task numbers in these documents" says how to read them.

## Chronological groups

- [The first handover document, 2026-07-16 to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md):
  milestone one,
  milestone two,
  milestone three's first round,
  initial provider behaviour,
  the corpus passes of late July,
  the per-call deadline and idle guard measurements and the panel-size confound,
  the early package state,
  and the work of 2026-08-13 to 2026-08-15 that the document recorded last.
- [2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md):
  roster repair,
  rebasing,
  the recall benchmark,
  the editor ensemble and the naturalness lane,
  measurable repair quality,
  the introduced-defect probe,
  the sensitivity checks of the probe and the checkers,
  and round three's preparation and first runs;
  the roster alias trap is also recorded in the first handover entry.
- [2026-08-07 to 2026-08-12](translation-repair-history-2026-08-07-to-12.md):
  run interpretation,
  deadlines,
  the Kimi-K3 schema-mismatch degradation,
  how the sample is drawn and its sheets joined to verdicts,
  the introduced-defect probe and the naturalness audit,
  and round-three grading.
- [2026-08-12 to 2026-08-15](translation-repair-history-2026-08-12-to-15.md):
  probe blindness,
  the overnight census of unread signals,
  coverage tables,
  the judge crosscheck,
  attribution,
  voting,
  cache incidents,
  the invisible character that welded paragraphs,
  and the channel marker that came back;
  the first handover entry records the nights of 2026-08-13 to 2026-08-15 too,
  so those nights appear in both entries.
- [2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md):
  the whole-day review of the night of 2026-08-15,
  the settled artifact's version,
  slice identity and cache keys,
  insertion anchors and the delivery ledger,
  the lane handed a passage the archive never translated,
  the removal of the proportional aligner,
  the block grouper,
  the section census and coverage probe,
  quote anchoring,
  the fidelity trial and the displacement screen,
  the state record after the displacement screen was rebuilt,
  judge bias,
  and the window trial's pre-flight,
  plumbing
  and blockers.
- [2026-08-19 to 2026-08-21](translation-repair-history-2026-08-19-to-21.md):
  reading-stage runs,
  window experiments,
  the critic voice-loss alarm and the code work it blocked,
  the end-to-end corpus pass stopped on purpose,
  and scope correction.
- [2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md):
  the document that began on the 21st and ended on the 23rd:
  consolidation,
  the document publisher,
  the ratio guard,
  the roster sweep and median baseline,
  the runaway emission bound,
  and the size note;
  six of its files are 2026-08-23 work,
  the far-longer tail and the note floor,
  the naturalness auditor window,
  repetition findings merged by occurrence,
  the settled-neither measurement,
  the size note boundary verification
  and the throw-assertion sweep finish,
  while the entry for 2026-08-23 alone is a separate record of that day.
- [2026-08-23, the work recorded for that day alone](translation-repair-history-2026-08-23-single-day.md):
  absence evidence and its decision,
  adopting option B for the archived text on the lane contest,
  self-certification,
  producing roster width,
  cross-language pairing,
  and two smaller fixes.
- [2026-08-23 to 2026-08-24](translation-repair-history-2026-08-23-to-24.md):
  work that reached the 24th:
  checker width,
  ballots,
  publication verification,
  generation changes,
  and branch tests for the modules the suite reaches only through a caller.
- [2026-08-24 to 2026-08-25](translation-repair-history-2026-08-24-to-25.md):
  work that reached the 25th,
  where the 2026-08-23 to 2026-08-24 entry stops at the 24th:
  the editor and writer calibrations,
  telemetry,
  error text,
  provider boundaries,
  the empty-runs verification fix and the combined build of the parked work,
  the naming of the two-lane artifact family and the landing rehearsal,
  the documentation of the report commands,
  refusal reporting with the policy on repeated messages,
  and the reach of the test suite.
- [2026-08-26, the package audit and the work after it](translation-repair-history-2026-08-26-audit-and-the-work-after-it.md):
  the audit,
  the major and minor findings it landed,
  and its close,
  then the overlap and straggler-grace measurements
  and the readiness signal the owner rejected.
- [Stop of 2026-09-01](translation-repair-history-2026-09-01-stop.md):
  Candidate M's terminal disposition and the links to the canonical report.

## Redaction timing

The
[package-local redaction policy](../../package/translation-repair/doc/redaction-timing.md)
defers documentation redaction while the package remains unfinished and not production-ready.
The historical files preserve corpus,
review,
calibration,
and takeover evidence for later package work.
Redaction becomes a required readiness gate before a finished or production-ready declaration.

The timing deferral does not authorize committing credentials,
API keys,
or raw provider payloads.
Those materials remain private throughout development and after release.
