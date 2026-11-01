# Translation repair history 2026-08-19 to 2026-08-21: the reading stage's first runs, the OCR gate and the provider's limits

The four defects the first real command-line runs of the reading stage found,
the deterministic OCR gate in front of the model readers,
the provider's refusal of AVIF,
the byte cap that turned out to be self-imposed,
and the silent defects behind it:
a store guard that rejected a reading kind on resume,
a stopped run that returned a verdict,
and a test file shaped so that it registered nothing.

Part of [Translation repair history: 2026-08-19 to 2026-08-21](translation-repair-history-2026-08-19-to-21.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-19: what the first real CLI runs of the reading stage found

Four defects,
all fixed,
all found by running `corpus-pass` rather than by reading code.
Full evidence in `doc/audit/reading-a-picture-at-the-user-boundary.md`.

### `sentinel-probe` cannot verify anything in the settle path

It calls `repairTranslation` directly and never reaches `settleEntry`.
A run over a picture-bearing entry completes green while proving nothing
about picture gathering,
because the code that gathers pictures is in `pass-entry.ts`.
The honest CLI check is
`corpus-pass -- --only <ids>` into a throwaway `TRANSLATION_REPAIR_RUNS_DIR`.
Anything that claims to verify the two-lane path must go through `settleEntry`.

### A failing picture reader killed the whole entry

`readImageAsset` is the ONLY model-calling stage that calls `client.chatText` directly.
Every other stage goes through `attemptStageCall`,
which contains a failure as a lost
voice and rethrows only on abort.
So a reader that looped,
and was cut off by the client's runaway guard,
rejected
`Promise.all` in `readImagePair`,
and the rejection travelled into `settleEntry`:
`status=ERROR`,
`processed=0`,
both lanes lost.
Fixed with `Promise.allSettled` plus `signal.throwIfAborted()`,
the same shape and order
`attemptStageCall` and `runStageRound` use.

WHEN A STAGE IS ADDED,
ASK WHETHER IT CALLS THE CLIENT DIRECTLY.
That one question would have caught this before a run did.

### Two refusals corroborated each other and would have been asserted as fact

`There is no text visible in this image.` and `No legible text is visible.`
agreed at 0.565 trigram overlap and were marked `corroborated`,
bound for the translator and judge sheets under
"WHAT THE PICTURES HERE SAY,
transcribed by two readers that agreed".
Both had slipped the phrase list by a single word:
one by word order,
one because `legible` sat between `no` and `text`.
The list had also silently lost its safety net,
since its own comment claimed the anchor clause caught what it missed
and the anchor clause had been deleted earlier the same day.

Replaced with `src/reading-refusal.ts`,
which tests SHAPE:
at most 160 characters,
containing a negation word,
containing a picture word,
all three required,
whole-word matched from a linear scan.
Six real transcriptions (390 to 632 characters) contain zero of either word list.

### The repair lane was deleting picture readings

`PICTURE_READING_NAMESPACE` was added to the store and not to `CLAIMED_PREFIXES`.
The repair lane is defined by SUBTRACTION,
as everything not claimed,
so it adopted `picture.*.json` and its discard deleted them
while logging that it discarded its own slices.

This handover already warned about this exact class,
and it had bitten three times.
This was the fourth.
The cure is now structural rather than a warning:
`slice-cache-namespace.unit.test.ts` walks every namespace the package defines
and fails if one is unregistered,
so a fifth cannot be added silently.

### Where the reading stage stands

A picture in this corpus is usually a photograph of a person,
not a document,
so most produce no reading and that is CORRECT rather than a miss.
Deterministic OCR confirms it:
on a smoke test of six assets,
tesseract read the three the models read,
at comparable length,
and found nothing in the three the models declined.

WHAT IS NOT YET SETTLED,
and the next thing to build:
the owner ruled that deterministic OCR should be tried FIRST,
that oversized assets should be re-encoded to AVIF at best quality rather than downscaled,
and that a picture with no text can simply be ignored.
`tesseract` (with `chi_sim`),
`avifenc` and `magick` are all installed.
An OCR-first order would also settle a real worry:
the same picture corroborated at 0.643 in one probe and disagreed at 0.087 in a CLI run,
so model readings are less repeatable than a five-pair sample suggested,
and a deterministic party on one side of the comparison halves that variance.

## 2026-08-19, later: the deterministic reader, and what the provider will not take

### OCR is a gate in front of the model calls, not a third reader

`src/image-ocr.ts` reads a picture with `tesseract -l chi_sim+eng` before any
model is asked.
`readImagePair` takes it as an injected `readOcr` collaborator,
required rather than defaulted,
so a unit test that forgets to supply a stub is
a TYPE ERROR instead of a slow test that shells out to whatever the machine has.

It gates rather than votes,
and that was settled by measurement rather than
preference.
Against the model readings already on record its trigram overlap is
0.019 and 0.023 on `Word1.webp` and 0.096 and 0.111 on `intro.webp`,
while those
same models agree with each other at 0.643 and 0.785.
It is not missing the
text:
on `Word1.webp` it returns 405 characters against their 390 and 394.
It
reads the same text and gets the GLYPHS wrong,
which leaves length intact and
destroys overlap.
Letting it vote would refuse readings that are fine.

What it is reliable at is PRESENCE,
six of six against the models in both
directions.
So:
no text found,
no model asked,
and the verdict is a new
`no-text` kind rather than an `unavailable`.
That is 119 of 191 assets.

### The provider refuses AVIF

`HTTP 400: Image type image/avif not supported. Only image/jpeg, image/png,
image/gif, image/webp, image/tiff, and image/bmp are supported.`

Every local measurement had said AVIF was the answer for the oversized assets:
it fit 16 of 17 under the cap with the text intact,
where webp fits 10 and
misses the three carrying the most text.
None of that mattered.
Verified by
sending the same picture twice,
one call apart,
as AVIF and as webp:
the AVIF
calls returned 400 and the webp calls returned 454 and 450 characters.

WHAT IS OPEN.
Without downscaling,
which the owner ruled out,
the three
text-heaviest oversized assets (`Aniloviraw/photo0.webp` 2965 characters,
`gqt/photo1.webp` 2329,
`Zha_Ke/letter.webp` 1718) cannot be put in front of a
model in any accepted format.
The deterministic reader reads all three,
so
their text is not lost;
what is missing is a second party to corroborate it.

A probe was in flight at the time of writing to test whether the byte cap is the
real obstacle at all.
The cap is SELF-IMPOSED:
`CONTEXT_SHARE = 0.5` in
`image-asset.ts`,
justified as leaving room for "the prompt,
the source,
the
archive wording and the reply".
A READING call carries none of those but a
200-character instruction,
so half a context is far more conservative than that
call needs.
If the provider accepts the pictures as they are,
the whole
re-encode problem dissolves and nothing needs to be lossy.
Read the probe's
result before building any re-encode.

Remaining option if it does not:
tiling.
Cutting `Zha_Ke/letter.webp`,
which is
1080 by 5645,
into vertical strips loses NO pixels,
which is what "no
downscaling" protects,
and a letter's natural reading order is already
top-to-bottom.
Untried.

## 2026-08-19, later still: the cap was ours, and two silent defects behind it

### The probe answered: nothing needs to be lossy

The probe named at the end of the previous section returned,
and it overturned the section.
The byte cap was never the provider's.
Sent as they are,
every oversized asset is accepted:
`gqt/photo1.webp` is 1274028 bytes,
four times the cap this package was enforcing,
and it comes back read for 2631 characters.
The corpus maximum,
1344454 bytes,
is accepted too.

So the cap is gone,
along with `CONTEXT_SHARE`,
`CHARS_PER_TOKEN`,
`BASE64_INPUT_GROUP` and `BASE64_OUTPUT_GROUP`.
`encodeImageAsset` now takes a plain `maxBytes`,
set to 8 MiB in the reading stage,
which nothing in this corpus approaches.
45 of 191 pictures had been refused on our own arithmetic.

No re-encode,
no format change,
no downscaling,
no tiling.
The tiling option recorded as "remaining,
untried" is not needed:
the three text-heaviest assets were only unreachable because of the cap that no longer exists.

WORTH KEEPING AS A METHOD NOTE,
and the reason this took a day to find.
The estimate divided a context length by an assumed characters-per-token
and compared the result against a base64 length.
Every step of that arithmetic was correct.
A vision model does not tokenize a picture by its base64 length:
it tokenizes by resolution,
in tiles.
The quantity being measured was one this package had invented,
so no amount of care in measuring it could have produced a right answer.
The check that would have caught it on day one costs one call:
send an oversized picture and see.

### The store rejected every `no-text` reading on resume

`isPairedReading` in `reading-cache-store.ts` checks the discriminant before the fields,
and it knew two kinds.
`no-text` was added to `PairedReading` the same afternoon and not to the guard.
Consequence:
every record carrying it was rejected on resume,
so the picture read as never gathered,
the deterministic reader ran again,
the same record was written back,
and the next pass rejected it again.
119 of 191 pictures end at that kind,
so two thirds of the picture work was re-done on every resume.

Nothing reported anything.
A run that silently re-does work is indistinguishable from a run that had nothing to resume,
which is why this needed a test to find rather than a log to read.

THE SHAPE OF THE MISTAKE IS THE SAME AS THE NAMESPACE ONE next door in the same directory:
a second place that has to learn about a new kind,
with nothing making it.
`reading-cache-store.unit.test.ts` now persists one record of every kind through the real store
and reads it back through a second open,
so a kind added to the type without being added to the guard fails there.
The same tests caught a second defect:
the optional `readings` on an `unavailable` record
were waved through unchecked for being optional.

### A stopped run returned a verdict instead of throwing

The OCR gate sits ahead of every model and consulted no signal of its own.
Its early return for `no-text` sits ABOVE the fan-in that rethrows on an aborted signal,
so a run already told to stop did not throw:
it spawned a decoder and tesseract for every remaining picture
and returned a `no-text` verdict that was then persisted.

`signal.throwIfAborted()` now runs before the gate.
The existing abort case could not have caught this,
because the call throws either way and the only difference is what it spent getting there;
the new case counts the asks and expects zero.
Removing the check makes it fail with `expected 'returned no-text' to equal 'AbortError'`.

### Verified at the settle path, not at the probe

`sentinel-probe` cannot verify any of this:
it calls `repairTranslation` directly and never reaches `settleEntry`.
The honest check is `corpus-pass -- --only <id>` into a throwaway `TRANSLATION_REPAIR_RUNS_DIR`.

On `wangzihao980`,
at the tip of this work:

```text
gatherEntryPictures  gathered 6 of 6 pictures
readImageWithOcr     picture1.webp: no text (0 characters, under 16)
readImagePair        picture1.webp: no text to read, so no model was asked
                     ... the same for picture2 through picture5 ...
readImageWithOcr     Word1.webp: read 205 characters without a model
readImageAsset       hf:moonshotai/Kimi-K3 read Word1.webp but the reading was refused: reads-as-refusal
```

Five pictures skipped both models in under a second,
and those five are exactly the ones that had produced the corroborated refusals.
The sixth passed the gate on 205 characters,
went to the roster,
and the shape screen caught a refusal from a real model on real content.

### A test file shaped wrongly passes silently, and costs an afternoon

This runner takes `describe({ name, children: [it({ name, fn, },),], },)`.
A file written as `describe('name', function () { it('name', fn,); },)` registers nothing,
prints nothing,
and exits 0.
Every runner in the chain reports success:
`mise run ... :test:unit -- <file>` exits 0,
and so does `node <file>`.

The tell is that a test file produces NO output at all,
not even a pass line.
A positive control on a known-good sibling separates it from a genuine null in one command,
which is the only reason it was caught here.
