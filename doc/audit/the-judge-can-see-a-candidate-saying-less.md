# The judge can see a candidate that says less

Measured 2026-08-20 with `judge-fidelity-probe`,
the harness `#84` built and never ran.
Spends quota.
No corpus text here:
counts and mechanics only.

## Why it was worth running now

Two findings sat in the record separately.

The repair lane deletes the archive's specifics:
ninety-seven of two hundred and sixteen distinctive words on `saurikissa`,
two hundred and fifty-five of one thousand two hundred and forty-eight pooled over
eleven pre-window entries.

And on CONTESTED slices the translate judge replaces the archive
thirty-seven times in thirty-nine,
94.9%.

The component that almost always prefers a fresh rendering
is the same component that would have to notice a rendering saying LESS.
If it could not,
nothing in this pipeline would catch content loss at DECISION time,
since the content-survival check runs at assembly,
after the choice,
and only reports.

## The result

```text
overall      16 of 16 chose the complete text
  deletion    8 of 8
  insertion   4 of 4
  alteration  4 of 4
```

## Why this is not a length preference or a status-quo reflex

The harness was built to make both of those score badly,
and they do not survive it.

A DELETION makes the faithful candidate the LONGER one,
so a roster that simply prefers more text scores perfectly on deletions alone.
An INSERTION splices in a sentence from elsewhere in the same document,
so the faithful candidate is the SHORTER one and the length habit scores zero.
Passing both,
eight and four,
is what shows the judges are reading.

BOTH DIRECTIONS AND BOTH BALLOT POSITIONS are run over the same pair.
Preserving is right when the incumbent is the clean text and wrong when it is the
damaged one,
so a judge that keeps whatever it is given scores half.
Sixteen of sixteen against a fifty percent null is one chance in sixty-five thousand.

The damaged twin is otherwise word for word the clean text,
so it cannot lose on fluency,
register or house style,
only on saying what the original says.

## What it settles

THE JUDGE READS FOR COVERAGE.
The 94.9% replacement rate is therefore not evidence of
carelessness:
these judges demonstrably notice a candidate that omits a proposition.

`#130`'s per-slice option is VIABLE on this evidence.
Handing the judge the repair lane's text as one more candidate asks it a question it
has been shown able to answer.

## What it does not settle

SIXTEEN TRIALS.
Decisive against the fifty percent null and not a precision estimate.

THE FIXTURES MOVE WHOLE SENTENCES.
A quietly dropped qualifier inside a sentence is
the harder case and is not measured here;
`#84` recorded that limit and it stands.

SELF-PREFERENCE IS STILL UNMEASURED,
since neither candidate is written by a model on
the roster.
In production both candidates ARE roster-written,
which is a different
question from this one.

TWO OF SIX JUDGES DECLINED on one trial,
holding that neither candidate covered the
original.
That is conservatism rather than failure,
and the ensemble verdict was still
correct,
but a lane contest where both candidates are imperfect will meet it more often.

## The same judge on a REAL lane contest: it declines

Measured the same day,
and it corrects the reading above.

The fidelity result made a per-slice contest between the lanes look viable,
so the contest was run:
both lanes' renderings of one slice on one ballot,
labelled by which retains more of the ARCHIVE SLICE's distinctive words,
so neither lane is favoured by name.

```text
contests with a retention difference       6
  the ensemble DECLINED                    5
  the ensemble decided                     1
  of those decided, it chose the
    higher-retention rendering             0
```

The one decision went to the rendering that kept LESS of what the archive said.

## Why the two results differ, and which one describes production

A CONSTRUCTED TWIN differs from its clean original by exactly one moved sentence
and is otherwise word for word identical.
One candidate is then unambiguously complete,
and the sheet's first criterion,
complete coverage,
picks it out cleanly.

TWO REAL LANE RENDERINGS are independently written.
Both paraphrase,
both drop something,
neither is complete.
The declining reasons recorded in the fidelity run say exactly this in the judges'
own words:
both candidates omit propositions,
so neither satisfies the criterion,
so the ensemble declines rather than ranking two imperfect texts.

PRODUCTION IS THE SECOND CASE.
So `16 of 16` says the judges can detect an omission,
and it does NOT say they will choose between two flawed renderings.
Those are different questions and only the first was measured before.

## What this does to `#130`

THE PER-SLICE OPTION IS NOT VIABLE AS SPECIFIED.
Handing the judge the repair lane's text as one more candidate produces a decline on
five slices in six,
which leaves the lane question exactly where it was
while spending a judging round per slice to get there.

What would have to change for it to work is a different question from the one the
judge is asked today:
not "which candidate is complete",
which neither is,
but "which of these two is closer to the original",
which the selection sheet does not currently pose.
Whether that question can be posed without inviting the register-and-fluency
preferences the sheet exists to suppress is unmeasured,
and is the next thing to
settle if this route is pursued.

A DETERMINISTIC FALLBACK LOOKED LIKE THE CHEAPER ALTERNATIVE,
since retention is
already computed with no quota:
on a decline,
keep the rendering that retains more of
the archive's specifics.
The wider run below shows why that is not the free win it
appears to be.

## The wide run: thirty-four contests, and a lane preference

Six contests could not settle this,
so the older pools supplied thirty-four.

```text
contests with a retention difference      34
  the ensemble DECLINED                   28      82%
  the ensemble decided                     6

of the six decided
  chose the higher-retention rendering     1
  went to the TRANSLATE lane               6
  went to the REPAIR lane                  0
```

THE LAST LINE IS THE FINDING.
Every contest the judge decided went to the translate lane,
and in five of those six the translate rendering was the one keeping LESS of what
the archive said.

That is not an arbiter between two lanes.
It is the same preference `doc/audit/incumbent-almost-never-wins-a-contest.md` already
recorded,
where the archive's own wording loses thirty-seven contests in thirty-nine:
the judges prefer a freshly written rendering.
Given repair-lane text,
which is an EDITED incumbent,
they treat it the way they treat
an incumbent.

So a per-slice cross-lane judge would decline four times in five,
and on the fifth would hand the slice to the translate lane
regardless of which rendering kept more of the page.

## Why the deterministic fallback is not free either

Retention favours the repair lane BY CONSTRUCTION,
for the same reason the judge
disfavours it:
the repair lane edits the incumbent,
so it keeps the archive's
vocabulary unless it deliberately removes it,
while the translate lane writes fresh
English and may say the same thing in different words.

Across these thirty-four contests the higher-retention lane was
repair twenty-four times and translate ten.

A rule that always keeps the higher-retention rendering therefore ships the repair
lane about seven times in ten,
and it does so partly because of how each lane is
built rather than because of which text is better.
That is the SAME structural bias
that killed the names-and-numbers discriminator in
`doc/planning/which-lane-ships.md`,
and it has to be stated wherever the rule is
proposed.

It remains the cheapest thing that decides every slice,
at zero quota.
It is not evidence about which lane is better.
