# Translation repair open decisions: what a thin roster's verdict is worth to the cache (question 6)

Question 6 on caching slices that fewer models than the stage asks for examined,
with the measured population and three options.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

## Question 6: what a thin roster's verdict is worth to the cache

BLOCKS NOTHING.
It changes how much a resumed run re-buys,
so it is worth
answering before a long run rather than during one.

WHAT IS ALREADY DECIDED AND BUILT,
so this question is only about the middle
ground:
a slice NOBODY examined is never cached.
Zero critics heard,
or zero
translators heard,
settles for the run and is left out of the cache,
so the next
attempt asks again instead of resuming an outage as a verdict.
A caller abort
now stops both drivers rather than settling the slices it interrupted.

THE MIDDLE GROUND is a slice that was examined by FEWER models than the stage
asks for.
Quorum is half the roster rounded up,
so on six critics a slice
decided by three met quorum exactly,
and one decided by two did not.

MEASURED,
on what is actually on disk:

-   Of the 150 cached repair slices,
    `heardCritics` runs 3 for 1 slice,
    4 for 3,
    5 for 92 and 6 for 54.
    Not one is below quorum,
    and one sits exactly on it.
-   Across all 56 settled artifacts,
    34 unmet-quorum findings appear,
    in 7
    entries,
    and every one of them is the REFINER.
    Critics,
    panel,
    editor,
    judges and checkers never fell short of quorum in the settled corpus.

So the population this question governs is small,
and it is concentrated in the
one lane that is optional:
naturalness refinement,
whose silence was traced to a
single model in `#73` and `#77`.

### Options

A.
 Cache only a slice where EVERY stage met quorum;
re-buy the rest next
    attempt.
    Pros:
what resumes is then work done at full strength,
and nothing thin
    survives into an artifact by being cached first.
    Cons:
the cache is per SLICE,
so a refiner that lost quorum re-buys the
    critics,
the panel,
the editor,
the judges and the checkers with it,
to
    retry one optional lane.
And it can loop:
the refiner's silence has a known
    chronic cause,
so those slices may never cache at all.

B.
 Cache anything that was examined at all,
which is the rule as it now stands:
    zero voices is not cached,
one voice is.
    Pros:
matches what the stages already decided to do,
since a stage short of
    quorum deliberately proceeds with findings rather than failing;
costs
    nothing to keep.
    Cons:
a slice decided by a thin roster resumes forever with no way to tell
    it apart from one decided at full strength,
unless a reader goes looking in
    the findings.

C.
 Cache it,
but RECORD that it was thin,
and let a later pass re-buy only
    those slices.
    Pros:
keeps the budget while making the population addressable;
the record
    already exists in `findings` and would only need a field a reader can filter
    on.
    Cons:
a schema field,
a version bump and a reader for a population that is
    34 slices in 7 entries today.

RANKING:
B > C > A.

B over C because the measured population is small and sits entirely in the lane
where thinness means "no improvement was attempted" rather than "nothing
inspected this",
and C's field cannot be added without a cache version bump,
which discards the 150 slices already on disk to gain a filter over none of them.
C over A because A pays for a whole slice to retry one optional lane,
and pays
it again on every attempt while the cause persists.

B
