# Translation repair history 2026-08-07 to 2026-08-12: the Kimi-K3 schema-mismatch degradation, the recall figure measured under it, its cause and its fix

The silent failures of the naturalness lane and the editor ensemble in run 013,
the recall re-measure
and its correction as a degraded-ensemble figure,
run 014,
the two-character channel marker that caused it all,
and the rosters widened so that no fan-out stage can meet quorum on one voice.

Part of [Translation repair history: 2026-08-07 to 2026-08-12](translation-repair-history-2026-08-07-to-12.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Run 013, and the naturalness lane failing without saying so

```text
DONE processed=5 of pending=50; artifacts=47/92 elapsed=43783207ms
```

Five entries settled,
none lost to the per-entry deadline,
which is a better
 return than run 012's two.
Entries:
`Y1Ran` (resumed from its run 012 abort,
per #61),
`SevenBird`,
 `Uekawakuyuurei`,
`TLL1122`,
`cheonwoomaeng`.

### What the numbers looked like, and why that was the tell

At 47 entries the accuracy probe had grown from 583 regions to 666,
while
 `REFINEMENT rewrittenSlices` had not moved from 9.
Not one of run 013's five entries carries a single `refined: true` record.

The cause is in the log and is unambiguous:

```text
24 refiner hf:moonshotai/Kimi-K3: schema-mismatch, voice lost
 6 refiner: retry round 1 for 1 lost voices    (also rounds 2 and 3)
```

Six refinement attempts,
four tries apiece,
every one lost.
Run 012's log carries ZERO refiner lines,
because a lost voice is what gets
 logged and run 012 never lost one.
So between two consecutive runs on unchanged pipeline code the lane went from
 working to producing nothing,
which makes it a provider-side change rather than
 a regression anyone introduced here.
`Y1Ran` reads `1/1 heard` only because its refine finding came back with its
 banked slices from run 012.

### Why nothing reported it

The lane is ONE model.
Every other stage retries to a quorum and reports a degraded roster;
 a roster of one has no quorum to lose,
so total failure moved no number that
 anything printed.
The refinement audit stayed at 9 and printed no note,
because its zero-note only
 fires when the total is zero and the total was non-zero from run 012.
A stage that had stopped working was indistinguishable from a stage nobody had
 asked to work.
That is the exact ambiguity the zero-note was written for,
arriving in the one
 shape the note does not cover.

### What landed

`score-probe` prints a LANE line,
counted from the per-slice findings the refine
 stage already wrote:

```text
LANE slicesOffered=101 slicesSilent=6 entriesWithRewrites=15/47
```

`slicesSilent` is slices where NO refiner answered.
Findings are read as plain strings and never validated into a vocabulary,
 because this count exists to notice a stage going quiet and throwing on drifted
 wording would silence it in precisely that case.

### What did NOT land, and why

The roster is unchanged.
Adding a second refiner costs a judge per selection round,
and round three
 already carries an accepted attribution cost from changing the roster,
the
 editor,
the checker set and this lane at once;
 changing it again mid-round would widen that further.
Whether the schema-mismatch is persistent or was a provider window is not
 established at ONE run,
and treating one run as a stable rate is the error
 already withdrawn twice in this document.
Recorded as task #64,
needing the user.
Read the next pass's refiner lines before proposing a roster change.

## The silent lane was the small half: the EDITOR ensemble degraded too

Chasing the refiner found the same failure one stage earlier,
in the stage the
 user's "no single model controls any part of the pipeline" rule was written
 for.

Kimi-K3 plays four roles here:
critic,
panel,
editor,
refiner.
Schema-mismatch counts across two consecutive passes on UNCHANGED pipeline code:

```text
run 012   Kimi-K3   0
run 013   Kimi-K3   61   (refiner 24, panel 13, critic 13, editor 11)
```

Critic and panel survive it:
they retry to a quorum,
and run 013 still shows 62
 chunk-runs at `critic stage: 6/6 heard`.
The EDITOR does not announce a stage line at all.
Its heard count lives only in a per-chunk finding,
and there:

```text
cheonwoomaeng   9 x editor-candidates (1/2 heard, 1 repairing)
TLL1122         3 x editor-candidates (1/2 heard, 1 repairing)
Toka_ls        10 x editor-candidates (2/2 heard, 2 repairing)   [run 012]
```

`cheonwoomaeng` repaired EVERY chunk it has with one editor.
Judges still chose what shipped,
so selection was not single-model,
but they
 chose among one model's proposals,
and the README's claim that "every editor in
 `editorModelIds` rewrites the chunk independently" is false for those chunks.

AND THE STAGE WAS BEHAVING CORRECTLY,
which is the part to understand before
 anyone fixes the wrong thing.
`stage-quorum.ts:154` computes `Math.ceil(modelIds.length / 2)`,
so a roster of
 two reaches quorum on ONE voice.
The editor stage met its quorum on every one of those chunks.
Nothing reported a fault because,
by the rule as written,
there was none.

This is not a malfunction,
it is two rules disagreeing.
"At least half the roster" is a sensible quorum for a six-model critic panel and
 a meaningless one for a two-model ensemble,
where half is one and the ensemble
 property is exactly what the second model was added to provide.
The disagreement is invisible while every model answers,
which is why it
 survived #45 and everything since.

That reframing changes the fix.
A per-stage MINIMUM,
the editor requiring both voices rather than half of them,
 addresses it without touching roster membership,
and is a far smaller change
 than swapping a model that holds four roles.
It also fails LOUDLY,
which is the direction this whole session has been
 arguing for.

### What landed

`summarizeStageRoster` replaces the refine-only version,
because a count that
 answers "could this stage speak" belongs to every stage that fans out.
`score-probe` prints:

```text
ROSTER editorOffered=322 editorDegraded=15 editorSilent=0
       refineOffered=101 refineDegraded=6 refineSilent=6
       entriesWithRewrites=15/47
```

Twelve of the fifteen degraded editor chunks are run 013's.
So the degradation is real but bounded at 15 in 322 across everything settled,
 and it is NOT a rate to quote from one pass.

### What this does NOT establish

That Kimi-K3 is permanently broken.
Two passes is two points,
and one of them is the only one showing the problem.
The next pass decides whether this is a provider window or a standing condition,
 and the refiner and editor lines are now the place to read it.
Do not change the roster before that,
and do not change it unasked:
 round three already carries an accepted attribution cost for changing the
 roster,
the editor,
the checker set and the lane at once.
Task #64 holds the decision.

## Task 51 is measured: recall on the current roster is 0.889

```text
SCORECARD dispatched=9 coverage=1.000 planted=27 detected=24 detectionRate=0.889
REPAIR judged=27 restored=23 partial=1 strict=0.852 lenient=0.889
```

From the scorecard JSON,
which the driver was not printing:
 `policyDeclinedSeeds=0`,
so `seedDetectionRateExcludingPolicy` equals the raw
 rate and all three misses are genuine.
That also removes one stated objection to comparability:
 the handover has said since 2026-08-06 that the house policy makes round-three
 recall non-comparable on that axis,
and on THIS run the policy never fired.

The driver now prints both fields.
Computing an attribution and leaving it in a file nobody opens is the same
 failure as the naturalness lane's:
the number that distinguishes two very
 different situations existed and reached no reader.

### What 0.889 against 0.981 does and does not say

It does NOT say recall regressed.
Two proportions,
24 of 27 against 53 of 54,
give a z of about 1.8,
which is not
 significant at any conventional threshold,
and the Wilson intervals overlap
 across roughly 0.90 to 0.96.
Three misses against one is also a difference of two events,
and this document
 already carries two withdrawn claims built on event counts that small.

The runs are not otherwise matched either:
 different roster (seven models against six),
different entries,
half the seeds,
 and several changed stages between detection and reporting.

What IS supportable:
the configuration running today detects seeded omissions at
 0.889 on this sample,
with no policy declines,
and the milestone-one figure is
 evidence about a configuration that no longer exists.
The README now says exactly that.

### Read this beside task 64

The recall run's own log carries the same refiner failure:
 `refiner hf:moonshotai/Kimi-K3: schema-mismatch, voice lost`,
through all three
 retry rounds,
plus a `restoration-judge` voice lost to the same cause.
So Kimi-K3's schema-mismatch is now observed in a THIRD run,
on a different
 driver,
hours after run 013.
That is no longer comfortably a provider window,
and it is the strongest
 argument yet for the per-stage minimum rather than waiting.
Still the user's call.

### CORRECTION, within the hour: 0.889 was measured on a degraded ensemble

The first version of the entry above,
and of the README paragraph,
presented
 0.889 as the current roster's recall.
It is not,
and the run's own log says so.

```text
recall run   Kimi-K3   312 schema-mismatches
             (refiner 96, critic 76, panel 69, editor 62, restoration-judge 9)
             GLM-5.2     3

critic stage:  72 x 5/6 heard    8 x 3/6 heard    1 x 0/6 heard
panel  stage:  64 x 5/6 heard    4 x 4/6 heard    2 x 3/6 heard
```

The critic stage NEVER reached 6/6 in this run.
One chunk was critiqued by NOBODY.
So the number describes a five-critic ensemble that occasionally fell to three,
 not the six the roster configures.

That is worth having,
because it is what the pipeline actually delivers under
 the condition it is currently in.
It is not worth calling "the current roster's recall",
and both documents now
 say which of the two it is.

TWO FURTHER FACTS,
both from the scorecard rather than inference:

-   All three misses are ONE entry,
    `Chinatsu_Suzuki`,
    which went 0 for 3 while
     the other eight entries went 24 for 24.
    An entry failing wholesale and a rate of 0.889 are different objects,
    and
     only the first is what happened.
-   That entry completed normally,
    `status=repaired` with 19 issues found,
    so
     its critics did run and did report.
    The `0/6` chunk cannot be attributed to it from the log,
    because the recall
     log carries NO per-entry markers.
    Not guessed either way;
     adding an entry marker to that driver would settle it next time.

### What this does to task 64

It removes the provider-window reading.
Kimi-K3 now shows schema-mismatch in THREE runs across two drivers,
rising:
 0 in run 012,
61 in run 013,
312 in the recall run hours later.
Every stage it sits in is affected,
and the two with no meaningful quorum,
the
 editor pair and the single refiner,
are affected worst.
The decision is still the user's,
but it should no longer wait on another pass
 to establish persistence.

## Run 014: the degradation is now the dominant fact about the pipeline

```text
DONE processed=9 of pending=45; artifacts=56/92 elapsed=53396640ms
```

Nine entries settled,
47 to 56,
one deadline casualty (`hulicaijia`).
Good throughput,
and almost none of it under the configuration the roster
 describes.

### Kimi-K3, four runs

```text
run 012        0 schema-mismatches
run 013       61
recall run   312
run 014      507
```

Run 014's critic stage reached its full roster ONCE in 166 chunk-runs
 (158 at 5/6,
5 at 4/6,
2 at 3/6,
1 at 6/6).

### What that has done to the two stages with no real quorum

```text
at 47 entries   editorDegraded=15/322   refineSilent=6/101
at 56 entries   editorDegraded=71/405   refineSilent=34/129
```

So run 014 alone contributed 56 degraded editor chunks and 28 silent refine
 slices.
The naturalness lane has now produced nothing at all for two consecutive passes:
 `entriesWithRewrites` is still 15,
unchanged since run 012 settled.

This is no longer a curiosity to report beside the real numbers.
A majority of the recent corpus was repaired by ONE editor of two,
and the
 milestone-two repair figures were measured when both answered.

### The evidence guard fired, and was right

`score-probe` refused to run at 56 entries:
 an envelope carried disagreeing probe copies.
The guard was correct and the CALLER was wrong.
Envelope ids are derived from the text they cover,
so they are unique within a
 document and not across a corpus,
and `summarizeProbeTelemetry` collapsed on
 the id alone across every artifact.
Two entries containing the same wording produced one id for regions serving
 different issues,
and the guard caught them disagreeing about which.

Measured before fixing,
because "how wrong were the old numbers" is the first
 question a reader will have:
 exactly TWO envelope ids span more than one entry,
and the old global collapse
 lost two regions out of 848.
No figure reported so far was materially wrong.
What the bug did was break the tool outright the first time a colliding pair
 disagreed,
and collisions only get likelier as the corpus grows.

The summary now takes readings grouped by entry and keys on the pair.
A test pins it:
two entries sharing an envelope id count as two regions.

## The model was never broken: a two-character prefix cost four runs

Kimi-K3 began emitting a `|>` channel marker in front of its JSON.
The JSON behind it was correct and complete every time.

```text
hf:zai-org/GLM-5.2      ok   {"count": 2, "first": "Mittens"}
hf:zai-org/GLM-4.7-Flash ok  {"count": 2, "first": "Mittens"}
hf:Qwen/Qwen3.6-27B     ok   {"count": 2, "first": "Mittens"}
hf:moonshotai/Kimi-K3   schema-mismatch   |>{"count":2,"first":"Mittens"}
hf:nvidia/...Nemotron   ok   {"count": 2, "first": "Mittens"}
hf:openai/gpt-oss-120b  ok   {"count": 2, "first": "Mittens"}
```

That single prefix produced 0,
then 61,
then 312,
then 507 schema-mismatches
 across four runs,
in every one of the five roles Kimi-K3 holds,
and everything
 attributed to "the degradation" in this document traces to it:
 the editor pair collapsing to one voice on 71 of 405 chunks,
 the naturalness lane silent on 34 of 129 slices,
 run 014's critic stage reaching a full roster once in 166 chunk-runs,
 and task 51's recall measured on an effectively five-critic ensemble.

### Why four runs went by without anyone seeing it

`schema-mismatch, voice lost` is where THREE different faults arrive wearing one
 label:
truncated thinking,
content that is not JSON,
and JSON the guard
 rejected.
`synthetic-client.ts` does distinguish them,
and says which at DEBUG level.
A corpus run records none of that.
So the logs could name the model and the stage and never the cause,
and four
 passes of evidence pointed at a model that was answering correctly.

THE DIAGNOSIS TOOK ONE CALL once the right question was asked.
`mise run //package/module/translation-repair:model-health` (moved on 2026-10-18 to
`//package/translation-repair/bench:model-health`) asks every roster
 model one trivial structured question and prints the raw reply.
Reach for it FIRST the next time a model looks dead.

### The fix, and what it deliberately does not do

`stripChannelMarker` removes a marker from a known list,
and only when what
 follows opens a JSON value.
A general "skip forward to the first brace" rule would have worked here and
 would also swallow a model that prefixes an apology before refusing,
turning
 content the refusal detector exists to classify into a silent parse success.
Verified live:
the identical call that returned `schema-mismatch` returns `ok`.

### What this does NOT fix

`stage-quorum.ts:154` still computes `Math.ceil(rosterSize / 2)`,
so a two-model
 editor roster still reaches quorum on ONE voice.
That is why one model's trouble could halve the ensemble silently,
and it stays
 true of whichever model has trouble next.
The user chose to widen the editor and refiner rosters and to switch them to
 `full-roster` retry,
but chose it believing Kimi-K3 was dead,
and that choice
 included dropping Kimi-K3 from both stages.
That specific membership change is now wrong.
Re-confirm before acting:
the premise changed,
not necessarily the decision.

## Every fan-out stage now has a quorum one voice cannot meet

Landed 2026-08-12 on the user's rule that the system must not have single-model
 failures.

```text
             before            after
editors      2 (quorum 1)      3 (quorum 2)
refiners     1 (quorum 1)      3 (quorum 2)
checkers     3 (quorum 2)      3 (quorum 2)
retry        quorum            full-roster on editor and refine
```

The arithmetic is the whole point.
`ceil(rosterSize / 2)` is ONE on a roster of two and cannot fail at all on a
 roster of one,
so the old editor pair could ship a repair written by a single
 model while reporting a met quorum.
That is not a bug in the stages;
 it is a quorum rule sized for a six-model critic panel being applied to a pair.

DEVIATION FROM WHAT WAS APPROVED,
recorded rather than buried:
 the option the user chose said refiners two.
Two leaves the quorum at one,
which would not have achieved what the change is
 for,
so refiners are THREE.
One line to revert if that is unwanted.

GLM-4.7-Flash takes the third editor and refiner seat because the constraints
 leave no alternative:
checkers must exclude every editor and refiner,
judges
 need two disinterested seats,
and the other three models hold the checker
 roster.
It is the model that most often loses its voice,
which now argues FOR seating it
 there:
a third editor that sometimes drops still leaves two,
while the same
 model among the checkers would cost proof rather than coverage.

Verified rather than assumed:
 `assertJudgeableEditorRoster` and `assertCheckerIndependence` both pass,
all
 three producing rosters sit at three voices with a quorum of two,
and three
 disinterested judges remain for each.

`pass9-run-001` is running on this configuration,
and is the first pass with
 the channel-marker fix,
the widened rosters,
and full-roster retry all in place.
Read its ROSTER line first:
 `editorDegraded` should collapse toward zero,
and if it does not,
the cause is
 something this session has not found.
