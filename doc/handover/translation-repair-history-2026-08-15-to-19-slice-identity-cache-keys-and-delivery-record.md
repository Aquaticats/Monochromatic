# Translation repair history 2026-08-15 to 2026-08-19: slice identity, cache keys and the delivery record

The slice-indexing assertion and the guard proof that argued against it,
the change that keys a slice by what it asks rather than where it sits,
the per-slice delivery record and the cache envelope that proves a record's own name,
the in-run memo,
central index stamping,
and the refusal sentence that had stored an index.

Part of [Translation repair history: 2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Slice identity, cache keys and the delivery record

### `#99`'s first step, and a guard proof that argued against its own guard

`d1fd32853` adds `assertSliceIndexing`,
which states the property every cache
key,
splice and cross-lane join rests on:
both sides of a prepared slice agree
about its index,
and that index is the slice's position.
`prepareDocumentPair`
runs it,
since that is the only place holding every slice of a document at once.
It changes no index,
so no cache moves,
and it gives `#99`'s reshape of how
indices are assigned something that fails if the reshape breaks the invariant.

THE PROOF IS THE INTERESTING PART,
because it did not say what I expected.
Mis-stamping the base index with the assertion REMOVED fails 16 tests:
an
existing `prepareDocumentPair` test already covers global document-order
stamping,
and 11 `repairTranslation` children fail with `AssemblyContractError`
from the assembly invariants.
So this defect class was already caught,
three
stages downstream,
as a complaint about a replacement rather than about the
stamping.

That makes the new guard a diagnosis improvement and a floor under the reshape,
NOT the only thing between a mis-stamp and a shipped document,
and it is
recorded that way in `#99`.
Worth doing the probe even when a guard is obviously
correct:
what it measures is not whether the guard works but what the codebase
already knew,
and the honest answer here was "more than I assumed".

The barrel it exports through is new.
`pipeline-barrel.ts` sat exactly at its
line budget,
so one document PAIR,
from the shared preparation to the driver
that runs both lanes over it,
became `document-barrel.ts`.

### A slice is now keyed by what it asks, not by where it sits

`5577324f5`,
and it is the change that makes the rest of `#99` and `#100`
affordable.
Both lane caches hashed the slice index beside the run shape and
both texts.
That meant any renumbering discarded every slice below the change,
however untouched its text,
and one-sided slicing renumbers BY DESIGN:
it
inserts a slice wherever a section has no translation.
Without this,
`#100`
would have rebought the corpus on its first run and again on every slicing
change after it.

WHAT A KEY IS FOR,
stated the way that settles the question:
two runs' slices
are the same slice when the models would be asked the same thing.
That is the
source text,
the incumbent,
the governance flag and the run shape.
Where the
slice sits is not part of the question,
so it is not part of the key.

WHAT IT COSTS,
measured rather than assumed.
Two slices carrying identical
source text,
identical incumbent and identical governance inside one document
now share a cache entry.
Their models would decide identically,
so the shared
record is right rather than merely cheap.
Across the 92 pinned documents and
1260 slices there is no such pair;
the probe was validated first on an invented
document with two identical sections,
where it finds the pair.
Both drivers now
stamp a resumed record with the index they asked under,
because the index the
record was computed with would otherwise name the wrong slice in every issue
record and replacement built from it.

The repair cache moves to 26 and the translate cache to 2,
which discards what
is on disk.
The user authorized that explicitly,
and the version histories in
both files say why it was spent here rather than later.

TWO TESTS CHANGED MEANING RATHER THAN WORDING,
which is worth noticing:
both
lanes used to REFUSE a cached record whose index disagreed,
and that refusal was
the very thing this change makes wrong.
They now assert the re-stamping,
by
resuming from records whose indices are all off by one and checking the document
and every index against the run that settled them.
A third test in
`document-lanes.unit.test.ts` had used the old refusal as its way to make the
repair lane fail with the signal live;
it now fails the lane's cache write
instead,
which is a first-lane failure of the same shape and does not depend on
a rule that no longer exists.

### Every slice now carries a delivery record, and a cache entry proves its own name

`2210fbbf8` builds `slice-delivery.ts`,
which is item 2 of `#102`:
one record
per slice naming the source,
the archive wording,
what the lane decided,
what
actually shipped,
and which of the three ways it shipped.
The distinction it
exists for is that a slice can DECIDE a replacement and still ship the archive
text,
because `guardFootnoteAssembly` withdraws replacements for footnote
damage,
for structural regression,
and for a set that reassembles to the archive
text.
Reading the document alone,
those are indistinguishable from a slice the
judges left alone.
Nothing calls `buildSliceDelivery` yet:
the wiring waits on
Question 5,
and items 5 and 6 (assert that the shipped set equals the
`replacement-shipped` records,
and that reassembling the ledger reproduces the
document) are still owed.

`358efd207` closes the hole the cache-key change opened.
A persisted slice is
named by a hash of its key,
and the loader trusted the file name to say which
question the record answered.
With the index in the key that was nearly
unfalsifiable;
without it,
a record moved,
copied or renamed answers for whatever
slice asks under that name,
and the driver splices its text into a slice it was
never computed for.
Every persisted record now carries an envelope stating the
key it answers,
and the loader refuses a payload whose envelope disagrees.

TWO CORRECTIONS FROM SOL ON THAT ONE,
both worth keeping.
First,
a reader cannot
know whether the run,
a later edit or a truncated write broke a record,
so the
refusal must not blame the run:
it names the file and what disagrees.
Second,
my
own commit message says the two remaining findings are recorded in `#99`.
They
are in `#95`.

### One question, asked once, whatever the cache holds

`40cf35737`.
Taking the index out of the key means two slices with identical
source,
incumbent and governance share an entry.
A WARM run resumes one record
for both,
which is right.
A COLD run asked the models twice,
kept two different
answers,
persisted both under one key and could use either.
Same input,
two
documents,
depending on whether a cache existed.
Both drivers now memoize what
they settle by key within the run,
so the cold path reuses what it just settled
exactly as the warm path resumes it.

ONE NUANCE RECORDED IN `#95` RATHER THAN DECIDED:
the memoization is
unconditional,
including for a record the driver deliberately did not persist
because no translator was heard.
An in-run twin therefore reuses an unheard
record while a warm run would ask again for both.

### The final name of a slice is stamped where the whole document is in view

`3afb233de`.
Subdivision was handed a base index and added its own offset,
which
is right only while every earlier section contributed exactly the slices the base
counted.
`#100` breaks that deliberately:
an insertion slice for an untranslated
section is a slice the base index never saw coming.
`prepareDocumentPair` now
restamps every carved pair through `reindexSlicePair` from where it actually
landed,
then asserts the invariant.
That closed `#99` on the minimal
alternative:
assert the property,
take the index out of the key,
and stamp
centrally,
rather than reshaping three stampings into one type.

### A refusal now names the slice it is actually for

`dba53968e`,
found by the advisor rather than by a test.
`alignmentRefusalFinding`
produces a sentence naming a slice by index,
and that sentence was STORED inside
the settled record.
Since the key no longer carries the index,
one record answers
for any slice asking the same question and is re-stamped when it does,
but the
stored sentence kept the index it was first settled under.
Two identical sections
therefore reported slice 0 twice and slice 1 never.
The sentence is now derived
in the driver from the record disposition,
its alignment measurements and the
index it carries after stamping.

MEASURED STRUCTURALLY,
because both caches had just been invalidated and disk
measures nothing:
this was the ONE index-bearing string stored in a persisted
record.
The repair lane stores none,
and every wire finding names a within-prompt
reference such as an edit region or a claim index,
which the key already covers.

THE PROMPT AUDIT THAT THE WHOLE KEY CHANGE RESTS ON is also closed,
recorded in
`#95`.
Of every file that builds model messages,
only `repair-contract.ts` and
`assembly-integrity.ts` mention `chunkIndex` at all,
and those are a type field
and a set of map keys.
The only index-derived value reaching a prompt is the
`lineStructured` governance flag,
which is in the key,
and `identityContext` is
in the run shape.
So identical texts really do mean an identical question.
