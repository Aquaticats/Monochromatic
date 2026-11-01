# Translation repair history 2026-08-21 to 2026-08-23: the size note for renderings far out of proportion

Why the planned fault names were refuted by their own measurement,
the free census over eleven settled artifacts,
the two trips shown to be page-only content,
what shipped instead as evidence plus a policy reading,
and the verification of that change.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-22, `#163` Commit B: the planned fault was refuted by its own measurement, and became evidence

Commits `2008ccc17`,
`bef5cba5c`,
`77294edd6`.
Both contests now carry a size note when one rendering of a passage is far out
of proportion to its Chinese,
and `CONTEST_POLICY` carries a reading for each
direction.
Neither direction carries a fault name.

### What the plan was, and why it could not be built

`#163` planned two named faults in `contest-ballot-wire.ts`,
one per tail
direction,
following the mechanism `#155` established.
The far-longer name contradicts the policy it would have joined.
`CONTEST_POLICY` already tells judges that where the Chinese is SILENT rather
than contradicting,
KEEPING page-only content is correct,
and it already
carries the sentence "A passage the Chinese states in one line can stand beside
a page region the archive spells out at length,
and the shorter candidate is
the one that lost something."
A candidate preserving such a region is far longer than the Chinese AND is the
right candidate.
Naming that a fault instructs a judge to penalise the exact behaviour `#155`
protects,
which is the shape of the criterion `#143` removed.

### The free measurement that settled it

Run over all 11 settled artifacts rather than buying a roster sample.
Ratios are computable without touching a passage,
because each artifact carries
`alignment.sourceCodePoints` and `incumbentCodePoints` per slice and the
candidates' own lengths.
184 rows,
116 eligible after the 80-character floor,
zero astral characters so
code points and UTF-16 units coincide.

  - Translate lane:
    0 trips in 58,
    ratios p05 1.00,
    median 3.03,
    max 4.59.
  - Repair lane:
    2 trips in 58,
    both on slices the lane returned unchanged.
  - Slices any lane PRODUCED:
    0 trips of 92.
    Repair-changed reaches 9.27
    against a 10 endpoint;
    translate-changed never passes 4.59.

So the endpoints transfer to this population,
and the thin 9.27 margin is a
second argument against a fault name rather than a reason to move an endpoint.

The proposal to gate far-longer on the candidate also exceeding the archive's
ratio is REFUTED:
at 100% of far-longer trips the candidate IS the archive,
so
such a gate could never fire.

### The two trips are page-only content, established without reading a passage

Attempting to print the passages was blocked,
which was correct,
and the
structural answer is better evidence.
Displacement implies a DONOR:
another slice whose source is large and whose
target is starved.
Profiling each whole document by size found none.

  - `dogesir_` slice 3:
    source 114,
    archive 1766,
    so 15.49 times,
    against a
    document running 2.75,
    with every other slice between 0.88 and 6.10.
  - `wangzihao980` slice 4 is the same shape just under the endpoint:
    source
    141,
    archive 1228,
    8.71 times,
    document 2.66.
  - `wangzihao980` slice 3 is the far-shorter trip:
    source 102,
    archive 66,
    0.65 times,
    where the archive genuinely under-renders the Chinese.

Both artifacts record `laneSelection` kind `pending-human-decision` and predate
`#155` and the contest,
so this measures lane outputs.
Lane outputs are what the contest judges,
so the population is the right one.

### What shipped instead

Evidence,
in `contest-size-note.ts`,
plus `SIZE_NOTE_POLICY` joined into
`CONTEST_POLICY`.
The two readings `#163` wanted from two names are carried by the policy:
far
shorter asks the judge to put the DROPPED question to that rendering;
far longer
opens two readings and says the surplus text decides,
naming the DROPPED-ALSO
rule for the page-only case and unsupported for the contradicting one.

Three threading decisions,
each made by measurement rather than taste:

1.  No document baseline.
    `PLAUSIBLE_BASELINE_MIN` 1.9 and
    `PLAUSIBLE_BASELINE_MAX` 4.5 sit strictly inside the 0.8 and 10 endpoints,
    so an absolute tail is already outside every norm the estimator accepts.
    Reading the baseline could not change whether a note appears.
2.  No verse marker.
    A fault name would have needed one,
    because verse expands
    unusually and a judge told "this is a fault" cannot discount it.
    The policy
    instead states outright that size settles neither reading and names verse as
    a reason a large ratio can be innocent.
3.  No cache version bump,
    following the precedent of every prior
    `CONTEST_POLICY` edit:
    `5de9d9085`,
    `8471664b7` and `d5407f7b4` all changed
    the policy and left `LANE_CONTEST_CACHE_VERSION` alone.
    Bumping would re-buy
    every cached ballot to deliver a note to the 1.7 percent of eligible slices
    that trip one.

The trigger calls the shipped instrument,
`sliceSizeOf` plus
`sliceImplausibility`,
rather than re-deriving a ratio,
so the note and the
baseline filter can never drift apart.
`block-count-gap` is excluded deliberately:
it describes the PAIRING rather
than the rendering,
and it was the sole cause for 20 of 36 flagged slices.

### Verification

Suite 531 PASS,
0 FAIL,
exit 0.
Lint 0 warnings and 0 errors,
types exit 0,
build exit 0.

GFP on the block-gap exclusion:
replacing the ratio-reason filter with a bare
emptiness check,
rebuilding and running failed exactly one test,
"SAYS NOTHING
for a block-count gap on its own",
with `expected 'SIZE NOTE for this
passage...' to equal ''`.
Restored and rebuilt green.

The `~10`-entry roster sample `#163` planned is SUPERSEDED rather than owed.
Its question was the false-fire rate,
now answered at 0 of 92 produced
candidates with an upper bound near 3 percent,
and ten random entries would
contain approximately no tails at all,
since 2 of 11 entries carry one.
