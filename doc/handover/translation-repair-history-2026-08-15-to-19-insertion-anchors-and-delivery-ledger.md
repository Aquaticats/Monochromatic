# Translation repair history 2026-08-15 to 2026-08-19: insertion anchors, separators and the ledger check

The two reviews that reordered the work on passages the archive never translated,
the placement model that lets a chunk name a place,
the span checks that prove a rendering can be written back,
separator ownership at assembly,
and the check that a delivery ledger agrees with the document it describes.

Part of [Translation repair history: 2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Insertion anchors, separators and the delivery ledger check

### Two sol reviews of `#100`, and what they changed

The first read the ten files of the slicing and alignment path and corrected the
recorded design in ways that reorder the whole task,
now rewritten into `#100`:
`alignHeadingsForced` emits no target cursor,
so the "place at the cursor" rule
was unimplementable;
`ambiguous` does not mean untranslated and treating it so can
DUPLICATE content;
a forced gap need not have a unique boundary;
therefore "emit
every source run exactly once" and "never place inside uncertainty" cannot both
hold,
and some source sections must stay unplaced.
Deleting `mergeOneSidedRuns`
alone makes `runToChunk` throw.
The landing order is now producers LAST:
placement
model,
then assembly,
then lane,
then paragraph subdivision,
then section-level
insertions.

The second was launched because the first had answered a question about
`translate-stage.ts` WITHOUT that file in front of it.
It confirmed the wrong-
success state (with both texts blank,
`wantsReplacement` is false,
the alignment
guard cannot fire,
and a missing translation settles as an ordinary unchanged
slice) and corrected two claims:
structural repair of candidates must NOT be
skipped in absent mode,
since it validates against the ORIGINAL,
and blank fresh
candidates already never reach the slate because `buildTranslateCandidates`
filters them.
It also found that a blank reply still counts toward
`heardTranslators`,
and that the incumbent fallback reports a producer for text
that does not exist.
Both are unmeasured:
no artifact under the runs directory
carries `translate-blank` or `translate-no-candidate`,
because the translate lane
has never run over the corpus.
They are recorded in `#100` landing 3 rather than
fixed speculatively now.

### A chunk can now name a place, and a span has to prove it can be written back

`26267a148`,
`faaa83ed8` and `f51c65549`,
which are landings one and two of
`#100` in the order the review set:
nothing produces an insertion yet,
and by
the time something does,
assembly will already refuse the shapes it must.

THE PLACEMENT MODEL,
in `chunk-placement.ts`.
A chunk is either CONTENT,
which
covers existing text,
or an INSERTION,
which names a boundary where a
translation belongs and none exists.
The discriminant is a field rather than
emptiness:
no constructor here produces an empty content chunk,
but the
exported type is structural,
so any caller can build one and a
`nodes.length === 0` test would silently promote that fabrication to an
insertion.

SOL ARGUED AGAINST MY FIRST SHAPE and was right.
I was going to hang an optional
`placement` field on the one broad chunk type;
that buys the word discriminator
without the protection,
since an insertion stays assignable to everything that
wants content.
The union it proposed,
with the discriminant optional on content
and required on an insertion,
changes no existing construction site and still
stops an anchor reaching a content-only parameter.
Exactly two places had to be
told they mean content,
both in `slice-pair.ts`,
because production makes
nothing else.

THE LINTER ARGUED WITH SOL AND WON A SMALLER POINT.
The review wanted an
insertion typed with an empty tuple for its nodes and an empty string literal
for its text.
`no-optional-escape` refuses both:
a zero-length container is
absence spelled as a value,
and the rule asks for a distinct non-empty domain
value instead.
The `kind` field is that value,
and the constructor keeps the
other two empty.

ONE RULE COVERS THE WHOLE REFUSAL LIST,
in `placement-layout.ts`:
every target
span starts at or after the previous span ends,
walked in slice order.
From that
follow no overlap,
no anchor inside a span,
no two spans starting at one offset,
no anchor after a span it starts with,
and no backward placement,
while every
legal shape stays legal.
Sol checked the equivalence claim against the code and
agreed there is no offset-only counterexample once the per-slice shape checks
pass.

WHAT IT FOUND THAT I HAD NOT:
array order is only slice order if the indices are
positions,
and `spliceSlices` never said so.
Its counterexample is two anchors
at one boundary carrying unique but shuffled indices,
where the descending-index
sort writes BA for slices that say AB.
The splice now asserts slice indexing
itself.
It also refuses blank text written into an anchor whose original says
something,
which needed the index map to carry the whole pair rather than the
target side alone.

THE TEXT-AGREEMENT CHECK IS MINE rather than the review's:
a span's text must be
what the document holds between its offsets.
Sol kept it,
on the ground that no
offset rule can catch stale or foreign slices whose ranges are valid and
ordered,
and corrected the message,
since a mismatch means stale text,
wrong
offsets or another document rather than only another document.

FIVE PROBES,
each shown to fail before being trusted:
the discriminant read as
emptiness (an empty content chunk becomes an insertion),
the ordering rule
removed (a backward placement passes),
the text agreement removed (foreign
slices pass),
the indexing assert removed (shuffled indices pass),
and the blank
refusal removed (an anchor ships nothing).
One earlier probe was rebuilt after
the first attempt failed for the wrong reason,
a missing import rather than the
guard:
a probe that fails on a `ReferenceError` proves nothing about the
assertion it was meant to test.

WHAT LANDING TWO STILL OWES,
recorded in `#101`:
separator ownership.
Assembly
writes model text verbatim,
so an anchor before a heading concatenates with that
heading and one at end of file concatenates with the previous paragraph.
The
review's rule is to strip only outer blank-line material from a fragment,
join
same-anchor fragments with one canonical blank line,
preserve existing
whitespace byte for byte,
and add one blank line only where an insertion creates
an adjacency that had no separator.
It also asks whether a MISSING replacement
for an anchor should be refused the way a blank one now is,
which cannot be
answered until the absent-incumbent lane work says whether assembly may ever
withdraw an anchor's replacement.

### The blank line between two blocks now has an owner

`e2c624fa9` finishes landing two of `#100` and closes `#101`.
Every replacement
until now went into a span that already sat between the right separators,
so
writing model text verbatim preserved them and nothing had to decide anything.
An anchor has no span:
written verbatim before a heading it produces
`...afternoon.## Habits`,
which still parses as Markdown and says something
else.

ASSEMBLY DECIDES,
not the prompt.
A prompt asking for correct leading and
trailing blank lines is a hope that fails silently,
and it cannot be right
anyway:
several fragments landing at one boundary,
each carrying its own blank
lines,
put two between every pair.
Only assembly knows what is on both sides of
the boundary,
how many fragments share it,
and what the document separates
blocks with.

THE RULES,
from the review and unchanged by implementing them:
strip only outer
blank-line material from a fragment and keep its indentation,
since a rendering
that begins with spaces is inside a list or a quote;
join same-anchor fragments
with one blank line;
preserve existing whitespace byte for byte and only top it
up;
use the document's own line ending,
which a Windows translation needs and a
diff would otherwise report as changes to lines nobody touched;
and treat the
end of the file as termination rather than as separation from nothing.

WHAT CHANGED SHAPE:
anchors sharing a boundary are now ONE edit rather than
several writes in sequence,
because the separators between them are decided
once for the group.
Content replacements still go in verbatim,
so nothing in
production moves:
no producer emits an anchor yet.

FOUR SPLICE EXPECTATIONS CHANGED,
which is the point rather than a regression.
Each asserted the verbatim write this replaces,
and two of them had an insertion
running into the paragraph after it.
A test that pins the old behaviour of the
thing you are fixing is not a regression test;
it is the defect,
written down.

PROBE:
composition replaced by joining the fragments,
which is exactly the old
behaviour.
Five cases fail,
including the heading case this exists for.

WHAT THE REVIEW LEFT OPEN,
recorded in `#101` and `#100`:
whether a MISSING
replacement for an anchor should be refused the way a blank one now is.
It
cannot be answered until the absent-incumbent work says whether assembly may
ever withdraw an anchor's replacement,
since withdrawing one restores nothing
where a translation belongs.

### A ledger now has to agree with the document it describes

`2920df105`.
`buildSliceDelivery` joins three reports from one lane,
and every
check it made was INSIDE that join:
a row cannot say shipped and undecided at
once,
and a shipped row's text is its accepted text by construction.
What no row
could check is whether the join describes the document the lane returned,
since
the document is not one of its inputs.

TWO CLAIMS,
both needing the document in hand:
the rows marked shipped are the
slices the result names,
and writing those rows over the archive reproduces the
returned text.
The second is the one that earns its keep.
It crosses from what
the lane DECIDED,
which is where a row's text comes from,
to what the document
CARRIES,
which the assembly guard decided;
those are two derivations that agree
today by construction and never said so.

THROUGH THE SAME ASSEMBLY rather than by concatenation,
which is what keeps the
comparison true once anchors exist:
the blank lines around an inserted rendering
are composed and belong to no slice,
so a row's shipped text is NOT a substring
of the document,
and a check that searched for one would refuse a document
nothing is wrong with.
There is a test for exactly that case.

`runDocumentLanes` returns a ledger per lane now,
each checked.
Derived rather
than decided:
each describes its own lane's document and neither mentions the
other,
so which document ships is still Question 5.
Writing one into an artifact
stays there too,
since the settled schema carries one lane (`#96`).
