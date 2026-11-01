# The third rendering: the bed, the shape rule and the sheet defects fixed before run 6

What the calibration bed can and cannot show,
the scorecard,
and the first slice that cleared the tripwire;
the measurement that neither text is a correct structural authority and the shape rule that replaced match-either;
the sheet and criterion defects found by reading ballots and by sweeping every prompt,
fixed before the bed was bought again,
except the verse addendum,
which was filed as a production follow-up.

Part of [The third rendering](the-third-rendering.md),
which lists every file of this record.

## What the bed can and cannot show

Three confounds were checked against the artifacts before the bed was scored,
because each would have made the readout mean something other than what it
claims.

NO SLICE IN THE BED CARRIES A PICTURE.
The translate lane is shown `pictureContext` in production and the calibration
arm passes none,
so a bed slice whose passage transcribes an image would put
blind consolidation producers against a candidate that could see.
Measured:
no bed slice source carries image syntax,
and no settled artifact for
the six entries records a picture reading at all.
The readout is clean on that axis,
and the wiring is still owed the day a bed
includes one.

AT 5 OF 13 SLICES ONE CANDIDATE IS THE ARCHIVE,
CHARACTER FOR CHARACTER.
The translate lane returned the archive verbatim at `lintong` slices 1 and 2 and
at `Zha_Ke` slice 1;
the repair lane returned it verbatim at `Acheron` slices 0
and 1.
The sheet renders that text twice,
once as ARCHIVE RENDERING and once as a
CANDIDATE,
and a producer weighing the apparent agreement of two sources is
counting one source twice.
This is production's behaviour and not the bed's,
since a lane declining to
change the incumbent means the archive was already right there.
Recorded as `#142`,
to be fixed after the bed rather than during it.

THE BED CANNOT SHOW THE SHEET AND THE GUARD DISAGREEING.
The sheet asks for the archive's shape and the guard checks the standing text's,
which diverge only where a lane wins with a reshaped slice.
No bed slice does,
so this readout says nothing about that case.
Recorded as `#140`.

## The scorecard, and what it refuses to decide

`consolidate-score.mjs` reads the run's log and buys nothing,
so it can be run
against a partial bed while the run is still in flight.

It COUNTS measurements 1 and 2:
how often the gate ships a consolidation over the standing text,
with the ballot
split at each slice,
and how many consolidations were structurally valid before
any return round.

It REFUSES to count measurements 3 and 4,
which ask whether a text is BETTER.
It prints those slices with the original,
both candidates and what shipped,
side
by side,
and names what to read for at each:
at the tied slices,
whether the
shipped text is one the reading would prefer to both lanes;
at the ten decided
slices,
whether anything the winning lane carried is gone.
The reading is the measurement,
and the sheet is its worksheet.

It also names a case the counters would otherwise bury:
slices where a return
round ran AND the contest ballots said a candidate DROPPED something.
A consolidation restoring what the standing text is missing is,
to a guard
anchored on the standing text,
a surplus to be deleted.
Nothing in the bed has hit that yet,
and the scorecard prints the ballots
whenever it does.

## First slice, and the tripwire it had to clear

`lintong` slice 1 was the slice that exposed the source anchor,
so it is the one
the restarted run had to clear before the other twelve were worth buying.

Under the page anchor all six consolidations were structurally valid with no
return round,
against none under the source anchor,
and every one of them kept
the block quote and its attribution line rather than flattening them.
The gate settled on the consolidation,
three of six ballots naming it and three
calling the pair equivalent.
What shipped is the winning lane's text with one unsupported word removed,
which
is measurement 4's criterion met rather than merely not violated:
nothing the
winner carried is gone.

## One anchor for three stages, and the question it leaves open

`#139`,
`#140` and `#142` are three symptoms of one unsettled question:
WHICH TEXT DECIDES THE SHAPE OF A RENDERED SLICE.

Today three stages answer it three ways.
The translate lane validates against the SOURCE.
The consolidation arm validates against the STANDING text,
which is whichever
lane won.
The consolidation sheet asks for the shape of the ARCHIVE.
Any two of those disagree wherever the archive is shaped differently from the
Chinese,
which is 4 of 19 measured comparison rows,
and the first bed slice is
one of them.

THE FIRST ANSWER WAS "ANCHOR EVERYTHING TO THE ARCHIVE",
AND IT IS WRONG.
It is kept here as a rejected option rather than deleted,
because the reason it
fails is the reason the guard ends up shaped the way it does.
The argument for it was that the archive is the only reference every stage can
share,
the source being another language's shape.
"Measured:
neither text is a correct structural authority" refutes it,
and
"What the guard should check instead" is what replaces it.

WHAT SURVIVES IS THE FRAMING,
NOT THE ANSWER.
The three stages must stop answering the shape question three ways,
and `#140`
still dissolves rather than needing a design of its own,
because the rule that
settles `#139` settles it too.

WHAT NOTHING HERE SETTLES,
and what no measurement in this bed can settle:
an archive whose own shape is wrong.
The rule below ACCEPTS the archive shape wherever a candidate matches it,
which
preserves a bad one as faithfully as a good one,
and the owner's standard is
that what we produce must be good even where the original is not.
The first bed slice is the easy direction,
where the archive's block quote and
attribution line are better than the Chinese paragraph they render.
The hard direction,
an archive that merged what the Chinese separates,
has not
been looked for and is not in the bed.
It is recorded here rather than guessed at.

## Measured: neither text is a correct structural authority

Anchoring every stage to the archive was the obvious answer,
and it is wrong.
Measured over every settled artifact on this machine,
68 distinct
entry-and-slice records (the same passage appears more than once where two runs
sliced it differently),
using the package's own `readSliceSkeleton`:

-   48 records:
    the archive and the Chinese carry the SAME block sequence,
    so
    the shape is not in doubt at all.
-   11 records:
    the archive carries MORE blocks,
    almost always an English
    rendering splitting one dense Chinese paragraph.
-   7 records:
    the archive carries FEWER blocks.
-   1 record:
    same count,
    different kinds.
-   1 record:
    unparseable under the strict grammar.

THE SEVEN ARE TWO DIFFERENT PHENOMENA,
and that is the finding.
At `lintong` the archive merges several Chinese paragraphs into a block quote
and an attribution line,
which is a better shape than the Chinese has.
At `Aniloviraw` slice 0 the archive is missing a whole block quote the Chinese
carries,
and at slice 2 it is missing two trailing elements.
The first is a reshape to preserve.
The second is content the archive DROPPED,
which is the whole reason this
pipeline exists.

SO AN ARCHIVE ANCHOR WOULD FORBID THE RESTORATION.
A rendering that puts back what `Aniloviraw`'s archive lost has one block too
many,
gets called invalid,
goes back to its author,
and comes back with the
restoration deleted.
That is the mirror of the defect the source anchor caused at `lintong` slice 1,
and it is worse,
because a lost block is lost silently while a flattened quote
is at least visible.

## What the guard should check instead

SUPERSEDED by "The shape rule,
settled:
the page is a floor,
not a ceiling".
What follows is the match-either rule as it was proposed and its two recorded
limits,
kept because the measurement that produced it still stands and because
the reason it was dropped is only legible beside it.
The rule that shipped treats the page as a floor rather than as one of two
acceptable sequences.

WHERE THE TWO REFERENCES AGREE,
which is 48 of 67 parseable records,
the guard
stays exactly as strict as it is now.
Both texts say the same thing about the shape and a candidate departing from it
has no defence.

WHERE THEY DISAGREE,
a candidate is valid if it matches EITHER the archive's
block sequence or the Chinese's.
The shape is a judgement call there,
the pipeline has no authority that settles
it,
and a guard that picks one anyway manufactures findings on 19 of 67 measured
records and sends models back to undo defensible work.
THAT SHARE IS NOT A CORPUS RATE:
the pool is the settled artifacts on this
machine,
five of the seven fewer-block records are one entry sliced by different
runs,
and one entry dominates the more-block records.
Matching either still catches the failure the guard was built for,
a model
returning one undifferentiated blob,
except where a reference is itself one
block.

THE PROTECTED ATOMS ARE UNAFFECTED and stay strict against both.
A footnote marker or a link that appears in neither reference is invented,
and
one that appears in both and not in the candidate is lost,
whatever the block
shape does.

TWO LIMITS,
RECORDED RATHER THAN BUILT AROUND.
A candidate matches one reference SEQUENCE ENTIRE,
so an archive that both
reshapes what it keeps AND drops a trailing block leaves the ideal rendering,
reshaped prefix plus restored tail,
matching neither.
No measured record does both,
and the `Aniloviraw` prefixes are kind-identical
so a restoration there matches the Chinese,
but it is unmeasured rather than
impossible.
And a one-sided slice has no archive text at all,
so the second reference has to
be OPTIONAL,
degrading to today's single-reference check rather than refusing
every candidate.

THE CONSOLIDATION BED STAYS VALID UNDER THIS RULE.
Its first slice is a disagreement record,
and what the six producers returned
matched the archive's sequence,
which the proposed rule accepts.
Nothing measured so far would have to be bought again.

## The bed stopped at five slices, and why

`lintong` slice 2 shipped a consolidation that signs a friend's note with the
DEAD PERSON'S name,
alias and city.
The Chinese signs it as left by a friend;
the standing text had that right;
the
consolidation put the archive's declared identity into the attribution line.

THE CAUSE IS A SHEET RULE,
and it is production's,
not the bed's.
`src/translate-selection-sheet.ts` gives every slate judge a criterion saying a
candidate dropping a declared name "has left something out",
and the
consolidation sheet says carrying one is correct "even where the passage never
spells it out".
Read literally,
both instruct a model to put the archive's identity block into
any passage,
and one judge had already abstained at the previous slice for the
stated reason that no candidate carried the declared location.
Recorded as `#143`,
with the measurement that put the rule there,
which the fix
has to keep satisfying.

THE RUN WAS STOPPED RATHER THAN FINISHED.
Five of thirteen slices were bought,
every one of them under the defective
wording,
so scoring them would measure the defect.
The owner's standing instruction is to fix and restart rather than to score
around a defect,
and this is the second time this bed has earned it.

## The shape rule, settled: the page is a floor, not a ceiling

Two candidate rules survived the measurement,
and the bed's own rejections
separate them.

MATCH EITHER REFERENCE is too weak.
At `lintong` slice 2 the Chinese is five paragraphs and the page is a block
quote plus an attribution line,
so a candidate matching the Chinese exactly
would be accepted with the quotation marks gone,
and the passage is a note
somebody left,
which is what the block quote says about it.

THE PAGE IS A FLOOR.
A candidate is structurally valid when the ARCHIVE'S BLOCK SEQUENCE APPEARS IN
IT IN ORDER,
and it carries no more blocks than the longer of the archive and
the Chinese.
Where the archive has no text for the slice,
the check falls back to today's
exact match against the Chinese.

WHAT THAT DECIDES,
on every case measured so far:

-   `lintong` slice 1,
    where the source anchor called six good renderings
    invalid:
    the archive's two blocks appear in each,
    so all six pass.
-   `Aniloviraw` slice 0,
    where the archive is missing a whole block quote:
    a
    rendering that restores it still contains the archive's one block in order
    and has no more blocks than the Chinese,
    so the restoration is no longer
    deleted.
-   The three rejections the bed actually produced,
    one rendering of six
    paragraphs and two that swallowed the attribution line into the quote:
    all
    three still fail,
    because the archive's sequence is not inside them.

## What landed before the bed was bought again

Five changes,
each with its guard shown failing when removed,
and the bed
restarted from zero afterwards because every earlier row was produced under the
wording being fixed.

THE PAGE IS A FLOOR (`3ca134e57`).
`validateTranslatedSlice` takes the original AND the text a candidate replaces,
requires the page's block sequence to appear in the candidate in order,
and
allows no more blocks than the longer of the two references.
Protected atoms take the larger demand of the two,
so a footnote the archive
added is kept and one the archive dropped is restored.

THE PAGE IS NOT ALWAYS THE INCUMBENT (`3accdf27d`).
`repairInvalidCandidates` takes `pageText` separately,
because a consolidator's
incumbent is the lane that won the contest while the page it replaces is the
archive.
It defaults to the incumbent,
so the translate lane is unchanged.

A DECLARED NAME IS A SPELLING AUTHORITY (`e0e822733`).
Both sheets said a candidate dropping one had left something out,
which is what
signed a friend's note with the dead person's name.

A CANDIDATE THAT IS THE ARCHIVE SAYS SO (`60f8d97ae`).

THE GATE STOPPED CHECKING ONE QUORUM TWICE (`9261c8efd`),
which the GFP pass
found:
removing the repeated conjunct failed no test,
because
`settleGateBallots` cannot answer `consolidated` on fewer voices than the
quorum.
A condition no case can reach reads as a second guard and is not one.

## What the floor rule dissolves, and what #144 cost

THE LIMIT THAT DISAPPEARED.
Match-either recorded a case it could not serve:
an
archive that both reshapes what it keeps AND drops a trailing block leaves the
ideal rendering,
reshaped prefix plus restored tail,
matching neither reference
entire.
The floor rule serves it outright.
A source of three paragraphs against an archive that merged two of them into one
block quote and dropped the third gives a floor of one block quote and a ceiling
of three,
so the ideal rendering,
block quote plus restored paragraph,
sits
inside both bounds and is valid.
The rule that shipped is therefore not merely simpler than the one it replaced;
it accepts a rendering the earlier proposal would have sent back.

WHAT #144 FOUND,
one slice into the second bed run.
Three sheets still told their models the ORIGINAL was the shape standard while
the guard floored candidates on the page.
The judge criteria were the dangerous one:
judges apply these criteria literally,
which run 1 proved when one abstained from a whole slate over the declared-names
wording,
so at every reshaped slice an archive-shaped consolidation could be
marked down by the very stage that is supposed to choose it.
The translator sheet and the repair follow-up were the expensive ones:
each buys
a return round at every reshaped slice by construction,
one by asking for the
source's shape and the other by announcing the check compared against the source
alone.

WHY THE JUDGE CRITERION NAMES NEITHER TEXT.
The existing translation reaches these judges anonymously,
as one candidate among
the others,
and never travels as labelled evidence.
A criterion naming the page would name a text the judge cannot see,
so the
criterion now asks only what is checkable from a candidate alone,
and says
outright that a shape the ORIGINAL does not have is not a fault.

THE BED WAS BOUGHT AGAIN,
for the second time in a day.
One row had landed under the old criteria and it was discarded rather than
scored,
on the standing instruction to fix and restart rather than to score
around a defect.

### Measured, not inferred: judges were already deciding slates on the source shape

The two archived bed runs carry 26 judge ballots between them.
SIX OF THE 26 NAME THE ORIGINAL'S SHAPE as a reason for their choice,
one of
them citing "Criterion 4" by number and rejecting two candidates in the same
sentence for what it called violating structure while praising the winner for
conveying the passage as one coherent paragraph rather than a disjointed list.
That is the criterion deciding a slate,
on a slice where the archive and the
Chinese disagree about how many blocks the passage has.

So the fix is not a precaution against a failure mode that might appear.
It removes a criterion that six ballots were visibly applying,
against the same
shape the guard was simultaneously demanding candidates depart from.
Neither archived run is scored,
because both were produced under wording that
has since changed,
but they are evidence about the instrument rather than about
the pipeline,
and that evidence survives the run being discarded.

### The ballot that failed a whole slate on both criteria at once

One of the six is not a passing remark.
At `lintong` slice 1,
the run I stopped,
a judge abstained from the entire slate
and gave two reasons:
that no candidate spelled the declared handle the way the
identity block does,
and that "the original is a single unbroken paragraph with
no block-quote markers,
yet every candidate wraps the text in block quotes and
most fragment it into separate quoted lines,
altering the markdown structure
(criterion 4)".

Every candidate wrapped the passage in block quotes because THE ARCHIVE PAGE
DOES,
and the guard requires it:
the page's blocks are the floor.
So the slate was refused for obeying the guard,
by a criterion that had not been
told the guard had changed,
and the refusal cost that slice its whole judged
decision.
Criterion 4 is closed by `#144`.
The criterion 3 half was NOT closed by `#143`,
which settled whether a declared
name is content a passage owes and said nothing about how to spell one;
the
spelling half is `#145`,
and the check that decided it is recorded in "Two
spelling authorities,
no precedence".

A tripwire in the scorecard prints any ballot whose stated reason reaches for the
source's shape or for a declared name,
and it was shown firing on this archived
row before being trusted on a live one.

### Two spelling authorities, no precedence

READ RATHER THAN ASSUMED,
through the package's own `collectIdentityLines`
against the corpus page and against the archived rows.
At `lintong`,
the identity block declares one romanised form of the handle and a
second as an alias.
The English page uses THREE forms across its length,
one of which the block
declares nowhere,
and that undeclared form is the one standing in the slice both
lanes worked from.
Every consolidation copied it.
So did both lanes.
So did the shipped text.

THE TWO SHEETS DISAGREED,
and neither said which wins.
The translator sheet said names already used by the existing translation are
authoritative and are kept exactly.
The judge sheet said declared names are used exactly as given.
Where an archive contradicts its own front matter,
producers were instructed to
write the form the judge was instructed to penalise,
and every candidate on the
slate inherits it at once,
so the fault cannot order them:
the judge's only
literal move is to refuse the whole slate,
which it did,
twice at the same slice.
The incumbent that survives that refusal is where the spelling came from.

SETTLED.
The declared spelling outranks the archive's usage,
in both producing
sheets,
because the front matter is what the archive DECLARES and its prose is
demonstrably inconsistent with itself.
Where the block offers more than one form,
any declared form is right;
where the
block is silent,
the archive's usage stands as before;
and inventing a third
spelling is forbidden as it always was.
The judge is separately told that a spelling every candidate shares cannot
separate them,
so it decides on the other criteria rather than declining.

WHY THIS WAS WORTH A THIRD RESTART.
One row had landed.
The alternative was
thirteen slices measured through an instrument that had already been observed
failing a slate for obeying its own guard.

### What the four refused slates were actually refusing

The archived bed rows carry 114 ballots.
FOUR OF THEM REFUSED AN ENTIRE SLATE,
and reading all four is what turned two
hand-found defects into a class:

-   `lintong` slice 1,
    twice,
    on the declared spelling and on the source's block
    shape,
    which are `#145` and `#144`.
-   `gaoyanger` slice 1,
    on tense:
    every candidate was said to have altered the
    time reference by rendering a tenseless Chinese copula in the past.
-   `keyword233` slice 0,
    on tense together with a declared location the passage
    does not mention,
    which is the `#143` direction.

TENSE IS IN THREE OF THE FOUR,
and the rule that settles it already existed.
`critic-prompt.ts` tells its critics that supplying what English requires and the
original can omit,
a pronoun,
a number,
an article,
A TENSE,
is required rather
than added.
No judge had ever been shown that sentence.
Judges were split in both directions on the same question:
one refused a slate
for rendering the present in the past,
another marked a candidate down for
keeping the present as awkward for a memorial page.

THE GAP WAS WIDER THAN TENSE.
Every producing sheet in the package splices `HOUSE_POLICY_BLOCK`;
no judging
sheet did.
The stages that decide what SHIPS were the only stages never told that reader
protection outranks completeness,
while criterion one asks them for every
proposition of the original with nothing left out.
A rendering that keeps a suicide method vague,
which is the corpus's own rule,
reads as an omission to a judge holding only the numbered list.

SETTLED as `JUDGE_POLICY_BLOCK`,
spliced once into `buildCandidateSelectMessages`
so all five judge panels get the same text and none can drift from the producers.
It states precedence outright,
because three of the four refusals came from
judges applying a numbered criterion literally against everything beside it.

FOUR DEFECTS,
ONE SHAPE.
`#143`,
`#144`,
`#145` and `#146` are all the same
failure:
a rule given to the models that WRITE and withheld from,
or contradicted
for,
the models that CHOOSE.
The bed has been started six times,
and every
restart was paid for by a defect that would otherwise have decided the corpus.

### The sweep that should have come first

Two of these were found by hand,
in ballots.
The rest were found in one pass that asked a different question:
which system
prompts in this package carry the corpus's rules,
and which do not.

-   Producing sheets carrying them:
    the translator,
    the consolidator,
    the
    critic,
    the coverage reader,
    the refiner,
    the adjudicator.
-   Deciding sheets carrying them:
    none.
-   The editor,
    which REWRITES text rather than judging it:
    none,
    though
    `house-policy.ts` names it as a consumer in its own comment.

THE EDITOR IS THE SERIOUS ONE.
The block's header states the failure it exists to stop:
a page keeps a suicide
method vague because the corpus's rule says to,
a critic ignorant of the rule
files it as an omission,
and the editor restores the detail.
The critic was given the rule and the editor was not,
so exactly half of that
sentence was guarded.
Its own sheet told it five separate times that every detail of the original must
survive and that an omission must be filled in full sentences with nothing
dropped.

WHAT THE SWEEP COST,
and what finding these one at a time cost.
The sweep is one `rg` over `role: 'system'` and one grep for the block name.
Finding `#144` and `#145` by reading ballots cost two bed restarts.
A sweep of the same kind belongs at the front of any measurement that depends on
what models were told,
and this one is now recorded so the next measurement can
start with it.

## Two checks run before run 6 could pay for them

Both were run while the bed was in flight,
so a hit would have cost one slice rather than ten.

### The overturned wording exists nowhere but the tests that pin it

The sweep that found #146 to #148 checked whether each sheet carried the policy block.
It could not have found #147,
which was a stale copy of wording #143 had already overturned,
sitting in a sheet that did carry its policy.
So the second sweep looked for the overturned phrasings themselves across every file under `src/`:
`HAS dropped`,
`has left something out`,
`of the ORIGINAL preserved`,
`are authoritative`,
`compared against the ORIGINAL by a mechanical`,
`omitting one`.

Every hit is either a test asserting the phrase is absent,
a comment recording why it was removed,
or the one live site that now carries the precedence clause after it
(`src/translate-wire.ts`,
where `are authoritative` is immediately followed by
`WHERE DECLARED NAMES IS SHOWN AND SPELLS THE SAME PERSON OR PLACE DIFFERENTLY, THE DECLARED SPELLING WINS`).
No sheet the bed exercises carries an overturned claim.

The sweep was widened past `role: 'system'` to the two files that build model-facing text without one.
`src/consolidate-brief.ts` renders judge findings under `Says what the original does not`
and `Omits what the original says`;
those are labels on ballot output,
and the ballots are now produced under the corrected policy,
so the labels describe what the judges were asked for.
`src/line-structure-addendum.ts` is the one open question and is recorded under its own heading.

### The findings recording is proven at runtime, not by reading the edit

`validityBefore` and `validityAfter` were widened from a bare validation kind to
`{validation, findings, unknownDetail}` several runs ago,
and nothing had ever shown the wider shape populated.
Run 3's archive (`consolidate-calibrate.jsonl.before-spelling-precedence-fix`) was written under the widened harness
and carries an invalid row,
so it is a positive control that costs nothing to read.

Its `lintong` slice 1 row records four models' verdicts,
and the invalid ones carry the check's own words:

- `The PAGE AS IT STANDS is 2 blocks (blockquote, paragraph) and your translation is 1 (blockquote).`

The harness records findings.
A `kind: 'invalid'` in run 6 will say what failed,
not merely that something did.

## The verse addendum still speaks for the original, and no bed slice can show it

`LINE_STRUCTURE_RULE` in `src/line-structure-addendum.ts` opens
`This region's ORIGINAL IS line-structured: each original line is a unit`,
then asks the editor to keep every existing line in place.
The second clause is the floor rule;
the first names the ORIGINAL as the authority on how many lines there are.
On the case the rule exists for,
`Toka_ls`,
the two disagree:
the Chinese chunk is 21 blocks at median 22 and the English rendering is 18 blocks at median 101.
An editor reading the first clause as licence may re-split the merged rendering,
which is the shape claim #144 removed from three other sheets.

It cannot affect run 6.
`isLineStructured` was run over all thirteen bed slices and returned false for every one,
so the addendum is empty on every call the bed makes.
It is a production follow-up,
filed with the other unswept sheets rather than restarting the bed.

## The measuring sheets now carry the rules too, and one of them needed a fourth verdict it did not get

The four sheets of #149 grade rather than write,
so nothing ships from any of them.
What they decide is which defects get worked on next,
and an instrument that penalises a page for obeying the rules it was written under
reports damage where the pipeline was right.

`JUDGE_POLICY_BLOCK` could not simply be spliced into them.
Its closing paragraph reads
`WHERE A CRITERION AND A HOUSE RULE DISAGREE`
and speaks of a candidate,
and a checker has neither:
it has issues,
regions,
verdicts and a text under review.
So the two shared paragraphs were lifted out
(`HOUSE_POLICY_BLOCK` and `FORCED_DIFFERENCES`)
and each consumer supplies its own close.
`JUDGE_POLICY_BLOCK` recomposes from the pieces and is byte-identical to what the running bed's judges see,
3037 bytes,
checked against a snapshot taken before the edit.

### What the resolution checker could not be told, and why it was not given a fourth verdict

`RESOLUTION_VERDICTS` is `fixed`,
`not-fixed`,
`worse`.
An issue asking an editor to restore a detail reader protection keeps out is an issue that should never have been filed,
and none of the three verdicts says that.
`checker-sensitivity.ts` already names the case,
its fabricated `ABSENT_ISSUE` is labelled `not-fixed-defect-was-never-there`,
and that harness exists to catch a checker answering `fixed` by agreeing with the sheet instead of reading the text.

Whether a fourth verdict was owed turned on one measurable fact:
what an unresolved issue costs.
It re-fires nothing.
`runCheckerStage` is called once in `repair-chunk.ts`,
after the editor's rounds,
`resolvedIssueIds` is credit-only,
and `notFixed` is read nowhere but `tally-resolution.ts` and the sensitivity probe.
The one real consequence is in `refine-phase.ts`,
where a previously confirmed issue going unresolved rolls the refinement back,
which is the conservative direction.
So the checker is told to answer `not-fixed` and told outright that the issue itself is wrong,
and the known limit is recorded here rather than papered over:
the tally cannot distinguish an unfixed defect from an issue that was never a defect,
so a protected-detail issue reaching the checker understates repair quality by one.

The splice bought something the sheet could not do before.
Where an editor DID restore a protected detail,
`worse` is the correct existing verdict,
and until now nothing gave the checker any reason to cast it.

### The prober had an active contradiction, not merely a gap

`PROBE_RULES_HEAD` says content the AFTER text drops is damage only if the ORIGINAL supports it.
On a protected detail the ORIGINAL does support it,
which is the entire reason the rule exists.
A block spliced beside that bullet would have left two live rules disagreeing,
with the older one written as a numbered rule.
`PROBE_HOUSE_RULE_CLAUSE` names the interaction in as many words.

### The restoration grader keeps its anchor

The 2026-07-17 directive grades restoration against the Chinese source,
and the block does not move that anchor.
An anchor on the source is what makes the bias possible:
a repair rendered vaguer than the Chinese because reader protection asks for it reads as a partial restoration.
The added line says such a repair is `restored`,
and that carrying a detail the rules keep out raises no verdict.

### A guard that reported success without running

The first version of `measuring-sheets-carry-house-rules.unit.test.ts` called `describe` without `await`.
The file registered nothing,
printed nothing,
and exited zero;
the whole suite then reported `exit=0` with no failing line while six cases had never run.
This was caught only by grepping the log for the suite's own name rather than trusting the exit code.
Every other test file in the repo already awaits its `describe`,
so this is a trap rather than a defect:
a suite is proven to exist by finding its name in the output,
never by the run's exit status.

## The verse rule and the page rule contradict each other, and the guard settles nothing

This section replaces a first reading that was wrong,
kept here because the wrong reading is the instructive part.

### What was claimed, and what refuted it

The claim was that `TRANSLATE_LINE_STRUCTURE_RULE` tells a producer to emit one line per ORIGINAL line
while the structural guard floors on the page as it stands,
so every verse candidate would be rejected on any page that had merged lines.

Measured,
and refuted.
`validateTranslatedSlice` has two sides,
and neither of them forbids unmerging on verse.
Every block of the page must appear in the candidate,
in kind and in order,
so a candidate that MERGES the page's blocks fails.
Extra blocks are allowed,
but only where the ORIGINAL has them:
a three-block candidate against a one-block page returns `valid` when the source is verse and `invalid`
when the source is prose,
with `Add a block only to carry something the ORIGINAL has and the text you are replacing left out.`

So on a line-structured chunk the guard actively PERMITS the unmerge,
because the original licenses every block it adds.
The guard and the verse rule agree;
it was the two sheet rules that did not.

The probe was positive-controlled before its null was trusted,
per QPC.
The control reproduces the archived finding verbatim:

- `The PAGE AS IT STANDS is 2 blocks (blockquote, paragraph) and your translation is 1 (blockquote).`

So the null on the verse cases is a real null,
and the guard has nothing to say about unmerging.

### What is actually wrong

On a line-structured chunk the producer's system prompt carries two rules that point opposite ways,
and until now neither deferred to the other.

- `TRANSLATE_RULES`,
  from #144:
  where the existing translation shapes the passage differently,
  keep the existing translation's shape.
- `TRANSLATE_LINE_STRUCTURE_RULE`,
  from #79:
  produce one output line per original line,
  and where the existing translation has merged lines,
  unmerge them.

On `Toka_ls`,
the entry the verse rule was written for,
the Chinese chunk is 21 blocks at median 22 against the English rendering's 18 blocks at median 101.
One rule says keep 18 and the other says restore 21.
Both arrive in one prompt,
and a producer meeting a contradiction resolves it however it likes.

The verse rule wins,
and now says so.
It rests on a measured failure and its own recorded decision,
while #144's bullet was written about prose reshaping.
Because the guard is a floor,
nothing downstream would ever have caught the wrong choice,
which is why the precedence had to be stated in the sheet rather than left to the check.

`LINE_STRUCTURE_RULE` in `line-structure-addendum.ts` is NOT the same defect.
It is given to an editor working on the standing text,
where keeping every existing line is the anti-reflow rule #79 built,
and its own documentation states the distinction.

### It could not have reached run 6 either way

The bed computes `lineStructured` with `isLineStructured` over each slice's source text,
and that predicate returned false for all thirteen bed slices,
so neither producing sheet emits the rule on any slice this run buys.

## The three instruments still discriminate under the longer prompt

A static guard proves the rules ARRIVE.
It does not prove a checker still tells a fix from a non-fix with several kilobytes more in front of it,
so all three sensitivity probes were run live against their pre-built fixtures.

Resolution checker,
every arm where the harness expects it:

- a genuine fix reads `fixed` on all three voices,
  an untouched text reads `not-fixed` on all three,
  and a fix that damaged its slice splits two `fixed` against one `worse`.
- on the mixed sheet the tense issue resolves,
  the meaning issue does not,
  and the FABRICATED issue draws two `not-fixed` against one `fixed`,
  so the majority still refuses to agree with a sheet describing something absent from the text.

Introduced-defect prober:

- `deletion/mislabelled`,
  where real damage sits under a false accepted issue,
  draws three removal-corroborated claims.
- `deletion/licensed`,
  the negative control where silence is correct,
  draws `none-found` on all three.

Rendering auditor:

- the flipped rendering draws an agreed `altered-polarity` finding on two voices.
- the clean rendering agrees at neither tier,
  which is the expectation;
  one voice filed a lone claim and no agreement formed on it.
- two degraded lines on the flipped arm are infrastructure rather than sheet:
  one voice lost and a 2-of-3 roster.
