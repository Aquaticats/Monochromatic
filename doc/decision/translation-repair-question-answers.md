# Answers to the translation-repair decision sheet

Superseded in part on 2026-10-25 by
`package/translation-repair/doc/design-commitments-votes-repairs-and-run-offs.md`:
the minimum weight of two a selection needs is gone,
and two answering judges decide whatever their weight (a half-weight self-vote still counts as one judge).

Decided 2026-08-16 by the repository owner,
on the seven questions drafted in
`doc/planning/translation-repair-open-decisions.md`.
That document holds the questions,
the evidence behind each,
and the options with their pros and cons.
This one holds only what was chosen,
what it means in code,
and what it moves.

Two answers carry an explicit ordering instruction rather than only a choice,
and both are recorded with the
decision they attach to:
measure judge quality before widening rosters,
and record who won each slice before the critic stage is
kept on the strength of evidence it does not yet produce.

The owner's answers cite items of the drafting session's task tracker by number;
each quotation keeps its number verbatim and names the item beside it.
Those ids are not GitHub issue numbers,
and the two namespaces collide (`AGENTS.md` rules XNS and XN2).

## Producing roster width: keep three, widen on evidence

Question 1,
answer D,
with "do `#84` first" (the judge-quality bench).

The producing roster stays at three until the judge-quality bench measures judge quality on preserve-or-replace.
Widening happens afterwards and on those numbers,
not before them.

This overrules the drafted ranking,
which put a named constant first on the argument that widening was already
decided and only its size was open.
The owner's reading is that the measurement comes first regardless,
so there is nothing yet to name.

What it means concretely:

-   No `PRODUCERS_PER_ROLE` constant is introduced now.
-   The self-certification weight stays blocked on the judge-quality bench rather than on this question.
-   ONE OF THE WORRIES BEHIND THIS QUESTION IS NOW MEASURED.
    Option C was rejected partly because with every
    model producing,
    "self-votes and self-certifications are the norm rather than the exception,
    which makes
    the discounts load-bearing in a way nothing has measured".
    The self-preference bench recorded under
    question 4 measures exactly that,
    and finds self-preference positive at every width from two to six with
    no trend distinguishable from the run-to-run band.
    It does not say a wider roster decides better,
    which is
    the other half and still open.
-   The bench numbers already gathered stand as the cost side of a decision whose quality side is missing:
    two producers to six multiplies calls by 1.7 and tokens by 1.8.
-   Nothing measured says a wider roster decides worse.
    That remains untested rather than refuted.

## Transcribed images: send the image, fall back to protecting the block

Question 2,
answer B with an A fallback:
"best effort,
fallback to A whenever an image's OCR doesn't make sense".

Supply the image to the translators and judges so the transcribed text has a source that can be CHECKED.
Where the reading does not make sense,
protect the block structurally instead:
keep it out of translation and
splice it back unchanged.

This is the only answer on the sheet that asks for a capability the pipeline does not have,
and it carried a
prerequisite the sheet named as a con:
the provider roster must actually hold models that read images.

### The prerequisite holds, checked against the provider

Settled 2026-08-16 by asking rather than assuming.
`GET https://api.synthetic.new/openai/v1/models` reports
`input_modalities` per model,
and two of the six already in the production roster read images:

    hf:moonshotai/Kimi-K3                               ["text", "image"]
    hf:Qwen/Qwen3.6-27B                                 ["text", "image"]
    hf:zai-org/GLM-5.2                                  ["text"]
    hf:zai-org/GLM-4.7-Flash                            ["text"]
    hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4   ["text"]
    hf:openai/gpt-oss-120b                              ["text"]

The provider lists ten entries and the only other vision ones,
`syn:large:vision` and `syn:small:vision`,
are
aliases of those same two models.
So the vision sub-roster is exactly two,
and widening it needs a different
provider rather than a different configuration.

TWO IS THIN,
and that is what this answer has to be built around.
Selection needs a minimum weight of two,
a
producer's ballot for its own work counts half,
and the judge-quality bench measured that on slices carrying any archive
imperfection the effective roster is already about four of six.
Where both vision models produce,
no
disinterested judge remains at all,
which is exactly the `no-disinterested-ballots` case the self-preference
instrument names.
Expect the A fallback to carry more of this than the answer's wording implies.

WORTH CONTRASTING WITH QUESTION 4,
whose answer turned out to rest on a premise that does not hold.
This one
holds.

What it means concretely:

-   `SyntheticModelInfo` has no modality field,
    so nothing in code can express which models read images.
    Adding one is part of the work rather than a blocker.
-   The image path is already in the markdown,
    so locating the asset is path resolution rather than a new
    corpus reader.
-   "Does not make sense" needs a stated rule rather than a judgement call,
    since it decides which of two
    behaviours a block gets.
    The population is small and enumerated:
    8 target-only blocks over 6 entries,
    plus
    one merged case,
    `shihai4h`,
    at 102 source characters against 1665.
-   The paired-quote ratio guard from option A is still needed for `shihai4h`,
    because no target-only rule
    will ever see it:
    it is aligned as an ordinary pair.

## Critics: keep them as evidence, remove every early return

Question 3,
answer B,
with "land `#83`" (recording who won each slice).

Critics stay in the translate path,
supplying named defects to the judges rather than deciding anything.
Every early return is removed,
so a critic can no longer block a document or return the original target and
discard translated slices that already succeeded.

The sheet's own con against B was that the numbers justifying the spend do not exist until selection records who won each slice.
The instruction to land that record answers it directly:
produce the numbers,
then keep paying with the evidence
in hand.

What it means concretely:

-   The critic-stage question is decided rather than open.
-   `repairChunk` must stop returning its input unchanged when non-translation votes stand,
    and the
    document-level dominance check must stop returning the whole original target.
-   Settling a document with no critics is confirmed as a defect on this answer.
    An empty critic roster is a misconfiguration here,
    not an
    intended configuration,
    because critics are kept.
    The guard can read an empty list.
-   Recording who won is blocked by driving the translate lane from the document driver,
    so landing it means landing the driver's outcome and cache first.

## Self-certifying checkers: a half, matching selection

Question 4,
answer A.

A checker's verdict on text it helped write counts for half,
the same discount selection already applies.

The sheet was explicit that consistency is the only argument for this number,
and that the arithmetic which
justifies a half in selection does not transfer:
selection needs weight 2 to carry a candidate,
so half-weight
self-votes cannot,
whereas resolution checking tallies verdicts about one claim and nothing in the arithmetic
picks a number.
The choice is made on consistency with that understood.

What it means concretely:

-   The self-certification weight is a half WHEN there is any self-certification to weigh,
    which today
    there is not.
    See "The half cannot take effect yet" before writing code for this.
-   The number was a stated preference rather than a derived one when it was chosen.
    It has since been
    measured,
    under "Measured after the fact,
    twice".

### The half cannot take effect yet, and that is not a defect

Checked in the source 2026-08-16,
after this decision was recorded as though it were immediately
implementable.
It is not,
and writing the weight now would be unreachable code.

`assertCheckerIndependence` in `repair-contract.ts` THROWS when any checker id also appears among the editors
or refiners.
A roster where a model could certify its own text is refused before any work happens,
so no
checker is ever a self-certifier and a half-weight branch would never run.

Relaxing that assertion is the "widen the producing roles" half of the self-certification work,
and question 1 answer D blocks
roster widening until the judge-quality bench reports.
So the ordering is forced:
the bench,
then the roles widen,
then the half
becomes reachable and is applied.
Weighing has to replace forbidding in one change,
not before it.

WHAT THE HALF STILL BUYS BY BEING DECIDED NOW:
whoever relaxes the assertion does not also have to pick a
number,
and the self-preference measurement in "Measured after the fact,
twice" says the number is about right.

### Measured after the fact, twice, and it supports the half at a smaller size than first reported

Run 2026-08-16 on the roster bench using the paired comparison in `src/self-preference.ts`:
each candidate
scored against itself,
its producer's own ballot against the rate among judges holding no stake in that same
text.
Two draws were taken,
and they share no slices,
so they are independent replications rather than one
sample.

    4 slices,  24 rounds   own 39 of 93   (0.42)   others 82 of 376   (0.22)   excess 0.20
    14 slices, 84 rounds   own 109 of 310 (0.35)   others 300 of 1361 (0.22)   excess 0.13

    pooled, 18 slices      own 148 of 403 (0.367)  others 382 of 1737 (0.220)  excess 0.147

THE SIGN REPLICATED AND THE SIZE FELL.
Take 0.147 as the estimate:
producers name their own candidate about
1.67 times as often as judges with no stake in it name the same text.
The first figure reported,
0.20 from
four slices,
was a small-sample reading and is superseded.
Note which half moved:
the disinterested rate is
identical across both draws at 0.22,
and the own rate fell from 0.42 to 0.35.

WHAT THAT SAYS ABOUT THE HALF.
The discount divides a self-vote by 2 against a measured lift of 1.67,
so it
is the right order and somewhat MORE aggressive than the effect it corrects.
Scaling a weight and describing
a rate are still different operations,
so this does not derive the number;
it bounds it.
Nothing here argues
for changing it,
and anything that later does should beat a 2x correction of a 1.67x effect on better
evidence than a bench.

STILL NO TREND WITH WIDTH,
on the wider draw either:

    width 2 pass 1   0.123       width 4 pass 2   0.182
    width 3 pass 1   0.241       width 5 pass 1   0.125
    width 4 pass 1   0.083       width 6 pass 1   0.093

The bench runs one width twice on purpose,
and width 4 came back 0.083 then 0.182.
That 0.099 spread is the
run-to-run band,
the variation between widths is not monotone,
and most of it sits inside the band.
Six runs
agree on the sign;
they say nothing about the ordering.

WHAT THIS IS NOT:
a census.
Eighteen slices at six widths means the opportunities are repeated measures over
eighteen texts.
The instrument is cheap to run wider,
and the second draw already showed why that matters.

## Replacement rate: widen the judged window and re-read it

Question 5,
answer E.

Widen the judged window on slices where the archive's layout differs from the original's,
then re-read the
replacement rate.
This separates "the archive is worse" from "the archive is laid out differently",
which is the question under
the 73 percent replacement figure.

### The rate this question is about is worse than the question says

Measured 2026-08-16 as a by-product of the self-preference bench,
over 18 slices at six roster widths,
108
rounds.
Every slice in the draw already had an archive translation.

    width 2   replaced 0.83        width 5   replaced 0.94
    width 3   replaced 0.83        width 6   replaced 0.94
    width 4   replaced 0.83

    the archive's English survived at least once on 10 of the 18 slices, and never on the other 8

The figure this question was written around is about 0.73.
On this draw it is 0.83 to 0.94.
THE TWO ARE NOT
DIRECTLY COMPARABLE:
that bench drew ten slices,
this one draws eighteen stratified by source size with an
incumbent on every one,
and the roster width varies here.
Same direction,
larger,
on more texts.

NO WIDTH TREND IS CLAIMABLE from the apparent 0.83 to 0.94 step.
The repeated width returned an identical
3 of 18 on both passes,
which looks like a band of zero and is not one:
at 18 rounds per width the sampling
error alone is around 0.09,
so two passes landing on the same integer is partly luck,
and the 0.11 step at
width 5 sits inside that.
Do not read the widening as causing more replacement.

WHAT THIS DOES NOT CHANGE:
the answer.
Option E measures the window before anything decides what to do about
the rate,
and a higher rate is a stronger reason to know how much of it is layout rather than quality.

What it means concretely:

-   The window trial is authorized,
    and its cost is the repriced one:
    three arms rather than two,
    with the candidate
    slate produced once and judged repeatedly,
    roughly 1760 exchanges over the flagged slices plus matched
    unflagged controls.
-   Splitting `runTranslateStage` into producing and judging halves is a prerequisite rather than a
    detour,
    because a two-arm comparison resamples the candidates and cannot measure the window.
-   None of A to D is chosen by this.
    Option E was ranked first precisely because it runs before that choice
    and changes what the choice is made on.

## Thin-roster slices: cache anything examined at all

Question 6,
answer B.

The rule as it already stands:
zero voices is not cached,
one voice is.

What it means concretely:

-   No code change,
    no schema field,
    no cache version bump.
    The 150 slices already on disk stay valid.
-   A slice decided by a thin roster resumes indistinguishably from one decided at full strength unless a
    reader goes looking in `findings`.
    That is accepted,
    on a measured population of 34 slices in 7 entries,
    all in the lane where thinness means no improvement was attempted rather than nothing inspected.

## Non-translation denominator: delegated

Question 7,
delegated.

Taking the drafted delegation answer:
option B now,
option C held until the artifact can carry an unexaminable
verdict.
That ordering costs nothing,
because C keeps B's denominator.

What it means concretely:

-   The ratio keeps reading over the SLICES,
    which is what the code already does,
    and the contract text stops
    calling the denominator the document's characters.
    This is documentation of existing behaviour,
    not a
    behavioural change.
-   `XIEPT2`,
    which produces no slices at all and settles as a clean unchanged document having examined
    nothing,
    keeps doing so for now.
    Reporting it as unexaminable is option C,
    and it waits on that artifact change.
-   Revisit when the artifact can carry that verdict.

## What the answers move

Unblocked and authorized:
the critic-stage question decided,
the window trial authorized behind the producing and judging split,
the self-certification weight settled,
the non-translation denominator settled as documentation.

Ordering the owner set:
the judge-quality bench before any roster widening;
the document-driven translate lane,
then recording who won each slice,
before the critic spend is
justified by its own numbers.

Still blocked on measurement rather than on a decision:
roster width,
which waits on the judge-quality bench.
