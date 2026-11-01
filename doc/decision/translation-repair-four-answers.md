# Four translation-repair decisions, taken on the best-quality guideline, 2026-08-16

Superseded in part on 2026-10-25 by
`package/translation-repair/doc/design-commitments-votes-repairs-and-run-offs.md`:
`MIN_SELECTION_WEIGHT` and the weights it was compared with are gone,
and two answering judges decide whatever their weight.

## How these were decided, and why an earlier version of this file is wrong

These four were first put to the owner as option sets.
The owner answered,
then withdrew the answers:
they had only half-understood the options,
and said the questions should not have been asked,
because
with "everything for the best quality" as the standing guideline the agent could have picked itself.

That criticism is correct,
and the reason is worth recording rather than apologising for.
THREE OF
THE FOUR OPTION SETS WERE PRICED ON COST.
They ranked a sample over a census,
a cheap landing over a
foundational fix,
and one trial arm against another,
and in each case the cheaper option won its
place in the ranking on the strength of what it saved.
Under a standing instruction that quality is
the objective and quota is not a constraint,
those rankings collapse:
the saving is not a benefit,
so the option that buys more evidence or better output simply wins.
A question whose options differ
only in cost is not a question for the owner under this guideline.

WHAT REMAINS A REAL QUESTION under this guideline:
anything where two paths differ in what they
VALUE rather than what they cost,
and anything that risks the owner's own data or reputation.
None
of these four were that.

So these are decided here,
on quality,
and TWO OF THEM DIFFER FROM WHAT THE OWNER CLICKED.
Both
differences are called out where they occur,
so a veto is cheap and informed.

## The rendering audit runs standalone over every settled artifact

DIFFERS FROM THE CLICKED ANSWER,
which was a sample.

A sample was ranked first only because it is "the cheapest way to get a RATE rather than a verdict".
Remove cost from that sentence and nothing is left of the argument:
a census over all settled
artifacts gives the same rate with no sampling error,
AND answers questions about particular entries,
which a sample explicitly cannot.

WHY NOT INSIDE THE LANE,
per slice,
which is the option that would catch a defect before anything
downstream reads it.
Two quality arguments,
not cost ones:

-   The instrument's production error rate is unmeasured.
    Its own control arms exposed a matcher that
    missed a unanimous defect,
    which is why it now reports two tiers.
    An instrument that has already
    been wrong about its own fixture does not belong inside the path that produces the output until
    its error rate is known from beside it.
-   Re-runnability is itself a quality property here.
    This instrument changed twice in one week,
    and
    a standalone reader can be re-run against artifacts a pass already wrote.
    Auditing inside the lane
    means every change to the instrument costs a full corpus pass before it can be believed.

So:
standalone,
over every settled version 2 artifact,
and revisit siting once its production error
rate has been measured that way.

## Declines get one retry AND the voice loss gets fixed

The clicked answer was one retry.
That was one of two options I offered as alternatives,
and offering
them as alternatives was the error:
they address different things and quality wants both.

RETRY ONCE,
because the ledger says it works:
retrying identically configured arms resolved 21 of 37
declines,
and only 8 slices of 109 declined under both judgings.

AND FIX THE VOICE LOSS,
because the retry treats the symptom.
The decline rate is 0.171 overall but
0.063 at a full panel against 0.287 at five voices and 0.692 at four,
so a decline is mostly a panel
that lost voices rather than a passage nobody could fill.
Separately,
135 of 327 judgings lost at
least one voice of six,
a rate of 0.413,
dominated by `hf:zai-org/GLM-5.2` being abandoned 60000 ms
after quorum.
The mechanism is in `candidate-select-model.ts`,
where `MIN_SELECTION_WEIGHT` is 2 and
a self-vote counts a half.

The stated objection to fixing it was that changing the roster or the deadline breaks comparability
with everything measured under the current configuration.
Under the best-quality guideline that
objection inverts:
comparability with a configuration that loses a voice in two judgings of five is
not worth preserving,
because the configuration is the defect.
Preserve the MEASUREMENTS as a record
of what the old configuration did,
and change the configuration.

SEQUENCE,
which matters and is the one thing cost-like reasoning gets right here:
the two-lane cost
run in flight is measuring the CURRENT configuration.
Let it finish,
so it reports that
configuration honestly,
then widen the deadline or seat a replacement,
then re-measure.

### The disposition name for what still declines

`no-candidate-backed`.
It names what the code observed,
that no candidate reached
`MIN_SELECTION_WEIGHT`,
and does not claim the passage is unfillable,
which the panel-width figures
contradict.
`panel-declined` was rejected for attributing a choice to a panel that mostly just lost
voices;
`unfillable` was rejected as the reading the measurements refute.

This is a reason rather than a new delivery kind,
which is verified rather than assumed:
`slice-delivery.ts:229` and `:289` both return `gap-remains` when `incumbentKind` is `absent` and
`incumbent-retained` otherwise,
and the comment at `slice-delivery.ts:226` states the rule outright,
that an unreached slice and an unheard one "both leave the incumbent standing wherever there is one,
and leave the gap wherever there is not".
A declining slate is an unheard slice by that definition.

The name appears in no source file and in no entry of `forbidden-strings.append.local.txt`,
and reads
in the same shape as the neighbouring reason values `assembly-integrity` and
`blocked-non-translation`.

## Insertion: fix the aligner first, then land section scale only

DIFFERS FROM THE CLICKED ANSWER,
which parked both landings.
Parking both is right about one of them
and gives up on the other.

The two landings are not one decision,
and presenting them as one option set was the second error.

### Paragraph scale is refused, permanently as far as the evidence goes

Across both runs of `mikaela_khara`,
ninety-six answers and NOT ONE VOTE FOR ABSENCE,
so at most one
candidate of twenty-two and plausibly none is a genuine omission.
The block census attributes the
rest to merges,
one entry rendering four consecutive source lines as a single English block,
and to
mispairings at the low end of the length signal.
Inserting there would duplicate content that is
already present in a different shape.
This is the landing the owner's park was right about.

### Section scale is real and gets landed, after the aligner

The section-scale absences are genuine and were confirmed against hand checking:
`XIEPT2` absent
eight times,
`XingZ60` sections 13 and 14 absent,
section 12 carried,
eleven of eleven correct,
on
which my own labels had been wrong nine times.
Leaving genuinely untranslated sections untranslated
is a quality loss in the product,
which is exactly what this guideline weighs highest.

Two things gate it,
and neither is a reason to abandon it:

-   THE HEADING DEFECT,
    understood and small.
    When a source section's counterpart is a heading with
    no body,
    the body belongs UNDER that heading.
    Landing five as designed inserts the whole source
    section,
    heading included,
    which would have produced eight duplicate headings on `XIEPT2`.
-   THE ALIGNER,
    which is the foundational one and comes FIRST.
    85 of 92 entries never reach the
    matcher at all,
    because equal section counts skip it (the equal-count fast path),
    and the scorer is broken with
    asymmetric preambles causing most fallbacks.
    Every section-scale number this section cites was drawn
    from the 7 entries that do reach it.
    Landing insertion on top of an aligner that never runs for
    92 percent of the corpus would ship a feature whose input is decided by a fallback.

ORDER:
the aligner's scoring and the equal-count skip first,
then re-run the section census over the population that then reaches
the matcher,
then land section-scale insertion with the heading defect fixed.

### What this means for the coverage measurement

Option A of `doc/planning/coverage-wire-rerun-trade.md` STAYS UNRUN,
and for the reason the earlier
version of this file gave:
it exists to say how much of the BLOCK-scale null to believe,
and block
scale is refused,
so it informs no decision that remains open.
Section scale does not need it,
since
the instrument was eleven of eleven correct there and its false-CARRIED weakness suppresses
insertion rather than causing it.

One piece survives on its own account,
being a durability fix rather than a measurement:
the coverage
probe writes with `console.log(JSON.stringify(...))` and nothing persists it,
which is why its
2026-08-16 numbers at both scales exist nowhere but a session transcript.

## The window trial is dropped

The clicked answer,
and it survives the re-decision unchanged,
because this one was NOT priced on
cost.

A fourth sham-context arm would distinguish "no effect" from "an effect this design cannot resolve".
That is a question about a MEASUREMENT,
not about the product:
the action it informs,
do not widen
the judge's context,
is the same under either answer.
Best quality of the output is not served by
resolving it,
so it is dropped on its merits rather than on its price.

Over the 109 slices read,
the wide arm decided differently from a narrow arm 18 times while the two
IDENTICALLY configured narrow arms differed 21 times,
and the exclusion rule producing that reading
was fixed in writing before any numbers existed.

WHAT DOES NOT LAPSE WITH THE TRIAL:
the negative control measured the per-slice preserve-or-replace
decision as about 19 percent unstable between identical runs.
Rates over many slices survive that;
per-slice claims and small comparisons do not.
