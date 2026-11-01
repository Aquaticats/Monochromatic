# Translation repair history 2026-08-12 to 2026-08-15: the introduced-defect probe's blindness, measured and then overturned

The false-negative measurement of the probe over every probed region,
the causes eliminated,
the labelling arm and the paired re-probe on damaged regions,
the control arms and their length matching,
the move of the pre-existing-issue defence from the prompt to the screen,
and the re-reading of the five damaged regions against the Chinese that withdrew the earlier probe figures.

Part of [Translation repair history: 2026-08-12 to 2026-08-15](translation-repair-history-2026-08-12-to-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The probe's blindness, measured 2026-08-12

### The false-negative rate over every probed region

Task `#66` asked for the introduced-defect probe's false-NEGATIVE rate.
Measured over all 857 distinct probed regions in the 56 settled artifacts of
 `node_modules/.monochromatic/translation-repair-runs/artifacts`:

```text
prober verdicts        2571  = 857 regions x 3 probers, exactly
no-introduced-defect   2438  (94.8 percent)
corroborated             86
removal-corroborated     40
contradicted              2
unanchored                3
uncertain                 2
regions where every prober found nothing   743 of 857 (86.7 percent)
```

### Three causes eliminated, so they do not get re-investigated

Lost prober voices absorbed into `noneFound`.
NO:
all 2626 recorded probe blocks read 3/3 heard,
and the verdict total is
 exactly three per region,
so no region is missing a voice.

The deterministic screen erasing true claims before they are recorded.
NO:
every one of the 131 raised claims persists in the artifact with its
 `admissibility`,
and only 5 were rejected,
2 contradicted and 3 unanchored.
`screenIntroducedDefects` is not where the damage disappears.

The sensitivity control being unrepresentative in size or issue count.
NO:
production regions have median `before` length 55 characters,
p90 115,
and
 median 1 issue per region,
p90 7.
The `OMITTING_REGION` fixture is 82 characters with 1 issue,
inside both
 distributions.

### What survives

The probers genuinely cast `no-introduced-defect-found`,
and their raise rate
 barely moves with how much text the edit removed:

```text
regions deleting over half their text   0.060
regions at 0.50 to 0.80 of before       0.057
regions at 0.80 to 1.20                 0.032
regions that grew                       0.066
```

A probe that could see removal damage would show a steep gradient there.
It shows none,
which is the signature of an instrument whose output does not
 depend on its input.

### Live hypothesis, and the experiment built for it

Each region reaches the prober with its accepted issues rendered under
 `PRE-EXISTING DEFECTS THIS EDIT TARGETED (these are NOT your findings)`,
and
 the rules forbid reporting one.
When an accepted issue is a FALSE POSITIVE calling source-supported content an
 unsupported addition,
the editor deletes that content and the prober is told
 the deletion was the repair.
The damage is then invisible by construction,
which would put probe blindness
 DOWNSTREAM of detection precision rather than in the probe.

`probe-sensitivity` gained a labelling arm that holds the deletion fixed and
 moves only the label,
cat fixtures only,
in
 `src/corpus-run/probe-sensitivity-input.ts`:

-   `deletion/unlabelled` deletes source-supported text,
    prior issue unrelated.
     A working probe reports damage.
-   `deletion/mislabelled` deletes byte-identical text,
    prior issue falsely
     calls it an addition.
    A working probe still reports damage,
    because the
     prober is shown the original.
-   `deletion/licensed` deletes content the original genuinely lacks,
    truthfully
     labelled.
    Silence is correct,
    and a claim here would disqualify the probe
     as a gate.

A gap between the first two measures how far a false accepted issue can talk the
 probe out of seeing real damage.

### Consequence for the re-plan if the hypothesis holds

`#66` stops being a separate defect and becomes a second symptom of the
 precision gap,
and the planning document's proposal to separate repair SAFETY
 from QUALITY needs the deterministic preservation check to carry the safety
 verdict,
since no model-based probe can be trusted while it is fed accepted
 issues as ground truth.

### The labelling arm ran, and it refutes the hypothesis it was built for

```text
deletion/unlabelled   prior = unrelated issue        3/3 removal-corroborated
deletion/mislabelled  prior = FALSE addition claim   2/3 removal-corroborated, 1 none
deletion/licensed     prior = TRUE addition claim    3/3 none, which is correct
```

Read them in order.
The probe DOES see a deleted clause the source supports.
A false accepted issue naming that clause costs exactly one voice of three,
so
 the labelling effect is real and modest rather than silencing.
The negative control is perfect,
and it is the one that matters most:
the probe
 stays quiet when the deletion really is licensed,
so it is reading the original
 rather than flagging every deletion.

So probe blindness is NOT simply downstream of detection precision.
That was the leading hypothesis and the measurement does not support it.

Two other candidate explanations died in the same pass:

-   Region count per call.
    Production sends one region per call in 848 of 854
     calls,
    which is exactly what the fixture sends.
-   Prior-issue count.
    Raise rate rises slightly with more listed issues,
    0.044
     at one issue to 0.061 at six or more,
    which is the wrong direction for a
     list that silences.

### What is left, and it is now a narrow question

The fixture damage is a clean clause deletion with a verbatim span present in
 BEFORE and absent from AFTER.
Production damage is usually a compression or rewrite,
and judging it needs the
 cross-lingual question of whether the deleted English was licensed by Chinese
 prose elsewhere.
The probe scores 3/3 on the first and near zero on the second.

Probers also disagree enormously about how often anything is worth raising,
 measured across 857 regions each:

```text
hf:openai/gpt-oss-120b                    81 claims   rate 0.095
hf:nvidia/NVIDIA-Nemotron-3-Super-120B    45 claims   rate 0.053
hf:Qwen/Qwen3.6-27B                        5 claims   rate 0.006
```

Qwen raised claims on both blatant fixtures,
so it is not broken;
it is far more
 conservative than the other two on real edits,
and a third of the ensemble
 contributing almost nothing drags a majority-of-three toward silence.

### Next action for `#66`

Re-probe the specific regions a human graded as damaged,
once with their
 accepted issues shown and once with them withheld.
That is the only remaining way to tell whether production damage is invisible
 because it is subtle or because the label suppresses it,
and unlike the cat
 fixtures it uses cases already known to be damaged.

### The paired re-probe on real damaged regions, and a correction

The cat fixtures said the labelling effect was modest,
one voice of three.
On
 the regions a human actually read as damaged it is total.
Five distinct regions,
rebuilt from the eight drawn positions,
each probed twice
 with the same three probers:

```text
entry              positions      issues  shown            withheld
Acheron            2+7+11+15      8       0 raised, 3 none  1 removal, 1 unanchored, 1 none
Anilovr            20             6       0 raised, 3 none  1 corroborated, 1 removal, 1 none
AmbeR_the_anpa     21             8       0 raised, 3 none  2 removal, 1 none
Chinatsu_Suzuki    37             7       0 raised, 3 none  1 corroborated, 2 none
Dethelly           43            17       0 raised, 3 none  1 removal, 2 none
```

With the accepted issues shown,
0 of 15 prober verdicts raised anything.
With them withheld,
7 of 15 raised,
and every one of the five regions drew at
 least one admissible claim.

So the earlier reading from the fixtures,
that probe blindness is not downstream
 of detection precision,
was WRONG,
and the fixture is why.
It carried ONE prior issue.
These regions carry six to seventeen,
and a region
 whose every part is labelled a known defect has nothing left that a deletion
 could damage.
The prompt conflates two instructions:
do not re-report the old defect,
and
 treat everything here as already accounted for.

### What that does NOT yet establish

Withholding the list is not obviously the fix.
The list exists because a prober shown a defective region reports the defect it
 was cut for,
and the withheld arm may be doing exactly that rather than finding
 the damage.
Nothing here separates those,
because all five regions are damaged by
 construction,
so there is no case where a raised claim would be wrong.

The missing measurement is a CONTROL:
regions from the same entries that the
 reader did NOT flag,
probed under both arms.
If the withheld arm raises claims
 just as often there,
it is re-reporting pre-existing defects and its 7 of 15 is
 worthless.
If it stays quiet there,
the label is suppressing real detections
 and the stage can be fixed.

### The shape a fix would take, if the control holds

`screenNonTranslationVotes` is the precedent:
deterministic evidence DISMISSES a
 claim rather than a prompt preventing it.
The same split applies here.
Let the
 prober run without the excusing list,
then discard deterministically any claim
 whose quoted wording falls inside a pre-existing issue's evidence span.
That
 keeps the defence against re-reporting the old defect while removing the
 blanket licence that currently silences the stage.

### The over-replacement statistic, re-derived on distinct regions

The round-three draw of 50 items sits on 40 DISTINCT regions;
six regions were
 drawn more than once.
Measuring each region's replaced text against the target-side span quoted by the
 issue that drew it:

```text
over 1.35x the DRAWN issue's span     13 of 40 regions   (19 of 50 items)
over 1.35x the WIDEST served span      0 of 40 regions
```

The second line is the one that matters,
and it is why `#67`'s framing was
 wrong.
An envelope merges OVERLAPPING and TOUCHING target-side evidence,
so a region is
 cut to cover every issue it serves,
and comparing its replacement against ONE
 of those issues measures the merge rather than the editor.
Against the widest span the region actually serves,
nothing over-replaces.

The comparison is conservative in the direction that matters.
The widest single
 served span is never longer than the union of the served spans,
so a union
 denominator would be larger,
the ratios smaller,
and the count still zero.

So the editor is not replacing far more text than the defect it was given:
it is
 replacing the envelope it was given,
and the envelope legitimately spans several
 accepted issues.
What remains true,
and is a different fault,
is that some of those replacements
 DROP source-supported content,
which is what the probe work is about.
The width
 of a replacement was never evidence of that,
and reading it as evidence pointed
 `#67` at the wrong stage.

### The control arm, run 002

Fifteen regions,
five a human read as damaged and ten unflagged from the same
 entries,
each probed with the accepted issues shown and again with them
 withheld:

```text
arm                regions  admissible verdicts  regions flagged
damaged/shown         5          0 of 15             0 of 5
damaged/withheld      5          5 of 15             4 of 5
control/shown        10          0 of 30             0 of 10
control/withheld     10          3 of 30             2 of 10
```

Two things follow,
and the first is larger than the question the run was built
 to answer.

The SHOWN arm is zero everywhere.
Across 45 verdicts covering damaged and
 undamaged regions alike,
the production configuration raised nothing at all.
A
 stage that answers identically whether or not damage is present carries no
 information,
which is a stronger statement than "it misses things".

The WITHHELD arm discriminates.
Four of five damaged regions drew an admissible
 claim against two of ten controls,
and 5 of 15 verdicts against 3 of 30.
That
 is a detector with signal rather than a prober re-reporting whatever defect it
 is shown,
which was the reading the control existed to rule out.

Run-to-run spread is real and worth carrying:
the damaged withheld arm gave 7 of
 15 in run 001 and 5 of 15 in run 002 on identical inputs,
while the shown arm
 gave 0 of 15 both times.
Five regions is a small denominator and the withheld
 rate should be read as a band,
not a point.

CAVEAT,
and it is why run 003 exists:
run 002's controls were NOT matched on
 replaced length,
and the gap is measured rather than suspected.

```text
damaged  n=5   60, 79, 99, 189, 268                          median  99, mean 139.0
control  n=10  12, 14, 32, 32, 41, 54, 63, 69, 69, 316       median  41, mean  70.2
```

Seven of the ten controls replaced less text than the SMALLEST damaged region,
 and a shorter replacement has less room to drop anything,
so the control's
 quiet is partly a statement about length.
An earlier note in this document gave the control range as 12 to 63 characters;
 that was read off the first four lines before the run finished,
and the full
 range is 12 to 316.
Control selection now orders unflagged regions by distance from the damaged
 region's replaced length in the same entry,
which removes the gap by
 construction rather than hoping it does not matter.

### Whether length-matched controls exist at all, per entry

Measured over every distinct edited region each entry has,
with the damaged
 region's own length shown for comparison and itself excluded from the pool:

```text
entry             damaged  pool  nearest available
Acheron              189      3  32, 12
Anilovr               99     20  95, 120
AmbeR_the_anpa        60     13  61, 51
Chinatsu_Suzuki       79     10  84, 62
Dethelly             268     52  316, 149
```

Four of the five entries match closely.
Acheron cannot be matched at all:
the
 whole entry carries three edited regions,
and once its damaged 189-character
 region is excluded the only candidates are 32 and 12 characters.

So run 003's control arm is well matched except for one entry,
and that entry's
 pair is a known exception rather than a silent weakness.
Excluding Acheron,
the
 controls run 51 to 316 characters against damaged regions of 60 to 268,
where
 run 002's controls had a median of 41 against a damaged median of 99.

### Pooled over three relabel runs, and a correction to "zero"

```text
arm                verdicts  admissible  rate   regions flagged
damaged/shown           45        2      0.044      2 of 15
damaged/withheld        45       18      0.400     14 of 15
control/shown           48        1      0.021      1 of 17
control/withheld        48        6      0.125      4 of 17
```

Correction first.
Runs 001 and 002 both gave 0 of 15 on the damaged shown arm,
 and that was recorded here as the production configuration raising nothing at
 all.
Run 003 gave 2 of 15 on identical inputs,
so the shown arm is not zero;
it
 is 3 admissible claims across 93 verdicts spanning both damaged and unflagged
 regions.
The substantive reading survives and is sharper with the extra run:
shown
 separates damaged from unflagged by 0.044 against 0.021,
on two claims against
 one,
while withheld separates them by 0.400 against 0.125.
Withholding the
 issue list raises the damaged rate ninefold and flags 14 of 15 damaged regions
 against 4 of 17 unflagged ones.

### The limit that decides whether this can gate anything

"Unflagged" is not "verified clean".
The controls were never read by anyone:
the
 human read 50 drawn items,
and these regions were not among them.
So the 0.125
 control rate bounds the false-positive rate only if those regions are actually
 undamaged,
which nothing establishes.

The cheap way to settle it is four items.
Exactly four control regions drew an
 admissible claim in the withheld arm,
and a human reading only those four says
 whether the withheld arm is finding damage nobody had looked at or inventing
 it.
That is a far smaller ask than another 50-item sheet and it is the last
 thing standing between this result and a gating decision.

### The prompt-to-screen move, and the limit it does NOT remove

The probe now withholds the accepted issues from the prober and dismisses a
 claim deterministically when its quoted wording restates one.
Measured on the
 labelling fixtures,
with the guard proven live by removing it:

```text
fixture                guard present            guard removed
deletion/unlabelled    3/3 removal-corroborated 3/3 removal-corroborated
deletion/mislabelled   3/3 PRE-EXISTING         3/3 removal-corroborated
deletion/licensed      3/3 none found           3/3 none found
```

The middle row is the finding,
and it is not the one the change was hoping for.
`deletion/mislabelled` deletes source-supported text under a FALSE accepted
issue naming that text.
All three probers detect the deletion,
and the screen
dismisses all three,
because the wording they quote is exactly the wording the
false issue complained about.

So moving the defence from the prompt to the screen did NOT recover detection of
 damage a false accepted issue licenses.
A deterministic check cannot:
it can
 see that a claim restates an accepted issue,
and it has no way to know whether
 that issue was right.

What the move does buy is real but narrower.
The claim is now RAISED,
SCREENED
 and RECORDED with its quoted text and its reason,
marked `pre-existing`,
where
 before the prompt stopped it being made at all and the region reported
 `noneFound`.
A count of dismissals is a measurement;
silence was not.
The
 `pre-existing` tally is now the visible price of every accepted issue the
 detector got wrong,
which ties repair safety directly to detection precision
 rather than leaving the link invisible.

`deletion/licensed` also deserves a caveat:
the probers reported nothing there
 under both conditions,
so that control passes without the screen ever being
 exercised.
It shows the probe is not flagging every deletion;
it does not show
 the screen dismisses correctly.

### Re-anchoring the question emptied the sheet, and reading the five explains why

Under the source-anchored prompt,
`probe-verify` wrote ZERO items:
44 of 45
 prober verdicts found nothing and the remaining one was unanchored.
The screen
 dismissed nothing,
so this is the probers themselves,
not the filter.

That looked like an over-correction until the five regions were read against the
 Chinese.
They are not damaged.
Each judged against the ORIGINAL rather than
 against the pre-edit English:

-   Acheron.
    Source:
    她在技术领域也颇有研究，曾经发过...视频。The BEFORE text adds
     "not just a gamer",
    "from hardware to software" and "shared her insights
     with the community",
    none of which the source says.
    The AFTER renders
     在技术领域也颇有研究 and drops the additions.
    The hi3861 video sentence is
     untranslated in BOTH texts,
    so the edit did not delete it.
    The earlier note
     in this document saying it did was wrong.
-   Anilovr.
    The BEFORE text reverses the source's sentence order and adds "On
     that day".
    The AFTER restores the order and drops the addition.
-   AmbeR_the_anpa.
    Source line is 条目贡献：UP 主纪念馆（...）with no "Bilibi".
     The BEFORE text carries "Bilibi - ";
    removing it is correct.
-   Chinatsu_Suzuki.
    Source:
    又像是哭诉又像是哀求。哀求 is PLEADING.
    The BEFORE
     text said "reminiscing",
    which is simply wrong,
    and the AFTER says
     "pleading".
    The edit fixed a mistranslation.
-   Dethelly.
    Source:
    私信里编号式地问上十几个问题。编号式地 IS "in numbered form"
     and 私信里 IS "via private messages".
    The earlier note calling those
     inventions was wrong;
    they are in the source,
    and the AFTER also restores a
     whole first half the BEFORE text omitted.

So all five are correct repairs,
and a probe reporting no introduced defect on
 them is RIGHT.

### What that overturns

The damaged set was never ground truth.
It was assembled by reading edits
 against the previous English,
which is the same reference error the prompt
 made,
so "the repairs are too broken to grade" rests on five items that read as
 damage only from that angle.

Every probe figure taken before the re-anchoring measured change rather than
 damage,
including 0.400 against 0.150 and 14 of 15.
They are withdrawn.

`#67`'s surviving half,
that the editor drops source-supported content,
now has
 no evidence behind it.
The one omission found in these five predates the edit.

What is NOT established:
that repairs are fine generally.
Five regions from four
 entries were read,
all drawn because someone thought they were bad,
and the
 probe agreeing with a re-reading of the same five is not a measurement of the
 corpus.
A fresh sample judged against the source is.
