# Ensemble voting: who is waited for, who judges, and what a self-vote is worth

Superseded in part on 2026-10-25 by
`package/translation-repair/doc/design-commitments-votes-repairs-and-run-offs.md`:
the minimum weight a selection needs (the weight the bench could award one candidate,
reached by four contributors)
is gone,
and two answering judges decide whatever their weight.
A self-vote still counts as one answering judge (the owner's ruling of 2026-10-20).

Decided by the user on 2026-08-15,
in three separate rulings during the session
that built the translate lane.
Each one reverses or narrows an earlier choice recorded in
`doc/handover/translation-repair-history.md`,
so this document is canonical for all
three and the older passages are history.

## The standing rule: one model's bad day must not delay the pipeline

User,
2026-08-15:
"The failure of any one model for the day must not delay the pipeline."

This is the principle the rest of this document applies,
and it is wider than
any one mechanism.
A model that answers nothing,
or answers very slowly,
is an ordinary operating
condition on this provider rather than an incident,
and the pipeline is expected
to keep its throughput through one.

Two mechanisms are measured against it here,
and they land differently:

-   RETRY TARGET,
    addressed:
    waiting for a whole roster spent four deadlines per
    gather on a voice that was not coming,
    which is delay in its purest form.
    Removed.
-   ROUND COMPLETION,
    addressed the same day:
    a round awaited every call
    together,
    so a stage finished no sooner than its SLOWEST voice even once
    quorum stood,
    and one model hanging until its deadline delayed every stage
    that seated it by up to 360 seconds.
    A round now abandons whatever is still in flight sixty seconds after quorum,
    `STRAGGLER_GRACE_MS` in `stage-round.ts`.

Cutting AT quorum was offered as the alternative and rejected by the user in
favour of the grace window.
The reason is arithmetic rather than caution:
quorum on a roster of three is
two,
so cutting there would discard the third voice on nearly every gather,
healthy or not,
and shrink every ensemble to its minimum permanently.
A window separates the two cases,
because a working third model answers within
seconds of the second and a hung one never answers at all.
Sixty seconds is the user's figure and is not derived from the latency
distribution;
run 013 measured time-to-first-byte at p50 45_837 ms and p90
163_296 ms,
so it should be re-read against a measured one.

The round assembles its result from what ARRIVED rather than by awaiting the
calls it abandoned,
so a client that ignored an abort would cost a voice rather
than hang the stage.

## No stage waits for its whole roster

`gatherStageVoices` retries lost voices and stops at QUORUM,
at least half the
roster rounded up.
There is no other target.

The user's reason,
verbatim:
"full roster should never be a retry target for anything,
because that will
block everything even if only one or two model of the provider is degraded for
the day."

What this reverses:
the editor and refiner stages passed `retryTarget: 'full-roster'` from
2026-08-12,
chosen on the belief that Kimi-K3 was permanently broken.
Under that target a stage re-asked a silent model for every retry round,
so one
model degrading for a day cost four deadlines per gather,
on every gather that
seated it,
for as long as the degradation lasted.

Why removing it is safe rather than a loosening:
the target existed to stop a single model deciding a stage,
and the quorum
arithmetic already prevents that on the rosters in use.
Editors,
refiners and checkers each sit at three,
so quorum is two,
and two
independent voices are exactly what the ensemble is for.
The option is deleted rather than left unused,
so it cannot return by default.

What is kept:
`stage-roster-incomplete (<stage> <heard>/<roster>)` is now emitted whenever a
roster ends short,
not only under the retired target.
The per-model `stage-voice-lost` findings say WHO went quiet and this says how
much of the stage that cost.

## Producers judge

`selectBestCandidate` seats the whole judge roster,
including every model that
produced a candidate in the set.

The user's reason:
"A model can both be a translator and a judge.
Yes,
a model judging its own
output would be worse than judging others' output,
but its own judgement would
still be somewhat valuable",
and separately,
on why a narrow judge pool is the
wrong shape:
"These models,
each of them has different blank spots."

What this reverses:
selection removed every producer from the roster,
which on a six-model roster
with three editors silenced half the panel to keep the other half
disinterested.
The readings thrown away were not replaced by anything.

## A self-vote counts half

`SELF_VOTE_WEIGHT` is `1 / 2`,
`FULL_VOTE_WEIGHT` is `1`,
and a winner needs
`MIN_SELECTION_WEIGHT`,
which is 2.

The user's ruling:
"Self-judge and self-vote should always be allowed,
just given less weight."

The arithmetic preserves the property the old exclusion protected,
on the
rosters in use.
A single-model candidate draws at most one half from its own author and a
three-contributor composite at most three halves,
both short of 2,
so no
candidate is selected by its own authors alone.

STATED PRECISELY,
because the first version of this document got it wrong and
so did three other files:
the discount attaches to a JUDGE AND CANDIDATE PAIR,
not to a judge.
A producer voting for another model's candidate carries FULL weight,
which is
the entire point of seating it.
Two consequences follow,
and the first version denied both:

-   Selection imposes no ceiling on roster width.
    Four editors judging each
    other's work select perfectly well,
    with or without a model that edits
    nothing.
-   The self-votes-alone property holds while no single candidate has four or
    more stakeholders,
    and fails at four,
    where halves sum to the threshold.
    A
    four-contributor composite is the reachable case.
    Nothing enforces the
    bound;
    it is a fact about rosters of three.

A half is not a measured figure and is not presented as one.
What the discount corrects is a TILT rather than a preference:
the judge sheet is anonymized and says so,
so a producer cannot see which
candidate is its own and cannot set out to back it.
Measuring the real self-preference rate is the judge-quality bench's work,
and every self-vote is
recorded by name (`select-self-vote (<model>)`),
weighed on its ballot,
and
counted in `SelectionTally.selfVotes` so that measurement has a population.

## Judges never learn who wrote what

Stated by the user as a check rather than a change,
and verified rather than
assumed.

`buildCandidateSelectMessages` renders candidates as `CANDIDATE 1` through
`CANDIDATE N` with fenced text,
and the system message tells the judge it does
not know which system produced which candidate and must not guess.
No producer name,
no role label,
and no "incumbent" marker reaches a judge.

The translate lane holds to this in the one place it could have leaked:
the existing translation stands on the ballot unlabelled,
and it is deliberately
NOT repeated in the evidence block,
where a judge could have matched it against
a candidate and identified it.
Declared names travel as evidence because a judge cannot check terminology
without them,
and they identify nobody.

## Consequences already applied

-   Slice cache version 25.
    Both behavioural rulings change who was heard and who decided while leaving
    every prompt identical,
    which is the class the structural guard cannot
    catch.
-   `SelectionOutcome` carries `voteWeight` rather than `votes`.
    The number is a weight,
    and a log line reading "won 2.5 votes" would be a
    lie about what was counted.
-   Every ballot leaves the selector with its model,
    choice,
    reason and weight.
    Reasons reached a log line and nothing durable before,
    and one lost pipe on
    2026-08-13 erased twenty minutes of them.

## Every producing role widens, and self-certification is weighed too

User ruling,
2026-08-15,
asked as a choice between holding the producing roles
where they were and widening them:
"All producing roles to 4,
just assign lower weights to self-certification."

Followed by the constraint that decides how it is written:
"The provider updates their offerings frequently.
Don't hardcode magic numbers
like 4 or 6."

### What the widening is NOT bounded by

A first version of this section derived the four as "roster size minus the
full-weight judges a selection needs",
on the claim that models had to stay
outside a producing role for full-weight ballots to exist at all.
That claim is FALSE,
and it is the second time the same error was written down
in one day,
so it is recorded here rather than quietly deleted.

The discount attaches to a JUDGE AND CANDIDATE PAIR.
A candidate's full-weight judges are everyone who did not write THAT candidate,
which is nearly the whole roster however many models produce.
Seat every model as an editor and a candidate by one of them still draws
full-weight ballots from every other one;
only its own author is discounted.
Nothing about selection breaks,
and nothing about it sets a ceiling.

### What actually moves with roster width

-   COST.
    Each added producer is another call per slice,
    and each candidate it
    adds becomes repeated input to every judge in the selection round.
    The
    growth is worse than linear in producers for that reason.
-   AGREEMENT.
    Ballots spread thinner across more distinct candidates,
    so the
    leader more often falls short of `MIN_SELECTION_WEIGHT` or ties.
    Both
    outcomes decline,
    and a decline keeps the incumbent,
    so widening the
    producers can quietly REDUCE how often anything is replaced.
-   COVERAGE.
    More independent renderings of the same slice is the thing the
    widening is for,
    on the user's own reasoning that these models have
    different blind spots.

None of those is a bound the code can compute.
What the roster size expresses is
a tradeoff,
and the number therefore belongs in run configuration with its
reasoning beside it,
not in a derivation that pretends to necessity.

### What is decided

Checkers stop excluding producers.
That WAS a real bound:
checkers excluded every editor and refiner,
so widening
a producing role starved the checker roster,
and the checkers left behind would
have sat at a quorum of one,
the single-voice failure the 2026-08-12 roster
change closed.
The ruling answers it by extending the selection discount to certification:
a
checker may certify text it helped write,
and that verdict counts for less than
a disinterested checker's.
`assertCheckerIndependence` stops being a refusal and becomes a weighting
(the self-certification weight).

What this does NOT decide,
and what the implementation must not assume:
the
certification weight.
`SELF_VOTE_WEIGHT` is a half by an argument about selection arithmetic that does
not transfer,
because resolution checking tallies verdicts about one claim
rather than ranking candidates against each other.
The self-certification weight work owns picking that number and saying what it rests on.

One consequence to expect rather than discover:
seating producers grew the
full-weight panel for any given candidate,
and widening the producing roles
moves it again.
Tie and decline rates will shift where nothing else changed,
and the judge-quality bench inherits
that too.

## An invalid candidate is sent back to its author, not dropped

User ruling,
2026-08-15,
on what a whole-slice validator should do with a
translation whose structure does not match the source.
Three options were offered,
to drop it,
to show judges everything,
or to drop
only on reference damage,
and all three were rejected:

"The pipeline should try fixing it by giving the findings to the original model
in the same chat,
and the original model can say it can fix it,
it can't fix it,
or for whatever reason the broken candidate it produced is the best possible
version for the information it has."

So validation is a CONVERSATION rather than a filter.
A candidate that fails carries its findings back to the model that wrote it,
in
the same exchange,
and that model answers with a revision,
an inability,
or a
defence of what it already produced.

The third answer is the one no filter could ever have collected.
A model that dropped a footnote marker because the definition it points at is
not in this slice is telling the pipeline something about the SLICING rather
than about itself,
and a validator that silently dropped that candidate would
have destroyed the only report of it.

Tracked as the invalid-candidate re-ask:
an invalid translated slice goes back to its author rather than being dropped.

## What the code enforced until 2026-08-15, and what it enforces now

The rulings this record states were recorded before the code matched them.
`assertJudgeableProducerRoster` still required two judges with no stake in any
candidate,
which is the rule the self-vote discount replaced.
On the six-model roster that capped producers at four and made the widest cases
of these rulings unreachable,
including the roster-width bench that was supposed
to measure them.

Landed in `285af2867`.
The guard now refuses only rosters that could not decide a round however they
voted:

-   producers non-empty and distinct,
    judges distinct;
-   the weight this bench could award ONE candidate,
    counting a producer seat at
    the self-vote weight and every other seat at the full one,
    reaches
    `MIN_SELECTION_WEIGHT`.

The second limb is the one that matters and a seat count would have missed it.
One producer judged by itself and one other model tops out at half a vote plus
one whole one,
which never reaches two,
so every round would decline while
reading as a stage that found nothing worth changing.
That case is now a test.

WHERE SELF-VOTES ALONE CAN CARRY A CANDIDATE,
stated because the arithmetic has
one exception and it should not be discovered in production.
When several models return byte-identical text their candidates collapse into
one,
and every contributor's ballot for it is a self-vote.
Four contributors reach the minimum weight with no outside judge at all.
That is deliberate:
agreement to the byte between independent models is itself
the corroboration.
Three contributors fall short and the incumbent survives.
Both are pinned in `candidate-select.unit.test.ts`.

## Provenance had to be fixed before the discount could work

Found by external review of the guard change and landed in the same session.
Two lanes lost the record of who wrote a candidate exactly when models agreed:
`selectPerEnvelope` kept only the first proposer of identical replacements,
and
the naturalness lane never deduplicated at all.
A model could therefore vote at FULL weight for its own words whenever another
model had written them first,
which is the discount silently not applying on
the slices where the ensemble agreed.
Identical candidates now merge their producers.
