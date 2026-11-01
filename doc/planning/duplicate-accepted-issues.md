# Duplicate accepted issues

Measured 2026-08-13 over the 56 settled artifacts in
 `translation-repair-runs`,
covering 2650 accepted issues across 54 entries.
This is a PROPOSAL for `#65`,
which asked whether duplicates count against
 precision and whether the pipeline should stop emitting them.
Nothing here is decided.

## The pipeline deduplicates on identity, and identity never repeats

```text
duplicates within one entry, extra copies beyond the first

  same issueId                  0    0.0% of accepted
  same summary text             4    0.2%
  same spans AND same category  412  15.5%
  same spans, any category      570  21.5%
```

The zero is the finding.
`issueId` is a hash of the adjudicated issue,
and the adjudicated issue carries
 the claim's summary wording,
so two critics describing one defect in two
 sentences produce two identities and both survive every gate.
Deduplication that keys on identity cannot see this and never could.

## They are the same defect, not two defects in one region

The obvious objection is that a span might genuinely carry two distinct
 problems.
Span width refutes it:

```text
span width in characters covered by the claim

  duplicate-span issues   n=570   median 60   p90 135   max 562
  first-of-span issues    n=2080  median 56   p90 123   max 564
```

The duplicated spans are the same size as everything else,
median 60
 characters.
That is roughly a sentence.
A 60-character span holding two separate omissions is possible;
170 of them are
 not.

The category pairings say the same thing.
The three commonest are a category paired with ITSELF:

```text
  170  accuracy/omission        +  accuracy/omission
   74  accuracy/addition        +  accuracy/addition
   71  accuracy/mistranslation  +  accuracy/mistranslation
   33  accuracy/mistranslation  +  accuracy/omission
   25  accuracy/addition        +  accuracy/mistranslation
   20  fluency/grammar          +  fluency/grammar
```

Cross-category pairs are the minority and some are legitimately distinct:
 a passage can be both mistranslated and emotionally flattened.
Same-category-same-span is not that.

## The cost is NOT duplicated work, and the first version of this said it was

An earlier revision claimed the editor repaired each sentence twice,
reading
 "both copies carried repair regions" as two repairs.
That was wrong,
and the correction matters more than the original claim.

```text
duplicate pairs served by the SAME repair envelope   567
duplicate pairs served by DIFFERENT envelopes          0
duplicate pairs where one copy had no envelope         3
```

Every one of them shares an envelope.
Both issues point at the same repair because that is how a merged envelope is
 recorded:
one envelope names every issue it serves,
so both copies list the
 same regions and a naive read counts one repair as two.

The pipeline therefore ALREADY collapses these at the layer where the cost
 would be,
and merging is pervasive by design:

```text
accepted issues served by one repair envelope

  median 1   p90 7   max 28
  envelopes serving more than one issue: 395 of 857  (46.1%)
```

So the editor writes one repair,
the checkers verify one repair,
and the probe
 judges one region.
No model work is duplicated.

Spread rather than concentrated:
44 of 54 entries carry at least one duplicate,
 so no single bad entry explains the issue-level count either.

## What it does to precision

The graded sheets drew issues uniformly,
so a defect claimed twice had two
 chances to be drawn.
That does not bias precision in a knowable direction,
because a duplicated
 defect is as likely to be a true positive as a false one;
what it does is
 weight the sample toward defects that attracted more than one critic.

Whether that is a distortion worth correcting depends on what the number is
 FOR.
As "what fraction of the defects we find are real",
counting duplicates is
 wrong,
since one defect is counted twice.
As "what fraction of what we ship is right",
the honest unit is the ENVELOPE
 rather than the issue,
because an envelope is what ships:
one repair serves a
 median of 1 accepted issue but a p90 of 7 and a maximum of 28.

That maximum is the part worth pausing on.
An entry whose repairs each serve many issues contributes far more issues than
 envelopes,
so an issue-denominated rate weights it far above an entry whose
 repairs serve one issue each,
without anyone choosing that weighting.
Both readings have been used in this project,
which is itself worth settling.

## Options

### Option C: count by envelope, and leave the pipeline alone

Change nothing about what the pipeline emits.
Report issue-denominated rates alongside envelope-denominated ones,
and draw
 grading samples over envelopes rather than issues.

Pros:
 targets the harm that is actually there,
which is counting rather than work;
 risks nothing,
since no stage changes behaviour;
 keeps the corroboration evidence,
which two critics independently claiming one
 span really is;
 envelope counts are already recorded in every artifact,
so this is a reader
 change rather than a pipeline change.

Cons:
 issue counts stay inflated wherever someone reads them without the envelope
 figure beside them;
 the adjudication panel still cannot see the agreement.

### Option B: collapse on exact span equality and category equality

Merge accepted issues whose spans match exactly and whose categories agree.

Pros:
 makes the issue count mean what a reader assumes it means;
 provably cannot merge defects sitting at different offsets;
 catches 412 of the 570.

Cons:
 buys no model work back,
since the envelope already merged them;
 discards the fact that two critics agreed,
unless the merge deliberately keeps
 both claim texts;
 changes what every existing measurement counted,
so prior numbers stop
 comparing for no operational gain.

### Option A: collapse on span overlap at the accept gate

The wider key:
merge where spans overlap rather than match.

Pros:
 catches all 570 including the 158 that overlap without matching.

Cons:
 everything wrong with B,
plus overlap is not identity,
so a genuine second
 defect in one sentence would be merged away and silently lost.

## Ranking

C > B > A.

This ranking is the reverse of the first revision's,
and the envelope
 measurement is why.
That revision ranked B first because duplicates looked like duplicated editor,
 checker and probe work;
once every pair turned out to share one envelope,
the
 argument for changing pipeline behaviour went with it.

**C over B** because the harm is now known to be confined to counting,
and a
 counting problem is fixed by counting differently rather than by changing what
 the pipeline emits.
B pays a real cost,
breaking comparability with every
 measurement taken so far,
and buys back no model work at all.

**B over A** because A's overlap key can merge two genuinely distinct defects
 and lose one,
and a lost defect is invisible in exactly the way this project
 keeps being bitten by.
Both share the same weak motivation;
A additionally can
 be wrong.

## What must be settled first

-   Which reading of precision the project uses,
    since the two readings
    disagree about whether duplicates should be counted at all.
-   Whether a duplicate should be kept as a CORROBORATION signal.
    Two critics
    independently claiming one span is evidence the defect is real,
    and the
    adjudication panel currently cannot see that they agreed,
    because the two
    claims arrive as unrelated issues.
