# When an image reading makes no sense

`#111`'s authorized answer is "B,
but best effort,
fallback to A whenever an image's OCR doesn't make sense",
and it says the rule has to be written down before it is implemented,
because it selects between two behaviours rather than describing one.
This is that rule.

Status:
proposal.
The default stated here is implemented so the work is not blocked,
and it is cheap to change because it is one function with its own tests.

## What each branch costs

The two branches are not symmetric,
and the asymmetry is what sets the threshold.

TRUSTING A BAD READING IS EXPENSIVE.
The reading becomes the source a transcript is judged against,
so a wrong one licenses replacing a human's careful transcription with something derived from a misreading,
and the judges have no way to tell,
because the reading is the only evidence they are given about the picture.

FALLING BACK IS CHEAP.
Option A protects the block structurally:
it stays out of translation and is spliced back unchanged.
That is exactly where the pipeline stands today for every transcript,
so a fallback loses nothing that currently exists.

SO THE RULE IS DELIBERATELY EAGER TO FALL BACK.
A reading has to earn its use.
This is the opposite of how a detector for a defect should be tuned,
and it follows from the costs rather than from taste.

## The rule

A reading is used only when every clause holds.
Otherwise the block falls back to structural protection.

### One: the reading has to say something

A reading under 16 characters after trimming is refused.
An image nobody could read returns an apology or nothing,
and both are shorter than any transcript worth having.

### Two: the reading must not be a refusal

A reading is refused when it carries a first-person inability in its first 200 characters:
wording of the shape "I cannot",
"I am unable",
"I can't make out",
"no text is visible",
"the image is unclear".
This clause is a heuristic and it is stated as one.
It is safe in the direction that matters,
because a false positive here costs only the fallback.

### Three: the two readers have to agree about what the picture says

This is the clause that does the work.
The vision sub-roster is exactly two models,
so a picture is read TWICE,
independently,
and the two readings are compared to each other.

Compare CHARACTER TRIGRAM sets.
Take every run of three consecutive characters in each reading,
whitespace collapsed,
and require the smaller set to share at least 30 percent of its trigrams with the larger.

Trigrams rather than anchors,
and rather than single characters.
Anchors are Latin words,
digit runs and handles,
so a picture of pure Chinese prose produces two readings carrying none,
and an anchor rule would refuse both however correctly they were read.
Single characters fail the other way:
two unrelated English texts share most of the alphabet,
so single-character overlap runs near one on a wrong pair.
Trigrams are starved by neither.

A reading that no second reader corroborates is UNAVAILABLE rather than usable-with-a-caveat.
That costs the 31 assets that fit one reader's context and not the other's,
and those keep exactly the protections they have today.

### Four: what this replaced, and why

An earlier version of this clause compared the reading against the transcript the archive already carried,
requiring two shared anchors.
It was measured against real traffic on 2026-08-19 and refuses correct readings.

Every reading of `Mio/7`'s two pictures was refused,
four attempts across two assets and two readers,
while the two readers agreed with each other at 0.967 and 1.000 character overlap.
The clause assumed a slice's target-only English transcribes that slice's pictures.
Where it is some other kind of addition,
every correct reading of every picture there fails.

The comparison was also structurally starved.
A reading comes back in the picture's own language,
usually Chinese,
and the archive transcript is English,
so the only tokens that can be shared are names,
handles and numbers.
Measured overlap was 2,
2,
4,
5,
5,
5 against a floor of 2:
two of six sat exactly on it.

Cross-model agreement has no such ceiling.
Over five pictures read by both models,
and every cross-pair as the control:

```text
same picture, different models     5 pairs   trigram overlap 0.643 to 1.000
different pictures                40 pairs   trigram overlap 0.000 to 0.129
```

The threshold of 0.30 sits between them with about a factor of two of margin on each side.

### Five: where nothing corroborates, there is no reading

A picture only one model can be sent has no second reading to agree with,
and an uncorroborated reading is UNAVAILABLE rather than usable with a caveat.
This is the same asymmetry the whole stage is built on.
Falling back costs nothing that exists today,
since the passage is protected and left alone,
while trusting a wrong reading licenses replacing a human's transcription with something derived from a misreading.
A caveat travelling downstream is a caveat somebody has to remember.

Measured over the 191 distinct assets a source-side reference names:
146 fit both readers,
31 fit only the larger context,
and 14 fit neither.
So this refuses 31 pictures outright.
Every transcript in the corpus is already held by a guard,
so none of those 31 loses a protection it has today.

### Six: two readers reporting nothing confirm a textless picture (2026-09-04)

The presence gate of clause one is drawn at 16 solid characters from `tesseract`,
and a painting's canvas can clear it as noise.
`Uekawakuyuurei/IMG_1308.webp`,
an oil painting of ships at sea,
returned 24 characters;
`img197.webp`,
an ink drawing of two cats,
returned 18;
`img370.webp`,
an ink drawing of a destroyer whose only text is the hull number,
returned 24.
Every model asked about them answered truthfully:
"There is no visible text in this image",
"I cannot read any text in this image.",
"There are no visible words",
and for the destroyer,
`DE581`.
Clause two screened the first two as refusals and clause one the third as too short,
each reader was asked four times to be told the same thing,
the pair ended `no-reader-available` and transient,
and the entry stopped at the visual-evidence gate on every run,
on both rosters.
`dogesir_/photo2.webp` and `gqt/photo3.webp` ended the same way.

A probe over those five pictures on 2026-09-04 found 14 of 15 refused replies naming text,
"no visible text",
"no words",
"no legible characters",
and one declining without naming it,
"I cannot read the image."
So the rule now tells the two apart.
A refusal that names text and says nothing about the picture's quality or the model's access is an absence report,
clause `reports-no-text`;
it describes the picture,
so it is not asked again.
A refusal that declines,
or names text it could not make out,
stays `reads-as-refusal` and is re-asked as before.
A reply under 16 characters that negates nothing is a short reading rather than an apology.

Two readers reporting absence,
or answering short,
or one of each,
confirm what the deterministic reader could not:
the picture is textless past its noise.
The pair records `no-text` with the readers that confirmed it,
the verdict is resumed on later runs like the deterministic one,
and nothing travels to the sheets from it,
exactly as before.
One absence report beside a decline,
or beside a full reading,
is still nothing corroborated.

## What this rule does NOT do

IT DOES NOT JUDGE TRANSLATION QUALITY.
Whether the reading is a good rendering is the judges' question,
and they are equipped for it.
This decides only whether the reading is about the right picture at all.

IT DOES NOT SCORE CONFIDENCE.
There is no threshold to tune between "probably fine" and "probably not",
because the branch is binary and the costs are lopsided.

IT DOES NOT ASK A MODEL TO JUDGE ANOTHER MODEL.
The second reader is not shown the first reading and is not asked whether it is sensible.
It is asked the same question about the same picture,
blind,
and the two answers are compared mechanically.
A judge would be another failure surface;
two independent witnesses are not.

## Two things a reader should be suspicious of

THE CORROBORATION CLAUSE ASSUMES THE TWO READERS FAIL INDEPENDENTLY.
Two models that share a training lineage could agree on the same misreading,
and nothing here would catch it.
That is a real limit and it is not measurable from this corpus.
What the sample does show is that they do not agree indiscriminately:
over 40 pairs of different pictures the overlap never exceeded 0.129.

THE SAMPLE IS FIVE PICTURES.
Five same-picture pairs is enough to show the separation exists and not enough to place the threshold precisely.
It is drawn from the only corpus slices carrying both a protected target-only run and a source-side picture reference,
so widening it means reading pictures no transcript describes,
which gives no ground truth to check against.

AN EARLIER VERSION OF THIS SECTION reported 37 to 142 anchors per known transcript against a requirement of two,
and read that as a wide margin.
It counted the ENGLISH archive transcripts,
which carry 73 to 142 anchors each because every English word of four or more letters is one.
The number that governed the clause was never that count but the OVERLAP a Chinese reading could reach against it,
which is 2 to 5.
The correction is recorded rather than deleted
because the mistake was reading a measurement of the wrong quantity as a safety margin.

A transcript of a handwritten note with no names,
dates or numbers would carry no anchors,
and the rule would fall back on it every time.
That is the safe direction,
and it is a real limit rather than a hypothetical one.

CLAUSE TWO IS PATTERN MATCHING ON APOLOGIES.
It will miss a refusal worded unusually.
Clause three catches most of what it misses,
because an apology shares no anchors with a transcript.
Where no transcript exists,
an unusual refusal would pass,
and the result is a bad transcript added where none was before.
That is the one path where this rule can make things worse rather than merely leave them as they are,
and it is the one worth watching in the first run.

## Where the reading comes from, and why it is its own stage

`#111` warns that the vision sub-roster is exactly two,
that selection needs a minimum weight of two,
and that a producer's ballot for its own work counts half,
so where both vision models produce,
no disinterested judge remains.
That is a real problem for one design and not for another.

READING IS SEPARATED FROM TRANSLATING,
which dissolves it.
A vision model is asked only to READ the picture,
producing text.
That text is then evidence,
like the source and the archive are evidence,
and the ordinary six-model roster translates and judges from it without any of them needing to see the picture.

This costs one or two calls per image and buys three things.
The roster is untouched,
so selection keeps its weights and its disinterested judges.
The reading is a value that can be screened,
which is what the rule above does and what it presupposes.
And the reading is cacheable per asset,
where a translation carrying an inline image would be cacheable only per slice.

THE ALTERNATIVE,
attaching the image to every translate and judge call,
was rejected on the roster arithmetic rather than on cost:
it puts the two vision models on both sides of their own work and leaves four models judging text they cannot check.

## Image size, and when to give up on one

Measured over the 284 assets in the pinned corpus:
median 71 KiB,
mean 154 KiB,
largest 1312 KiB.
Base64 inflates by a third,
so the largest asset would arrive as roughly 1.75 MiB of prompt,
which is around 450 thousand tokens and does not fit the context of either vision model.

The images the known transcripts describe are smaller,
`Zha_Ke/letter.webp` at 613 KiB being the largest of them,
which is about 817 KiB encoded and does fit,
expensively.

THE RULE:
an asset above a stated encoded size is not sent,
and the block falls back to structural protection.
`sharp` is already in this workspace and downscaling is therefore available,
but it is deliberately NOT the first move:
a downscaled photograph of handwriting is exactly the input that produces a confident wrong reading,
which is the failure this whole rule exists to avoid.
Refusing a picture too large to send is honest;
shrinking it until it fits and then trusting what comes back is not.
