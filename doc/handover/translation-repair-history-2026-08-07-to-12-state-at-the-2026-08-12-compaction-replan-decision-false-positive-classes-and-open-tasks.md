# Translation repair history 2026-08-07 to 2026-08-12: the state at the 2026-08-12 compaction: the decision to re-plan, the two classes of false positive, the open tasks and what landed

The state at the 2026-08-12 compaction:
the two user decisions taken after the repair sheet was read,
to stop and re-plan the milestone and to let the running pass finish with its repairs discarded,
the analysis of the eight round-three false positives as two classes,
the open tasks in the order the plan touches them,
and everything that landed in the session.

Part of [Translation repair history: 2026-08-07 to 2026-08-12](translation-repair-history-2026-08-07-to-12.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## State at the 2026-08-12 compaction

Two user decisions,
both taken after the repair sheet was read:

-   STOP AND RE-PLAN the milestone rather than start the next fix.
    The proposal is `doc/planning/translation-repair-milestone-replan.md`.
    It is a PROPOSAL,
    not a decision,
    and it ends with three open questions the
    user has not answered.
-   `pass9-run-001` LETS RUN,
    and its REPAIRS ARE TO BE DISCARDED.
    Its detection output stays valid,
    since the editor defect does not touch
    which issues are accepted.
    NOTE THE GAP,
    which was stated when the option was chosen and is still true:
    nothing today can re-repair a settled entry without recomputing it whole,
    so
    "discard the repairs" means recomputing those entries when the editor is
    fixed.
    Whoever picks this up should not expect a cheap re-repair path to
    exist.

### The analysis that should survive compaction

The eight round-three false positives are two classes,
not eight problems.
FIVE are one class:
a claim that content is unsupported,
filed because the
 licensing evidence sits outside the window the critic judged.

```text
 4  "adds she"        the original uses she/her throughout
 7  "adds gamer"      the original does say so, in another sentence
41  "adds in heaven"  context shows they went to the afterlife
43  "omits confidant" it IS translated, elsewhere in the passage
50  "adds She"        pronouns are she/her from context
```

THREE are a smaller class,
the critic being more literal than the user's policy:
 `20` "on that day" enhances fluency,
`24` 解脱 has no better English rendering
 and is not vague,
`26` 总会 can be "often".

Removing the first class alone would put this sample near 39 of 42,
about 0.93,
 which clears the bar.
That is the entire precision gap,
and it is now explained rather than mysterious.

DO NOT read that as "widen the context window".
Tasks #40 and #41 already
 widened judged context and already render a source context window for
 addition-class claims,
and both are complete;
these five still got through.
An addition claim asserts content appears NOWHERE in the source,
which a window
 cannot establish at all.
The planning doc proposes a document-wide absence
 check instead.

### Open task numbers, in the order the plan touches them

-   `#66` probe reports `noneFound` on damage a reader sees at once.
    Blocks
     cheap verification of any repair change.
-   `#67` editor replaces far more than the defect span,
    21 of 50 edits beyond
     1.35x,
    and separately writes unidiomatic English.
-   `#65` duplicate accepted issues,
    14 percent of the last sample.
    The gate
     excludes them now;
    the pipeline still emits them.
-   `#31` judge crosscheck,
    still deferred and now clearly downstream of #66.
-   `#60` is SUPERSEDED in its framing:
    it asks for the probe's false-positive
     rate,
    and #66 shows the false-negative rate is the problem.

### Everything landed this session, for the record

Channel-marker fix recovering Kimi-K3 across five roles;
widened rosters with
 every fan-out stage at three voices and a quorum of two,
plus full-roster
 retry;
draw digest binding sheets to an exact draw;
narrowed telemetry claim
 types;
per-entry scoping of the probe region collapse;
ROSTER reporting for
 stage degradation;
duplicate as a first-class grade verdict;
the round-three
 gate verdict;
and the recall re-measure.
All committed and pushed on `translation-repair-rebased`.
