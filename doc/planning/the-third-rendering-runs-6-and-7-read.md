# The third rendering: runs 6 and 7 read against the four measurements

The sixth run scored at thirteen of thirteen slices,
with the tied slices read,
the missing wrapper and the tense finding;
the five declines shown to be real indecision,
the changed slices read in full and what run 7 had to show;
run 7 read partway and then scored,
including the deletion it did not repeat.

Part of [The third rendering](the-third-rendering.md),
which lists every file of this record.

## Measurement 3, read on the three slices the reading called tied

Read while the run was still buying its last slices,
because the reading is the slow half and these three were already settled.
The question fixed in advance was whether a consolidation gives these slices
a text the reading would prefer to BOTH lanes.

### keyword233 slice 0: no consolidation, and that passes

The judges declined and the standing text shipped.
By the rule fixed for this slice,
where the reading recorded two good texts,
a tie passes.
The lanes differed on two words:
the repair lane wrote `opened her own channel` and `heartfelt reflections`,
the translate lane `set up her own channel` and `spiritual reflections`.
`心灵感悟` is reflection of the heart,
and `spiritual` invites a religious reading the Chinese does not carry,
so the standing text that shipped is the better of the two.

### keyword233 slice 1: better than either lane

The consolidation took `坚强不屈` as `unyielding` from the repair lane,
where the translate lane's `resilient` drops the `不屈` half outright.
It took `这在我看来很棒` as `which I find wonderful` from the translate lane,
where the repair lane's `I thought` moves a present-tense judgement into the past.
And on `她用她的智慧和热情` it wrote `She used her wisdom and passion to add a touch of brightness`,
restoring an agent that the repair lane dropped (`Her wisdom and passion brought`)
and that the translate lane demoted to an adverbial (`With her wisdom and passion`).
Nothing in either lane is missing from it.

### Weideriche_ slice 0: better than either lane, and one of the wins is a house rule

The original's block quote ends `有伤口不要用酒精x`,
where the trailing `x` is a typed flourish that says nothing in English.
The repair lane copied it across as its own line.
The translate lane dropped it.
The consolidation dropped it,
and one contest ballot cited the rule by name,
saying the translate candidate `adheres to the house rule for omitting meaningless punctuation`.
That is the punctuation rule of `HOUSE_POLICY_BLOCK` deciding a live ballot.

It also kept `伤口` singular,
`a wound`,
where the translate lane had pluralised it,
which another ballot had flagged.
And it kept the recipient in `也会向其他人表达自己的关心`,
which the translate lane dropped;
the shipped `to them` resolves an ambiguity the Chinese leaves open,
but preserving the recipient beats dropping it.

So two of the three tied slices got a text better than both lanes,
and the third correctly changed nothing.

## The consolidation has no assembly step, so nothing wraps it

Every SHIPPED consolidation in this run is one long line per paragraph while both lane texts are semantically wrapped.
The cause is structural rather than a stray setting:
`wrapReplacementText` is called from exactly two places,
`wrapTranslateRecords` in `translate-assemble.ts` and `wrapRepairOutcomes` in `repair-assemble.ts`,
and both are lane assembly steps.
A third rendering that no lane assembles inherits neither.

Nothing in production is wrong today,
because consolidation is not wired.
Wired as it stands,
every consolidated slice would ship unwrapped into a corpus #122 converted to semantic wrapping.
Filed as its own task,
blocking the wiring half of this work.

## Measurement 4 has already found one: the consolidation changed the tense of a page

`gaoyanger` slice 1 is a slice where the reading and the contest AGREED on the translate lane,
which is the population measurement 4 exists to protect.
The gate settled on a consolidation anyway,
and that consolidation moved the slice from past into present.

```text
standing: Gaoyang was a cute, gentle, and kind girl. She grew up in a single-parent family with difficult living conditions, but she was always very strong.
shipped:  Gaoyang is a cute, gentle, kind girl. She is from a single-parent family, and from childhood her living conditions were difficult, but she has always been very strong.
```

The rest of the page is past.
Chunk 0's standing text reads `Gaoyang indulged in the cradle of happiness.`,
and chunk 1's own standing text reads `Gaoyang was a pretty, tender and kind girl.`
A reader meets both tenses on one short memorial page.
The shipped sentence also disagrees with itself:
`She is from a single-parent family, and from childhood her living conditions were difficult.`

The shipped text is not worse on any other axis.
It separates `单亲家庭` from `从小生活条件不好`,
which the Chinese does carry as two clauses and the standing text had fused.
So this is a tense defect specifically,
not a bad consolidation.

### The mechanism, read off the sheets rather than guessed

`consolidate-wire.ts` splices `HOUSE_POLICY_BLOCK`,
which says nothing about tense,
so the producer choosing the wording had no guidance at all,
with the past-tense standing text in front of it.
`contest-ballot-wire.ts` splices `JUDGE_POLICY_BLOCK`,
whose forced-differences paragraph tells the gate that rendering a tenseless copula in past
or present is a choice English forces and never a fault.
The producer was free and the decider was told not to object.

That paragraph was added by #146 for a measured reason and is not the thing to undo:
three whole-slate refusals came from judges treating a forced tense as an alteration of the time referred to,
and Chinese does mark no tense.
What no sheet has ever carried is the other half.
A slice joins a page that already has a tense,
and absent a reason to change it,
the page's tense is the one to match.
Consistency with the page is a different claim from whether a forced choice is a fault,
so stating it does not reopen the refusals.

This is the run reporting a behavioural finding rather than an instrument defect,
which is what a swept instrument is supposed to produce.

## The sixth run scored complete, at 13 of 13 slices

Run 6 finished every slice.
The four measurements fixed in advance are answered
below.
No passage is quoted here:
findings are described structurally,
and the
texts stay in the run artifacts.

### Measurement 1: how often the roster backs a consolidation

The gate ran at 7 of 13 slices and shipped the consolidation at 7 of 7.

The other six never reached the gate.
Five declined at the judging step with
`declined-indecision` and shipped the incumbent (`lintong#2`,
`keyword233#0`,
`Weideriche_#1`,
`Acheron#1`,
`Acheron#2`),
and `Zha_Ke#0` has no standing text
at all,
so there was nothing to contest.

The 7 of 7 needs one qualification before it reads as endorsement.
Three of the
seven shipped on a minority of decisive ballots,
with refusals at or above the
winning count:
`keyword233#1` at two consolidated against four `neither`,
`Acheron#0` at two against four,
`Weideriche_#0` at three against three.
A
`neither` ballot means the two texts differ only in wording and neither is more
faithful,
which is the churn case the sheet elsewhere warns against.
Whether a
plurality of refusals should keep the standing text is a counting-rule question
this run raises and does not settle.

### Measurement 2: structural validity without a repair round

63 of 78 candidate renderings were valid on the first attempt,
and 6 of the 13
slices were clean at 6 of 6.
The failures cluster on two shapes:
pages whose
standing text is a block quote,
where producers flatten or multiply the quote,
and `Zha_Ke#1`,
where three producers emitted text that would not parse as MDX
at all.

### Measurement 4: damage where the contest already agreed with the reading

The answer is no,
in two different ways,
and both are real.

#### `gaoyanger#1`: the tense finding, with the ballots behind it

Recorded already as issue #152 and fixed since.
The scorecard supplies the
mechanism the earlier reading could only infer.
The panel split three to three.
Three judges preferred the past-tense candidate,
one of them saying outright
that the present tense wrongly implies the person is alive,
which in a memorial
is a defect of dignity rather than of style.
Two of the three judges who
preferred the present-tense candidate justified it by appealing to the
original's unmarked tense and its present copula,
which is exactly the reading
the forced-differences paragraph invites when nothing names the English as the
authority.
The present-tense candidate won and shipped.

This is a positive control for the #152 fix rather than an argument against
#146:
the judges who were right were already right,
and lost the vote to a
reading the fix now removes.

#### `Zha_Ke#1`: the consolidation deleted a person's will from the page

The most serious finding of the run,
and it outranks everything else open under
#138.

The slice's ORIGINAL is a single parenthetical content warning.
Its standing
English is that warning FOLLOWED BY the entire will:
a `<details>` and
`<summary>` wrapper,
dozens of lines,
a numbered list of final wishes.
The
shipped consolidation is the warning alone.
Everything after it is gone,
and
the gate voted six to nil to ship it.

Three separate defects compose to produce that,
and each is independently
actionable.

    1.  The slice pairs one line of Chinese against a span of English many times
        its size. Whatever the pairing intended, the consolidation was asked to
        render a note and handed a will as the text it replaces.
    2.  The structural floor could not stop it. The scorecard's floor section
        reports `Zha_Ke#1` as `shipped=valid` while `standing=INVALID:1`: the
        standing text does not parse, so the guard has no block list to floor
        on, and a one-paragraph candidate passes against a page it cannot read.
        The floor is disabled exactly where it mattered most.
    3.  The faithfulness criterion then rewards the deletion. One ballot
        objected to candidates that "add an entire 'My Will' document not
        present in the ORIGINAL", reasoning correctly from the criterion it was
        given. Another ballot objected in the opposite direction, that rival
        candidates "omit the body of the will". The panel had no rule telling it
        which of those two readings governs.

Nothing here is specific to consolidation.
A selection between two lanes would
face the same three defects;
the third rendering is what made them visible.

### Two findings about the wrapper, pointing opposite ways

Every one of the seven shipped consolidations is unwrapped:
a single long line
per paragraph,
where both lane candidates arrive semantically wrapped.
This is
#151 observed live rather than inferred from the call graph.

The second finding constrains how #151 may be fixed.
Judges actively penalise
semantic wrapping in their ballots,
calling it arbitrary line breaks,
unnecessary line breaks,
and breaking mid-sentence in a way that undermines
coherent reading.
At least five ballots across four slices demote a candidate
for it.
So the wrapper must run AFTER selection and never on a candidate a judge
is shown,
or fixing #151 will cost the wrapped lane its votes.

### Correction to the `Zha_Ke#1` account, and the real cause

The finding stands and the mechanism is confirmed,
but the evidence first cited
for it was the wrong field,
and the cause is one layer deeper than described.

WHAT WAS WRONG.
The account cited the scorecard's `standing=INVALID:1` as proof
that the page does not parse.
`standingText` in the bed is not the archive page:
`standingOf` returns the WINNING LANE'S candidate,
the text that would ship if
the consolidation lost.
The page is `incumbentText`.

WHAT WAS MEASURED INSTEAD,
by running the production validator directly over the
run-6 state rather than reading the scorecard's summary:

    -   `sourceText` is 41 characters on one line.
    -   `incumbentText` is 3875 characters over 73 lines.
    -   `repairText` and `translateText` are both 3875 characters: BOTH LANES
        KEPT THE WHOLE THING.
    -   `shippedText` is 164 characters on one line.
    -   `validateTranslatedSlice` returns `valid` for the shipped text and
        `invalid` for the page itself, whose finding is that it could not be
        parsed as Markdown.

So the conclusion holds exactly as stated:
the floor was disabled because the
page does not parse,
and a candidate that drops 3711 of 3875 characters passed
against the source alone.

THE CAUSE,
which the first account did not reach.
The strict grammar's complaint
is `Expected a closing tag for <details>`.
The slice boundary cuts THROUGH an
HTML element:
the opening tag is inside this slice's span and the closing tag is
not.
The span is therefore not a well-formed document fragment,
and no strict
parse of it can succeed no matter what any producer writes.

The same page parses cleanly under the plain-markdown grammar,
which reads it as
two blocks,
a paragraph and an html block.
Under that reading the floor bites
correctly and the one-paragraph candidate is refused.

This reframes the slicing task:
not "one line of Chinese paired against a large
English span" alone,
but a slicer that splits inside an HTML element.

## The five declines are real, not voice loss

Measured from the archived run-6 state,
no quota.
This answers the question
#130 had to settle before any decline policy could be written on top of it,
because a policy calibrated on infrastructure failures would be calibrated on
noise.

EVERY ONE OF THE FIVE HAD A FULL PANEL.
All five record `judgesAvailable: 6`
and `ballots: 6`:
every judge answered,
and none was lost to an empty reply or
a cut stream.
The earlier declines in this bed that were caused by voice loss
were a different thing,
and none of these is one.
They are genuine indecision.

They are also near misses rather than chaos.
The picks,
by candidate index:

    -   `lintong#2` scattered across four candidates, best supported by two.
    -   `keyword233#0` across four, best supported by three, and every one of
        the six ballots was a self-vote.
    -   `Weideriche_#1` split three against three between only two candidates.
    -   `Acheron#1` across five, best supported by two.
    -   `Acheron#2` across four, best supported by two.

The seven slices that DID reach the gate look barely different:
their best
candidate carried three or four votes against the declines' two or three,
and
`gaoyanger#1` proceeded on the same three-against-three shape between two
candidates that made `Weideriche_#1` decline.
What separates them is the
weighting rather than the raw count,
and self-voting is doing much of that
work:
`keyword233#0` weighed to nothing because all six judges preferred their
own candidate.

That last observation belongs to #91,
which is about weighing self-certification
rather than forbidding it,
and it now has a measured case behind it.

### What this settles, and what it does not

It settles that the decline rate is behaviour rather than infrastructure,
so it
can be reasoned about.

The remaining half of #130 is a values choice rather than a measurement:
what
SHOULD happen when a full panel cannot agree.
The current behaviour keeps the
incumbent,
and that is the reading consistent with everything else decided for
this corpus,
where reader protection outranks completeness and a tie already
keeps the standing text.
It is adopted on that basis rather than left open,
and
it is the owner's to overrule.

## Run 6's changed slices read in full, and what run 7 had to show

### Measurement 4 in full: the three changed slices that lost nothing

Five run-6 slices shipped text differing from the standing rendering.
Two were
read and recorded as damaged:
`gaoyanger#1` drifted into the present tense,
and
`Zha_Ke#1` shipped a short note where the archive carried a will.
The other
three were flagged for reading and never got a recorded verdict,
so measurement
4 stood as a bare "no,
twice" without saying what the rest of the changed
slices did.
All three are now read against the same question,
which is whether
anything the standing text carried is lost.
The passages stay in the run
artifacts under `~/temp/agent`;
what follows describes the changes rather than
reproducing them.

`lintong#1` differs in exactly two places.
The subject's name changes from a
two-word capitalised spelling to the lowercase single-word form the entry
declares,
which is the ground every judge quoted in the run-6 name tripwire gave
for their pick.
One sentence loses a trailing English connective that the
original marks nowhere.
The attribution line survives intact and every other
line is identical.
Nothing is lost.

`Zha_Ke#3` makes two changes and both are what the ballots argued for.
A
connective the standing text had read as additive is read as contrastive
instead,
and a verb phrase regains the object the original names,
which the
standing text had left implicit.
The outbound link survives unchanged.
Nothing
is lost,
and two readings improve.

`Acheron#0` is word for word identical to the standing text.
It differs only in
that the semantic line breaks are gone,
which is #151 rather than a loss of
content.

Measurement 4 therefore reads three clean against two damaged,
rather than two
damaged against silence.
Both damaged slices now have a fix and a guard behind
them,
in #152 and #153,
and confirming those hold in production is what run 7 is
for.

### What run 7 must show, written before run 7 is read

Recorded before the run finishes,
for the same reason the `gaoyanger#1`
prediction was worth recording:
an expectation written afterwards cannot be
wrong,
and so cannot be evidence.

`gaoyanger#1` must not ship a present-tense rendering.
The #152 sheets now name
the English as the authority for tense wherever the original leaves it unmarked.
A present-tense rendering shipping again means #152 failed in production,
which
is an escalation rather than a cue to re-read the ballots.

`Zha_Ke#1` must ship nothing.
Every candidate at that slice should be refused by
the floor,
the incumbent retained,
and the will preserved.
Expect repair-round
churn there,
because the finding the floor emits is a kind mismatch no candidate
can act on while #154 is open.
That churn is #153 working rather than a hang or
a fresh defect.
The one thing worth watching is whether the bed retries an
unfixable refusal past its bounded rounds,
which would be a bound to fix in the
bed script rather than a pipeline defect.

The #155 change to DROPPED carries an overcorrection risk,
because a criterion
counting anything the archive holds and a candidate omits structurally favours
the lane that starts from the archive.
Run 6 settled 8 contests to translate,
4
to repair,
and 1 to neither.
Translate wins collapsing toward two or three would
mean the criterion overshot,
and it would need the bounding pass #143 got.

Decline counts and `neither` counts may move in either direction,
because the
sheets themselves changed.
That is #130 data rather than noise,
and it should be
read as a fresh measurement rather than compared for stability.

### #152 holds in production

Run 7 shipped a fresh consolidation at `gaoyanger#1`,
the same slice run 6
damaged.
Run 6's rendering opened in the present tense and then switched to the
past inside a single sentence,
which is the exact pair of faults the #152 rule
names.
Run 7's rendering holds the past throughout and no longer disagrees with
itself.

The expectation was recorded before the run was read,
so it is evidence rather
than a reading fitted to a result.

## Run 7 read partway, and the runaway stream it exposed

### Run 7 at seven of thirteen slices

Reading the run against the expectations recorded before it,
on the seven
slices settled so far.

The #155 tripwire is not tripped.
On these same seven slices run 6 settled four
contests to translate and three to repair,
and run 7 settles three and four.
One slice moved,
`keyword233#1` from translate to repair.
That is a single
change rather than the collapse toward two or three that would have meant the
DROPPED criterion overshot.

Decisions moved more than contests did.
Two slices,
`keyword233#0` and
`Weideriche_#1`,
went from `declined-indecision` to a settled judgement and now
ship,
and `Weideriche_#0` did the same earlier.
So the changed sheets are
resolving panels that previously could not agree.
That is #130 evidence and it
points the same way the decline classification did:
the declines were behaviour
rather than infrastructure,
and changing what the judges were told changed the
behaviour.

`gaoyanger#1` and `Zha_Ke#1`,
the two slices the run exists to check,
are one
settled and one still pending.

### A runaway shape the guard was not built for

The run log carries three `StreamCutShortError` events,
two of them at 1089815
and 2014974 characters,
both from the same model,
both ended by the 180-second
deadline rather than by any guard.

That matters because the client-side runaway guard from #119 and #120 is a
degeneracy detector:
it looks for repetition and raises
`StreamDegenerateError`.
Neither of these is that error,
so the guard never
fired.
The premise that a runaway repeats itself does not cover a model emitting
two million characters the scanner does not read as degenerate,
and nothing else
caps output size.

Filed as #156 rather than folded into #154,
with the cap left to be derived from
the observed distribution of completed responses rather than guessed.

## Run 7 scored, and the deletion it did NOT repeat

### What the headline did, and why it cannot yet be credited to anything

Counting the same way for both runs over the same 13 slices:

- Run 6:
  the gate ran at 7 slices,
  shipped consolidated at all 7,
  and 5 slices ended `declined-indecision`.
- Run 7:
  the gate ran at 10 slices,
  shipped consolidated at all 10,
  and 2 slices ended `declined-indecision`.

The fixes that landed between the two runs were aimed squarely at judges who could not agree,
so the direction is the one they were built to produce.
That is not evidence they produced it.
Twelve of the 13 slices moved between the runs,
including slices where nothing changed but the wording,
because the models are redrawn on every run and the bed buys every call fresh.
A shift from 7 to 10 out of 13 is well inside the spread two draws of that size can produce on their own,
and nothing here has measured that spread.
Run 8 does NOT bound it either,
and saying so here corrects a line this document briefly carried.
A run-to-run band has to be measured on ONE UNCHANGED BUILD.
Run 8 carries ONE change run 7 did not:
the `#158` scanner fix,
which is not telemetry-only.
Arming the degeneration detector on 47 percent of traffic that was previously invisible to it
can change outcomes on its own,
because a call whose thinking repeats past 131072 characters is now cut where run 7 let it complete.
So run 8 against run 7 still carries a confound,
though a narrower one than run 7 against run 6.

The bed's slices are NOT the second change,
and a line this document carried saying otherwise was wrong.
The bed does not slice anything.
It replays static artifacts under `~/temp/agent/readable-20260820` and `~/temp/agent/pairfix-20260820`,
every one of them written on 2026-08-20 between 17:34 and 19:29.
The `#154` widening landed at 2026-08-21 14:50:40,
about twenty hours later.
Run 8 therefore replays PRE-WIDENING slices,
exactly as run 7 did.

That static input is what makes run 9 against run 8 a clean pair:
identical build,
identical slices,
nothing touched between them.
Until that exists the honest reading is that the gate reached more slices in run 7 and the reason is unmeasured.

### `Zha_Ke#1`: run 6 shipped a 96 percent deletion, and run 7 did not

THE RUN ATTRIBUTION HERE WAS WRONG IN AN EARLIER VERSION OF THIS DOCUMENT,
which read as though run 7 shipped the deletion.
It did not.
Read straight off the two state files:

```text
run 6   Zha_Ke#1   decision=judged                origin=fresh       incumbent=3875   shipped= 164
run 7   Zha_Ke#1   decision=declined-indecision   origin=incumbent   incumbent=3875   shipped=3875
```

Run 6 deleted the letter.
Run 7 kept every character of it.

WHAT CHANGED BETWEEN THEM IS THE DECISION,
NOT THE JUDGEMENT.
Run 7's panel still reasoned the deleting way:
its recorded selection ballot calls the letter-preserving candidates
"unacceptable because they add the entire letter text,
names,
and metadata not present in the ORIGINAL".
The panel did not converge,
the gate returned `declined-indecision`,
and the fallback to the standing text is what saved the letter.
So the letter survived run 7 by the safety net,
not by the panel changing its mind.

RUN 7 IS ALSO THE FIRST RUN CARRYING `#155`.
The policy clause making deletion of page-only content a named fault landed at 13:45:20,
and run 7's state file was created at 13:46:48,
eighty-eight seconds later.
Run 6 finished at 13:21,
before it existed.
That makes run 6 against run 7 the only before-and-after pair for that clause,
and one slice is not enough to attribute the change to it:
`declined-indecision` is exactly what a panel split by a new fault clause looks like,
and also exactly what an ordinary tie looks like.

TWO OTHER ROWS NEED SEPARATING FROM THE DELETION COUNT.
Run 6's `Zha_Ke#0` and run 7's `Acheron#0` both carry `shipped=0` with `decision` undefined.
Those are slices that failed rather than slices that deleted,
and counting them as deletions would overstate the rate in both runs.

Run 7 does carry one large reduction that is not a failure:
`Acheron#1` shipped 172 characters against 328 standing,
judged and fresh,
a 48 percent cut.
Nothing has read that slice.

What follows describes RUN 6's deletion,
which remains the clearest evidence in the bed.

Run 6 replaced 3875 characters of standing text with 164 characters,
and the gate ballots were 6 for consolidated,
0 against.
Every other slice in both runs shipped between 90 and 111 percent of what stood.
This one shipped 4 percent.

The cause is not the judges.
Their own recorded reason is that the standing candidate
"includes the entire letter body and a metadata sentence not present in the original passage",
which is true,
and which they were right to notice.
They were shown 41 characters of source against 3875 characters of standing English and asked which was faithful.
Given that pairing the short answer IS the faithful one.

The pairing is the defect.
Every other slice in the bed has a source-to-standing ratio between 2.1 and 4.0,
which is what a Chinese to English pair looks like.
`Zha_Ke#1` sits at 94.5.

`Zha_Ke`'s English page carries ONE container spanning offsets 372 to 4045,
a span of 3673 characters,
and its Chinese source carries none at all.
The letter is content the English page has and the Chinese does not,
and the slicing paired it against a note of 41 characters.

### The structural check filtered in the wrong direction

Three of the six candidates at that slice reproduced the letter at about 3800 characters,
and all three were marked `invalid`,
every one of them for the same reason:
`MdxParseError: MDX body refused to parse`.
The three candidates marked `valid` were 136,
161 and 164 characters,
all of them deletions.

So at this slice the structural check refused exactly the candidates that preserved the content
and admitted only the candidates that dropped it.
The floor did not fail to protect the letter.
It selected against it.

That parse failure has a named cause,
and it is `#154`:
these slices were cut before the container widening,
so the container's tags belonged to no node and the slice text carried a letter body without its opening
and closing tags.
A candidate faithfully reproducing that text reproduces the imbalance,
and unbalanced MDX does not parse.
A candidate that rewrites it into a short clean note parses fine.

### What run 8 does NOT test

An earlier version of this section said run 8 was the first run whose slices are cut after the widening,
so it would be the live test of the residual expectation recorded here as unobserved.
That was wrong,
and the timestamps settle it:
the bed replays artifacts written 2026-08-20,
and the widening landed 2026-08-21 14:50:40.
Run 8 never exercises the widening.

The consequence for scoring matters more than the correction.
At `Zha_Ke#1`,
a still-failing faithful candidate is NOT evidence the widening failed,
and a passing one is NOT evidence it worked.
The path was not walked.
Testing the widening needs slices cut from the current build,
which the bed as written never does.

`Zha_Ke#1` should also come OUT of run 8's accuracy count for the band comparison.
Its `expect: 'repair'` label was set by a reading that did not know
93 percent of the slice's standing text was material the pairing had already declined to pair.
A malformed slice scoring as a clean accuracy point in a thirteen-item bed
moves the number this document is trying to bound.
Score it,
record it,
and exclude it from the comparison with a note.

The widening does not fix the pairing,
and neither does anything else here.
Forty-one characters of source against a 2909-character letter the roster explicitly refused to pair
is a slice no judge should be shown at all,
whatever parses.
That is `#157`,
and this is the strongest evidence for it in the corpus so far:
not a pairing that produced a confidently wrong answer,
but a CORRECT pairing whose refusal the slicer discarded,
coming within a single unanimous ballot of deleting a letter someone wrote about a person who died.
