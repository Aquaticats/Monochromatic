# The rendering audit over every settled version 2 artifact

Task `#115`.
The population was measured before a single model was asked anything,
from the four archived version 2 artifacts at `~/translation-repair-v2-archive/`
and the corpus clone at `~/one-among-us/data`.
The results were added after each run landed,
under rules this document fixed in advance.

## What this answers, and what it does not

IT DOES NOT ANSWER the question it was opened to ask,
and it fails for TWO INDEPENDENT REASONS,
either of which would be enough on its own.

CONFOUNDED WITH THE ENTRY.
Every corroborated archive defect sits in one of the two entries
and the single corroborated fresh defect sits in the other,
so neither entry shows the pattern the pooled table shows.
More slices cannot separate them.
Only more entries can,
which means more settled version 2 artifacts,
which is what task `#108` would produce.

AND WIDER THAN THE EFFECT, measured rather than feared.
The population was audited THREE times.
Over 40 digest-verified pairs of identical text,
the corroborated total moved from four to ten,
ten of the forty subjects flipped between claiming something and claiming nothing,
and across the three readings the archive corroborated count ran 5, 3, 7
while the gap being measured ran 4, 2, 4.
The rule fixed in advance puts a move of about four inside the noise;
this is six.

IT DOES ANSWER several things worth having:

-   The instrument runs on real lane output at all,
    over 40 subjects, at zero cost to the producing path.
-   Both `#107` relocation candidates found in the first run anchor against
    their shared slice boundary,
    which is the geometry `#107` predicts,
    so the mechanism is corroborated even though neither pair is proved to be
    one move,
    and the second run found no candidates at all.
-   The order-of-magnitude gap `#68` measured over the introduced-defect probe
    appears in NEITHER run here,
    so that finding still looks specific to the probe task rather than to a model.
    The SIZE of the gap at this task is not measurable from these runs:
    two runs disagree about it.
-   `claimed` counts CLAIMS and not distinct problems,
    because one voice can anchor two categories to one span,
    and it did.
-   Voice loss is not spread evenly across the population.
-   A band measured on the six texts one run audits twice is NARROW,
    and it is narrow because those six are the easy ones.
    It disagrees with the population it was drawn from.

NOTHING HERE MAY GATE WHAT SHIPS.
The instrument's own production error rate is unmeasured;
`#66` is the false-negative half and is open.
This is telemetry.

## Two things are called a run here, so this document calls them apart

A SETTLEMENT is one pass of the production pipeline over an entry,
which is what produced an artifact.
Two of them exist per entry,
in the archive subdirectories `two-lane-cost-2026-08-16`
and `grace-remeasure-2026-08-17`.
Nothing in this task re-ran one.

An AUDIT RUN is one pass of this telemetry probe over all 40 subjects.
Three of them exist,
numbered 1, 2 and 3 throughout,
and they differ only in when they were bought.

Both were called "runs" in earlier drafts,
which is the `#99` defect in prose:
one word carrying two meanings that a reader has to infer from context.
Where this document says "run" without a qualifier it means an audit run.

## It is 40 subjects, and "every artifact" costs no more than one settlement did

The archive holds four artifacts:
two entries settled twice,
under `two-lane-cost-2026-08-16` and `grace-remeasure-2026-08-17`.

```text
                                          slices   decided   retained   replaced
two-lane-cost-2026-08-16/Aniloviraw            5         5          1          4
two-lane-cost-2026-08-16/zheermao101          15        15          6          9
grace-remeasure-2026-08-17/Aniloviraw          5         5          2          3
grace-remeasure-2026-08-17/zheermao101        15        15          7          8
                                              --        --         --         --
                                              40        40         16         24
```

EVERY delivery row in the translate lane is `decided`.
There is no `not-evaluated` row and no withdrawal anywhere in the archive,
so the audit population is the whole delivery ledger rather than a subset of it,
and the earlier estimate of roughly 40 audits was right by accident:
it was reasoning about one settlement per entry,
and 40 is all four artifacts.

## 16 of the 40 audit the archive, not a fresh rendering

`incumbent-retained` means the judges preferred the wording the archive already had,
so the document carries it unchanged.
Auditing that slice audits a human translation from years ago.
`replacement-shipped` means the document carries something the lane produced.

The instrument was built for output with no BEFORE text,
which is the second case.
Reading both in one denominator would blur the first real measurement it produces,
so `auditsArchiveText` is recorded per subject and the two are reported apart.

The invariant that makes this free was checked on all 40:
`outcome.acceptedText` equals `shippedText` everywhere,
`shippedText` equals `incumbentText` on every retained slice,
and differs from it on every replaced one.

## The two settlements are directly comparable, slice for slice

Both entries produced the SAME preparation identity under both builds,
at the same corpus commit,
under different pipeline digests:

```text
Aniloviraw     sha256-preparation-v1:d8f64be98e6143dee73f36   5 slices
zheermao101    sha256-preparation-v1:100b954149dcaf28a05a1f  15 slices
corpus         a41fc607     digests  3e6902fb  and  8a63a9cd
```

Same slices, same source text, two independent settlements.
That buys two readings nothing was paid for:

-   SIX SUBJECTS ARE CHARACTER-IDENTICAL ACROSS BOTH SETTLEMENTS
    (`zheermao101` slices 0, 2, 7, 8, 9, 11),
    so the instrument can be asked the same question twice
    and its own audit-to-audit band read off real text rather than fixtures.
    Five of those six are retained-in-both, identical by construction.
    The sixth, slice 9, is `replacement-shipped` in BOTH settlements with identical text:
    two independent productions converged on the same characters.
-   SIX OF TWENTY SLICES CHANGED THEIR DECISION between the settlements
    (`Aniloviraw` 1, 2, 3; `zheermao101` 3, 6, 13).
    That is the per-slice instability measured on identical inputs,
    and it is the reason a single run of anything here settles very little.

## What re-preparation is for, and what it is not for

The artifact carries `sourceText` on every delivery row.
So the audit reads the two texts straight out of the artifact and never re-slices.
Auditing re-sliced text would audit a different input than the one the judges saw.

Re-preparation still happens, for two things that live only in the preparation:

-   `identityContext`.
    Both entries declare real names,
    and the producing judges were given them:
    236 characters for `Aniloviraw`, 218 for `zheermao101`,
    declaring name, alias and location on both sides.
    An auditor shown a rendering WITHOUT that block sees a name
    it cannot derive from the source
    and has every reason to call it a fabrication.
    That is the defect `#36` was opened for,
    and rediscovering it as findings would poison every name-bearing slice.
    So it is passed, and recorded as a tagged value rather than an optional string,
    because "this pair declared nothing" and "nobody recorded whether it did"
    mean opposite things when a name-shaped finding turns up.
-   Provenance.
    All four artifacts VERIFY against a preparation recomputed at HEAD.
    Slice source text at HEAD is character-identical to what the artifacts carry,
    so nothing in the slicing has moved under them.

A verification refusal is carried as a value rather than thrown.
A slicing that moved is a finding about that artifact,
not a reason to refuse to read the rows it settled.

The corpus pin comes from each artifact's own `corpusSha`,
never from `RUN_CORPUS_PIN`,
and the read goes through the commit object rather than the working tree,
so neither a moved pin nor a dirty clone can change the answer.
The clone was clean when this was measured, and it would not have mattered.

## What this population cannot answer

Two entries is not a census in any useful sense,
whatever the word "every" suggests about four files.
Nothing about a particular entry can be settled from two of them.

The instrument's own production error rate is still unmeasured:
`#66` is the false-negative half and is open,
`#68` records that one of the three checkers raises claims
at a tenth the rate of the others.
So nothing this run says may gate what ships.
It is telemetry, and the producing path does not change.

Read `corroborated` and `agreed` apart.
Four runs of `audit-sensitivity` now show the strict tier
reporting a unanimous defect as nothing.

## Decided before the run, so the result cannot move the goalposts

`#107` is open:
per-slice judging cannot tell a RELOCATION from a fabrication.
A passage the archive carried across a slice boundary
reads as an omission on one slice
and as an unsupported addition on its neighbour.

So: paired omission and addition findings on ADJACENT slices of one entry
get checked against each other and counted as one relocation,
not as two defects.
This is written down before the tally exists,
because deciding it afterwards is exactly the move that was refused
when the voice-loss re-read was read on the at-risk population.

The 16 retained subjects are where `#107` bites hardest,
since those are the archive's own text,
which is what the relocation claim is about.

## What the first full run said

Run `2026-08-17T14-25-51.584Z-b7d84b5b`,
all 40 subjects,
three auditors each:

```text
                subjects   drew a claim   claims   corroborated   agreed   near   degraded
ARCHIVE text          16             10       30              5        7      8          4
FRESH   text          24             14       25              1        1      4          1
```

```text
hf:openai/gpt-oss-120b                 asked 40   spoke on 17   claims 25   dropped  3
hf:nvidia/NVIDIA-Nemotron-3-Super      asked 37   spoke on 13   claims 16   dropped  9
hf:Qwen/Qwen3.6-27B                    asked 38   spoke on 11   claims 14   dropped  4
```

Relocation candidates under the `#107` rule: two.
`grace-remeasure-2026-08-17/Aniloviraw` pairs an omission at slice 1
with an addition at slice 2,
and `grace-remeasure-2026-08-17/zheermao101` pairs an omission at slice 1
with an addition at slice 0.
Both are named rather than subtracted:
this reading can say which pairs a human should look at,
and cannot say whether either is really one move.

### The archive-versus-fresh split is confounded with the entry split

This is the finding that matters most,
and it does not depend on the band at all.

```text
                       subjects   spoke on   claims   corroborated   agreed
Aniloviraw   ARCHIVE          3          3        6              0        0
Aniloviraw   FRESH            7          5       12              1        1
zheermao101  ARCHIVE         13          7       24              5        7
zheermao101  FRESH           17          9       13              0        0
```

EVERY corroborated archive defect is in `zheermao101`.
The one corroborated fresh defect is in `Aniloviraw`.
Neither entry shows the pattern the pooled table shows.

So the headline reading,
that archive text draws more corroborated defects than fresh text,
is indistinguishable from the reading
that `zheermao101` draws more corroborated defects than `Aniloviraw`,
which would say nothing about archive text at all.
Two entries cannot separate those,
and no number of subjects inside two entries can either:
adding slices makes each cell tighter
without adding a single degree of freedom to the comparison.

The denominators make it worse rather than better.
The archive half's 16 subjects are 3 from one entry and 13 from the other,
so the pooled archive figure is very nearly `zheermao101`'s archive figure.

WHAT WOULD FIX IT: more ENTRIES, not more slices.
The band from a second run answers a different question,
which is whether one entry's reading is stable,
and it is worth having for that.
It cannot unconfound this.

### Voice loss is not spread evenly

Five subjects lost a voice and finished on two of three auditors.
All five are `zheermao101`:

```text
two-lane-cost   zheermao101 #2   Qwen lost
two-lane-cost   zheermao101 #8   Nemotron lost
two-lane-cost   zheermao101 #13  Nemotron lost
grace-remeasure zheermao101 #2   Qwen lost
grace-remeasure zheermao101 #13  Nemotron lost
```

None on `Aniloviraw`,
which is the entry with the shorter slices.
Both entries were audited by the same roster under the same settings,
so this is a fact about which subjects are hard to answer,
not about which models were reachable that afternoon.

It also means the two halves were not audited by equally complete panels.
A subject heard by two voices cannot reach the strict corroborated tier
as easily as one heard by three,
and every one of these sits on the entry that carries the corroborated defects.
Another reason the pooled comparison is not readable.

### What gets claimed, by category

```text
   9  altered-referent   ARCHIVE        9  altered-referent   FRESH
   7  omission           ARCHIVE        5  altered-time       FRESH
   3  altered-modality   ARCHIVE        2  altered-number     FRESH
   3  unsupported-addition ARCHIVE      2  omission           FRESH
   3  altered-actor      ARCHIVE        2  altered-relation   FRESH
   3  altered-time       ARCHIVE        2  altered-identity   FRESH
   1  altered-identity   ARCHIVE        1  altered-actor      FRESH
   1  altered-relation   ARCHIVE        1  altered-modality   FRESH
                                        1  unsupported-addition FRESH
```

`altered-referent` is the most claimed category on both halves and by the same count,
which is at least a sign the instrument is not simply louder about one half.

`omission` is the category that separates them,
seven on archive text against two on fresh.
That is the category `#107` says per-slice judging cannot tell from a relocation,
and both relocation candidates are archive-side omissions.
So the one categorical difference between the halves
sits precisely on the failure mode that is known to be unresolved.

### Both relocation candidates have the geometry `#107` predicts

`#107` says a passage the archive carried across a slice boundary
reads as an omission on one slice and an unsupported addition on its neighbour.
That predicts WHERE inside each slice the two claims should anchor:
against the shared boundary.

Measured, from the persisted spans and the artifacts' own slice lengths.
No text, only offsets:

```text
Aniloviraw   omission  at slice 1, source 189 chars, anchored [149..161]   79% to 85% through
             addition  at slice 2, shipped 278 chars, anchored [0..44]      0% to 16% through

zheermao101  omission  at slice 1, source  80 chars, anchored [7..30]       9% to 37% through
             addition  at slice 0, shipped 265 chars, anchored [176..218]  66% to 82% through
```

Both pairs anchor against their shared boundary,
the first at the end of one slice and the start of the next,
the second at the start of one slice and the latter third of the one before it.
That is corroboration for the MECHANISM `#107` describes,
measured rather than assumed,
and it is the strongest thing two candidates can supply.

It is still not proof that either is one move.
Confirming that needs the two texts read side by side by a person,
which is what `#107` asks for and what this reading deliberately does not do.

### One voice can claim one region twice, and the claim count adds them up

On `zheermao101` slice 0,
Qwen anchored BOTH an `altered-actor` and an `unsupported-addition`
to the identical candidate span, characters 176 to 218.
One voice, one region, two claims.

So `claimed` counts CLAIMS and not distinct problems,
and the two are not the same number.
Anything that ever reads a threshold over this figure
has to decide whether one voice describing one region twice
should count once or twice,
and the honest answer is not obvious:
a passage that both invents material and misattributes it
arguably is two defects.

Recorded here because it is invisible in the tally
and only exists to be found because the whole report is persisted
rather than a count.

### The one thing this run cannot support is the comparison it was bought for

Archive text drew five corroborated defects over 16 subjects
and fresh text drew one over 24.
That is the headline shape,
and it is NOT QUOTABLE from this run,
because this run measured no band.

`QNB` says a comparison resolves nothing narrower than the spread
the instrument moves through on unchanged input,
and that spread is known to be wide here without being known to be how wide.
It was seen directly during this task:
`grace-remeasure/Aniloviraw#0` read `claimed=1 corroborated=0` on a capped buy
and `claimed=5 corroborated=1 agreed=1` on this run,
identical text,
identical roster,
minutes apart.

The rows of this run carry no recorded text identity,
because the field that records it landed after the run started,
so the six character-identical subjects cannot be paired inside it
and the report says so instead of printing zeroes.
A second full run over the same 40 supplies the pairing at full width.
Until that lands,
the two halves are two single readings and the difference between them
has no scale to be read against.

### What is already worth saying

The per-voice rates do not reproduce `#68`.
Over the introduced-defect probe,
Qwen raised claims at about a tenth the rate of the other two.
Here the three voices are within a factor of two of each other
on both subjects-spoken-on and total claims,
and Qwen is not the quietest on subjects spoken on.
That is one run of 40 subjects against 857 probed regions,
so it does not overturn `#68`;
it says the quiet-voice finding is specific to that stage or that population
rather than a fixed property of the model,
and `#68` should be re-read on the stage it is going to gate.

Nemotron dropped 9 claims at the screen against 3 and 4 for the others,
and answered 37 of 40 against Qwen's 38 and gpt-oss's 40.
A voice that answers less often and anchors less well
contributes less than its seat suggests,
which is the same shape as `#68` in a different quantity.

## What the second full run said, and it settles the headline

Run `2026-08-17T15-06-37.885Z-c157db5b`,
the same 40 subjects,
the same roster,
about forty minutes after the first,
and this one recorded its pipeline digest at run start,
so it answers for itself.

```text
                subjects   drew a claim   claims   corroborated   agreed   near   degraded
ARCHIVE text          16             10       31              3        4     13          3
FRESH   text          24             10       21              1        2      3          3
```

Set beside the first run,
over identical input:

```text
                       run 1    run 2
ARCHIVE corroborated       5        3
FRESH   corroborated       1        1
ARCHIVE near               8       13
relocation candidates      2        0
```

THE ARCHIVE CORROBORATED COUNT MOVED FROM FIVE TO THREE
with nothing changed but the hour.
The relocation candidates went from two to none:
the `#107` pairing found nothing at all the second time,
on the same forty subjects.

### Applying the pre-committed rule to that, without rounding it

The rule fixed in advance says
that a corroborated total moving by about four between two readings
means the archive-versus-fresh difference is inside the noise,
and that a move of nothing or one means it stands.

The observed move is TWO.
That is between the two markers,
so the rule as written DOES NOT DECIDE IT,
and reading it toward either marker would be choosing an answer
and then calling it a rule.
State it as it is:
undecidable under the pre-committed rule,
which was under-specified in the middle.

Nothing rests on closing that gap,
because the confound settles the headline independently,
which is exactly why an honest in-between reading costs nothing here.
The rule is worth keeping and worth sharpening for the next population.
It would have been worth much less
if it had been quietly stretched the first time it landed between its own markers.

### What the numbers still say

The archive-versus-fresh gap was 5 against 1,
then 3 against 1.

This is a DIFFERENT measurement from the band rule just applied,
and it is worth not blurring them.
The band rule asks how far apart two audits of ONE TEXT land,
and its answer here is undecided.
This asks what the POPULATION AGGREGATE did between two readings,
and its answer is that it moved by two while the claimed effect is four.
An effect cannot be read off an aggregate
that moves by half the effect's size when nothing changes,
whatever the per-text band turns out to be.

And the confound with the entry was already fatal to it independently.
The headline is dead twice over,
for two unrelated reasons,
and neither is a reason to distrust the two entries:
they are reasons this population cannot answer this question.

### The within-run band is narrow, and it is narrow on the easy subjects

Over the six texts this run audited twice,
which the pairing found exactly as predicted:

```text
pairs 6   same claim count 3   widest gap 4   total gap 8   silent on one side only 1
claims 8 against 8, corroborated 1 against 1
```

Read alone,
that looks reassuring:
the corroborated totals do not move at all,
and by the rule fixed in advance
a corroborated total that moves by nought means the tier is stable.

Read against the population figures above, it is misleading,
and the reason is already recorded:
those six are almost all archive text inside one entry,
five of six retained in both settlements.
They are the subjects two independent productions agreed about.
The corroborated tier moved by two on the full population
while moving by nought on them.

So a narrow band measured on a convenience sample of a population
is not a band for that population.
The six pairs were free, and worth having,
and they would have licensed exactly the wrong conclusion on their own.

### The per-voice rates are not stable either, which revises `#68`

```text
                                    run 1                 run 2
hf:openai/gpt-oss-120b        spoke 17  claims 25   spoke 14  claims 20
hf:nvidia/NVIDIA-Nemotron     spoke 13  claims 16   spoke 12  claims 24
hf:Qwen/Qwen3.6-27B           spoke 11  claims 14   spoke  6  claims  8
```

An earlier note on `#68`,
written from run 1 alone,
said the three voices sit within a factor of two of each other here
and so do not reproduce the order-of-magnitude gap seen over the probe.
Run 2 does not support the "factor of two" part.
Nemotron claims 24 against Qwen's 8,
which is three to one,
and Qwen's own claim count nearly halves between the two runs
on identical input.

What survives, and it is the part that mattered:
the ORDER-OF-MAGNITUDE gap from the introduced-defect probe does not appear
in either run,
so that finding still looks specific to the probe task.
What does not survive is any claim about the size of the gap here.
Two runs disagree about it,
so a third figure would be a third number rather than a confirmation.

## The band, at full population width, and it decides

Run `2026-08-17T15-44-00.750Z-eeda8b9b` is the third audit of the same 40 subjects.
Paired against the second:

```text
paired        40      predicted 40
textMoved      0      predicted 0
unverifiable   0      predicted 0
within-run pairs 6    predicted 6
```

All four predictions were recorded before the run landed and all four hold,
so the pairing is doing what it claims
and the archive did not move under any of the three audits.

THE BAND OVER 40 IDENTICAL TEXTS, about thirty-eight minutes apart:

```text
pairs 40   same claim count 19   widest gap 4   total gap 37   silent on one side only 10
claims 52 against 63, corroborated 4 against 10
```

### The pre-committed rule decides this one, and the answer is noise

The rule fixed in advance:
a corroborated total moving by about four between two readings
puts the archive-versus-fresh difference inside the noise.

The corroborated total moved from FOUR TO TEN.
That is a move of six, on identical text, with an identical roster.
It is past the marker rather than between the markers,
so unlike the run-one-to-run-two comparison this one is not undecidable:
the rule applies and the answer is that a difference of four is not readable.

### Ten of forty subjects flipped between silent and speaking

`silent on one side only 10`.
A quarter of the population drew at least one claim in one audit
and nothing at all in the other,
on the same characters.

That is the number any future gate should be shown first.
A threshold reading "at least one claim" would have flipped
on a quarter of these subjects for no reason in the text,
and a quiet subject is indistinguishable from a clean one.

### Three readings of one population

```text
                        run 1   run 2   run 3
ARCHIVE corroborated        5       3       7
FRESH   corroborated        1       1       3
gap                         4       2       4
relocation candidates       2       0       0
```

The archive figure ranges from three to seven across three readings of one
unchanged population.
The gap that the whole audit was bought to measure ranges from two to four
while the quantity it is computed from swings by four.

The relocation candidates appeared once and never again,
which is worth stating plainly:
the two candidates whose geometry corroborates `#107` so neatly
were not reproduced by either later audit of the same text.
The GEOMETRY finding stands, because it is a fact about where those two claims
anchored.
The RATE finding does not exist: two, nought and nought is not a rate.

### What this does not say

It does not say the archive's English is as good as a fresh rendering,
or worse, or better.
It says this instrument on this population cannot tell,
and now says so with a measured band rather than as a caution.

It also does not say the instrument is broken.
An instrument with a wide band is still an instrument;
it needs either a larger population or repeated readings averaged,
and this population can supply neither
because it is two entries.

## How the band was to be read, written before it existed

Two bands, resting on different things,
and the write-up must say which is which rather than quoting the friendlier one.

WITHIN one run,
the six character-identical subjects two artifacts of one entry already contain.
Six pairs,
one build,
one roster,
minutes apart,
every pair verified by recorded digest on both sides.
Small, and free.

AND IT IS NOT A GENERAL SAMPLE, re-derived from the artifacts before the run landed
by digesting both texts of every slice in both run sets:

```text
Aniloviraw    0 identical of  5
zheermao101   6 identical of 15   (slices 0, 2, 7, 8, 9, 11)
```

`Aniloviraw` contributes NOTHING.
Not one of its five slices shipped the same English in both settlements,
which is itself worth noticing:
two of them were `replacement-shipped` in both settlements and still differ,
so two independent productions of one source never converged there.

Five of the six pairs are `incumbent-retained` in both settlements,
so they are archive text.
Only slice 9 is a fresh rendering,
and it qualifies because two productions happened to converge on the same characters.

So the within-run band is measured
almost entirely on ARCHIVE text,
entirely within ONE entry.
It can say how stable the instrument is when re-reading the archive's own English.
It cannot say anything about its stability on fresh renderings,
which is the half the instrument was built for.
Only the across-run band reaches those.

A CHECK ON THE PAIRING ITSELF, recorded before the run lands:
the population section says those six are
`zheermao101` slices 0, 2, 7, 8, 9 and 11,
measured from the artifacts back when the population was counted.
`auditRepeatsWithin` should find exactly those six and no others.
It reaches that answer by a completely different route,
from digests of the text each audit was actually shown
rather than from the artifacts' own delivery rows,
so agreement is real corroboration and a different count is a defect
in one of the two readings.

ACROSS two runs,
every subject in both.
Forty pairs,
which is the band at the full width of the population.
The first full run cannot supply this:
its rows predate the recorded text identity,
so pairing it against anything yields forty subjects
that are matched by position and cannot be vouched for.
The report says exactly that,
in its own sentence,
kept apart from the sentence about the archive having moved.

THE HEADLINE GAP IS A CORROBORATED GAP,
five over 16 archive subjects against one over 24 fresh ones,
so the band that decides whether it is readable
is the band on CORROBORATED counts,
not on raw claims.
`AuditRepeatBand` carries both sides' corroborated totals for that reason.
If two runs of identical input move the corroborated total by about four,
the archive-versus-fresh difference is inside the noise and must not be quoted as a finding.
If they move it by nothing or one,
it stands.
Say which,
in those words.

### What run 3 must produce, recorded before it lands

Both runs read their texts straight out of the same four artifacts,
which nothing has touched.
So pairing run 3 against run 2 must give:

```text
paired        40
textMoved      0
unverifiable   0
```

and `auditRepeatsWithin` on run 3 must again give six.

Any other numbers are a finding rather than a result.
A non-zero `textMoved` would mean the archive changed under the two runs,
which nothing should have done and which would invalidate the band.
A non-zero `unverifiable` would mean one of the two runs did not record
what it was shown,
which both builds do.
Fewer than 40 paired would mean the two runs bought different populations.

Writing this down turns 40, 0, 0 into a confirmation
instead of a description of whatever turns up.

### What "the same build" can and cannot mean here

`pipelineDigest` hashes the whole built tree,
so any two runs separated by any commit carry different stamps,
including commits the audit runner never loads.

The runner's actual dependency closure,
read off its own import statements in the built entry:

```text
./run-config-DHZxe1GN.mjs
./rendering-audit-settled-digest-DhVzr2R3.mjs
./rendering-audit-settled-input-PANwDHyw.mjs
./pipeline-digest-DDwOLNUW.mjs
./probe-store-CIfFH-eS.mjs
./rendering-audit-Cw-Y7Wq9.mjs
```

The reading modules that changed between runs compile into
`rendering-audit-settled-report.mjs` and `rendering-audit-settled-band-*.mjs`,
which are not in that list.
So runs separated only by reading-side commits executed identical auditing code
while reporting different digests.
Task `#116` proposes recording the closure alongside the tree digest,
since the closure is the thing a band measurement actually needs
and it costs nothing to read.

## Provenance of the first run, which its own file gets wrong

The run file records pipeline digest `...b7d84b5b`.
That is not the build that produced these rows.

Every probe read its digest at the END of a run until this task,
and `dist` was rebuilt at 14:38:56Z while this run was in flight,
having started at 14:25:51Z.
So the file stamps a build that landed mid-run.
The build that actually ran is `...291b354f`,
which the capped run at 14:22:55Z recorded three minutes earlier.
That is an inference from the surrounding evidence rather than a reading:
nothing records a build in the 176 seconds between,
and nothing rules one out either.

The persisted file is left exactly as written.
The store is the record,
and editing a run to say something it did not say is worse
than a run with a wrong field and a note next to it.
All three probes now read the digest at run start,
so the second full run answers for itself.
