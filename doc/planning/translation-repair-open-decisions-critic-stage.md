# Translation repair open decisions: whether the critic stage survives (question 3)

Question 3 on whether critics stay in the translate path,
what dropping them saves,
the blocking behaviour that cannot stay,
and the options and ranking.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

## Question 3: does the critic stage survive

BLOCKS `#86`,
and the answer changes the cost of every entry.

Critics exist to find defects for an editor to repair,
and the decided shape
repairs nothing.
Dropping them removes 582 calls per corpus and adds 105,
making the new shape CHEAPER than the one it replaces.
Keeping them gives the
judges evidence and keeps a stage that reasons about the source,
which
Question 2 argues for.

There is a third answer I did not see until the review:
if critics stay,
their
BLOCKING behaviour cannot.
`repairChunk` returns the input unchanged when
non-translation votes stand,
and the document-level dominance check can return
the whole original target,
discarding translated slices that already succeeded.
On a sparse target,
which is exactly what the lane exists for,
that is the
common case rather than the rare one.

THIS QUESTION ALSO DECIDES WHETHER `#93` IS A DEFECT.
An empty critic roster
today settles a document rather than refusing it,
which is wrong when the roster
is empty by MISCONFIGURATION and right when critics were dropped on purpose.
Answer A makes an empty critic roster the intended configuration,
so any guard
`#93` grows has to refuse a MISSING STAGE rather than an empty list,
or be
placed where the stage is requested rather than where its roster is read.

### Options

A.
 Drop critics from the translate path.
    Pros:
cheapest;
removes the blocking behaviour by removing the stage;
    the judges already compare against the source.
    Cons:
loses the only stage that names WHY a passage is wrong,
which is what
    every grading sheet has been built on.

B.
 Keep critics as evidence for the judges,
with every early return removed.
    Pros:
keeps the diagnosis and the sheets;
judges get named defects rather
    than only two texts.
    Cons:
pays for a stage whose output no longer decides anything,
and the
    numbers that would justify it do not exist until `#83` lands.

C.
 Keep critics only where the incumbent is substantial,
and skip them where
    it is thin.
    Pros:
spends the calls where a critic can see something.
    Cons:
"substantial" is a threshold,
which is the kind of number you
    rejected when it was called a coverage ratio.

RANKING:
B > A > C.

B over A because the sheets are the instrument every quality claim rests on,
and losing the critic loses the vocabulary they are written in;
the cost is
recoverable later and the instrument is not.
A over C because C reintroduces exactly the magic threshold you rejected in the
pipeline-shape decision,
and a threshold that decides whether a passage is
examined at all is worse than one that decides how it is routed.

B,
land #83
