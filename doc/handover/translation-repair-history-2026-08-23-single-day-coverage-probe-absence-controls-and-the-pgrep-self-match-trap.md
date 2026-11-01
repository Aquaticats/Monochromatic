# Translation repair history 2026-08-23: whether the coverage wire can vote absence, whether the votes are right, and the background-wait trap met on the way

The coverage probe's persistence gap that was already closed,
the background-wait trap met while running it,
the grading of carried verdicts,
the control that deletes the roster's own evidence and the decoy cut that answers it,
the breadth run over six entries,
and the page-length signal that corroborates the absence verdicts without a model.

Part of [Translation repair history: 2026-08-23, the work recorded for that day alone](translation-repair-history-2026-08-23-single-day.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Task 106, one of its two decision-independent items is already done

Checked 2026-08-23,
while looking for work that does not wait on question 28.

`#106` records:
"The probe writes its result with `console.log(JSON.stringify(...))` and nothing
persists it",
and concludes "Persisting probe output into the runs directory is worth doing whichever
option wins."

THAT IS NO LONGER TRUE.
`coverage-probe.ts:438` calls `persistProbeRun` into the resolved runs
directory,
keeping the rows,
the roster,
the corpus pin,
the pipeline digest and the runner closure,
and deliberately keeps standard output as well.
`persistProbeRun` landed in `probe-store.ts` on
2026-08-17,
one day after the note was written,
alongside a commit whose own subject records that
"the redirect this probe was built around never worked".

So the durability gap is closed and a later session acting on that line would rebuild what exists.
This is the same failure the width probe kept producing in reverse:
there,
comments claimed invariants the code did not have;
here,
a task claims a gap the code has since filled.
Both come from reading a recorded sentence as a fact rather than as a claim to check.

WHAT IS STILL TRUE:
the 2026-08-16 numbers themselves are gone.
No `coverage-probe` directory exists
under any runs directory on this machine,
so the probe has not been run since persistence landed,
and
those verdicts survive only in session transcripts as the task says.
Re-running is the only way to get
them back,
and a re-run now keeps them.

## Waiting on a background run: `pgrep -f` matches the agent's own shell

Cost two false "it finished" notifications on 2026-08-23,
once on width draw B and once on the
coverage probe,
and it will keep costing them until someone writes it down.

Waiting for a spawned run by `PID=$(pgrep -f 'some-probe'); while kill -0 "$PID"; do sleep; done`
fails two ways at once.

FIRST,
`pgrep -f` matches on the WHOLE command line,
and the agent's own `bash -c` wrapper contains
the pattern,
because the pattern is part of the command being run.
So the pgrep frequently returns
the wrapper's own pid rather than the probe's,
and the wait ends the moment that shell exits,
which
is immediately.

SECOND,
a mise task with `depends = ["build"]` spends its first half-minute building.
A pid captured
too early belongs to a transient build-phase process that exits on its own,
so the waiter reports
completion while the probe has not started.

WHAT WORKS:
wait for the real interpreter process to appear,
match it precisely,
and confirm it
before arming.

```bash
# The node process, never the wrapper: filter the agent's own shell out.
pgrep -af 'coverage-probe.mjs' | grep -v 'bash -c'
```

Then check the pid with `ps -o args= -p "$PID"` and confirm the command line is the one expected
before waiting on it.
`kill -0 "$PID"` alone proves a process exists,
never that it is the right one.

## `#106` option A: the carried verdicts are self-consistent, which is not yet an answer

Ran the coverage probe over one entry at the current wire,
`mikaela_khara`,
cap 16,
into `~/temp/agent/106-coverage`.
Sixteen block-scale verdicts persisted:
`carried` 15,
`partly-carried` 1,
zero absence votes and zero unanchored,
reproducing the recorded block-scale null on a fresh run.

Graded those verdicts with three deterministic signals,
no model in the loop.
The question option A exists to answer is whether an anchored quote RENDERS the source passage
or merely exists somewhere in the translation.

-   REUSE,
    one span offered as evidence for two different blocks: 0.
-   INVERSION,
    a later block anchored earlier in the page than an earlier one:
    0 of 15 adjacent pairs.
-   THINNESS,
    evidence short enough that it cannot be rendering its source block:
    0 of 16.
    The shortest is 2.08 English characters per source character;
    the spread runs 2.08 to 6.78.

So the degenerate failure is not what is happening here.
The evidence spans are block-sized,
distinct per block,
and walk the page in block order.

### The first version of that grader reported a finding that was not one

It keyed reuse by span and pushed the block label once per evidence quote,
but each row carries one quote per voter,
so six voters agreeing on one span registered as six offers and tripped the `length > 1` test.
Every group it printed was one block repeated.
Corrected by counting DISTINCT blocks per span,
which reads 0.

Same shape as the fabricated `neither shipped: 2` earlier in this work:
an analysis script that reports a number nobody checked it could produce correctly.
Every detector now runs a positive control on a planted case before it reads the run,
and the reuse detector additionally runs a negative control proving
within-block agreement does not trip it.

### What this does NOT establish

The controls prove the DETECTORS can fire.
They do not prove the COVERAGE PROBE can vote absence at all,
which is the actual question behind the unanimity.
A wire that never returns `absent` produces exactly this reading on a complete translation
and on a defective instrument alike,
and one entry could simply be well translated.

The control that separates them:
take a case the roster voted `carried` on,
delete THE ROSTER'S OWN EVIDENCE SPAN from the translation,
and ask again about the same source passage.
The rendering the roster pointed at is then provably gone.
Voting `carried` again,
anchored on some other span,
is the contamination hypothesis demonstrated;
flipping to absent or partial is the wire shown able to see absence,
which makes the null benign and answers `#106`.

Damage cannot be chosen without asking first:
coverage candidates are by construction the passages the aligners REFUSE to pair,
so nothing outside the roster's own answer says which target text renders them.

## `#106` ANSWERED for the block scale: the coverage wire can vote absence

Ran `coverage-control-probe` over `mikaela_khara` at the current wire,
roster of six,
118 seconds.
Three cases reached `carried` with deletable evidence and were damaged:

-   `pair 0 block 6`:
    `carried` to `absent`,
    absence votes 0 to 6,
    cut 6 spans of 112 chars.
-   `pair 0 block 7`:
    `carried` to `partly-carried`,
    absence votes 0 to 0,
    cut 6 spans of 75 chars.
-   `pair 0 block 9`:
    `carried` to `absent`,
    absence votes 0 to 5,
    cut 6 spans of 74 chars.

Two of three flipped to a unanimous or near-unanimous absence verdict
once the spans the roster itself had anchored on were deleted.

THE INSTRUMENT IS NOT BLIND.
The recorded block-scale null,
ninety-six answers with not one vote for absence,
therefore describes the translations rather than the wire.
The wire returns `absent`,
and returns it with every voice,
when the rendering is actually gone.

Read beside the deterministic grading of the same entry
(no cross-block evidence reuse,
no order inversions,
no evidence too thin to render its source),
the `carried` unanimity is benign on the measured entry:
the spans are block-sized,
distinct per block,
in document order,
and their removal is detected.

### What one case shows and what it does not

The middle case is worth keeping in view.
Deleting the anchored spans moved it to `partly-carried` with no absence votes at all,
which is what a passage partly rendered somewhere else in the page should produce.
That is the wire discriminating,
not failing:
it declined to call absent something it could still partly find.

Breadth is a separate claim.
This is one entry and three cases,
which is enough for REACHABILITY,
the question the control was built to settle,
and not enough to say the corpus carries everything.
A reachability control needs one unambiguous flip,
not a sample.

### The objection still open when this was recorded

A wire that answered `absent` after ANY deletion would produce the same flip
and would be as useless as one that never votes absence.
The middle case argues against that (a 75 character cut produced no absence votes),
but it is not a size-matched cut taken from somewhere the roster did not point at.
`coverage-control-decoy.ts` exists to make exactly that cut:
same document,
same passage,
same run,
same number of characters,
taken as late in the page as it fits so the frontmatter and title are not what gets deleted.
A sound wire keeps saying `carried` under it.

### A background waiter reported completion while the probe was still running

Third false finish in this work.
The `until ! pgrep` waiter exited immediately with an empty log
while `ps -o args=` showed the probe alive under the pid captured at launch.
Do not trust a waiter's completion notification;
confirm against `pgrep -af '<runner>.mjs' | grep -v 'bash -c'` and the pid before reading a result as final.

## `#106` option A is DONE: the wire reads the passage, so the null is about the corpus

The paired control settles the objection the first run left open.
Same entry,
same roster,
run again with both cuts made per case:

-   `pair 0 block 6`:
    targeted cut `carried` to `absent`,
    0 to 6 absence votes;
    DECOY of the same 112 characters at offset 10545:
    `carried`,
    0 absence votes.
-   `pair 0 block 7`:
    targeted `carried` to `partly-carried`,
    0 to 0;
    decoy of the same 75 characters at 10582:
    `carried`, 0.
-   `pair 0 block 9`:
    targeted `carried` to `absent`,
    0 to 5;
    decoy of the same 74 characters at 10583:
    `carried`, 0.

Absence votes appeared on two targeted cuts and on NONE of the three decoys.
The wire is not answering the damage;
it is answering the question.

### The decoy cut ordinary prose, checked rather than assumed

All three decoys land in the same tail region,
which invites the reading that they deleted a footer
the roster never cared about.
Classified the region structurally,
without printing any of it:
offset 10545 of a 10657 character page covers ONE PROSE LINE of 111 characters plus a trailing blank.
Not a heading,
not a list,
not links,
not html.
So the decoy deleted a real body sentence of comparable size and the roster still said `carried`.

### It replicated, and the path has no cache

The targeted arm returned the same three verdicts on both runs.
`coverage-stage.ts`,
`coverage-verdict.ts` and `coverage-control.ts` contain no cache of any kind,
so the second run bought its own answers.
Two independent runs agreeing on all three,
including the `partly-carried` middle case,
is the discrimination holding rather than a coin landing the same way.

### What `#106` option A now says

Four readings,
all pointing the same way on the measured entry:

-   no evidence span serves two blocks,
-   no block's evidence sits out of document order,
-   no evidence is too short to be rendering its source,
    the thinnest being 2.08 English characters
    per source character,
-   deleting a rendering is detected and deleting an equal amount of unrelated prose is not.

The block-scale null,
ninety-six answers with not one vote for absence,
is therefore a statement about the translations rather than about a wire that cannot say no.
Question 28's premise stands on measurement now instead of on an ambiguity.

STILL GATED ON THE OWNER,
unchanged:
do not wire `groupSourceFirst` into `subdivideChunkPair`,
and do not emit insertion pairs from `alignDocumentSections`.
That is a decision about what the pipeline should DO with a reachable absence verdict,
and this only establishes that the verdict is reachable.

Breadth remains the honest limit:
one entry,
three damaged cases,
sixteen graded verdicts.
Enough to settle reachability,
which needs one unambiguous flip and its negative control,
and not enough to claim the corpus carries everything.

## `#106` BREADTH: the block-scale null was a fact about ONE ENTRY, not about the wire

Ran the absence control over six entries carrying coverage candidates,
chosen off a zero-quota census of all 92 pairs which found 26 such entries.
Two clean groups came back,
and the second is the one that matters.

### Group one: damage is detected, and only the right damage

-   `mikaela_khara`:
    HELD.
    2 of 3 targeted cuts produced absence votes (6 of 6,
    5 of 6);
    the middle case moved to `partly-carried`.
    Decoys:
    0 of 3.
-   `TianqiChen666`:
    HELD.
    3 of 3 targeted (5 of 6,
    6 of 6,
    6 of 6).
    Decoys:
    0 of 3.
-   `Futajuhuacha`:
    HELD.
    3 of 3 targeted (2,
    6,
    5 votes).
    Decoys:
    0 of 3.

Across the three:
absence votes on 8 of 9 targeted cuts and on 0 of 9 equally sized cuts
taken where the roster had not pointed.

### Group two: the roster refuses to call the passage covered WITH NOTHING DAMAGED

-   `shi_Yumiaoya`:
    8 of 8 candidates.
    Seven read `absent` with 4 to 6 absence votes,
    one `split`.
-   `Aniloviraw`:
    3 of 3 read `absent`,
    5 to 6 absence votes.
-   `XingZ60`:
    4 of 4 not covered;
    section 13 `absent` with 5 votes,
    sections 12 and 14 and one block `partly-carried`.

Fifteen of fifteen undamaged candidates on those entries were NOT called covered.
Not one refusal in the whole run was an anchoring failure:
the count of `evidence-not-locatable` across all six entries is zero.

`XingZ60` also REPRODUCES its recorded section-scale verdicts:
section 12 reads `partly-carried`,
which is exactly what `#106` records for it
(0 full,
5 partial),
and section 13 reads `absent` as recorded.
Section 14 drifted from `absent` to `partly-carried` with one absence vote.

### What this changes

`#106` records,
of `mikaela_khara`,
"ninety-six answers from six models,
NOT ONE VOTE FOR ABSENCE was cast on sixteen passages the block aligner refuses to pair",
and `#100` refuses paragraph-scale landing on that evidence:
"at most one of twenty-two,
and plausibly none".

THAT WAS A FACT ABOUT ONE ENTRY.
It is not a fact about the wire,
which votes absence
near-unanimously the moment it is shown a different page,
and it is not a fact about the corpus.
`mikaela_khara` is a well translated entry whose block candidates really are covered;
`shi_Yumiaoya` and `Aniloviraw` are not,
and the same wire says so without being prompted.

So the paragraph-scale evidence base needs re-deriving over a spread of entries
before it can carry the weight `#100` puts on it.
This does NOT decide question 28 and nothing has been wired:
`groupSourceFirst` stays unwired and `alignDocumentSections` still emits no insertion pairs.

### What is NOT established

Whether those absence verdicts are CORRECT.
They could be genuine omissions or the aligner
mispairing,
and this control does not tell them apart.
The argument that they are omissions rather than merges is that a merge puts the content
somewhere in the translation,
which is what `carried` reports,
while `absent` claims it is nowhere.
That is reasoning,
not a measurement,
and checking it means reading those passages against their source.

### The instrument said "not damageable" for two opposite reasons

The first breadth run printed one line for both,
and half the entries hit it,
so half the run was unreadable.
A roster that never called a passage covered is the wire
voting absence on undamaged text,
the strongest reading available;
a quote that cannot be found in the page is an anchoring failure that says nothing about coverage.
Fixed in `02a65109f`:
every refusal now carries its reason,
the undamaged verdict and its
absence votes,
and the control no longer throws when nothing was damageable,
because a page
offering nothing to cut is a result rather than an empty run.
It still cannot HOLD on zero rows,
since holding is a claim about what damage does.

## The absence verdicts are corroborated by a signal that consults no model

The breadth run left one thing unestablished:
whether those absence verdicts are CORRECT.
A deterministic page-level measurement settles it without reading a passage.

English characters per source character,
whole page,
over all 92 pairs as `readCorpusFile` sees them:
p5 1.42,
p25 2.28,
p50 2.65,
p75 3.00,
p95 4.52.
Chinese becoming English expands,
so a page far below that band is missing content by arithmetic.

The six entries the control ran on separate completely along that ratio:

-   `shi_Yumiaoya`,
    absent on 8 of 8:
    ratio 0.37,
    the SECOND SHORTEST translation of 92.
-   `Aniloviraw`,
    absent on 3 of 3:
    ratio 1.88,
    rank 11.
-   `XingZ60`,
    not covered on 4 of 4:
    ratio 2.00,
    rank 12.
-   `Futajuhuacha`,
    HELD and carried:
    ratio 2.79,
    rank 55.
-   `mikaela_khara`,
    HELD and carried:
    ratio 3.09,
    rank 77.
-   `TianqiChen666`,
    HELD and carried:
    ratio 3.11,
    rank 80.

No overlap.
The three entries where the roster refuses to call anything covered are the three
most under-translated of the six,
at ranks 2,
11 and 12 from the short end;
the three where it calls passages carried and notices their deletion sit at 55,
77 and 80.

SO THE ROSTER IS NOT INVENTING ABSENCE.
It votes absent exactly where an independent
character count says the English cannot hold the content,
and carried where the English is
the length a full translation would be.
That is the correspondence check `#106` asked for,
obtained without a model and without
quoting a line of the corpus.

Note on two different character counts for the same entry:
`git show | wc -c` reports 9795 source and 1630 English for `shi_Yumiaoya`,
while `readCorpusFile` reports 3935 and 1458,
since the reader strips frontmatter.
The pipeline's own reader is the one used here.
Both put the entry at the extreme low end
(0.17 and 0.37 against a p5 of 1.42),
so the reading does not turn on which is used.

### What this means for option C

Question 28's option C proposes inserting only where absence is corroborated by more than the
matcher's refusal,
naming "the target document is shorter than the source by about that
section's size" as the corroboration.
Measured at PAGE scale that corroboration and option A's
model verdicts agree on all six entries.
That is evidence for A's verdicts being trustworthy,
and it is also the first evidence that C's underlying signal is real rather than tuned,
though C would still need it at SECTION scale,
where it was only ever tried on two entries.
