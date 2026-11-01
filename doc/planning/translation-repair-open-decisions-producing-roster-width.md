# Translation repair open decisions: how wide the producing rosters should be (question 1)

Question 1 with its roster-width measurements,
options and ranking,
the 2026-08-23 measurement that width does not decide the outcome,
and the owner's ruling that keeps three producers and sends checker width to measurement.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

## Question 1: how wide should the producing rosters be

BLOCKS the roster half of `#91`,
and through it the first long run under the
new shape.

The GUARD half of `#91` no longer waits on you and landed overnight in
`285af2867`:
the code still forbade what you had already ruled,
so a roster
where every model produces threw before any model was asked.
Details in the
"decisions I took without you" section.
What is still yours is how many
producers to seat by default.

You ruled "All producing roles to 4" and then "don't hardcode magic numbers like
4 or 6".
I tried to derive the four from the roster size and got it wrong twice in one
day,
both times by the same mistake,
so I stopped deriving and am asking.

WHAT IS NOT A CONSTRAINT,
verified in the code rather than assumed:
selection imposes no ceiling at all.
The half-weight discount applies to a judge's ballot for its OWN candidate,
so a
candidate's full-weight judges are everyone who did not write it.
Four editors judging each other work fine;
so would six.

WHAT ACTUALLY MOVES with width:

-   COST,
    now measured rather than reasoned about,
    in
    `doc/audit/translation-repair-lane-budget.md`.
    Per slice:
    width 2 costs 7.2
    calls and 22118 tokens,
    width 4 costs 10.2 calls and 34567 tokens,
    width 6
    costs 12.4 calls and 40294 tokens.
    Over the whole corpus that is 27.9M,
    43.6M and 50.8M tokens for one pass.
    So the widest roster costs about 1.8
    times the narrowest,
    not six times,
    and the reason is where those tokens go.
    MEASURED PER STAGE on 2026-08-15 from the same rows,
    at no new cost,
    and it
    corrects what this bullet said yesterday:
    the judge round takes 5.4 calls
    per slice at EVERY width from three up,
    because the judge roster does not
    widen when the producing one does,
    but its tokens still rise 58% from width
    2 to width 6.
    Which half of a ballot grows is not knowable from those rows,
    which carry one total per exchange:
    the prompt repeats every candidate to
    every judge,
    and the answer may carry a verdict per candidate.
    The judge
    round dominates at both ends either way,
    60% of a slice's tokens at width 2
    and 52% at width 6.
    So widening buys more candidates at a discount rather than for free,
    and a
    cheaper decision procedure would save more than a narrower roster.
-   AGREEMENT.
    Ballots spread thinner across more candidates,
    so the leader more
    often falls short of the minimum weight or ties.
    Both outcomes decline,
    and
    a decline keeps the incumbent,
    so widening can quietly REDUCE how often
    anything is replaced.
-   COVERAGE,
    which is what the widening is for,
    on your reasoning that these
    models have different blind spots.

### Measured overnight, so this is no longer a guess

`mise run //package/module/translation-repair:slice-census` (moved on 2026-10-18 to
`//package/translation-repair/slice-cache:slice-census`) reads the pinned
corpus,
slices it exactly as the pipeline does,
and spends no quota.
First run,
92 complete pairs:

    1260 slices
    slice source chars   p50 101, p90 174, p99 374, max 1313
    slice target chars   p50 299, p90 486, p99 1512, max 10959
    slices over 4641 target chars   1 of 1260

COST,
which is now arithmetic rather than an adjective.
Translate calls are
slices times producers,
so the whole corpus costs:

    3 producers   3780 translate calls
    4 producers   5040
    6 producers   7560

Selection is 1260 rounds whatever the producer count,
one call per judge,
so
7560 judge calls at the current six-model judge roster.
What producer count changes there is the SIZE of each judge prompt,
since every
candidate is repeated to every judge:
about 101 source characters plus one
candidate of about 299 per producer,
plus the incumbent.

    3 producers   about 1300 characters per judge prompt
    4 producers   about 1600
    6 producers   about 2200

So widening from three producers to six doubles the translate calls and adds
about seventy percent to each judge prompt.
Neither is a cliff,
and neither is
free.

THAT SEVENTY PERCENT WAS AN OVER-ESTIMATE,
and the bench now says by how much.
Measured per stage on 2026-08-15:
a ballot costs 2794 tokens at width 3 and 3861
at width 6,
which is 38% rather than 69%.
The character arithmetic counted only
the part of the prompt that grows,
and a ballot also carries the policy,
the
source and the incumbent,
none of which widen with the roster.
Use 38% when
pricing a width,
and keep the arithmetic for the shape of the effect rather than
its size.

THE TAIL IS ONE ENTRY.
Exactly one slice of 1260 exceeds 4641 target
characters,
which is the size the translate probe already watched time out at
six minutes and return schema-invalid output.
It is in `shihai4h`,
at 10959
characters,
and that entry also owns the largest target-only block count.
So
the oversized-call risk is a single entry rather than a distribution problem,
and it can be handled without changing the roster at all.

THE AGREEMENT LIMB IS BEING MEASURED NOW,
and it is the half of this question
the corpus cannot answer:
whether more candidates make the judges converge less.
`mise run //package/module/translation-repair:roster-bench` (moved on 2026-10-18 to
`//package/translation-repair/bench:roster-bench`) runs the same ten
stratified slices through the translate lane at every width from two to the
whole roster,
interleaved so each width meets the same provider weather,
with
one width run twice for a run-to-run band.
Results are folded in below as they
land.

THE FIRST THING IT SHOWED IS THE BAND ITSELF,
and it is wide.
On one slice,
at
one width,
the two passes disagreed about the outcome:
one judged and replaced
the archive text at weight 4.5,
the other declined for indecision and kept it.
Same slice,
same width,
same roster,
minutes apart.
Any difference between
widths smaller than that is noise,
which is exactly why the repeat was built in
before the sweep rather than after.

THE SWEEP IS FINISHED,
60 rows,
and it lands inside that band.
Per width,
over
the same ten slices:

    width 2   kept 3   declined 0   72 calls    221k tokens    70s per slice
    width 3   kept 2   declined 1   90 calls    300k tokens    91s
    width 4   kept 3   declined 2  102 calls    344k tokens    88s
    width 4   kept 4   declined 3  101 calls    347k tokens    97s   (repeat)
    width 5   kept 2   declined 1  113 calls    417k tokens   111s
    width 6   kept 2   declined 1  124 calls    403k tokens    93s

THE TWO WIDTH-FOUR PASSES DIFFER BY AS MUCH AS THE WIDTHS DO.
They disagree
about the outcome on three of the ten slices,
and their kept counts,
3 and 4,
span the whole range every other width falls in.
So the honest reading is that
WIDTH DOES NOT MEASURABLY CHANGE AGREEMENT on ten slices,
not that some width
won.
Ten slices cannot resolve a difference smaller than the noise,
and this
noise is large.

The agreement worry that motivated the question is not supported either:
the
widest roster declined once,
the narrowest declined never,
and nothing in
between trends.
Ballots spreading thinner across more candidates did not produce
more declines here.

WHAT THE SWEEP DOES SETTLE IS COST,
which now has numbers rather than an
estimate:
going from two producers to six multiplies calls by 1.7 and tokens by
1.8,
and the whole bench spent 602 calls,
2.03 million tokens and 1.5 hours of
wall time on ten slices.

So this question is now a coverage-versus-cost decision and not an agreement
decision.
Nothing measured says a wider roster decides worse;
what it costs is
`1.8x` at six.

WHAT THIS BENCH DOES NOT MEASURE:
every slice it drew already has a translation,
94 to 302 characters so far.
The numbers therefore describe preserve-or-replace
only.
Filling a GAP,
a slice whose incumbent is blank,
is the case the new shape
exists for,
and no row here covers it:
the draw is stratified by source size over
slices the aligner paired,
and one-sided sections are not sliced at all,
which is
`#90`.
Nothing in this question needs a redraw to answer,
but no answer here
transfers to the gap case.

### Options

A.
 Fixed count in run configuration,
named and commented,
e.g.
    `PRODUCERS_PER_ROLE = 4`.
    Pros:
says exactly what it means;
one line to change when the provider
    changes;
no false derivation.
    Cons:
it is the literal you told me not to write,
and it silently means
    "all but two" at six models and "half" at eight.

B.
 A share of the roster,
e.g. two thirds,
rounded down.
    Pros:
tracks the provider with nothing edited,
which is what you asked for.
    Cons:
the fraction is as arbitrary as the count,
and rounding makes it
    jump at odd sizes.

C.
 Every model produces in every role,
and the discounts carry the whole load.
    Pros:
no number at all;
maximum coverage;
the simplest rule to state.
    Cons:
most candidates per slice,
so the largest judge prompts and the
    thinnest ballots;
and with every model a producer,
self-votes and
    self-certifications are the norm rather than the exception,
which makes
    the discounts load-bearing in a way nothing has measured.

D.
 Keep the current three until `#84` measures judge quality,
then widen on
    evidence.
    Pros:
the only option that spends nothing before the measurement that says
    whether wider slates help;
leaves every other decision intact.
    Cons:
it is what you already overruled,
and it delays the coverage you
    widened for.

RANKING:
A > D > B > C.

A over D because you have already decided to widen and a named constant is the
honest way to write "four" while you decide whether four is right;
D would be
re-litigating a decision you made.
D over B because a fraction invents precision nobody has:
two thirds of six is
four only by coincidence,
and at seven models it silently becomes four again.
B over C because C makes both discounts load-bearing on every slice at once,
and the self-preference rate they compensate for is exactly what `#84` has not
measured yet.

D,
do #84 first

### MEASURED 2026-08-23: the width does not decide it

This question was gated on a number nobody had.
There is one now,
from two disjoint draws of twenty
slices each over the same forty-slice sample,
thirty-one of which carried work.
The instrument,
its three fixed defects and its positive control are described in
`doc/handover/translation-repair-history.md`.

Widening the editors from three to six changed what ships NO MORE OFTEN than re-running the same
three did.
The comparison is paired,
one bit per slice from changing the width and one from changing
nothing,
so only slices where the two disagree carry information:

-   moved WITHOUT churning,
    which is width doing something:
    1 of 31
-   churned without moving,
    which is the lane alone:
    2 of 31

Quality was asked separately,
by putting both arms' repairs on one slate before the fixed panel in
both seating orders.
Of 23 contests,
7 were decided by SEAT rather than text.
The 16 that survived
the swap split 10 wide to 6 narrow,
which arrives about one time in four under no true difference.
Draw A alone had read 5 to 1 and looked like something;
draw B read 5 to 5.
The held-back half is the
only reason that is known rather than believed.

TWO FACTS NOW BEAR ON THE "ALL PRODUCING ROLES TO 4" RULING that were not available when it was made,
and the ruling is yours to keep or revise:

1.  Between three and six,
    widening buys nothing measurable on either reading.
2.  FOUR IS NOT REACHABLE as the rosters stand.
    `RUN_MODELS` partitions the six-model roster exactly,
    three editors who are also the three refiners plus the disjoint other three as checkers,
    because
    `assertCheckerIndependence` forbids a checker from being a writer.
    Four writers leaves at most two
    checkers,
    and nothing refuses that:
    the only floor,
    `assertRostersConfigured`,
    refuses an EMPTY
    role and nothing more.
    At two checkers,
    one `fixed` against one `not-fixed` resolves nothing,
    since `tallyResolutionChecks` needs `fixed > (notFixed + worse)`.

So implementing the ruling costs checker independence and buys no measured quality.
RECOMMENDATION:
stay at three,
and leave `assertCheckerIndependence` alone.
That also leaves the checker-side self-vote discount unreachable,
which is the standing state
recorded in `doc/decision/translation-repair-question-answers.md`,
and closes the applying task
unbuilt rather than shipping a relaxation nothing needs.

### RULED 2026-08-23: three producers, and the checker width goes to measurement

The owner read both facts and REVISED the "all producing roles to 4" ruling.
The producing roles
STAY AT THREE.
`#186` measured that exact comparison and found nothing,
and widening is not free
here because it forces a checker change nothing has measured.

The checker roster was ruled separately,
and NOT settled by opinion.
`assertCheckerIndependence`
relaxes behind a run-configuration switch,
the same entries run once with the disjoint three and
once with all six,
per-issue resolution is compared,
and whichever wins ships while the loser is
deleted.
The reasoning is the standing instruction to prototype and measure where the answer is
unknown,
and this one is unknown:
no measurement anywhere covers checker width,
only producer width.

WHY THIS IS NOT THE SAME QUESTION `#186` ANSWERED,
which is the misreading to guard against.
`#186`
compared three writers against six writers and found the ships indistinguishable.
Checker width asks
something else:
whether six ballots per issue,
three of them discounted for having written the text
under review,
resolve better than three undiscounted ballots from models that wrote nothing.
A null
on producers says nothing about that,
because the two arms differ in ballot COUNT rather than in who
produced the candidate.

THE DISCOUNT IS ALREADY BUILT AND ALREADY LIVE ELSEWHERE,
so this measurement adds no new weighting
mechanism.
`tally-resolution.ts:310` picks the weight PER ISSUE through `wroteTextForIssue`,
so a
checker is halved only on issues whose shipped text it helped write and keeps full weight on the
rest.
`candidate-select.ts:276` runs the same overlap-plus-discount pattern on every chunk today.
The checker stage is the only place in the pipeline that excludes rather than discounts.

The arithmetic that makes the measurement worth running:
at all six,
an issue written by the three
writers faces three full independent votes against three halved self-votes,
three against one and a
half,
so independents stay dominant while the panel doubles.

A SIDE FINDING WORTH MORE THAN THE ANSWER:
the panel's ability to separate two serious repairs of one
passage is not stable between runs.
Draw A decided 6 of 12 contests by seat;
draw B decided 1 of 11
that way,
hours apart on the same models and the same sheet.
Any later measurement resting on this
panel resolving fine differences should measure that decisiveness first rather than assume it.
