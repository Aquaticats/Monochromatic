# Reading a picture at the user boundary: the settle-path run and the whole-corpus census

Part of [Reading a picture at the user boundary](reading-a-picture-at-the-user-boundary.md),
which lists every file of this record.

## The settle path, exercised at the tip

Everything in [the first probes](reading-a-picture-at-the-user-boundary-first-probes-and-attachment.md),
[the first CLI run](reading-a-picture-at-the-user-boundary-first-cli-run-and-refusal-containment.md),
[the OCR census file](reading-a-picture-at-the-user-boundary-ocr-census-store-proof-and-ocr-gate.md)
and [the AVIF refusal file](reading-a-picture-at-the-user-boundary-avif-refusal-and-the-size-cap-that-was-ours.md)
was measured either in a unit test or in a standalone probe.
Neither reaches `settleEntry`,
so neither can show that a picture reading arrives where the lanes read it.
`corpus-pass -- --only wangzihao980` into a throwaway runs directory closes that gap
for the gate and for the shape screen.

NOT FOR THE CEILING,
and the claim that it did was wrong when first written here.
This entry's largest asset is `Word1.webp` at 71288 bytes,
and every other one is smaller,
so nothing in this run comes near the 294912 the old estimate enforced
and nothing in it could have been refused by that estimate either way.
The ceiling's evidence remains the direct `readImageAsset` calls recorded in
[the AVIF refusal file](reading-a-picture-at-the-user-boundary-avif-refusal-and-the-size-cap-that-was-ours.md),
which sent `gqt/photo1.webp` at 1274028 bytes and got 2631 characters back.
A settle-path witness for the ceiling needs an entry that actually carries an oversized asset.

```text
gatherEntryPictures  gathered 6 of 6 pictures for wangzihao980
readImageWithOcr     picture1.webp: no text (0 characters, under 16)
readImagePair        picture1.webp: no text to read, so no model was asked (0 characters)
readImageWithOcr     picture2.webp: no text (0 characters, under 16)
readImagePair        picture2.webp: no text to read, so no model was asked (0 characters)
readImageWithOcr     picture3.webp: no text (0 characters, under 16)
readImagePair        picture3.webp: no text to read, so no model was asked (0 characters)
readImageWithOcr     picture4.webp: no text (0 characters, under 16)
readImagePair        picture4.webp: no text to read, so no model was asked (0 characters)
readImageWithOcr     picture5.webp: no text (0 characters, under 16)
readImagePair        picture5.webp: no text to read, so no model was asked (0 characters)
readImageWithOcr     Word1.webp: read 205 characters without a model
readImageAsset       hf:moonshotai/Kimi-K3 read Word1.webp but the reading was refused: reads-as-refusal
```

The five textless pictures were decided in 0.9 seconds between them,
with no model call at all.
Under the previous order each would have gone to two vision models,
and `picture1.webp` through `picture5.webp` are precisely the assets whose two refusals
corroborated each other at 0.565 and would have been asserted to the translator and the judge
as what the picture says.
The gate and the screen now catch that case twice over,
at different layers,
on real content.

`Word1.webp` is the counterexample that keeps the gate honest:
it carries text,
the deterministic reader found 205 characters of it,
and the picture went to the roster exactly as before.
A gate that refused it would be a gate that had learned the wrong lesson from the photographs.

## Two defects the store and the stop were hiding

### Resume rejected the verdict it had just written

`isPairedReading` checks the discriminant first and knew `corroborated` and `unavailable`.
The `no-text` kind was added to `PairedReading` and not to the guard,
so every record carrying it was rejected on resume:
read again,
written back,
rejected again on the pass after that,
forever.

The cost is bounded but the silence is not.
119 of 191 pictures end at `no-text`,
so two thirds of the picture work was re-done on every resume,
and a run that re-does work reports exactly what a run with nothing to resume reports.
No log line,
no finding,
no counter moved.

It was found by writing the tests the package owed rather than by reading the code,
and the twelve cases were committed red first so the record shows what they caught:

```text
RESUMES a no-text verdict
  AssertionError: expected undefined to deeply equal { kind: 'no-text', characters: 3 }
REFUSES an unavailable record whose kept readings are malformed
  AssertionError: expected { kind: 'unavailable', ...(3) } to equal undefined
```

The second was unlooked for.
The kept readings exist so a disagreement can be diagnosed rather than only counted,
and the guard accepted any junk in that field because the field is optional.
Optional is not unchecked.

### The gate outran the stop

`readImagePair` checked the abort at the fan-in,
below the gate.
The gate returns early for `no-text`,
so that check was unreachable for exactly the two thirds
of pictures the gate decides.
A run told to stop therefore kept spawning a decoder and tesseract per picture,
and returned verdicts that were then persisted.

Removing the new `signal.throwIfAborted()` reproduces it:

```text
AssertionError: expected 'returned no-text' to equal 'AbortError'
```

Which is the sharper reading of the defect.
The cost of the subprocesses was the obvious complaint;
the real one is that a stopped run was still producing answers.

## A refusal is a roll, and it was costing two readings in three

The settle-path run above ended `Word1.webp` at `one-reader-only`:
the deterministic reader had found 205 characters on it,
`hf:Qwen/Qwen3.6-27B` transcribed 384,
and `hf:moonshotai/Kimi-K3` declined to read it,
so nothing corroborated and a good transcription was discarded.

An earlier probe had both models reading that same asset at 390 and 394 characters,
corroborating at 0.643.
Two runs,
opposite outcomes,
identical input.
That is not a fact about the picture,
so it was measured:
six asks per model,
same bytes,
same instruction.

```text
hf:Qwen/Qwen3.6-27B     read read read read read read      6 of 6   376 to 397 characters
hf:moonshotai/Kimi-K3   refused refused read read refused refused
                                                            2 of 6   377 and 403 characters
```

The picture is not ambiguous and the readings are not marginal.
One model transcribes it every time in a tight band,
and the other transcribes the same thing when it does not decline.

WHY THAT COSTS MORE THAN THE RATE SUGGESTS.
Corroboration needs BOTH readers,
and the vision sub-roster is exactly two because the provider offers exactly two models that read images,
`syn:large:vision` and `syn:small:vision` being aliases of those same two.
So the pair's success rate is the WEAKER reader's read rate,
not an average of the two.
A reader declining two asks in three does not cost a third of the readings.
It costs two thirds of them.

At the measured one-in-three,
asking again is worth a great deal and costs little:
one ask retains 33 readings in 100,
two retain 56,
three retain 70,
four retain 80.
Asking stops at the first reading,
so the expected cost at a limit of four is about 2.4 asks,
and it only ever fires on the pictures the deterministic gate already passed,
60 of 191 here.

SCOPED TO THE REFUSAL CLAUSE ALONE.
A model that does not read images,
a picture too large to send,
and an empty reply
are properties of the input or the roster:
asking again spends a call to be told the same thing.
Only `reads-as-refusal` has been measured to vary between identical asks,
so only it is re-asked.

### The same entry, after the change

```text
readImageWithOcr   Word1.webp: read 205 characters without a model
readImageAsset     hf:moonshotai/Kimi-K3 read Word1.webp but the reading was refused: reads-as-refusal
readPastRefusal    hf:moonshotai/Kimi-K3 declined Word1.webp, asking again (2 of 4)
readImageAsset     hf:Qwen/Qwen3.6-27B read Word1.webp: 389 characters
readImageAsset     hf:moonshotai/Kimi-K3 read Word1.webp: 403 characters
readImagePair      Word1.webp: corroborated by 2 readers at overlap 0.653
```

The reading the previous run threw away is the reading this run keeps.

## Resume, proved on the store rather than on the type

The `no-text` verdict is the kind the store used to reject,
and the resume proof that existed predated the kind entirely,
so the case that mattered had never been exercised on a real store.

Stopping the run above once its pictures were settled and starting it again
into the SAME runs directory:

```text
gatherEntryPictures   gathered 6 of 6 pictures for wangzihao980
readDocumentPictures  reading 6 pictures for this document
readDocumentPictures  picture1.webp: resumed, no-text
readDocumentPictures  picture2.webp: resumed, no-text
readDocumentPictures  picture3.webp: resumed, no-text
readDocumentPictures  picture4.webp: resumed, no-text
readDocumentPictures  picture5.webp: resumed, no-text
readDocumentPictures  Word1.webp: resumed, corroborated
```

Zero `readImageWithOcr` lines in the whole run,
against six in the run that produced these records.
Before the guard learned the kind,
those five would have re-decoded and re-read on every pass,
written the same records back,
and had them rejected again,
while the log looked exactly like this one minus the six resumed lines.

## The ceiling, witnessed at the settle path on the asset that needed it

The withdrawal above said a settle-path witness for the ceiling needs an entry that carries an oversized asset.
`gqt` does:
`photo1.webp` is 1274028 bytes,
4.3 times the 294912 the old estimate enforced.

WHAT COUNTS AS THE WITNESS,
since the cap was removed rather than raised.
The absence of a `too-large-for-model` line proves nothing:
it is true by construction now.
The discriminating evidence is a MODEL-PRODUCED reading of that asset through the settle machinery,
and there is one:

```text
readImageWithOcr  photo1.webp: read 2329 characters without a model
readImageAsset    hf:Qwen/Qwen3.6-27B read photo1.webp: 2718 characters
readImageAsset    hf:moonshotai/Kimi-K3 read photo1.webp: 2748 characters
readImagePair     photo1.webp: corroborated by 2 readers at overlap 0.989
```

At 0.989 this is the strongest corroboration recorded anywhere in this audit,
on the single largest text-bearing asset in the corpus,
which the pipeline used to refuse without asking anyone.

## The gate's marginal band, and what the re-ask costs there

The same run turned up the case that the re-ask makes expensive.
`photo3.webp` cleared the gate on 19 characters,
one above `MIN_OCR_CHARS`,
and both readers declined every one of their four asks:

```text
readImageWithOcr  photo3.webp: read 19 characters without a model
readImageAsset    hf:Qwen/Qwen3.6-27B ... reads-as-refusal        (four times)
readImageAsset    hf:moonshotai/Kimi-K3 ... reads-as-refusal      (four times)
readImagePair     photo3.webp: 0 of 2 readers produced a reading, so nothing is corroborated
```

Eight calls,
no reading.
That is not the re-ask misbehaving:
two readers refusing eight asks between them is strong evidence the picture carries nothing worth reading,
and the deterministic reader's 19 characters were noise.
It is the GATE letting a marginal picture through,
and the re-ask multiplying what that costs
from two calls to eight.

HOW MANY PICTURES SIT IN THAT BAND,
over the whole corpus:

```text
0 characters        119
1 to 15              12   below the gate, never sent
16 to 31              9   the marginal band
32 to 63              8
64 to 127            11
128 or more          32
```

So the exposure is nine assets,
and the question the number cannot answer is whether they
behave like `photo3.webp` or carry real text that a higher threshold would throw away.
Raising `MIN_OCR_CHARS` on one observation would be trading a measured cost for an unmeasured loss.
The corpus-wide re-run records the outcome of each of those nine,
which decides it on evidence rather than on this one case.

## The whole corpus, read through the production call

Everything above,
and the runs in [the first CLI run](reading-a-picture-at-the-user-boundary-first-cli-run-and-refusal-containment.md) and [the OCR census file](reading-a-picture-at-the-user-boundary-ocr-census-store-proof-and-ocr-gate.md),
is one entry at a time.
The reading lane runs before either translation lane
and takes nothing from them,
so it can be measured on its own:
`~/temp/agent/reading-census.mjs` walks all 93 entries,
prepares each pair,
gathers its pictures
and reads them through the same `readDocumentPictures` call the pass uses.
47 entries carry pictures,
191 distinct assets,
one entry lacks a complete pair.

```text
no-text        131
corroborated    54
unavailable      6   no-reader-available 4, one-reader-only 1, readers-disagree 1
```

THE 131 RECONCILES WITH THE 119 recorded in [the OCR census file](reading-a-picture-at-the-user-boundary-ocr-census-store-proof-and-ocr-gate.md)
rather than contradicting it.
That figure came from a standalone tesseract sweep counting assets with zero characters.
The gate's own verdict adds the 12 assets that returned 1 to 15 characters,
which are below `MIN_OCR_CHARS` and therefore textless as far as the pipeline is concerned.

CORROBORATION OVERLAPS,
over all 54:

```text
min 0.544   p10 0.727   median 0.930   max 1.000
below 0.50: 0     below 0.40: 0
```

The threshold is 0.30 and nothing lands within 0.24 of it.
The gap between the threshold and the weakest real corroboration is wider than the threshold itself,
so the number is not doing marginal work on this corpus and moving it would change nothing.

THE ONE DISAGREEMENT IS NOT A NEAR MISS EITHER,
which is the other half of that claim
and the half a distribution over corroborations cannot show.
`noname3031/postcard8.webp` is the corpus's only `readers-disagree`.
The kept-readings field exists so a case like it can be diagnosed rather than merely counted,
and read for shape alone it says this:

```text
deterministic reader   122 characters
hf:Qwen/Qwen3.6-27B     50 characters
hf:moonshotai/Kimi-K3   31 characters
overlap                  0.069
```

0.069 sits inside the 0.000 to 0.129 band measured for readings of DIFFERENT pictures.
The two readers agreed with each other about as much as they would have
had they been shown unrelated images,
so the refusal is the threshold doing exactly its job
rather than a boundary case it decided arbitrarily.

The shape also says why.
The deterministic reader found 122 characters,
so the picture is not blank and the gate was right to pass it,
yet both models returned readings far shorter than that and unrelated to each other.
That is the signature of text present but hard to read,
where each reader guesses differently instead of failing outright.
A single reading here would have been shipped with no sign anything was wrong,
which is the entire argument for requiring two.

## The marginal band decides itself, and the answer is to leave it alone

Nine assets return 16 to 31 characters from the deterministic reader,
the band `gqt/photo3.webp` sits in after costing eight calls for nothing:

```text
Chinatsu_Suzuki/photo4.webp   17   corroborated 1.000
Chinatsu_Suzuki/photo6.webp   23   corroborated 0.924
Susiethegamer/photo9.webp     22   corroborated 0.616
Uekawakuyuurei/IMG_1308.webp  24   unavailable  no-reader-available
Uekawakuyuurei/img197.webp    18   unavailable  no-reader-available
Uekawakuyuurei/img370.webp    24   unavailable  no-reader-available
gqt/photo3.webp               19   unavailable  one-reader-only
hakureico/photo2.webp         24   corroborated 0.767
mone/messages2.webp           17   corroborated 0.727
```

Five of nine corroborate,
one at 1.000.
Raising `MIN_OCR_CHARS` to clear the four failures would throw away five real readings,
including a perfect one,
so the threshold stays where it is.
The eight wasted calls on `gqt/photo3.webp` are the price of the five that work.

## The re-ask, measured at corpus scale, refutes what one picture suggested

`REFUSAL_ASK_LIMIT` was set from six asks on `Word1.webp`,
where one reader declined four times
in six.
Over the corpus that rate does not hold:

```text
reader and picture pairs that reached a model   119
  read on the first ask                         109
  read only after a re-ask                        1
  never produced a reading                        9

the 9, by the clause each pair exited on:
  reads-as-refusal, through all four asks        6
  empty reply, never re-asked                    1
  too-short, never re-asked                      2
```

SO A DECLINE IS USUALLY ABOUT THE PICTURE,
not the roll,
which is the reverse of the conclusion
drawn from one asset.
Seven pairs ever returned `reads-as-refusal` and six of them refused through all four asks.
The single recovery,
`gqt/photo3.webp` on its third ask,
moved that asset from
`no-reader-available` to `one-reader-only`,
which is not a usable reading either.

ONLY TWO OF THE FOUR `no-reader-available` VERDICTS ARE STUBBORN REFUSALS.
`Uekawakuyuurei/IMG_1308.webp` and `dogesir_/photo2.webp` are:
both readers refused all four asks on each.
The other two never exercised the re-ask at all.
One reader of `Uekawakuyuurei/img197.webp` returned an empty reply while the other refused four times,
and both readers of `Uekawakuyuurei/img370.webp` exited on `too-short` after a single ask.
Neither clause is a refusal,
so `readPastRefusal` was right not to ask again.
The distinction matters because the log line reads
"read X but the reading was refused: <clause>" for every screen,
so a tally that keys on the sentence rather than the clause
counts length screens and empty replies as refusals,
and reports the roster as more stubborn than it is.

`Word1.webp` is genuinely a roll:
it declined into `one-reader-only` on one run,
corroborated at 0.653 after a re-ask on the next,
and corroborated at 0.597 on first asks
during this census.
It is the exception rather than the sample.

WHAT THAT COSTS AND WHY THE LIMIT STAYS.
Re-asking spent 20 extra calls across the whole corpus,
against 119 first asks.
That is cheap enough to keep for the roll case,
which does occur and did produce a corroboration on a real settle-path run.
The projection it replaced,
"four asks retain four readings in five",
is refuted and has been removed from the source comment and the README.

THE METHOD LESSON IS THE SAME ONE AS THE BYTE CAP,
one layer up.
Six asks on one picture is a real measurement of that picture and not of the corpus,
and the arithmetic done on it was correct.
A rate measured on a single item cannot be multiplied out to a population,
however carefully the item was measured.
