# Whether to add the source-sentence field to the coverage wire, and rerun

Queue item 3 from `doc/handover/two-lane-outcome-vocabulary.md`,
written 2026-08-16.
It is a proposal,
not a decision:
question 28 in `doc/planning/translation-repair-open-decisions.md`
is still the user's,
and nothing here has been landed.

## The question

`#106` records a measured weakness in the coverage prototype.
An anchored quote proves the English EXISTS,
not that it renders the passage asked about.
On `XingZ60` section 14,
two of six voices quoted a real sentence belonging to the introduction,
and both quoted the same irrelevant sentence,
so voice agreement does not separate it.
The recorded fix is a second field naming the SOURCE sentence a quote renders,
checked against the
passage,
which is a wire change plus a rerun.

The handover flags a trade against doing it,
and the trade is real:
option A of question 28 is strong partly because it is BUILT AND MEASURED TWICE,
and changing the wire makes both measurements describe code that no longer exists.

## The part of the trade that was stated backwards

The handover's caution treats the wire change as a cost to question 28's evidence and nothing more.
Reading the failure direction against what each scale concluded says something sharper.

At SECTION scale the instrument produced absence verdicts and was right about all of them:
`XIEPT2` eight times absent,
`XingZ60` sections 13 and 14 absent,
section 12 carried,
eleven of eleven correct against hand checking,
on which the recorded labels were wrong nine times.
A bias toward false CARRIED did not stop it finding absence there.

At BLOCK scale the conclusion is a null:
across both runs of `mikaela_khara`,
ninety-six answers,
not one vote for absence,
from which `#106` concludes the case against landing four is stronger
than first measured,
at most one of twenty-two and plausibly none.

A false CARRIED is precisely the answer a permissive evidence rule manufactures.
So the block-scale null is the measurement MOST exposed to this defect,
not the least,
and it is the one question 28 leans on when it argues against landing four.
The section-scale verdicts are comparatively safe,
because absence findings are not what the
defect produces.

Two readings fit the block-scale unanimity,
and the existing evidence does not separate them:

-   BENIGN.
    Block candidates are single lines inside a document that IS translated,
    so a covering
    span usually genuinely exists,
    and the census already attributes the high end of the length
    signal to merges and the low end to mispairings rather than to omissions.
-   CONTAMINATED.
    The evidence rule admits any non-empty quote,
    `#106` records that the single word
    `September` was accepted as evidence and stopped only by the locator's ambiguity check,
    and correspondence is never checked at all.

Nothing recorded distinguishes them.
The defences already in `#106` do not:
the unanchored-quote analysis,
`f75b5a85c` carrying the
document's own text,
and `b46c8823b` judging uniqueness over the broadest accepted form all concern
whether a quote is really IN the document,
which is the question that was already being answered.
Correspondence is the question that is not.

## What a rerun would cost, and what nothing currently keeps

The coverage probe writes its result with `console.log(JSON.stringify(...))` and nothing persists it.
The 2026-08-16 numbers at both scales therefore exist only in session transcripts:
a search of `~/temp/agent`,
the corpus clone and the worktrees for retained coverage output finds
`merge-census.json` from the block census and no probe verdicts at all.

That has a consequence for every option:
correspondence CANNOT be scored over the existing
measurements,
because the pairs of candidate passage and submitted quote were never written down.
Any correspondence number requires generating fresh pairs.

It is also worth fixing on its own account,
independent of this question.
A probe whose output survives only in a transcript cannot be re-read after a session ends,
which is the same failure the handover exists to prevent.

## Options

### A. Measure the false-CARRIED rate first, over the unchanged wire

Rerun the probe exactly as it stands,
retain the candidate and quote pairs this time,
and score correspondence as a SEPARATE judgement that no production path consumes.

-   Pros:
    answers whether the wire change is needed before paying for it;
    leaves question 28's option A built and measured as it was,
    since nothing shipped changes;
    the correspondence rate is decision-relevant to question 28 itself,
    because it says how much of
    the block-scale null to believe;
    fixes the retention gap as a side effect.
-   Cons:
    costs a probe run and a scoring run;
    produces a THIRD sample at section scale,
    whose
    ordinary variance against the first two has to be read as variance rather than as change;
    does not fix the defect,
    only sizes it.

### C. Leave the wire alone until question 28 is answered

-   Pros:
    preserves the evidence base of an open user decision exactly as it was measured;
    costs nothing.
-   Cons:
    leaves a known bias unmeasured,
    in the direction of the conclusion being relied on;
    the user answers question 28 without knowing how much the block-scale null is worth.

### B. Add the source-sentence field and rerun both scales now

-   Pros:
    fixes the defect that was actually observed,
    at the point where it occurs;
    a rerun under the new wire is the number that would then be true.
-   Cons:
    makes both existing measurements describe code that no longer exists,
    while the decision
    they support is open;
    the user would be choosing option A of question 28 on evidence that no longer applies to the
    artifact they would be choosing.

### D. Add an identifying-evidence constraint to the wire guard, and rerun

The constraint `#106` already records as not acted on:
any non-empty quote is admissible today.

-   Pros:
    smaller change than the source-sentence field;
    addresses a recorded gap.
-   Cons:
    pays the same invalidation cost as B for less;
    and it would NOT have caught the failure that motivates this question,
    because
    `XingZ60` section 14's quote was a real,
    substantial sentence that simply rendered a different
    passage.

## Ranking

A > C > B > D.

-   A OVER C,
    because the contamination runs in the direction of the conclusion question 28 relies
    on,
    so measuring it changes what the user is choosing between,
    while C leaves them to decide
    without it.
    A's cost is one cheap probe run.
-   C OVER B,
    because both leave the instrument unfixed for now,
    and B additionally destroys the
    evidence base of an open decision.
    The prototype is wired to nothing,
    so the defect harms no
    production path while the question waits.
-   B OVER D,
    because B targets the failure actually observed and D does not:
    a minimum-evidence
    rule rejects the single word `September` but admits a whole irrelevant sentence,
    which is the
    case that was measured.

## What A would consist of

Recorded so the work is not re-derived,
not because it is authorized.

1.  Persist the probe's output into the runs directory instead of stdout only,
    so a verdict survives
    the session that produced it.
    This is worth doing whichever option is chosen.
2.  Rerun `coverage-probe` at both scales on the same candidates `#106` names,
    the eleven section candidates and `mikaela_khara`'s sixteen block candidates.
3.  Score correspondence separately:
    for each anchored quote,
    ask whether it renders the candidate
    passage,
    as its own judgement over the retained pair,
    consuming nothing in the production path.
4.  Report the false-CARRIED rate per scale,
    beside the unchanged coverage verdicts,
    and read the block-scale null against it.

## What this must not do

Nothing here may run while the two-lane cost run holds the roster.
The window trial and the corpus pass already competed for the same six models once,
and a probe launched alongside would both slow that run and confound the voice-loss rate it reports.
The task tracker carries the cost run as `#114`.
