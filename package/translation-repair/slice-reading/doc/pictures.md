# Reading the pictures a document shows

Part of [the `slice-reading` package README](../README.md).

A document that shows a picture carrying text is a document whose translation cannot be judged from its text alone.
The pipeline reads those pictures and puts the reading beside the source and the archive,
as evidence a later stage may consult.
Nothing in the reading decides what ships.

## Deterministic first, and usually last

`tesseract` runs before any model is asked,
with `chi_sim+eng`.
Under 16 solid characters the picture is recorded as carrying no text,
no model is asked about it,
and no finding calls that a shortfall,
because it is the correct answer:
119 of the 191 pictures in the reference corpus carry no text at all,
most of them being photographs of people.

It gates rather than votes,
which was settled by measurement and is the opposite of what it looks like it should be.
Its trigram overlap against the model readings is 0.019 and 0.023 on one asset and 0.096 and 0.111 on another,
while those models agree with each other at 0.643 and 0.785.
It is not missing the text:
on the first asset it returns 405 characters against their 390 and 394.
It reads the same text and gets the **glyphs** wrong,
which leaves length intact and destroys overlap,
so letting it vote would refuse readings that are fine.
What it is reliable at is **presence**,
six of six against the models in both directions.

Noise can clear its line:
a painting's canvas returned 24 characters on `Uekawakuyuurei/IMG_1308.webp`,
so the models were asked and every one reported that the picture carries no text.
Since 2026-09-04 such a reply is an absence report (`reports-no-text`),
not a refusal:
it is not asked again,
and two readers reporting absence,
or answering with fewer characters than a transcript (the hull number on `img370.webp`),
confirm the picture textless past the noise.
The verdict is recorded with the readers that confirmed it and resumed like the deterministic one;
a reader that declines to read (`reads-as-refusal`) is still re-asked and still leaves the picture unread.
The refused reply's opening is logged,
since it carries the model's words about the picture and never the picture's text.

## Several readers, and a reader asked again

A reading may be used only when a second model,
shown the same picture and not the first model's answer,
agrees with the first at the corroboration threshold.
A single reading is refused rather than passed along with a caveat.

The cross-provider vision sub-roster is six models when every provider is wet:
`RUN_READER_MODELS` in `package/translation-repair/run/src/corpus-run/run-config.ts` takes every roster model some
provider can show a picture,
less the ones held as unmeasured readers and the ones the owner culled.
Each entry seats its readers off the meters
(`package/translation-repair/corpus-pass/src/corpus-run/pass-seated-pictures.ts`),
so a model withheld on the provider that would serve it reads nothing that entry.
The bench is counted by the providers that can show a picture (`visionReachOf`),
not by the ones that serve the model text,
and the `JUDGE SEATS` line prints `seeing=` beside `readers=` (ledger B722).
`gemma-4-26b-a4b-it` was listed as reading pictures on OpenRouter but held out of that reach by
`openrouter-pictures-unmeasured` until `reader-measure` passed (ledger B721 and B723).
It passed on 2026-10-21 (UTC) and the hold was lifted:
a first and a second run (08:44 UTC and 08:48 UTC) matched 3 of 291 pictures to 3 recent trusted records
and were not measured,
since the recent runs directories hold picture readings of one entry only (ledger B800),
and a third run (09:02 UTC) over 192 trusted records of 10 entries read 9 pictures,
all of them corroborated by every seated reading of their picture and none by none,
against a seated floor of 1.00 and ceiling of 0.00 over 43 readings.
The 9 pictures came from two entries (Mio 7,
Toka_ls 2),
which is the limit of the verdict,
and the measurement cost 15 calls.
`roster-changes.md` in the family `doc/` carries the full account.
Corroboration still requires independent agreement,
and a declined reading is still asked again up to four times.
The corpus-wide retry measurement predates the current reader roster
and must not be read as any current reader's refusal rate.

Measured over the whole corpus,
119 reader and picture pairs reached a model,
109 read on the first ask,
and 1 more read only after being asked again.
Of the 9 that never produced a reading,
6 refused through all four asks,
while 2 exited on the length screen and 1 on an empty reply.
Neither of those is a refusal and neither is asked again.
So a decline is usually about the picture,
and sometimes about the roll:
on one text-bearing asset asked six times per model,
`hf:Qwen/Qwen3.6-27B` read it six times of six and `hf:moonshotai/Kimi-K3` twice of six,
transcribing the same text either way.
Re-asking costs about 20 extra calls over a corpus pass and recovers the roll case when it happens.

A reply the length limit cut is the one empty reply that is asked again
(since 2026-10-21,
ledger B718).
`glm-5.3-flash` on NeuralWatt spent its whole completion cap reasoning and answered `text:""` with
`finishReason:"length"`,
which says the call ran out of budget and nothing about the picture.
The ask goes once more with the next reading perspective,
to another wet provider that serves the model where the router has one,
else to the same provider,
and the log line names the picture,
the finish reason,
the provider that cut it and the route the second ask takes.
A second empty reply stays `empty-reply`,
and an empty reply that ended on `stop` is still asked once.

## What is sent, and what is not

Pictures are sent as they are.
No re-encode,
no format change,
no downscaling,
no tiling.
An earlier byte ceiling was this package's own estimate of what a vision model would accept,
derived from base64 length against context length,
and it was wrong in kind:
a vision model tokenizes a picture by resolution in tiles,
not by base64 length.
Sent unchanged,
an asset four times that ceiling comes back read for 2631 characters.
A plain 7 MiB ceiling remains (`READING_MAX_BYTES` in `src/image-reading-stage.ts`,
8 MiB until 2026-08-22,
when that was found to permit request bodies the gateway rejects),
which nothing in the reference corpus approaches.

The provider accepts `image/jpeg`,
`image/png`,
`image/gif`,
`image/webp`,
`image/tiff` and `image/bmp`,
and refuses AVIF with an HTTP 400.

## Deployment dependency

`tesseract` must be on the path,
with the `chi_sim` and `eng` language data installed,
or every picture is recorded as unreadable by the deterministic reader and every one of them is sent to two models.
Decoding needs `dwebp` for webp assets;
ImageMagick is tried as a fallback and cannot be relied on for that format.
