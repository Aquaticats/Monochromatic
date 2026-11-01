# Translation repair open decisions: where the rendering audit runs (question 8)

Question 8 on where the rendering audit runs now that it works,
answered on 2026-08-16 with the standalone option over every settled artifact,
kept with the superseded ranking as the record of what the answer was chosen against.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

## Question 8: where the rendering audit runs, now that it works

ANSWERED 2026-08-16:
option B,
standalone over EVERY settled artifact.
Recorded with three other decisions of that date in
`doc/decision/translation-repair-four-answers.md`.

The ranking below puts C first and is SUPERSEDED,
deliberately and for a reason worth keeping:
C won
its place as "the cheapest way to get a RATE rather than a verdict",
and under the standing
best-quality guideline cost is not a constraint,
so the saving is not a benefit.
A census has no
sampling error and additionally answers questions about particular entries,
which C says outright it
cannot.
Everything else in the ranking still holds,
including why the audit stays OUT of the
producing path until its production error rate is known.

The options are kept as written,
because they are the record of what the answer was chosen against.

The absent-baseline instrument (`#85`) is built,
tested,
and MEASURED against
its own planted defect,
three runs of each arm.
On the positive fixture the
dropped negator was found in all three runs:
by all three auditors twice,
and by
two of three once,
where the remaining voice pointed at it but quoted wording
that could not be anchored.
On the faithful fixture every auditor claimed
nothing at all,
every time.

It is wired to nothing.
That is the last decision on it,
and it is yours because
the options differ in what they cost you rather than in whether they work.

WHAT EACH CALL COSTS,
structurally:
one exchange per voice per audited unit,
against whatever roster it is given.
The multiplier is how many units you audit,
and that is what the options differ on.

### Options

A.
 **Per slice,
inside the translate lane,
on every slice it renders.**
    -   For:
every rendering is audited before anything downstream reads it,
and
        the findings arrive while that slice's context is still in hand,
so a
        future gate could act on them without a second pass.
    -   Against:
it multiplies a corpus pass by the roster width,
does that on
        slices nobody has reason to doubt,
and puts an instrument whose
        production error rate is unmeasured inside the path that produces the
        output rather than beside it.

B.
 **Standalone,
over settled version 2 artifacts,
after a pass.**
    -   For:
nothing in the producing path changes,
so a bad audit run cannot
        cost a pass.
It reads the artifact the pass already wrote,
and it can be
        re-run against those same artifacts when the instrument changes,
which
        is exactly what this week needed and did not have.
    -   Against:
it cannot influence what ships without a second pass,
and it
        re-reads text the pass already had in memory.

C.
 **Standalone over a SAMPLE of settled artifacts,
not all of them.**
    -   For:
the cheapest way to get a RATE rather than a verdict,
which is what
        an instrument in shadow mode is for.
A sample is enough to say how often
        the lane's renderings carry corroborated defects.
    -   Against:
it says nothing about any particular entry,
so it cannot settle
        a question about one.

D.
 **Leave it unwired,
and run it by hand when investigating.**
    -   For:
costs nothing,
and the sensitivity arms already show it works when
        pointed at something.
    -   Against:
nothing accumulates.
Every question about the lane's fidelity
        would start from zero,
which is how an instrument built at some expense
        quietly stops being used.

### Ranking

C > B > A > D.

-   C over B,
    because the first thing anyone will ask of this instrument is a
    RATE,
    and a sample answers that for a fraction of the cost.
    B's advantage,
    covering every entry,
    only pays once a rate exists and someone wants to act
    per entry.
-   B over A,
    because re-runnability is worth more than immediacy while the
    instrument is still changing:
    the matcher was rebuilt twice this week,
    and
    under A each rebuild would have needed a fresh corpus pass to re-measure,
    while under B the artifacts are still on disk.
-   A over D,
    because A at least accumulates evidence,
    and D's cost argument is
    real but its outcome is an unused instrument.

WHAT I WOULD DO IF YOU DELEGATE THIS:
C now,
B once a rate exists and per-entry
action is wanted,
and A only if a gate is ever built on the audit,
which would
need its false-positive rate measured on real renderings rather than on one
faithful fixture.

DELEGATE
