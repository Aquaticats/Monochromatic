# Which lane ships

Status:
PROPOSAL.
Nothing is decided here.
`artifact-v2-build.ts` records `laneSelection: pending-human-decision` and says why:
which lane ships is the owner's question.
This document supplies the measurement that question needs,
and reports that the obvious automatic answers do not work.

Date:
2026-08-20.
Zero quota:
every number comes from settled artifacts and the pinned corpus.

## Why it matters now

The pipeline cannot emit a finished document for an entry until one lane is chosen.
Everything else can be production ready and this would still block delivery.

## The damage instruments cannot separate the lanes

Every zero-quota damage check,
over every settled artifact carrying a delivery ledger:

```text
lane       entries  slices  shipped  adjacent  document  severed  unfilled
repair          11      92       62         1         0        1         0
translate       11      92       71         0         1        0         0
```

The repair lane carries two damage events and the translate lane one.
That is not a difference.
Three events across one hundred and thirty-three shipped slices cannot rank two lanes,
and any ratio computed from them ("half the damage rate") is one event wide.

RECORDED SO IT IS NOT RE-DERIVED AS A FINDING:
these instruments are built to catch specific,
rare defects.
Their rarity is the good news about the pipeline
and the reason they are useless as a lane discriminator.

## The verdict the pipeline already computes

Each comparison row carries a `verdict`,
over ninety-two rows:

```text
both-differ       51   55.4%    both lanes changed the slice, and differently
translate-only    18   19.6%    only the translate lane changed it
archive-stands    12   13.0%    neither changed it
repair-only        9    9.8%    only the repair lane changed it
both-agree         2    2.2%    both produced the same text
```

Two things follow.

FIRST,
the translate lane acts where the repair lane declines twice as often,
eighteen slices against nine.
That is a coverage difference,
not a quality one,
and it is the strongest signal available without judging text.

SECOND,
and decisively:
on more than half of all slices both lanes produced different text.
No count,
ratio or guard can say which of two different renderings is better.
That is a judgement about meaning.

## So a whole-document lane choice is the wrong shape

Picking one lane for the corpus discards the better rendering on roughly half the slices,
whichever lane is picked.
The verdict distribution says the decision is PER SLICE or it is arbitrary.

## The option that fits what already exists

The translate lane already judges:
`judgeTranslateSlate` picks among candidate renderings,
with the incumbent entered as one of them,
and records who won.

The same stage could be handed the repair lane's text as one more candidate.
That turns `both-differ` from an unanswerable question into the question that stage
already answers,
and it needs no new judging concept,
roster or prompt shape.

WHAT IT COSTS:
one judging round per `both-differ` slice,
which is about 55% of slices.
Not free,
and not measured here,
because the shape has to be agreed before it is priced.

WHAT WOULD HAVE TO BE CHECKED FIRST,
since `#84` measured this judge on
preserve-or-replace and not on choose-between-two-rewrites:
whether it can rank two candidate rewrites at all,
or whether it only recognises an incumbent worth keeping.
That is a measurable question and it gates the whole option.

## What is NOT proposed

Choosing a lane by damage counts.
The measurement above shows those counts cannot carry the decision,
and a rule built on three events would be overfitting to noise
rather than to this corpus.

## A lane discriminator that looked decisive and was not

Tried 2026-08-20,
refuted the same hour,
recorded so it is not retried naively.

THE IDEA:
a rare-word test cannot tell the translate lane's legitimate re-wording from
deletion,
but NAMES AND NUMBERS survive any faithful translation.
A lane that drops a person,
a place or a figure is damaging the page in either lane,
so counting those should make the two comparable.

THE PROXY:
a token capitalised in the archive whose lowercase form never appears
lowercase there,
plus any token containing a digit.
Sentence-initial common words were supposed to be excluded
because they appear lowercase elsewhere in the document.

IT PRODUCED A CLEAN-LOOKING SPLIT:

```text
POOLED repair      kept 362 / 389   93.1%
POOLED translate   kept 303 / 389   77.9%
```

READING WHAT IT COUNTED KILLED IT.
The supposedly lost names are dominated by ordinary words that happen to appear only
capitalised in a short document,
at sentence starts,
in headings and in list items,
together with list markers counted as numbers.
Genuine names are a small minority of the total.

AND THE FAILURE IS BIASED,
which is what makes it dangerous rather than merely noisy.
The repair lane EDITS the incumbent,
so it keeps the archive's sentence openings and
scores well on those false positives by construction.
The translate lane writes fresh prose and starts sentences differently,
so it loses them without losing any content at all.
The measurement therefore rewards the repair lane for a property that is not quality.

WHAT A REAL VERSION NEEDS:
an actual named-entity list rather than a capitalisation
heuristic,
or the corpus's own structured fields,
which carry names directly.
Until then `#130` stands where it did:
no telemetry here ranks the lanes.

## What reading a whole entry showed, which no telemetry did

`lintong` was read complete against its Chinese,
both lanes,
three slices.
Written up in `doc/audit/the-critics-are-shown-the-wrong-paragraph.md` under
`## Reading lintong complete, both lanes against the Chinese`.

THE REPAIR LANE CAUGHT A FACTUAL ERROR ABOUT THE PERSON.
The archive says her partner called her "a bit insecure";
the Chinese says 占有欲比较强烈,
strongly possessive.
It also restored 粘人 and 可能是因为我吧,
both absent from the archive,
un-softened 逼着 from "ask others" back to "force you",
and corrected 熟人 from "friends" to "acquaintances",
which had reversed who she
was willing to trouble.

THE TRANSLATE LANE PRESERVED EVERY ONE OF THEM.
On this entry it changed exactly one slice,
the `## 简介` heading;
both prose slices came back as the incumbent unchanged.

WHY THIS MATTERS FOR THE DECISION.
The reasoning that makes the translate lane look like the safe default is that
it owes the archive nothing and so cannot inherit its mistakes.
On this entry the opposite happened:
the lane that owes the archive nothing kept an archive sentence that
misdescribes a dead person,
and the lane that edits the archive is the one that read the Chinese closely
enough to catch it.

WHAT IT DOES NOT SETTLE.
One entry is not a corpus.
The `saurikissa` reading in the same document shows the repair lane losing
register,
which is a real cost on a memorial page.
Both readings together say the lanes fail differently rather than one dominating,
which is what makes a per-slice contest the right shape and a whole-document
lane choice the wrong one.
That was already the recommendation here;
it now rests on read passages rather than on telemetry that could not separate
the lanes.

## Eight entries read complete: the lanes fail in opposite directions

Full reading in `doc/audit/eight-entries-read-against-the-original.md`.
Thirteen slices where the lanes produced meaningfully different text:
four to the repair lane,
six to the translate lane,
three tied.

THE COUNTS ARE NOT THE FINDING.
They are one reader's judgement and several calls are close.
The finding is that the two lanes fail in ways that do not trade off against
each other:

-   THE REPAIR LANE FAILS BY INHERITANCE.
    It edits the archive,
    so an invention no critic flagged ships untouched.
    Two entire fabricated sentences shipped this way in six entries,
    including
    an invented characterisation of what a dead person did with her community.

-   THE TRANSLATE LANE FAILS BY DISCARD.
    It owes the archive nothing,
    so accurate detail the archive knew and the
    source does not carry is lost,
    and on a page whose content lives in
    pictures it has nothing to translate and cannot improve the English at all.

WHY THIS MATTERS FOR THE DECISION.
A rule preferring one lane,
on any measurable property of a slice,
is choosing
which of the two failures to accept everywhere.
That explains the record above better than anything already written here:
every automatic discriminator was indefensible because the discriminating fact
is not a property of the slice at all.
It is a question about the archive's text:
does this passage contain English the source does not support?

Where the answer is yes,
the translate lane is right.
Where the archive carries accurate detail beyond the source,
or the source is
not in the document text at all,
the repair lane is.
Both are per-slice judgements about evidence,
which is exactly what the panel
already does.

THE RECOMMENDATION IN `## The option that fits what already exists` therefore
stands,
and now rests on read passages rather than on telemetry that could not
separate the lanes.
It was the recommendation before this reading;
the reading is why it is the
right one rather than merely the available one.
