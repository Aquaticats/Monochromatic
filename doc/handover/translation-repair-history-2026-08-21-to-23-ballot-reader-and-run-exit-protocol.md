# Translation repair history 2026-08-21 to 2026-08-23: the ballot reader, a matcher control and the run-exit protocol

The reader built and validated while a live pass ran,
the finding that both tail preconditions reproduce on the earlier run,
the control that shows the class-instance matcher can fail,
what work was and was not available during the pass,
and the ordered steps to take when the pass exits.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-22 late: the ballot reader is built and both tail preconditions reproduce

The `#163` boundary verification is waiting on a live pass,
so the instrument it
needs was built and validated against settled artifacts first.
It is `${HOME}/temp/agent/163-ballots.mjs`,
and it answers the precondition
question before the ballot question,
because a slice where the note never fires
verifies nothing.

WHAT IT READS.
Ballots live at `.laneSelection.slices[].ballots[]`,
each
carrying `choice`,
`unsupported`,
`dropped`,
`unsupportedRaw`,
`droppedRaw` and
`reason`.
The three renderings come from `.comparison[]` as `incumbentText`,
`repairText`
and `translateText`,
and the Chinese from `.lanes.repair.delivery[].sourceText`,
matched by `chunkIndex`.
Artifacts settled before `laneSelection` existed still carry `comparison`,
so
the reader falls back to it and reports geometry without ballots,
which is what
made the two known tail entries readable at all.

IT REBUILDS THE REAL NOTE RATHER THAN A LOOKALIKE.
It imports `contestSizeNote`
from `dist/final/node/index.mjs` and passes the same three labels
`lane-contest-wire.ts` passes,
so "did the note fire" is the pipeline's answer.
Judge prose never reaches stdout:
ids,
character counts,
ratios,
tail names,
verdict kinds and ballot enums only,
with `reason` and the raw arrays written to
`<runsDir>/163-ballot-detail.json`.

VALIDATED BEFORE IT WAS TRUSTED,
because its first real result was a null:
17
slices across 6 entries of `vub-run1-20260821`,
0 notes fired.

-   Synthetic positive control on invented text,
    a 96-character original
    against a 2250-character rendering:
    `target-far-longer` at 23.44 times,
    and
    against a 15-character one,
    `target-far-shorter` at 0.16.
    The note fires and
    the reader reports FIRED,
    so the null is a real absence.
-   Three figures recorded in `contest-size-note.ts` before this reader existed
    reproduce exactly:
    `dogesir_` slice 3 at 15.49 times,
    its neighbours across
    0.88 to 6.10,
    and a produced ratio of 9.27 against the 10 endpoint,
    which
    turns out to be `wangzihao980` slice 4.

BOTH PRECONDITIONS HOLD on `translation-repair-runs-flagged-20260818`:

-   `dogesir_` slice 3:
    Chinese 114 characters,
    archive and repair both 1766 at
    15.49 times,
    translate 215 at 1.89.
    Tails `target-far-longer` and
    `block-count-gap`.
    Repair equals the archive byte for byte,
    which is the
    documented trip condition:
    every tail is a lane handing back the archive.
-   `wangzihao980` slice 3:
    Chinese 102 characters,
    archive and repair both 66
    at 0.65 times,
    translate 184 at 1.80.
    Tail `target-far-shorter`.

WHAT A CORRECT BALLOT LOOKS LIKE,
read off `SIZE_NOTE_POLICY` rather than
invented for the occasion.
At `dogesir_` slice 3 the surplus is page content the
Chinese is silent about,
so the DROPPED-ALSO rule governs and the SHORTER
candidate is the one that lost something:
judges should put `translate` in
`dropped` and keep the long rendering.
At `wangzihao980` slice 3 the archive and
repair are far shorter,
so the DROPPED question goes to them in particular.

THE LIVE RUN MAY NOT REPRODUCE THE SLICING.
These figures are the 08-18 run's,
and `#131`,
`#157` and `#159` all changed pairing since.
If the live pass slices
either document differently,
or the repair lane changes slice 3 instead of
returning the archive,
no note fires there and the run verifies nothing.
That is
a finding to record,
not a failed verification,
and the reader reports it as
`note=silent` rather than as a judge ignoring evidence.

### toBeInstanceOf was never shown able to fail, and now has been

Every conversion `#127` landed rests on `toBeInstanceOf`,
and no failure of that
matcher had been observed this session:
every failure seen was `toBe` or a scope
error.
A broken-permissive matcher would have made all 25 class conversions pass
identically,
so the family was resting on an unvalidated probe.

Controlled in both forms,
against committed files,
restored afterwards with
`git checkout --`:

-   Async,
    `runs-lock.unit.test.ts:94`,
    `RunsDirectoryBusyError` to `TypeError`:
    FAIL,
    exit 1,
    `expected RunsDirectoryBusyError... to be an instance of
    TypeError`.
-   Sync,
    `pipeline-digest.unit.test.ts:277`,
    `TypeError` to `RangeError`:
    FAIL,
    `expected TypeError... to be an instance of RangeError`.

Both files pass again after restore.
This supersedes any plan to revert a source
error class and rebuild to prove the same thing:
each converted file already
runs against real `dist`,
and one wrong-class failure closes the gap for the
whole family without a build.

### What is actually available while the pass runs, and what is not

Checked against the tracker rather than assumed,
because the obvious next item
turned out to be blocked in the opposite direction from the one expected.

-   `#98` records `Blocked by: #106`,
    not the reverse,
    and its own fix order
    says not to route the equal-count fast path through the current aligner
    just to route it:
    the gate waits until heading scoring gains a signal for
    handle-free headings.
-   `#106` blocks `#98` and `#100`,
    and ends with an explicit instruction not
    to wire `groupSourceFirst` into `subdivideChunkPair` nor emit insertion
    pairs from `alignDocumentSections` until the user answers question 28 in
    `doc/planning/translation-repair-open-decisions.md`.
-   `#106` also must not start while a pass holds the roster,
    which the live
    verification run does.
-   `#94` is held with `#100` for a stated reason that still holds:
    `#100`
    changes what `alignmentPairCount` means,
    which is the denominator confusion
    the rename exists to prevent,
    so renaming first means renaming twice.

So the alignment cluster is gated on a USER DECISION,
not on the build.
Nothing
in it should be started unasked.

### Run-exit protocol, in order

Ballots outrank the build,
because the run is the perishable thing.

1.  `node ${HOME}/temp/agent/163-ballots.mjs ${HOME}/temp/agent/163b-verify`.
    The sharp question is whether the new slicing still produces a slice 3
    where the repair lane returns the archive.
    If it does not,
    no note fires
    and the run verifies nothing,
    which is a finding rather than a failure.
2.  `mise run //package/module/translation-repair:build` (moved on 2026-10-18 to
    `//package/translation-repair/<package>:build`),
    which only becomes
    safe once the pass has exited.
3.  `mise run //package/module/translation-repair:test:unit` (moved on 2026-10-18 to
    `//package/translation-repair/<package>:test:unit`),
    to confirm the
    baseline holds with the nine classes `56ba8f1dd` added.
4.  The 24 of `#127`,
    per `${HOME}/temp/agent/127-step2-mapping.txt`:
    convert,
    run each file with `node <file>`,
    then `test:unit` again.
    Never run
    `test:unit` before the build;
    it declares no `depends` and passes vacuously
    against a stale `dist`.
