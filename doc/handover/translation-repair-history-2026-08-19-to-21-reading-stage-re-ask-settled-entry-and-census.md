# Translation repair history 2026-08-19 to 2026-08-21: the reading stage's re-ask, its first settled entry and the reading census

Asking a declining reader again,
the resume proof on the reading kind that needed it,
the guard bite and the state of the lane,
the first picture-bearing entry settled end to end with its cost,
and the separate census of the reading lane,
whose first null result was a bug.

Part of [Translation repair history: 2026-08-19 to 2026-08-21](translation-repair-history-2026-08-19-to-21.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-19, last: a declined reading is asked again

The settle-path run discarded a good reading,
and the reason was not the picture.
`Word1.webp` carries text that the deterministic reader gets 205 characters out of.
Asked six times per model with identical bytes and identical instruction:

```text
hf:Qwen/Qwen3.6-27B     6 of 6 read it, 376 to 397 characters
hf:moonshotai/Kimi-K3   2 of 6 read it, 377 and 403 characters
```

A refusal is a property of the ROLL.
That costs more than the rate suggests,
because corroboration needs both readers
and the provider offers exactly two models that read images,
so the pair's success rate is the WEAKER reader's read rate rather than an average.
A reader declining two asks in three costs two thirds of the readings,
not one third.

`readPastRefusal` asks the same model again,
up to four asks,
on the refusal clause alone.
At the measured rate that retains four readings in five where one ask retained one in three,
and it costs about 2.4 asks per picture because asking stops at the first reading.
It fires only on pictures the gate already found text on.

On the same entry afterwards,
`Word1.webp` corroborated at 0.653,
after Kimi-K3 declined once and read 403 characters on its second ask.

WHY NOT A THIRD READER instead.
There is no third to have:
the catalog carries `readsImages: true` on exactly two models,
and the vision aliases the provider lists are aliases of those same two.
The two-model sub-roster was a design choice about judge disinterest;
it is now also a fact about supply.

### Resume, finally proved on the kind that needed it

Stopping that run once its pictures were settled and starting it again into the same
runs directory resumed all six,
five `no-text` and one `corroborated`,
with ZERO `readImageWithOcr` lines against six in the run that wrote them.
The resume proof that existed before today predated the `no-text` kind,
so the case the store had been silently rejecting had never actually been resumed.

### What is left

The README now describes the reading stage and names `tesseract` and `dwebp` as
deployment dependencies,
so package completeness is met for this lane.

Still owed,
and none of it blocks the lane:
a settle-path witness for the size ceiling needs an entry carrying an asset past
294912 bytes,
which `wangzihao980` does not have (`gqt` does);
and the corpus-wide re-run under the corrected pipeline has not been done,
since every batch started before today's fixes carries picture verdicts
produced by the estimate,
the missing gate,
and the missing retry.

### The last guard bite, and the coupling it revealed

The refusal shape clause had been added without its bite shown,
because a subagent
was rebuilding the shared dist at the time.
Disabling the `readsAsRefusal` call in `image-reading-sense.ts` fails the screen-level case with

```text
AssertionError: expected 'usable' to equal 'refused'
```

which is the two production refusals passing as usable readings again.
It also fails four cases in the re-ask suite,
because that suite drives its refusal fixture through the real screen rather than stubbing the clause,
so the clause is pinned from two directions and a rename cannot quietly detach it.

### Where the lane stands

Built,
tested,
and witnessed on the settle path:
the gate,
the shape screen,
the re-ask,
the ceiling,
and resume.
Unit suite 429 passing and 0 failing,
lint 0 warnings 0 errors,
types clean.

NOT YET WITNESSED:
one picture-bearing entry reaching `status=SETTLED` end to end under this build.
Every run so far was stopped deliberately once its picture phase had been read,
and the entry that did settle predates all of today's changes.
That is the one thing standing between this lane and done,
and it gates the corpus-wide pass,
because a full pass is hours of quota and one settled entry is the cheap check
that picture context flows through `slicePictures` into both lanes without a new surprise.

## 2026-08-19, closing: the entry settles, and the reading lane is measured separately

### One picture-bearing entry settled end to end

```text
TALLY wangzihao980 status=SETTLED slices=6
  repairStatus=repaired repairIssues=43 repairAccepted=35 repairResolved=35
  repairFindings=78 repairChanged=5
  translateStatus=complete translateChanged=4 documentsDiffer=4
  alignmentFindings=0 selection=pending-human-decision ms=3660309
DONE processed=1 of pending=1
```

That was the gate.
Both lanes completed on a document whose picture context came from the
gate,
the shape screen and the re-ask,
and the aligner raised nothing.

READ THE COST FIGURE BEFORE PLANNING A CORPUS PASS:
3660309 ms is 61 minutes for ONE entry,
and the corpus holds 93.
A serial end-to-end pass is days of wall clock,
not hours.

### So the reading lane is measured on its own

The reading lane is separable:
it runs before either translation lane and takes nothing from them.
`~/temp/agent/reading-census.mjs` walks every entry,
prepares its pair,
gathers its pictures,
and reads them through the same `readDocumentPictures` production call,
recording the verdict per asset into `~/temp/agent/reading-census.json`.
That buys the corpus-wide picture rates for the price of the vision calls alone,
instead of waiting days behind two translation lanes that no picture question depends on.

It answers three things the single-entry runs cannot:
how many pictures reach each verdict corpus-wide;
how the nine assets in the 16 to 31 character band behave,
which is what decides whether
`MIN_OCR_CHARS` sits in the right place;
and how many re-asks the refusal roll actually costs,
which is what the ask limit should be
set from rather than from one picture.

### A null result that was a bug, twice in one session

The census printed `entries listed: 93` and `assets read: 0` on its first run,
in under a second.
Not a corpus without pictures:
`readCorpusFile` takes `relPath` and had been passed `path`,
so every read threw and a bare `catch { continue; }` turned 93 failures into a clean zero.

This is the same shape as the test file that registered nothing and exited 0 earlier today.
Both were caught by asking whether the probe could have shown a positive at all,
and in both cases the answer took one command.
The census now counts and reports the entries it could not read,
so its zero can never again mean two different things.
