# Translation repair open decisions: the straggler grace's one-model tax (question 9)

Question 9 on who pays the straggler grace's remaining cost,
the six options from leaving the window to re-asking a cut voice,
and the ranking.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

## Question 9: what to do about the straggler grace's one-model tax

`STRAGGLER_GRACE_MS` is 180000,
unchanged since `doc/decision/translation-repair-straggler-grace.md`
widened it from 60000 and measured a roughly three-and-a-half-times drop in voice loss.
First
production traffic now shows WHO pays that window's remaining cost:
of 19 cuts over 2025 streams,
16 are `hf:zai-org/GLM-5.2` (4.88 percent of its 328 streams) and 2 are `hf:Qwen/Qwen3.6-27B`,
newly appeared and on its two largest deliveries in the log (3258415 and 3209277 characters).
Three
of six models have never been cut.

THE WINDOW ALSO CANNOT TELL A FULL-SPEED CALL FROM A CRAWLING ONE,
which is a second,
separate
finding:
the audit's two example abandonments were both cut at exactly 180000 ms,
so dividing
delivered characters by that shared floor gives an upper bound on each rate,
at most 16196
characters a second for one and at most 686 for the other,
twenty-three times apart.
Both are
upper bounds,
since the true duration is censored at the cut,
but the shapes read as different:
one
was still producing,
the other was closer to crawling.

AND NOTHING RE-ASKS A CUT VOICE,
by design.
`stage-quorum.ts` stops its retry rounds once quorum is
satisfied,
so a voice lost to the grace after quorum already stood is never re-dispatched.
The
consequence named in the audit is the one worth deciding on:
on the stages this happens,
the
ensemble is effectively five of six,
always missing the same member.

WHAT REDOING THE ORIGINAL DERIVATION WOULD NEED,
and does not have:
the 180000 figure came from
per-model whole-call latency percentiles over a 602-exchange bench under the roster of the time.
Nothing read for `#121` measures that under the CURRENT six-model roster;
first-byte and mid-stream
gap are a different measurement,
and cut rate by model is a rate rather than a latency.
A fresh
per-model latency bench is the prerequisite for computing a new number rather than guessing one,
and running it spends quota this task was not authorized to spend.

### Options

A.
 **Leave `STRAGGLER_GRACE_MS` at 180000.**
    -   For:
it already measured a roughly three-and-a-half-times drop in voice loss over the
        window it replaced,
costs nothing to keep,
and the residual is concentrated on models this
        ensemble is built to tolerate losing occasionally rather than a class of hangs.
        `doc/decision/translation-repair-straggler-grace.md`'s own backlog already says the LAST
        widening's cost needs a run-to-run band before it needs a decision,
and that band still
        does not exist.
    -   Against:
it keeps taxing the same voice on the same stages repeatedly rather than fixing
        anything,
and 84 percent of cuts landing on one model is the shape the audit specifically
        flagged as worth deciding on rather than absorbing quietly.

B.
 **Widen `STRAGGLER_GRACE_MS` further,
pending a fresh per-model latency bench.**
    -   For:
directly targets the harm the audit's full-speed example shows,
a call still
        delivering real content when the grace ends it,
and the prior widening (60000 to 180000)
        is precedented,
measured,
and worked.
    -   Against:
no fresh per-model latency data exists to derive a new number from under the
        current roster,
so any figure chosen now is a guess dressed as a measurement;
it also
        raises the cost on the crawling-call shape (the 686 characters-a-second example),
and the
        prior widening's own remeasure found a real,
unbanded wall-time cost (+21 percent
        aggregate,
+28.6 percent on one entry) that widening again would compound before it is
        even priced.

C.
 **Give `hf:zai-org/GLM-5.2` (or any model crossing a cut-rate threshold) a longer grace than
    the roster default,
rather than moving the shared window.**
    -   For:
targets exactly what was found,
one model's concentration,
without paying a wider
        window's cost on the other five models' much rarer cuts.
    -   Against:
`runGatherRound` takes one shared `graceMs` per round with no per-model concept
        today,
so this is a real implementation change layered on top of the policy choice,
and it
        privileges one vendor's model by name in code,
which stays meaningful only while the
        roster does not change.

D.
 **Switch from a fixed-duration grace to a production-rate cutoff.**
    -   For:
the audit's two examples show the fixed window conflates "still producing" with
        "crawling",
and a rate test would keep the first kind of call and cut the second,
which is
        closer to what the grace is actually meant to discriminate.
    -   Against:
exactly two examples exist to validate any rate floor against,
both upper bounds
        with an unknown true rate,
so a specific threshold chosen now fits n=2;
it is also a larger
        implementation than a constant edit.

E.
 **Drop `hf:zai-org/GLM-5.2` from the roster.**
    -   For:
removes the tax at its source,
and the model's cut rate under the CURRENT roster
        (4.88 percent,
sixteen times the next-highest model measured here) is now a direct
        measurement rather than an inference from an older bench.
    -   Against:
the straggler-grace decision already measured this model answering in full on a
        large share of its seated stages,
so dropping it trades a slow-but-mostly-working voice for
        a permanently five-model ensemble on every stage rather than most of them;
`#105`'s framing
        was explicitly "widen the deadline,
or seat a replacement",
and the straggler-grace
        decision already answered against replacement once.

F.
 **Re-ask a straggler once the grace cuts it,
instead of leaving the round at whatever quorum
    already heard.**
    -   For:
recovers the lost answer even when the round itself could not wait for it,
so a
        working-but-slow reply is not simply discarded.
    -   Against:
this reverses deliberate,
documented design;
`stage-quorum.ts`'s retry rounds stop
        once quorum stands specifically so a satisfied round is not delayed further,
and re-asking
        after the round has moved on means judging the straggler against a slate the rest of the
        panel no longer shares,
close to the confound `#109`'s window-trial ledger exists to avoid.

### Ranking

A > C > D > B > F > E.

-   A over C,
    because C is a real,
    unvalidated behavior change naming a single vendor's model in
    code,
    while A costs nothing and `translation-repair-straggler-grace.md`'s own backlog already
    says the LAST widening's cost needs measuring before the next one is decided;
    acting again
    before that band exists repeats a mistake this document family has already corrected twice.
-   C over D,
    because both target the concentration precisely,
    but C only has to choose one number
    (a per-model grace) where D has to fit a rate floor to two examples,
    both upper bounds;
    being
    wrong about C's number costs one model some extra minutes,
    being wrong about D's rate risks
    cutting any slow-starting healthy call on any model.
-   D over B,
    because D at least targets the signal the audit found,
    rate rather than duration,
    where B pays the same wider-window cost on all six models' rare cuts to fix a problem that is
    concentrated on one.
-   B over F,
    because B is at least the same kind of change,
    a constant,
    with a measured precedent
    behind its last move;
    F fights the pipeline's own documented design intent and risks the exact
    judging confound `#109` was built to remove.
-   F over E,
    because F costs a design conflict and a possible confound,
    both recoverable;
    E is a
    roster change already argued against once in this codebase's own history,
    and it discards a
    model shown to answer usefully on most of the stages that seat it.

WHAT I WOULD DO IF YOU DELEGATE THIS:
A now,
then C once the run-to-run cost band the
straggler-grace backlog already calls for is measured,
since C is the option that answers "the
grace's cost is concentrated on one model" without guessing at a number the current roster has no
bench for.

DELEGATE
