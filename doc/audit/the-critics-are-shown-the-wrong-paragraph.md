# The critics are shown the wrong paragraph

Found 2026-08-20 by reading one entry's source and translation end to end,
after the owner said to stop asking and measure,
and not to hesitate to read the corpus.
No corpus text here:
slice indices,
lengths and ratios only.

## What reading `saurikissa` showed

Its Chinese and English say the same things in the same order.
The slicer still paired them wrongly,
because the ENGLISH SPLITS A PARAGRAPH THE
CHINESE KEEPS WHOLE:
a request for advice sits inside the previous Chinese paragraph
and stands alone in English.
Everything after that point runs one block out of step.

Reading each pair the critics were actually given:

```text
slice 3    Chinese: a two-clause reply          English: a four-line passage that
                                                 belongs to the PREVIOUS Chinese block
slice 5    Chinese: a one-line quotation        English: a passage about eyeglasses
slice 6    Chinese: uniform, collections        English: travel, a cat
slice 7    Chinese: travel, a cat               English: relationships
slice 8    Chinese: relationships               English: a death, and grief
slice 9    Chinese: two quoted exchanges        English: the third day after a death
```

SIX OF ELEVEN SLICES PAIR UNRELATED PARAGRAPHS.
The artifact records `alignmentFindings: 0`.

## This explains almost everything chased this week

A critic shown English that genuinely is not in the Chinese beside it
files an unsupported-addition claim and is RIGHT to.
The panel accepts it and the editor removes the passage.
Every stage behaves correctly on the input it was handed,
and the result is that content the original DOES carry leaves the page.

It also explains why `#107`'s relocation window helps:
when the pairing is off by one block,
the correct source IS the neighbour,
so showing the neighbours hands the critic the right text by accident.
THE WINDOW IS A WORKAROUND FOR MISPAIRING,
not a fix for it,
and its measured benefit should be read that way.

It explains the panel accepting less under the window,
the judges declining a lane
contest,
and the repair lane deleting the specifics of a life.
One cause underneath.

## How widespread

Chinese is denser than English,
so a correctly paired slice sits in a narrow band of
character ratios.
Against each entry's own median,
at a 2.5x threshold either way:

```text
  lintong             4 slices   median 2.36   outliers 3
  saurikissa         10 slices   median 4.10   outliers 4
  wangzihao980        6 slices   median 1.49   outliers 2
  Chinatsu_Suzuki    12 slices   median 2.58   outliers 4
  GLaDOSister        10 slices   median 2.47   outliers 1
  dogesir_           10 slices   median 2.43   outliers 1
  Acheron, AmbeR_the_anpa, Anilovr, Arita        outliers 0

  POOLED             15 of 80 slices        18.8%
```

THIS UNDERCOUNTS.
On `saurikissa` the ratio test flags four slices and reading found
six.
A pair can be completely mismatched and still have an ordinary ratio,
which is
exactly what happens when two adjacent paragraphs are similar in length.
Treat 18.8%
as a floor.

## What follows

FIX THE PAIRING,
and everything downstream is measuring something different.
Judging any stage,
lane or rule on runs made with mispaired input is judging it on
noise,
and several conclusions recorded this week rest on exactly that.

THE REPAIR LANE IS NOT TO BE DISCARDED over this,
per the owner on 2026-08-20:
some entries were translated by a person,
and those translations contain passages done
right and better than a fresh rendering.
The incumbent is evidence and a starting point
worth preserving where it is right,
which is what
`doc/decision/translation-repair-output-goal.md` already decided.
The defect here is that the pipeline cannot currently tell where it IS right,
because it is comparing it against the wrong paragraph.

## It is not stale artifacts, and it is not `#98`

TWO CANDIDATE EXPLANATIONS,
both checked and both wrong.

NOT STALE.
`align-blocks.ts` landed 2026-07-26,
nearly a month before these runs,
and
it is wired into subdivision through `group-aligned.ts` and `slice-pair.ts`,
whose own
comment says pairing by shared index "was tried and is wrong".
The run settled TODAY
mispairs `saurikissa` exactly as the older ones do.

NOT `#98`.
That holds the SECTION-level shortcut,
where equal chunk counts pair by
index and report nothing.
`saurikissa` has one section on each side,
so its section
pairing is correct.
The drift is INSIDE the section,
between paragraphs,
which is
`alignBlocks`' job.

## Why the block aligner cannot see it

Its scoring is,
by design,
language-neutral:
block kind,
shared script-neutral tokens,
and a plausible length ratio.
On this entry:

KIND CARRIES NOTHING,
because every block on both sides is a paragraph,
so the
kind-match score is constant across every candidate pairing.

TOKEN OVERLAP CARRIES ALMOST NOTHING,
because the tokens it counts are Latin words,
digit runs and component names.
Chinese and English paragraphs of ordinary prose share
none.
`AirPods` is the rare exception.

LENGTH RATIO IS ALL THAT REMAINS,
and one weak signal cannot hold a monotone walk on
course across eleven blocks.

THE ANCHORS EXIST AND IT CANNOT READ THEM.
This entry carries transliterated names on
both sides throughout:
a friend's name,
three place names,
a subculture's name,
and a
person referred to by a Latin initialism.
Every one is a strong pairing signal and
every one is invisible to a matcher comparing Latin tokens,
because the Chinese side
is not Latin.

## The fix is one already written down for headings

`#71` proposed exactly this for the section aligner:
"a matcher that scored heading
similarity,
including transliteration and shared Latin substrings".
The same matcher is
what the BLOCK aligner needs,
and for the same reason.

That makes transliteration-aware scoring a shared dependency of `#71`,
`#98` and this,
rather than three separate alignment tasks,
and it moves it ahead of every stage-tuning
question:
no prompt,
window,
panel rule or lane choice can be judged on runs whose
critics were shown the wrong paragraph.

## Settled with correct pairing

Two runs on the corrected build,
2026-08-20.

FIRST RUN found two defects in the pairing code itself,
which is what a live run
is for.
`saurikissa` paired 16 of 16 and settled cleanly;
`lintong` collapsed
from five slices to one,
because all six models paired one translation block
with two originals and the reader refused every reply.
A translation may MERGE
paragraphs exactly as it may split them,
and the reader forbade the mirror of the
case it allowed.
Separately,
pairing was bought on every resume,
which
`pass-entry`'s own test caught.

SECOND RUN,
with merges carried and the pairing cached:

```text
                    slices   alignment findings   pairing
  saurikissa             9                    0   16 pairs, 6 of 6 voices usable
  lintong                3                    0    9 pairs, 6 of 6 voices usable
```

`lintong`'s nine pairs over seven original and five translation blocks is more
pairs than target blocks,
which only a merge can produce.

### What moved

```text
                    resolved   accepted   archive retention
  saurikissa
    baseline              23       63%          55.1%
    window                 6       35%          91.2%
    paired                34       44%          78.2%
  lintong
    baseline              23       79%          71.4%
    window                18       70%          77.1%
    paired                18       83%          71.4%
```

`saurikissa` resolves more than any earlier arm,
and `lintong`'s panel accepts a
larger share than any earlier arm.

### What this does NOT show

ARCHIVE RETENTION FELL against the window arm,
and that is not evidence either
way.
`doc/decision/translation-repair-output-goal.md` decides the output is
judged against the ORIGINAL,
never against the input translation,
and retention
counts archive words.
The window scores 91% partly by resolving six issues:
an
arm that resolves thirty-four necessarily diverges more from the incumbent,
and
a rare-word test cannot separate "removed what it should not have" from
"repaired a great deal correctly".

So the pairing work establishes that every stage now reads the paragraph it
should.
It does not establish that the output is better,
because that is a
source-anchored judgement and nothing here makes one.
It is the same wall
`doc/planning/which-lane-ships.md` reaches.

## Reading the paired output against the Chinese

The measurements above cannot say whether the OUTPUT is better,
because they
compare against the incumbent.
This is a reading against the original,
which is
the decided standard.

`saurikissa` slice 1,
the opening of the story.

WHAT THE ARCHIVE SAID:
that she stepped into the internet under her present name
and was met with suspicion;
that she was transparent about her missteps "when she
was a child";
and that "after she realized the dangers of Esu culture" she hoped
to use her experience to help friends.

WHAT THE PAIRED RUN SHIPS:
that she reappeared online under her present name A
SECOND TIME and was initially not trusted;
that she did not shy away from
discussing her participation during her MINOR YEARS;
and that she hoped to use
her experience to help friends.

THREE GAINS,
each checkable against the Chinese:

```text
第二次        "a second time"    the archive dropped it entirely; restored
未成年        "minor years"      the archive said "when she was a child"
                                 which is vaguer and slightly wrong
"After she realized the dangers of Esu culture"
                                 GONE. The Chinese carries no such clause
```

THE THIRD IS THE ONE THAT MATTERS.
That inserted clause is the defect four
critics independently reported under the mispaired build,
which the panel
rejected two votes to three,
and which
`doc/audit/the-judge-can-see-a-candidate-saying-less.md` recorded as a rejection
with no relocation available to justify it.
With the paragraph paired correctly
the claim survives and the clause is gone from shipped text.

### What is worse, stated too

REGISTER SLIPS IN PLACES.
The mother's exasperated-but-fond line becomes "Ah,
what a
headache of a child she is",
where the archive had "Oh,
you troublesome,
endearing soul".
The paired rendering is closer to the words and further from the
voice,
and it uses the present tense of someone who has died.

SPACING ARTEFACTS appear inside sentences,
two spaces where one belongs,
which no
stage currently checks.

### The verdict this supports

ON FIDELITY TO THE ORIGINAL,
which is the decided standard,
the paired output is
better than the archive on this passage:
it restores a dropped fact,
sharpens a
vague one,
and removes an invention.

ON VOICE it is somewhat worse in at least one place,
and that is a real cost on a
memorial page rather than a rounding error.

Neither observation generalises from one passage.
What it does establish is that
the question is now ANSWERABLE by reading,
which it was not while the critics
were shown the wrong paragraph.

## Reading `lintong` complete, both lanes against the Chinese

`lintong` is three slices,
short enough to judge entirely rather than in part.
Read at corpus `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`,
against the run in
`~/temp/agent/pairing2-20260820`.

### The repair lane corrects a factual claim about the person

The archive opens the partner's message with "Lin Tong is a girl who is a bit
insecure."
The Chinese says 占有欲比较强烈:
strongly possessive.
Insecure and possessive are not the same claim about someone,
and the archive's is not what her partner wrote.
The repair lane returns "a girl with a rather strong possessive streak".

Four more in the same passage:

-   粘人 is absent from the archive.
    The repair lane restores it as "she was clingy,
    always wanting to be close".

-   可能是因为我吧 is absent from the archive.
    The repair lane restores it as "Maybe that's because of me",
    which is the partner taking responsibility and the most personal clause in
    the passage.

-   The clause about what she made you do turns on 逼,
    to force or compel.
    The archive softens it to "her tendency to ask others to do things when they
    are not in the mood for it".
    The repair lane returns "loved to force you into doing things you didn't
    like".

-   熟人 is acquaintances,
    not friends.
    The archive says "not her friends",
    which reverses who she was willing to
    trouble.
    The repair lane returns "not mere acquaintances".

### The translate lane preserved every one of them

On both prose slices the translate lane returned the incumbent unchanged:
`changedSliceCount` is 1,
and the one slice it changed is the `## 简介` heading.
So on this entry the two lanes are not close.
The lane built to re-translate from the original kept an archive sentence that
misdescribes the person,
and the lane built to repair the archive is the one that caught it.

This is the concrete evidence `#130` was missing.
It does not settle `#130`:
one entry is not a corpus,
and the reading in `## Reading the paired output against the Chinese` shows the
repair lane losing register on `saurikissa`.
It does refute the simplest framing,
that the translate lane is the safer
default because it owes the archive nothing.

### Costs, stated

TENSE IS INCONSISTENT ACROSS ONE PASSAGE:
"Lin Tong **is** a girl",
"**were** her favorites",
"she **was** clingy",
"**Likes** to trouble her partner".
Chinese is tenseless so the choice is the renderer's,
and on a memorial page the present tense for a deceased person is a decision
rather than a slip.
Nothing in the pipeline checks tense consistency within a passage,
and the archive was not consistent either.

瓶 IS A BOTTLE,
rendered "can".
Minor,
and the archive says "can" too.

### One paragraph never reached a slice at all

The closing Chinese paragraph,
the friend's closing paragraph,
its dream request and its date,
appears in no slice's source,
while its English rendering sits in the incumbent of slice 2.
The repair lane,
shown English with no original behind it,
deleted the rendering of the dream request and left a bare `> ` line in the shipped
text.

This is not a judgement call the lane got wrong.
It is `blockPairingToSteps` losing a block:
an original whose first rendering an earlier original already claimed,
and which also renders further blocks of its own,
was never placed on either side.
A sweep over 20000 admissible monotone pairings lost a source block in 6.8% of
them.
Fixed in `4434618ea`,
with the sweep reporting zero violations after.

CONFIRMED AGAINST THE ROSTER'S ACTUAL PAIRING,
not inferred from the shape of
the output.
A settled run discards its pairing cache by design,
so a fresh single-entry run was read mid-flight,
before settlement:

```json
[{"source":0,"target":0},{"source":1,"target":1},{"source":1,"target":2},
 {"source":2,"target":3},{"source":3,"target":3},{"source":4,"target":3},
 {"source":5,"target":3},{"source":6,"target":3},{"source":6,"target":4}]
```

Original 6 is paired with translation block 3,
which originals 2 through 5 already claimed,
AND with block 4,
which nothing had claimed:
exactly the merge-then-split shape.
Under the old rule it matched neither branch and was placed nowhere.
Under the fix all seven originals are placed,
and slice 2 carries `block/6` beside the incumbent it belongs to.

So the incident is this defect rather than `#90`/`#100` one-sided slicing,
which was the other candidate and would have needed a different fix.

### The stray space in every wrapped line is ours

The shipped blockquotes carry `>  she was clingy,` where the archive carries
`> `.
It is not a model artefact:
the `semantic-line-breaks` fixer inserted its break in FRONT of the space that
separated the two clauses,
so the space opened the continuation line.
The same defect put a three-space indent under every `- ` marker it wrapped.
Fixed in `9a4643877`.
Passages settled before that keep the older shape,
which the rule now leaves untouched rather than re-wrapping,
so a cache replay cannot return wording no lane produced.

## The instrument that would have caught it

Nothing reported the missing paragraph.
The entry settled with `alignmentFindings: 0`,
every stage behaved correctly on the input it was given,
and the only reason it was found is that somebody read the whole entry against
its Chinese.

WHY THE EXISTING CHECK COULD NOT SEE IT.
`assertSpanContiguity` compares a slice against the range it claims,
which catches a slice carrying fewer blocks than its own offsets cover.
A block that reached NO slice has no range covering it,
so no range disagrees with itself and the check passes.

`assertSliceCoverage` closes that,
comparing the carving against the chunk pair that went in,
per side,
and refusing three distinct faults:
a block placed nowhere,
a block placed twice,
which inflates a run past its budget,
and blocks placed out of document order,
which gathers text from two places into one slice.

MEASURED BEFORE IT WAS ALLOWED TO THROW,
because a guard that fires on a legitimate case breaks production rather than
protecting it:
all 92 corpus pairs prepare cleanly on the deterministic path.
Landed in `38af0a261`.

CORPUS-WIDE CHECK OF WHAT ALREADY SHIPPED:
of 26 settled two-lane artifacts,
four carry a body paragraph that reached no slice.
Three of those are HTML contribution-credit comments,
which the pipeline masks by design.
`lintong` is the only genuine loss.

## Verified at the artifact, not at the unit test

A fresh single-entry run of `lintong` into `~/temp/agent/pairfix-20260820`,
on the fixed build,
settling 3 slices with zero alignment findings:

-   The closing Chinese paragraph now reaches slice 2's source.

-   the dream request ships,
    as "Visit me in my dreams when you have time...",
    where the previous run deleted it.

-   The date ships,
    as "October 2022."

-   No continuation opens with a stray space,
    and no line ends in whitespace.

-   No blockquote stump remains.
    Counting only a `>` line NOT followed by more blockquote,
    since a `>` line
    between two paragraphs INSIDE a blockquote is ordinary and the archive
    carries several:
    archive 0,
    before the fix 1,
    after the fix 0.
    A first count that flagged every bare `>` reported a failure that was not
    one,
    which is why the discriminator is stated here rather than the verdict
    alone.

WHAT THE REPAIR LANE RETURNED THIS TIME,
on the passage read earlier:
"a girl who is quite possessive" for 占有欲比较强烈,
"She was clingy,
too."
for 粘人,
"Maybe it's because of me."
for 可能是因为我吧,
"her tendency to force you to do things you don't like" for the 逼 clause,
and "not her acquaintances" for the 熟人 clause.
Every correction the earlier reading identified survives a re-run with a
different sample,
which is worth more than any one of them appearing once.

STILL UNCHECKED BY ANY STAGE:
the tense is still mixed within the passage,
"Lin Tong **is** a girl" beside "she **was** clingy" and "**Likes** to trouble".

## The guard immediately found a second way to lose text

Within an hour of being wired in,
`assertSliceCoverage` caught a defect nobody had looked for.

`mergeOneSidedRuns` folds a run that ended up with nothing on one side into a
neighbour,
holding leading one-sided runs until a two-sided run arrives to carry them.
If none ever arrived it discarded them,
justified by a comment:
"anything still held had no two-sided run anywhere:
one side of the section is empty,
and the caller's own fallback owns that case."

THAT PREMISE IS FALSE.
A supplied pairing that pairs NOTHING makes every step one-sided,
and a budget no single block fits inside then puts each in its own run.
Both sides carry blocks,
so the caller's empty-side fallback never fires,
and the entire section left the document with nothing recording it.

MEASURED by fuzzing the whole slicing path with randomised in-range pairings
over all 92 corpus pairs:
10 of 920 lost a section,
zero after the fix.
The deterministic path was 92 of 92 before and after,
which is why nothing had ever seen this:
it needs a pairing only the roster produces.

THE FIX FLUSHES held blocks into a final run when BOTH sides are held,
not either.
With only one side held the section really does have an empty side,
a one-sided run is a slice nobody can review,
and the module's stated exception survives untouched.
Landed in `73e37c83d`.

WHAT THIS SAYS ABOUT THE FIRST FUZZ ROUND.
An earlier sweep with indices drawn without regard to a section's real block
counts reported 378 of 920 failing.
That number is meaningless:
`readBlockPairing` refuses an out-of-range pair,
so production never produces
one,
and the sweep was measuring a shape that cannot occur.
Bounding the indices by each section's actual counts is what turned a scary
number into the one real defect.
