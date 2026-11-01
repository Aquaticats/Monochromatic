# Reading a picture at the user boundary: the provider refuses AVIF and the size cap was ours

Part of [Reading a picture at the user boundary](reading-a-picture-at-the-user-boundary.md),
which lists every file of this record.

## The provider refuses AVIF, so the re-encode cannot use it

The sweep in [the OCR census file](reading-a-picture-at-the-user-boundary-ocr-census-store-proof-and-ocr-gate.md)
found a quality for almost every oversized picture and checked
the text survived,
which made AVIF look settled.
It is not,
for a reason no
amount of local measurement could reach:
the provider will not accept the
format at all.

Sent one small AVIF,
built from a picture both readers had already transcribed
successfully as webp:

```text
avif   hf:Qwen/Qwen3.6-27B     HTTP 400
avif   hf:moonshotai/Kimi-K3   HTTP 400
webp   hf:Qwen/Qwen3.6-27B     OK, 454 characters
webp   hf:moonshotai/Kimi-K3   OK, 450 characters
```

The error names the whole accepted set:

```text
Image type image/avif not supported. Only image/jpeg, image/png, image/gif,
image/webp, image/tiff, and image/bmp are supported.
```

Same picture,
same models,
same instruction,
one call apart.
So the format is the variable and nothing else is.

WORTH KEEPING AS A METHOD NOTE.
Every local measurement said AVIF was the
answer:
it fit 16 of 17 where webp fits 10,
and it preserved the text on every
asset that carried any.
A module built on that would have been correct in every
respect except the one that decides whether it works.

### Webp is accepted and much weaker

Re-running the same sweep with `cwebp`,
which is the format the corpus already
uses and the provider already takes:

```text
asset                          ocr before   fits at   bytes    ocr after
Aniloviraw/photo0.webp               2965    NO QUALITY FITS DOWN TO q20
gqt/photo1.webp                      2329    NO QUALITY FITS DOWN TO q20
Aniloviraw/photo1.webp               1914      q50    275158      1895
Zha_Ke/letter.webp                   1585    NO QUALITY FITS DOWN TO q20
Chinatsu_Suzuki/photo2.webp           536      q40    293238       526
MizuharaNagisa/letter.webp            446      q40    276904       390
zhangyubaka/photo1.webp               298      q70    270548       359
```

Ten of seventeen fit,
and the three carrying the most text are among the seven
that never do.
Those three are exactly the ones worth rescuing.

WHAT STILL READS THEM.
The deterministic reader has no cap and no format
restriction,
and gets 2965,
2329 and 1718 characters out of the three webp
cannot shrink.
So their text is not lost;
what is lost is a second party to
corroborate it.

## The size cap was ours, not the provider's

Before building any re-encode,
the premise it rests on was tested:
that the
oversized pictures are refused at all.
They are not.

Sent unchanged,
in their original webp,
to both readers:

```text
Zha_Ke/letter.webp        628180 bytes   Kimi-K3  ACCEPTED, 1623 characters
Zha_Ke/letter.webp        628180 bytes   Qwen     ACCEPTED, 1613 characters
Aniloviraw/photo0.webp    547484 bytes   Kimi-K3  ACCEPTED, 3094 characters
Aniloviraw/photo0.webp    547484 bytes   Qwen     ACCEPTED, 3329 characters
gqt/photo1.webp          1274028 bytes   Kimi-K3  ACCEPTED,  625 characters
gqt/photo1.webp          1274028 bytes   Qwen     ACCEPTED, 2631 characters
```

`gqt/photo1.webp` is 1274028 bytes.
The pipeline's own cap for Qwen is 294912.
The provider took a picture more than FOUR TIMES the size this package refuses
to send it,
and read 2631 characters out of it.

So every `too-large-for-model` in every run so far was OUR GUARD REFUSING,
not
the provider.
45 of 191 assets,
a quarter of the corpus's pictures,
were never
offered to a reader because of an estimate made here.

### Why the estimate is wrong in kind, not by a factor

`encodedCharsThatFit` takes half the model's context,
converts tokens to
characters at three each,
and compares that against the picture's base64 length.
Every step is defensible for TEXT.
None of it describes an image.

A vision model does not tokenize an image by the length of its base64.
It
tokenizes by resolution,
in tiles.
The base64 length is an artefact of the
compressor and can vary by a factor of ten between two pictures of identical
dimensions.
Measuring an image's cost that way is not conservative;
it is
unrelated.

The arithmetic shows how unrelated.
`gqt/photo1.webp` at 1274028 bytes is
1698704 base64 characters,
which at three characters a token is 566234 tokens,
which is more than DOUBLE Qwen's entire 262144-token context.
It was accepted
and answered.
Whatever the provider counts,
it is not this.

The half-share is separately wrong for this call in particular.
Its own comment
justifies it as leaving room for "the prompt,
the source,
the archive wording
and the reply".
A reading call carries a 200-character instruction and nothing
else.
There is no source and no archive wording to leave room for.

### What replaces it

Nothing lossy,
which is the point.
No re-encode,
no format change,
no
downscaling,
no tiling.
The pictures go as they are.

The reading stage stops deriving a byte ceiling from a context length and
applies a plain one instead,
set above the corpus's largest asset.
A picture the
provider genuinely will not take now announces itself as an HTTP error,
and that
error costs one reading and nothing else,
because a reader that throws was
contained earlier today.

THAT ORDERING MATTERS.
Letting the provider be the authority on what it accepts
is only safe once a refusal cannot take an entry down with it.
Before the
containment fix,
replacing a conservative guess with a real request would have
traded 45 unread pictures for lost entries.

## A figure this record reported is now void

Earlier in [the OCR census file](reading-a-picture-at-the-user-boundary-ocr-census-store-proof-and-ocr-gate.md),
and in the tracker,
and in the module's own comment:

> Of the 191 distinct assets a source-side reference names,
> 146 fit both
> readers,
> 31 fit only the larger context and 14 fit neither.
> So 31 pictures can
> never be corroborated.

That was measured carefully against `encodedCharsThatFit`,
and it described that
function rather than the world.
With the derivation gone,
all 191 assets can be
offered to both readers,
so the count of pictures that cannot be corroborated
FOR REASONS OF SIZE is zero.

It is recorded rather than deleted because the mistake is the reusable part.
The
measurement was real,
the arithmetic was right,
and the conclusion was wrong,
because the quantity being measured was one this package had invented.
A number
carrying that much detail reads as evidence about the provider,
and this one
never was.
