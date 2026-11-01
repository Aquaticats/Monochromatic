# Reading a picture at the user boundary: the first probes and where a reading attaches

Part of [Reading a picture at the user boundary](reading-a-picture-at-the-user-boundary.md),
which lists every file of this record.

## What the provider accepts

The endpoint takes a content-parts array.
Both models the catalog marks as reading images accepted it on the first attempt,
with no transcoding and no shape negotiation:

-   `hf:Qwen/Qwen3.6-27B`,
    262144 context
-   `hf:moonshotai/Kimi-K3`,
    524288 context

Both accepted `data:image/webp;base64,` directly.
That matters more than it sounds:
187 of the 191 distinct assets a source-side photo reference names are `.webp`,
so a webp rejection would have forced a transcoding stage into the pipeline.
It did not.

Ten calls were made across four entries.
First-byte latency ran 2.8 to 4.2 seconds and whole exchanges 5 to 51 seconds,
which puts a reading in the same cost band as an ordinary translation call rather than in a band of its own.

## What the readings are

The instruction asks for transcription in the picture's own language,
so a reading of a Chinese picture comes back in Chinese
while the transcript the archive carries is the English a human translator wrote.
The two are never comparable by length:
`dogesir_/intro.webp` reads as 590 to 632 characters against a 1538-character English transcript,
which is the ordinary Chinese-to-English expansion rather than a truncated reading.

## Where the screen stands, measured

`readingMakesSense` refuses a reading sharing fewer than two anchors
with the transcript the archive already carries.
Over the four corpus slices that carry both a protected target-only run and a source-side photo reference,
five assets,
two readers,
ten attempts:

-   six attempts produced a usable reading,
    with shared anchors of 2,
    2,
    4,
    5,
    5,
    5
-   four attempts were refused,
    all four `describes-another-picture`,
    all four on `Mio/7`

Two of the six sit exactly on the floor.
One anchor more lost to paraphrase and `zheermao101/photo3.webp` fails for both readers.

## The margin is not what an earlier note claimed

`doc/planning/when-an-image-reading-makes-no-sense.md` recorded 37 to 142 anchors per known transcript
against a requirement of two,
and read that as a wide margin.
That measurement was of the English archive transcripts,
which carry 77 to 130 anchors each because every English word of four letters or more is one.
The operative number is not that count but the OVERLAP with a Chinese reading,
and the overlap is 2 to 5.

The ceiling is set by how many Latin-script and digit tokens the picture itself contains,
since those are the only tokens that survive into both a Chinese reading and an English transcript.
The source side of these five slices carries 3 to 11 anchors in total.
A picture of pure Chinese prose carrying no names,
handles or dates
cannot clear a floor of two however correctly it is read.

## Cross-model corroboration is far stronger

Two models reading the SAME picture agree with each other in the same script,
so nothing is lost to translation.
On `dogesir_/intro.webp`:

-   cross-model shared anchors 8 of 9
-   distinct-character overlap 245 of 246,
    ratio 0.996

Against 4 and 5 shared anchors for the same two readings measured against the archive transcript.
The vision sub-roster is exactly two models and both are already being asked,
so this signal costs nothing that is not already spent.

## The archive-transcript clause refuses correct readings

Every reading of `Mio/7`'s two pictures was refused as `describes-another-picture`,
four attempts across two assets and two readers.
Calling the transport directly,
bypassing the screen,
shows what was thrown away:

-   `photo6.webp` reads as 100 and 98 characters,
    7 anchors each,
    0 shared with the archive transcript
-   `photo7.webp` reads as 590 and 178 characters,
    10 and 7 anchors,
    0 shared with the archive transcript

The readings are real.
The two models agree with each other almost exactly,
1.000 and 0.967 distinct-character overlap.
What they do not agree with is the 2066-character target-only English on that slice,
which is simply not a transcription of either picture.

So the clause fires on a premise it cannot check.
It assumes a slice's target-only English transcribes that slice's pictures,
and where the target-only English is some other kind of addition,
every correct reading of every picture there is refused.

## Cross-model corroboration separates cleanly, on a small sample

Over the ten readings saved by these probes,
five pictures across four entries,
comparing every reading against every other:

-   5 pairs of the same picture read by different models:
    shared anchors 5,
    6,
    7,
    8,
    9;
    character overlap 0.829 to 1.000
-   40 pairs of different pictures:
    shared anchors 0 to 2;
    character overlap 0.326 to 0.667

Both criteria separate with a gap and no overlap.
A shared-anchor floor of 3 keeps 5 of 5 same-picture pairs and admits 0 of 40 different ones.
A character-overlap threshold of 0.75 does the same.

The different-picture group is the positive control:
it is the case that must move,
and it moves.

STATED AS THE SMALL SAMPLE IT IS.
Five same-picture pairs is enough to show the separation exists
and not enough to place the threshold precisely.
It is drawn from the only corpus slices carrying both a protected target-only run and a source-side photo,
so it cannot be widened without reading pictures that no transcript describes.

## What a reading is actually for, checked rather than assumed

The wiring was going to be justified as protection:
a transcript the source cannot account for gets deleted,
and a reading gives it a source.
Checking that premise against the nine known transcripts shows it is already false.
Every one of them is held by a guard today:

```text
wangzihao980/4     split(1138) + quotes
zheermao101/8      alignment + quotes
zheermao101/11     split(1129) + quotes
dogesir_/3         split(1538) + quotes
Zha_Ke/1           alignment
MizuharaNagisa/3   alignment + quotes
Mio/7              split(2066) + quotes
Mio/16             alignment + quotes
shihai4h/14        alignment + quotes
shihai4h/52        alignment
shihai4h/53        split(10736) + quotes
```

Exactly one corpus slice sits in the gap the reading was supposed to fill:
`shihai4h/3`,
ratio 4.1,
284 characters of target,
no quote block,
no protected run.
The remaining unguarded rows are ordinary slices at ratios of 2.3 to 4.4,
which is the ordinary Chinese-to-English expansion rather than a transcript.

So the reading does not buy protection.
It buys the three things protection cannot:

-   CHECKING a transcript against the picture it claims to transcribe,
    which is what the owner asked for and what no guard does
-   ADDING a transcript where the archive carries none,
    which is half of what reading pictures is for
-   giving translators and judges the picture's content as evidence
    rather than a hole in the source they must translate around

The acceptance test for the wiring follows from that.
It is not "a transcript survives",
since they already do.
It is that a reading reaches the models and changes what they can check.

## Where a reading attaches

Source-side references reach 79 of 1260 slices,
and no slice names a picture on the target side without naming it on the source,
so the source's own references are the whole attachment surface.

They do not reach `Zha_Ke/1`,
whose 3652 characters transcribe a letter
named one slice later,
in `Zha_Ke/2`.
That is the same adjacency the fidelity window was built for,
measured on 2026-08-18 as every relocation pair in the corpus being adjacent,
so the readings of a slice's NEIGHBOURS travel with it
exactly as the neighbouring source and neighbouring archive already do.

## A picture finding is telemetry, and the key deliberately omits it

The slice cache key appends `['pictures', pictureContext]` only when a reading arrived.
The findings do not enter the key at all.

That asymmetry is intentional.
The key must cover everything that changes WHAT THE MODELS ARE ASKED,
and a finding changes nothing a model sees:
`picture Word1.webp: no reading, readers-disagree` is a line in the stored record,
never a line in a sheet.
Feeding it into the key would evict every slice on a reason string
that no model ever read.

The consequence is worth naming so nobody rediscovers it as a defect.
A slice settled on a run where nobody gathered a picture's bytes
records `picture Word1.webp: not read`.
A later run that DOES gather those bytes,
but whose two readers disagree,
computes the same empty picture context,
so it hits the same key and resumes the old record with the old reason.
The stored reason then describes the earlier run's gap rather than this run's disagreement.

Both runs are correct about the thing that matters:
no reading reached the models,
so no reading should.
Only the explanation is stale,
and only in the direction of an older,
less specific reason.
A run that gathers bytes and CORROBORATES a reading computes a different context,
so it misses the key and re-settles,
which is exactly the behaviour required.

The alternative,
keying on findings,
would trade an occasionally stale telemetry string
for cache misses on every slice of every entry whose reader availability shifted.
That is the wrong trade for a field nothing gates on.

## The paired-quote ratio guard is refused on its own measurement

The fallback half of this work was specified as two pieces:
keep every target-only block out of translation,
which is built and wired,
and add a paired-quote ratio guard "for the one merged case",
a paired blockquote whose English side runs over five times its Chinese side.

Measured over the whole pinned corpus rather than the entry that raised it,
that second piece protects nothing and costs eight slices.

There are 211 aligned blockquote pairs.
Their growth band is p50 2.89,
p90 4.00,
p99 8.49,
max 26.20.
Nine pairs sit over 5.0:

```text
Susiethegamer/15    src 76   tgt 382    ratio 5.0
shihai4h/18         src 70   tgt 364    ratio 5.2
CutOceanHeyFis1/4   src 15   tgt 83     ratio 5.5
mikaela_khara/0     src 16   tgt 98     ratio 6.1
a2581911655/3       src 12   tgt 78     ratio 6.5
Weideriche_/0       src 11   tgt 73     ratio 6.6
shihai4h/46         src 55   tgt 467    ratio 8.5
shihai4h/14         src 98   tgt 1649   ratio 16.8
Rentable_A/0        src 10   tgt 262    ratio 26.2
```

Three things follow,
and each on its own is enough.

The outlier the guard was designed around is `shihai4h/14`,
not `shihai4h/3`.
The earlier note named the entry rather than the slice,
and the two are different slices with different shapes.
`shihai4h/14` already carries both an alignment guard and a quote guard,
so the ratio test would be its third.

The slice that actually needs covering cannot be reached by any quote rule.
`shihai4h/3` is 70 source characters against 284 of target,
ratio 4.1,
and it carries NO BLOCKQUOTE ON EITHER SIDE.
A guard that fires on paired blockquotes over five times their source
misses it twice over:
wrong shape,
and under the threshold anyway.

The remaining eight firings are ordinary text.
Five of them have source quotes of 10 to 16 characters,
where a ratio is arithmetic on almost nothing:
`Rentable_A/0` reaches 26.2 because ten Chinese characters became a sentence.
Freezing those out of translation is a cost with no matching benefit.

So the guard is refused,
on the population it was meant to serve.
What covers `shihai4h/3` is the reading,
which is what the ruling asked for first:
the slice names `photo3.webp`,
18550 bytes,
comfortably inside the smaller reader's allowance of 294912,
so both readers can be sent it and a corroborated reading can reach the sheets.
The structural half of the fallback that DOES earn its place,
keeping target-only runs out of translation,
is built,
wired into `translate-slice.ts`,
and unaffected by this.
