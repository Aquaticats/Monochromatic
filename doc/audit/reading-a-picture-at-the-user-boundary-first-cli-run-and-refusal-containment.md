# Reading a picture at the user boundary: the first CLI run, the refusal shape and the containment

Part of [Reading a picture at the user boundary](reading-a-picture-at-the-user-boundary.md),
which lists every file of this record.

## What the first real CLI run showed

`corpus-pass --only wangzihao980` into a throwaway runs directory,
which is the pipeline as it actually ships rather than a probe around it.

The wiring reaches production:
`gathered 6 of 6 pictures`,
then `reading 6 pictures for this document`,
then six distinct readings persisted under the `picture.` namespace.
Their verdicts:

```text
corroborated                          overlap 0.565   reading lengths 39/27
corroborated                          overlap 0.565   reading lengths 39/27
unavailable   readers-disagree        overlap 0.087
unavailable   one-reader-only         (Qwen3.6-27B: too-short)
unavailable   no-reader-available     (both readers: reads-as-refusal)
unavailable   no-reader-available     (too-short; reads-as-refusal)
```

Three observations worth keeping.

The screen is doing real work rather than passing everything.
Four of six pictures produced no usable reading,
and each refusal names which reader failed and how.

Corroboration is doing real work too.
One pair was compared and REFUSED at 0.087,
which sits inside the 0.000 to 0.129 band measured for readings of different pictures.
That is the mechanism separating on live traffic rather than on a sample.

THE TWO CORROBORATED ROWS WERE NOT READINGS.
Both readers had declined,
in different words,
and the two declines agreed.
That finding and its fix have their own section;
what belongs here is that the figure alone did not reveal it.
An overlap of 0.565 sits between the two measured bands
and reads like a weak but real agreement,
which is exactly what it is:
two short texts that genuinely resemble each other.
The number was right and the conclusion drawn from it was wrong,
because agreement about a picture presumes both parties were describing one.
Reading the stored text was what settled it,
and no threshold could have.

## Two refusals corroborated each other, and the fix is a shape rather than a list

The corroborated pair recorded above was checked rather than trusted,
and it was not a reading of anything.
Both entries hold this:

```text
hf:Qwen/Qwen3.6-27B      "There is no text visible in this image."
hf:moonshotai/Kimi-K3    "No legible text is visible."
```

Two models declining to read,
agreeing with each other at 0.565 trigram overlap,
and marked `corroborated`.
Nothing downstream could tell:
a corroborated reading travels to the translator and the judge
under the heading `WHAT THE PICTURES HERE SAY, transcribed by two readers that agreed`,
so both sheets would have asserted
that the picture says `There is no text visible in this image.`

That is worse than no reading.
A refused picture is silent,
and silence is what the design is built to fall back to.
An asserted falsehood is evidence pointing the wrong way.

### Why the existing screen missed it

The per-reading screen carried a phrase list including
`no text is visible` and `no visible text`.
The first reply misses by WORD ORDER.
The second misses because `legible` sits between `no` and `text`.
Each slipped by exactly one word.

The list had also lost its safety net without anyone noticing.
Its own comment said "the anchor clause catches most of what it misses",
and the anchor clause was deleted earlier the same day
for refusing correct readings.
Removing a clause left the clause that depended on it standing alone,
which is the ordinary way a screen quietly narrows.

### The replacement, and why it is not a longer list

`src/reading-refusal.ts` tests SHAPE rather than wording.
A refusal talks about the picture:
it negates,
it names the picture or its text,
and it is a sentence rather than a passage.
A transcription reproduces what the picture holds and does none of those.
So a reading reads as a refusal when all three hold:
at most 160 characters after trimming,
containing a negation word,
and containing a word for the picture or its text.

All three are required because each alone refuses real readings.
The word lists are compared against whole words from a linear scan,
so `not` cannot fire inside `note`.

### Measured, both directions

Positive control,
the two replies that caused this:
both now refused,
clause `reads-as-refusal`.

Negative control,
the six real transcriptions kept from the boundary probe:
all six usable,
at 390,
394,
448,
454,
590 and 632 characters.
Between them they contain
ZERO English negation words and ZERO picture words,
so neither list can reach them even before the length bound is consulted.
Their shortest is more than twice the bound.

The separation is not marginal on this sample,
and the sample is small enough to say so plainly:
six real readings and two refusals.
What makes it worth shipping is not the count
but that the two populations differ in kind rather than in degree.
A Chinese transcription has no occasion to write the English word `image`.

## Why the reading stage alone could fail an entry

Both CLI defects share a root,
and naming it is worth more than either fix.

Every other model-calling stage in the pipeline goes through `attemptStageCall`
(`src/stage-call.ts`),
which wraps the exchange and ends with:

```ts
  catch (error) {
    // Aborts must always win so user steering can stop a fan-out.
    if (signal.aborted)
      throw error;
    l.warn(`${stage} ${modelId}: ${String(error,)}, voice lost`,);
    return { heard: false, };
  }
```

A model that fails loses its voice;
the stage carries on with the voices it has;
an abort still travels.
That is why `translate-repair.ts` can call `Promise.all` over its candidates
without any of the exposure the reading stage had:
each call inside it is already contained.

`readImageAsset` calls `client.chatText` DIRECTLY.
It is the only stage that does.
So it inherited none of that,
and a reader that threw
took its partner's reading,
the picture,
the document and the entry with it.

The fix places the same two rules at `readImagePair`,
which is the reading stage's own fan-in:
`Promise.allSettled` so a failure costs one reading,
then `signal.throwIfAborted()` so a stop still wins.
Deliberately the same shape and the same order as `attemptStageCall`
and as `runStageRound`,
rather than a third way of saying it.

Routing readings THROUGH `attemptStageCall` was considered and not done.
It is built around a JSON response format and a validator,
and a transcription is neither,
so the fit would be a parameter that means
"skip most of this function".
The containment is what matters and it is now stated in both places.

WHAT TO CHECK IF ANOTHER STAGE IS ADDED:
whether it calls the client directly.
That question,
asked once,
would have caught both of these before a run did.
