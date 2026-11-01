# Reading a picture at the user boundary: the OCR census, the store proof and the OCR gate

Part of [Reading a picture at the user boundary](reading-a-picture-at-the-user-boundary.md),
which lists every file of this record.

## What the corpus's pictures actually hold, measured deterministically

Every figure about readings in [the first probes](reading-a-picture-at-the-user-boundary-first-probes-and-attachment.md)
and [the first CLI run](reading-a-picture-at-the-user-boundary-first-cli-run-and-refusal-containment.md) was a figure about MODELS.
None of them said how many corpus pictures carry text at all,
so none of them could say whether a picture producing no reading was a miss
or the correct answer.

Tesseract 5.5.3 with `chi_sim` and `eng`,
over all 191 distinct
source-referenced assets,
no model involved and no quota spent:

```text
CARRY TEXT          72
NO TEXT AT ALL     119
missing on disk      0
undecodable          0

text length over the 72 that carry any:
  min 1   p50 122   p90 759   max 2965
```

So 119 of 191 pictures,
close to two thirds,
are photographs with nothing to transcribe.
A reader declining those is RIGHT,
and every earlier reading of "no usable reading"
as a shortfall was reading a correct answer as a failure.

The `min 1` matters as much as the maximum.
Tesseract on a photograph returns a few characters of noise rather than a clean zero,
so "carries no text" needs a floor taken from this distribution
rather than a strict comparison against zero.

### Deterministic OCR agrees with the models, in both directions

On the six assets where a model reading is on record,
six of six agree:

```text
wangzihao980/Word1.webp       675x1200   tesseract  205   models read 390 / 394
zheermao101/photo3.webp      1225x533    tesseract  388   models read 448 / 454
dogesir_/intro.webp          1080x984    tesseract  540   models read 590 / 632
wangzihao980/picture4.webp    313x679    tesseract    0   both models: no text
wangzihao980/picture2.webp    270x360    tesseract    0   both models refused
Uekawakuyuurei/img231.webp   2024x1492   tesseract    0   both models refused
```

Tesseract reads what they read,
at comparable length,
and finds nothing exactly where they found nothing.

It also has NO CONTEXT CAP,
so it reads the 45 assets no model can be sent at all,
and it is deterministic,
which the models are not:
the same picture corroborated at 0.643 in one probe and disagreed at 0.087 in a CLI run.

## The size cap, and why "best quality" cannot mean the highest number

SUPERSEDED,
and kept for the method rather than the conclusion.
The allowance of 294912 bytes was this package's own estimate,
not the provider's limit,
and the provider accepts every one of these assets unchanged.
Read "The cap was ours" in [the AVIF refusal file](reading-a-picture-at-the-user-boundary-avif-refusal-and-the-size-cap-that-was-ours.md)
before acting on anything in this section.

45 of 191 assets sit past the smaller reader's allowance of 294912 bytes.
Re-encoding to AVIF at `--qcolor 100` fits ZERO of them,
because it makes every one LARGER,
by four to twelve times:

```text
Uekawakuyuurei/IMG_1308.webp   1344454 -> 12302890 at q100
gqt/photo1.webp                1274028 ->  7929946 at q100
Mio/photo5.webp                1129330 ->  8224189 at q100
```

The source is lossy webp and near-lossless AVIF is not,
so "the best quality setting" has to mean the highest quality that FITS,
found by sweeping rather than assumed.

Only 17 of the 45 carry any text,
so only those 17 are worth rescuing.
Sweeping `--qcolor` downward until the re-encode fits,
then decoding it back and re-reading it to check the text survived:

```text
asset                          ocr before   fits at   bytes    ocr after
Aniloviraw/photo0.webp               2965      q30    230558      3015
gqt/photo1.webp                      2329      q30    267639      2258
Aniloviraw/photo1.webp               1914      q60    288031      1926
Zha_Ke/letter.webp                   1585      q40    285611      1588
Chinatsu_Suzuki/photo2.webp           536      q50    273734       530
MizuharaNagisa/letter.webp            446      q40    213174       335
zhangyubaka/photo1.webp               298      q60    273039       298
Uekawakuyuurei/IMG_1308.webp           24    NO QUALITY FITS DOWN TO q30
```

Sixteen of seventeen fit,
and on every asset carrying substantial text
the re-encode preserves it:
`Zha_Ke/letter.webp` reads 1585 characters before
and 1588 after,
`zhangyubaka/photo1.webp` exactly 298 both times.

NO RESIZING IS INVOLVED.
`Zha_Ke/letter.webp` stays 1080 by 5645 pixels.
The bytes come down by re-encoding,
which is what was asked for,
rather than by discarding resolution,
which was not.

The one that never fits,
`Uekawakuyuurei/IMG_1308.webp` at 5737 by 3970,
carries 24 characters,
and the sweep's own read-back shows why that number
is noise rather than text:
the low-yield assets swing wildly across the
re-encode,
45 to 25,
18 to 43,
1 to 11,
while the text-bearing ones hold steady.
Nothing is lost by leaving it unread.

## The store, proven on the real thing

A settled entry RETIRES its own caches,
so the settled path can never show a
resumed reading:
after `wangzihao980` settled,
its `slice-cache/wangzihao980/`
directory was gone and only the artifact remained.
The case the store exists for is therefore an entry CUT OFF mid-run,
which is what a deadline,
a crash,
or the owner's REISUB produces.

Run twice over `DarlinChit`,
the first interrupted after its reading landed,
the second identical and into the same runs directory:

```text
FIRST RUN
  19:58:19.997  gathered 1 of 1 pictures for DarlinChit
  19:58:19.997  reading 1 pictures for this document
  19:58:28.113  hf:Qwen/Qwen3.6-27B read photo1.webp: 168 characters
  19:58:35.034  hf:moonshotai/Kimi-K3 read photo1.webp: 163 characters
  19:58:35.034  photo1.webp: corroborated by 2 readers at overlap 0.866

SECOND RUN
  19:58:39.959  gathered 1 of 1 pictures for DarlinChit
  19:58:39.959  reading 1 pictures for this document
  19:58:39.960  photo1.webp: resumed, corroborated
```

Zero `readImageAsset` lines in the second run,
against two in the first.
The resume lands one millisecond after the gather,
where the first run spent
eight seconds and then fifteen.

Two things worth keeping beyond the resume itself.

This is the first CORROBORATED READING OF REAL CORPUS CONTENT on record here:
168 and 163 characters of the same picture,
agreeing at 0.866,
which sits inside the 0.643 to 1.000 band measured for readings of one picture
and far outside the 0.000 to 0.129 band measured for readings of different ones.
Every earlier "corroborated" row in [the first CLI run](reading-a-picture-at-the-user-boundary-first-cli-run-and-refusal-containment.md) turned out to be two refusals.

The entry that used to die now settles.
`corpus-pass --only wangzihao980` finished `status=SETTLED slices=6`,
`processed=1 of pending=1`,
`artifacts=1/92`,
where the same command before the
containment fix finished `status=ERROR` with `processed=0`.
STATED PRECISELY:
no reader threw during that run,
so it does not re-exercise the
containment.
What it shows is that the wiring runs end to end and an entry with
six pictures settles.
The containment itself is held by the unit guards,
which
were shown to fail with it stripped.

## OCR is a gate, not a third reader, and the measurement is why

The obvious design was to let the deterministic reading join the corroboration
as a third party:
it cannot hallucinate,
so anchoring the comparison to it
looked strictly better.

Measured against the model readings already on record,
it does not work.

```text
picture                  OCR vs Kimi-K3   OCR vs Qwen3.6-27B   the two models
zheermao101/photo3.webp       0.626             0.619              0.990
dogesir_/intro.webp           0.096             0.111              0.785
wangzihao980/Word1.webp       0.019             0.023              0.643
```

On two of three pictures the OCR reading fails the 0.30 threshold against BOTH
models,
while those same two models agree with each other comfortably.
Wiring OCR into the comparison would therefore refuse readings that are fine.

The reason is visible in the lengths rather than the scores.
On `Word1.webp` the OCR returns 405 characters where the models return 390 and
394,
so it is not missing the text:
it is reading the same text and getting the
GLYPHS wrong.
Tesseract on handwritten Chinese recovers the layout and the
character count and substitutes lookalike characters,
which destroys trigram
overlap while leaving length untouched.
`photo3.webp` scores 0.62 because it is
cleanly printed,
and that is the exception.

### What it is reliable at

Presence.
Six of six agreement with the models on whether a picture has text at
all,
in both directions,
which is exactly the question the ruling asked it to
answer:
"if something has no text,
we can ignore it."

So the deterministic reading is a GATE in front of the model calls:

-   OCR finds no text,
    which is 119 of 191 assets:
    the picture is settled as `no-text` and NO MODEL IS ASKED.
-   OCR finds text:
    the two models are asked and corroborated against each
    other exactly as before.
    The OCR reading does not enter that comparison.

That keeps corroboration as strong as it was,
spends nothing on two thirds of
the corpus,
and does not rest on a glyph-level agreement that OCR cannot supply.

### Where the AVIF re-encode fits, now that this is settled

SUPERSEDED TWICE.
The provider refuses AVIF outright,
and there is no cap to fit under.
Nothing in this subsection was built.

`Zha_Ke/letter.webp` is 628180 bytes,
past both readers' allowances,
so no model
can be sent it at all.
The deterministic reading gets 1718 characters out of it,
which is not usable on its own,
since nothing corroborates it.

Re-encoded at q40 it becomes 285611 bytes,
inside both allowances,
with its
text intact at 1588 characters and its 1080 by 5645 pixels untouched.
Then the
models can be asked and can corroborate each other in the ordinary way.

So the two pieces are not alternatives.
The deterministic reading says whether a
picture is worth any calls at all,
and the re-encode is what lets the calls
happen on the pictures too large to send.
