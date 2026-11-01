# Translation repair history 2026-08-15 to 2026-08-19: the lane handed a passage the archive never translated

The translate and repair lanes when the incumbent is absent,
the removal of the wrong-success state,
the departure from the review recorded for veto,
and the external review of that landing with its four fixed defects and recorded items.
Its first paragraph is the third landing of a task whose first two landings are recorded in `translation-repair-history-2026-08-15-to-19-insertion-anchors-and-delivery-ledger.md`.

Part of [Translation repair history: 2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The lane handed a passage the archive never translated

### The lane can be handed a passage the archive never translated

`a5091af5e` and `0ba633b62`,
the third landing of `#100`.
Nothing produces such
a slice yet;
landings four and five are the producers.

THE WRONG-SUCCESS STATE REMOVED:
every fallback in the translate stage ships the
wording already in the archive.
That is right for a slice that HAS one,
since
leaving a passage as it stands is the state the run began in,
and shipping text
no judge vetted is a new claim about the archive.
For a slice with none,
the
same fallback shipped the empty string and reported a settled slice,
so the run
read as having delivered a translation it never produced.

ABSENCE IS A MODE,
NOT AN INFERENCE.
`incumbentKind` is decided from the target
chunk being an insertion anchor,
never from the text being blank:
a content span
holding only whitespace is the archive's own wording,
thin as it is,
and the two
ask different questions.
That is also why it is in the CACHE KEY rather than
only in the record:
both carry identical texts,
so a key over texts alone would
hand one the other's answer.
Schema version 3,
and the bump discarded nothing,
measured first.

NOT STORED ON THE RECORD,
against the review's suggestion,
because the prepared
slice is the source of truth and cannot go stale,
while a copy inside a cache
record could be resumed against a slice of the other kind.

AN ERROR RATHER THAN A RESULT at the stage boundary,
because there is no honest
result to build:
the stage returns the text that ships plus who produced it,
and
here nothing ships and nobody produced it,
so every field would be invented.

WHERE I DEPARTED FROM THE REVIEW,
recorded as decision 26 for veto:
it wanted
the lane to throw and leave the entry unsettled.
The driver instead catches the
refusal per slice,
records the passage as unfilled with the stage's findings,
caches nothing,
and lets the rest of the document settle.
The document keeps the
gap the archive already had,
which states nothing false;
a decline depends on
which judges answered,
so throwing discards every other slice's work over
something that varies between runs;
and `unfilledChunkIndices` names those
slices,
so a missing passage cannot be read as one the judges kept.

THIS ANSWERS WHAT `#101` LEFT OPEN,
and the answer is in the lane rather than in
assembly:
a MISSING replacement for an anchor is legitimate and writes nothing,
because that is exactly what an unfilled slice produces,
while a BLANK
replacement for an anchor whose source says something is still refused,
because
that is a lane claiming delivery.
The two were never the same case.

A BLANK TRANSLATOR REPLY IS NOW A LOST VOICE.
`{"translation": ""}` satisfies
the schema,
so it arrived as a heard voice,
was dropped further down as an
unusable candidate,
and its model counted as answered and was never re-asked.
The wire guard refuses it,
so the roster asks again.
Decision 27 records what
that changes in a count.

THE REPAIR LANE SAYS THE QUESTION DOES NOT APPLY.
Its critics compare a
translation with its original,
its editor rewrites the regions their defects
name,
its checkers confirm they are gone;
handed an anchor,
every stage is asked
about text that does not exist.
`notApplicableRepair` states that,
with no
exchange spent,
and the outcome list stays position-aligned with the slice list,
which is what a skip would have broken.
Measured rather than asserted:
the same
preparation with and without an anchor spends 14 exchanges,
against 17 when the
branch is neutered.

TWO FILES WERE SPLIT AT THEIR LINE BUDGET rather than raised.
`translate-stage-result.ts` holds what a round DECIDED,
which is the record every
later reader joins to,
and `translate-slice-attempt.ts` holds one slice's two
honest endings.
The second split was forced by a rule worth knowing about:
`no-nullish-union` refuses `TranslateSliceRecord | undefined` as a return type,
so the two endings had to be named rather than one of them spelled as absence.

FOUR PROBES,
each shown to fail before being trusted:
the absent-mode
no-candidate refusal removed (only the no-candidate case fails),
the absent-mode
decline refusal removed (only the two decline cases fail),
the wire guard's
blank rejection reverted (the re-ask case fails on the heard count,
3 against
2),
and the repair lane's anchor branch removed (the exchange count moves from
14 to 17).

### What the review of that landing found, and what is still open

A source-bearing external review of the absent-incumbent work,
run over the
whole lane,
the selector and the assembly path.
Four defects were real and are
fixed;
three items are recorded rather than built,
and one claim it made is
answered by a measurement rather than by code.

WHAT COULD HAVE RECORDED A TRANSLATED PASSAGE AS A MISSING ONE,
which is the
class that mattered.
A blank winner raised the ABSENCE error,
so a deletion for
a slice the archive does translate would have been caught by the driver and
written into the unfilled list.
It now raises its own error,
in either mode,
since a deletion is a defect rather than an outcome.
Two more layers refuse the
same shape:
the attempt layer accepts an unfilled result only for an anchor,
and
the wording builder refuses an unfilled index naming a slice with archive
wording.
None of the three was reachable through the normal path;
all three are
one regression away from being reachable,
and the failure is silent.

THE ALIGNMENT REFUSAL WAS NOT GATED BY MODE.
It restores the incumbent,
which at
an anchor is nothing,
so a selected rendering could have been turned into a
settled blank anchor:
the exact wrong-success state this work removes,
arriving
by the back door.
Gated to a present incumbent,
with a check that a record for an
absent slice carries a translation before it is built.

A DOCUMENT WITH HOLES NOW SAYS SO.
`status` is `complete` or `unfilled`,
and the
gaps are entries carrying their reason and the stage findings rather than a list
of indices.
The old shape was nameable and still missable:
a consumer reading
the text and the counts saw an ordinary success,
and several unfilled slices
flattened their evidence into one list where nothing said which passage each
belonged to.

THE JUDGE SHEET WAS PROMISING A FALLBACK THAT DOES NOT EXIST.
Every judge is told
declining is safe because the caller keeps text it already trusts;
at an anchor
there is none,
so the sentence bought a missing passage with the caution it
asked for.
The consequence is now the caller's to state,
defaulted to the old
sentence so no other caller changes,
and the test reads the sheets the judges
received rather than trusting the builder.

ONE CLAIM ANSWERED BY MEASUREMENT rather than by code:
the review doubted that
the repair lane spends nothing on an anchor,
since the not-applicable outcome
still reaches refinement.
It does reach it,
and costs nothing,
because
refinement derives its envelopes from the outcome text and an empty text yields
none.
The test measures the exchange count with and without an anchor:
14 both
ways,
against 17 when the branch is neutered.

ONE CORRECTION TO MY OWN REASONING,
worth keeping because the rationale was
wrong while the change was right.
The cache key carries the incumbent kind,
and
I justified it with a collision between an anchor and a whitespace-only content
span.
There is no such collision:
the whitespace span carries its whitespace
rather than the empty string,
and a content chunk covering nothing is refused by
the layout guard.
The field stays because the QUESTION differs,
which is a
better reason and holds even if those two facts change.

WHAT IS RECORDED RATHER THAN BUILT,
all in `#100` and `#102`:
a machine-readable
disposition on the repair not-applicable outcome,
which is prose-only today;
a
structured attempt ledger with an explicit retry policy,
which the review ranks
above both the current always-retry behaviour and an expiring negative cache,
and which needs the decline rate measured first;
and the settled-record list
still holding only filled slices rather than a full-length union,
which the
status field and the unfilled entries make legible but do not make positional.

ONE THING DELIBERATELY LEFT AS IS,
since the review flagged it as a cost:
two
identical anchors in one document are both bought,
because an unfilled slice is
memoized nowhere.
That is the cold-warm agreement rule rather than an oversight.
A warm run finds nothing cached for an unfilled slice and buys it again,
so an
in-run memo would make a cold run cheaper than a warm one over the same
document,
which is the divergence that memo exists to prevent.
