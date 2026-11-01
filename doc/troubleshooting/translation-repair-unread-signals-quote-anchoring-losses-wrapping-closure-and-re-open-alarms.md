# Unread signals: quote anchoring losses, the closed wrapping hypothesis and the false re-open alarms

The investigation of the 398 critic claims discarded at quote anchoring:
the soft-wrapping hypothesis,
its bounds and its closure at 30 misses,
what the misses are not,
and the later alarms that tried to re-open it and turned out to be pooling artifacts and one outlier entry.

Part of [Signals the pipeline emits and nobody reads](translation-repair-unread-signals.md),
which lists every file of this record.

## Quote anchoring discards 398 critic claims, and the cause is still unknown

`quote-not-found` 225,
`empty-quote` 129,
`ambiguous-quote` 30,
 `quote-outside-blocks` 12,
`no-quotes` 2.
Each one is a critic claim that never reached adjudication.
Confirmed at the call site rather than inferred from a TSDoc example:
 `repair-stages.ts` pushes the reason and returns an empty array,
so the claim
 is dropped.

The buckets are different failures and must not be pooled.
`empty-quote` is a malformed model response,
 `ambiguous-quote` is a quote found more than once,
 and only `quote-not-found` is a location miss.

For that last bucket there is a mechanism the code cannot currently see.
The corpus soft-wraps its prose,
so a paragraph holds line breaks that are not
 paragraph breaks.
A critic quoting across a wrap returns a space where the
 document holds a line break.
`locateQuote` searches byte-exact first,
then
 falls back to `normalizePunctuation`,
whose map covers curly punctuation,
 CJK corner brackets and U+00A0.
Line breaks are deliberately absent from it,
 so both searches miss.

The competing explanation was ruled out first:
if the critic prompt re-wrapped
 the text,
a space-joined quote would be faithful to what the model was shown
 and the fix would belong at the rendering boundary instead.
`critic-prompt.ts` interpolates `sourceText` and `targetText` raw,
so the model
 saw the line breaks and collapsed them itself.

### What was done about it

Telemetry only,
in `a6bbeca50`.
`quote-not-found` now carries a suffix naming
 what a soft-line-break collapse would have produced:
 `[line-break-collapsible]` for a unique hit,
`[line-break-ambiguous]` for one
 that lands twice,
nothing at all when the quote is genuinely absent.
No claim changes fate.
`SLICE_CACHE_VERSION` went to 9 because findings are
 part of the cached payload.

Counting outcome TRANSITIONS rather than "does it locate now" is the point.
Collapsing line breaks makes the haystack more uniform,
so some misses will
 become ambiguous rather than located,
and a fix that traded silent discards
 for silent ambiguity would look like a win in a naive count.

Admitting those quotes was NOT done,
and is blocked rather than merely
 deferred.
A repair anchored to a quote spanning a wrap replaces several lines
 with one,
which is exactly the line-structure question left open in
 `doc/planning/naturalness-lane-reach.md`.
Landing it quietly would decide that
 question without asking.

### How much of the 225 the wrapping explains, bounded before the telemetry lands

Two attempts,
one worthless and one usable.
Both are recorded because the
 worthless one looks convincing.

The first compared the newline rate among quotes that DID locate (1.03% of
 5458 target anchors) against the rate a 41-character window covers a newline
 when slid across the same prose (28.06%).
A 27-fold deficit,
and meaningless.

It fails twice over.
The corpus wraps at semantic boundaries,
so lines are
 short (median 49 characters against a median quote of 41) and quotes are
 line-locked:
29.7% of located quotes are EXACTLY whole lines,
39.9% start
 flush after a line break and 46.0% end flush before one.
A uniform-random
 window is not what a critic produces.
Worse,
the sample is survivors only,
so
 "located quotes rarely contain newlines" is what BOTH explanations predict:
 critics rarely crossing wraps,
and cross-wrap quotes failing and vanishing
 from the sample.

The second test avoids survivors entirely.
If wrapping causes misses,
entries
 whose prose wraps more should miss more,
and the miss rate is computed against
 attempts rather than against successes:

```text
  54 entries with at least 10 anchoring attempts; median line length per entry
  spans 12 to 149 characters, so the split has room to show an effect

  prose mostly wrapped (share > 0.72)   26 entries   4.5% of attempts missed
  prose less wrapped   (share <= 0.72)  27 entries   3.1% missed

  median line <= 54                     27 entries   4.2% missed
  median line >  54                     26 entries   3.6% missed
```

Both splits move in the direction the mechanism predicts,
and neither moves
 far.
Treating the less-wrapped rate as the non-wrap baseline attributes
 roughly a quarter of the misses in the wrapped group to wrapping,
so the
 expectation to hold going in is that soft wrapping explains a MINORITY of the
 225,
not the bulk of them.

That is a bound,
not an answer:
the split is coarse,
and entries that wrap more
 may differ in other ways.
The suffix landed in `a6bbeca50` measures it
 directly,
one failed quote at a time,
and is the figure to believe.

### What the direct measurement says so far, against that prediction

At 3 settled entries `pass13` has produced 13 `quote-not-found` findings and
 NONE of them carries either suffix.
The telemetry is confirmed live,
because a
 bare reason can now only be produced by the line that always calls
 `lineBreakSuffix`.

Thirteen is enough to test the quarter:

```text
  if the true wrap share were   probability of seeing zero in 13
    25%                            2.4%
    20%                            5.5%
    15%                           12.1%
    10%                           25.4%
     5%                           51.3%

  one-sided 5% upper bound: true share <= 20.5%
```

So the prediction this section tests is REJECTED at the conventional level.

### CLOSED at 30 misses: soft wrapping is not a material cause

The count reached the pre-registered threshold with none suffixed:

```text
  observations   5% upper bound on the true wrap share
      13              <= 20.5%
      17              <= 16.1%
      25              <= 11.2%
      30              <=  9.5%     <- threshold, 10% now rejected at 4.24%
```

Thirty `quote-not-found` findings,
23 source and 7 target,
and not one carried
 either suffix.
The point estimate was zero and the ceiling 9.5%.

AMENDED ONE HOUR LATER,
and the amendment matters more than the number.
At 33
 misses one IS suffixed,
`quote-not-found (source) [line-break-collapsible]`,
 meaning a quote that collapsing soft line breaks would have located uniquely.
So the mechanism fires in practice,
and "not one" expired exactly as two
 earlier universal claims about accumulating data did.

The bound LOOSENS with a positive observation rather than tightening:

```text
  1 of 33, point estimate 3.0%
  one-sided 95% upper bound  <= 13.5%   (was <= 9.5% at 30 with zero)
```

THE VERDICT IS UNCHANGED and now rests on the point estimate rather than on a
 zero.
At 3.0%,
admitting wrap-spanning quotes recovers about 7 of the 225
 misses in the settled population.
Seven claims is not worth a behaviour change
 that also decides the line-structure question,
and the change would still
 replace several lines with one.

What changed is the honesty of the claim,
not the decision:
this is "a small
 real effect",
not "no effect".

THE SAMPLE WAS CHECKED BEFORE THE NULL WAS TRUSTED,
because a lightly-wrapped
 sample would produce this result for an uninteresting reason.
It is not
 lightly wrapped:
the 7 settled entries have a median wrapped-prose share of
 69%,
against 69% for the other 85 entries in the corpus.
Individual shares run
 from 0% to 97%,
so the medians matching is the relevant comparison.

VERDICT.
The mechanism is real and is proven by unit test:
a quote spanning a
 soft wrap,
returned with a space,
fails both the exact and the
 punctuation-normalized search.
Its INCIDENCE in practice is about 3%,
with a
 one-sided 95% ceiling of 13.5%.
Admitting wrap-spanning quotes would therefore
 recover roughly 7 of the 225 misses in the settled population,
so the
 anchoring change is not worth making.

The telemetry stays.
It costs nothing,
it is what closed this,
and it will
 notice if a future corpus wraps differently.

CONSEQUENCE BEYOND THIS ISSUE:
the line-structure question in
 `doc/planning/naturalness-lane-reach.md` now governs ONE change,
the
 naturalness lane,
rather than two.
That decision got simpler rather than more
 valuable,
reversing what was recorded earlier.

### What the misses are NOT, which narrows what they can be

With wrapping ruled out,
the next candidate was a pipeline bug rather than
 model behaviour:
that the text a critic is SHOWN differs from the text
 `locateQuote` SEARCHES.
Any divergence there would defeat quotes wholesale and
 would be ours to fix.

There is none.
`repair-chunk.ts` parses `documents` from exactly the
 `sourceText` and `targetText` it then passes to the critic phase,
and both
 travel together into `runChunkCriticPhase`.
`RepairDocument.text` is the
 original source byte-for-byte rather than the masked variant,
so masking does
 not separate them either.

So the remaining candidates are all model behaviour:
paraphrasing instead of
 quoting,
eliding with an ellipsis,
quoting across the chunk boundary it was
 shown,
or inventing text.
Those cannot be told apart without the failing
 quote,
which nothing retains.

That is worth stating positively rather than as another dead end.
The expensive
 branch,
a systematic defect in our own plumbing,
is eliminated.
What is left is
 a telemetry gap with a known and cheap remedy.

The direction matters for the decision this feeds.
If soft wrapping explains
 little,
the anchoring lead closes,
and the line-structure question in
 `doc/planning/naturalness-lane-reach.md` governs one change rather than two.

### The side split varies by entry, so do not compare a sample to a population

The quote-family findings carry a side,
`source` or `target`,
and across all 56
 settled entries `quote-not-found` is almost exactly even,
112 against 113.

At 7 settled entries `pass13` shows 17 source against 3 target.
Under an even
 split that is about a 0.26% outcome,
which looks like a pipeline change and is
 not one.

The controlled comparison is the same entries in both runs,
and it dissolves
 the effect:

```text
                          pass13    old run, SAME 7 entries
  quote-not-found source     17        12
  quote-not-found target      3         6
  empty-quote source          6         7
  empty-quote target          4         3
```

Those seven entries were ALREADY source-skewed in the old run,
12 against 6,
 while the corpus overall was even.
The same holds for `empty-quote`,
whose
 corpus-wide skew of 112 source against 17 target is driven by entries outside
 this sample entirely.

So the side distribution is an entry property,
and comparing a 7-entry sample
 against a 56-entry population measures which entries were drawn rather than
 what the pipeline did.
Compare the same entries,
or do not compare.

This is the second finding in one session to dissolve the same way;
the other
 was a `needs-human` share that looked like a threefold shift and turned out to
 be one entry.
Both were caught,
but both were nearly written down.

### A gap this exposed

The failing quote is not retained anywhere.
Artifacts keep adjudicated issues
 only,
2650 accepted plus 1033 rejected plus 415 needs-human,
and a claim
 discarded at anchoring never becomes one.
So the incidence of each mechanism
 cannot be measured from the existing 56 entries at all,
only from a pass run
 after the suffix landed.
A recorded signal that omits the evidence needed to
 act on it is the same failure this document is about,
one layer down.

## The quote-anchoring re-open alarm was a pooling artifact, and the closure stands

Fired 2026-08-13 by the run monitor,
which put the quote-anchoring verdict at risk on `pass13`:
8 of 58
 wrap-explained,
where the closure assumed about 3%
and this was nearer 10%.

The figure is arithmetically right and the conclusion is wrong.
It pools two
 populations that the artifact tree keeps separate:

```text
  artifacts     39 quote-not-found   1 collapsible    2.6%
  slice-cache   19 quote-not-found   7 collapsible   36.8%
  pooled        58                   8              13.8%
```

The quote-anchoring finding was closed on the SETTLED-ARTIFACT population,
and that population reads
 2.6% here against the roughly 3% the closure assumed.
It is the same number.
The closure stands and needs no re-taking.

This is the third time this session that a share computed over the wrong
 population produced a false alarm,
after the needs-human share and the
 quote-side split.
The tell is identical each time:
a rate that moves by a
 factor of three or more between a small reading and an established one,
where
 the small reading silently spans a different set of things.

THE MONITOR SHOULD BE FIXED RATHER THAN THE DOCUMENT.
It globs every `.json`
 under the run root,
so it will keep pooling `slice-cache` with `artifacts` on
 every future check,
and every future firing will overstate the same way.

### The slice-cache reading is a real leading indicator, not noise

36.8% against 2.6% is not a rounding difference,
and it deserves an explanation
 rather than a dismissal.

The likely one is timing rather than behaviour.
`artifacts` holds SETTLED
 entries and `slice-cache` holds slices of entries still in flight,
so the cache
 is a window onto work that has not yet reached an artifact.
If those 7
 collapsible failures settle as they stand,
the settled share rises and the finding
 genuinely would need re-taking.

Held as a WATCH ITEM rather than a finding,
for two reasons.
The cache total is
 19 observations,
which is small enough that a couple of entries dominate it.
And the direction is not established:
nothing here shows a cached failure
 reaching an artifact unchanged,
which is the step the inference needs.

Re-read both populations separately when `pass13` settles more entries.
The
 comparison that matters is artifact-share then against artifact-share now,
and
 it is the only one that speaks to the closure.

### It fired again on the settled population, and it was one entry

The fixed monitor,
counting `artifacts` alone as it should,
alarmed at 12
 settled entries:
8 of 59 wrap-explained,
13.6%,
against the roughly 3% the
 closure assumed.
On the right population,
with the pooling bug gone.

It is still a false alarm,
and the reason is the one this session keeps
 relearning:

```text
  Futajuhuacha        7/20     35%
  Aniloviraw          1/6
  every other entry   0/33

  all 12 entries      8/59     13.6%
  excluding the top   1/39      2.6%
```

Seven of the eight hits are ONE entry.
Drop it and the share is 2.6%,
which is
 the closure's number to within noise.
The finding stands,
again.

The watch item recorded when the pooling bug was fixed did predict this
 correctly in one respect:
the in-flight `slice-cache` hits were 7 of 19,
and
 they settled into `artifacts` unchanged,
so the cache was a genuine leading
 indicator of what would land.
What it led to was an outlier,
not a population
 shift.

#### The guard that generalises all three false alarms

Three alarms this session,
three different surface causes,
one shape:

-   `needs-human` at 31%,
    which was `AmbeR_the_anpa` alone.
-   Wrap-explained at 13.8%,
    which was `slice-cache` pooled with `artifacts`.
-   Wrap-explained at 13.6%,
    which was `Futajuhuacha` alone.

A threshold on the share cannot separate these from a real shift,
and the
 second and third cases prove that fixing one surface cause does not stop the
 next.
The rule that does separate them is stated once and applies to all:

A share is a POPULATION signal only if it survives dropping its largest single
 contributor.

The monitor now computes both figures and alarms only when BOTH clear the
 threshold.
It reports the excluded-top share on every progress line too,
so the
 concentration is visible before it becomes an alarm rather than after.

#### Left open, and worth a look when the pass ends

`Futajuhuacha` really is 35% wrap-explained where every other settled entry is
 near zero.
That is a fact about one entry rather than about the pipeline,
but
 nothing here explains WHY one entry's critic quotes should fail line-break
 collapse at ten times the corpus rate.
Cheap to check once the pass stops:
 whether that entry's target text is wrapped differently from the rest.
