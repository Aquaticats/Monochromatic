# Translation repair history 2026-08-15 to 2026-08-19: the whole-day review and its fixes

The night of 2026-08-15,
when the day's code went to a second reader in full,
the twelve findings it returned and the one nobody found,
two measurements taken while fixing,
and the second round of review on those same fixes.

Part of [Translation repair history: 2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Night of 2026-08-15: a whole-day review, twelve findings, and one nobody found

### The review's answer and what landed from it

The day's code went to a second reader in full,
and the answer named twelve
things.
Ten were real and are fixed;
one was a duplicate of what `#103` already held;
one is held open by Question 3 rather than by doubt.
`#103` carries the disposition of each with its commit.

FIRST,
THE ONE NO REVIEW FOUND,
because it is the one that would have cost a
document.
The naturalness lane stamped `changed: true` from the REWRITER's
verdict,
and a rewriter is measured against the accuracy text it rewrites rather
than against the archive.
A refinement that lands back on the archive's own words is a slice nothing
happened in,
and it was being recorded as a change.
That names the slice in the shipped set carrying the archive wording,
which
`assertReplacementsChange` refuses,
so a run the models got right would have
aborted the whole document at assembly.
It now reads the archive text,
and drops that slice's resolved-issue credit on
the same rule the accuracy stage already applies:
nothing it returns can have
resolved anything.

REFINEMENT ALSO HAD NO ABORT PROTECTION AT ALL.
A torn-down exchange surfaced as whichever stage happened to fail,
so a caller
could not tell a spent deadline from a provider fault,
and a phase that settled
under an abort returned a document that read as a finished run.
Both rules now live in `repair-refine-step.ts`.
The second one is CONDITIONAL on the lane having asked somebody something,
and
that condition is load-bearing:
the slice loop deliberately lets a fully cached
document finish under an abort,
because what a stopped run cannot do is BUY what
it is missing,
and an unconditional check broke that rule in the probe.

WHAT ELSE LANDED,
in one line each:

-   `guardFootnoteAssembly` checks its own replacements,
    so a direct caller
    cannot hand it a no-op and have the net-zero branch adopt it as an honest
    empty result.
-   `deriveShippedIndices` states the call order as a precondition and names it
    in the message.
    That refusal is the one a blameless run can reach,
    and only
    by calling it before the guard.
-   A structural regression withdrawn in the same round a footnote took the
    blame is now recorded.
    It used to vanish,
    because a regression the
    withdrawal also fixed never reached another round.
-   Both lanes check a freshly settled record against its own text before
    caching it,
    not only when resuming one.
    Only one direction of that
    contradiction was ever caught downstream:
    a record DENYING a change it made
    is dropped in silence,
    since only changed records become replacements.
-   `winnerChangedText` reads the text alone,
    and `selectRepairCandidate`
    refuses a slate whose unchanged candidate carries anything else.
    Its
    measurements stay unchecked on purpose,
    so an archive that genuinely fails
    an integrity check is still expressible.
-   The comparison refuses repeated rows on the repair side too,
    and refuses a
    repeated shipped index rather than folding it into a set.
-   `runDocumentLanes` preflights both lanes in one call,
    with lane-prefixed
    role names:
    both lanes have a `judgeModelIds` and one object cannot hold
    that key twice.

### A harness fact worth knowing before you read any removal proof

`await describe(...)` runs at module scope,
so a FAILING suite stops every later
suite in the same file.
A probe that breaks an early suite hides whatever the later ones would have
said.
This was found the honest way:
a probe reverted two guards in
`select-candidate.ts` and only one test failed,
which looked like the second
guard being unpinned.
Reverting them one at a time showed both were load-bearing.
Read a probe's failure list as a floor rather than a total.

### Two guards this night added are pinned by no test, on purpose

`assertSettledRecordAgrees`'s CALLS in both lanes are vacuous by construction:
neither lane can produce a contradictory fresh record now that both derive
`changed` from their own text,
so removing the calls turns nothing red.
The function itself is tested.
This is the same shape as the blocked exit's vacuous check,
and it is here for
what it costs rather than for what it catches.

`winnerChangedText`'s wiring is still unpinned for the reason recorded above.

### Two measurements taken while fixing, both zero quota

IMPORTS NOTHING USES.
The finding was six left behind in one file by an earlier
extraction,
on a lint run reporting zero warnings and zero errors.
`eslint/no-unused-vars` is off across this workspace by a decision documented in
place,
and its reasons are all about local variables.
A census over 3459 TypeScript sources finds 93 names used nowhere but their own
import line;
19 were in this package and are gone.
The policy question is `doc/planning/unused-import-lint-policy.md`,
ranked
A > B > C,
and it blocks nothing.

THE DOMINANCE DENOMINATOR IS THE SLICED FRACTION,
not the document.
`assessNonTranslationDominance` sums slice characters on both sides of its
ratio,
so an unpaired or unsliced section is in neither term.
Measured over the 92 pinned pairs:
slices cover 92.5% of an average English
document,
14 entries fall under 90%,
and two fall under half.
`XIEPT2` produces ZERO slices from 17 alignment refusals,
so the lane settles it
as a clean unchanged document having examined nothing.
Nothing behavioural changed,
because which denominator is right is a decision;
`#104` holds it,
and only the contracts that misdescribed it were corrected.

### The night's own fixes went back to the same reader, and nine things came back

Reviewing a review's output is not ceremony here:
the fixes above were written
in a few hours and touch the two assertions the whole assembly contract rests
on.
All nine findings were acted on,
and `#103` carries each with its commit.

THE ONE WORTH REMEMBERING is a finding added hours earlier,
in the same night's
work.
A structural regression withdrawn in a round where a footnote identifier
took the blame was being recorded as `assembly-structure-reverted`,
and the
round cannot know that.
Withdrawing what the footnote named MAY answer the parse
damage or may not;
the next round is what says.
It now reads
`assembly-structure-observed`.
The general lesson is worth more than the string:
a finding written from inside the round that produced it will reach for the past
tense of an outcome that has not happened yet.

TWO JUDGEMENTS WERE TAKEN RATHER THAN FIXED,
both recorded in `#103` with the
reasoning:

-   The refinement abort rule stays coarse.
    It fails an entry whose work is
    finished but was overtaken by an abort,
    because telling that apart from a
    torn-down exchange needs the phase to report whether a voice was abandoned,
    and every stage swallows a failed voice by design.
    A retry costs one entry;
    the alternative ships a cut-short document as whole,
    which costs a corpus.
    The narrower variant is `#103` item 10.
-   `selectRepairCandidate` refuses REPEATED unchanged identifiers,
    and nothing
    more general.
    The reviewer asked for unique candidate identifiers across the
    slate;
    the slate is always exactly two candidates built by
    `settleChunkVerdict`,
    so a general uniqueness check would be one more guard
    no test could pin.
    Recorded so it is not re-raised as an oversight.

THE SLICE CACHE WAS NOT BUMPED,
and the reasoning is now in the version history
in `repair-slice-key.ts` rather than only in a session.
Two changes moved what
`changed` MEANS.
Refinement outcomes are never persisted,
so that half needs
nothing.
And a version-25 accuracy record written before `winnerChangedText`
started reading the text can only over-claim a change,
never deny one,
because
the old rule answered `false` whenever the unchanged candidate won and that
candidate carries the archive text.
Over-claiming is exactly what
`sliceRecordAgrees` discards on resume,
at one recomputed slice.
Bumping would
discard every settled slice in the corpus to fix what the discard path fixes one
at a time.

WHAT IS STILL OPEN FROM BOTH REVIEWS:
`#103` items 8,
9 and 10.
Item 8 is the
same tense problem in the FOOTNOTE findings,
which is a scorecard change rather
than a string change and wants doing with `#102`.
Item 9 asks whether the
archive's own integrity should be measured rather than assumed,
which
`selectRepairCandidate` now makes expressible and nothing yet does.
Item 10 is
the narrow abort rule above.

FOR THE MORNING:
the dominance denominator is now Question 7 in
`doc/planning/translation-repair-open-decisions.md`,
with options,
a ranking,
and what I would do if it is delegated.
It was tracker-only before,
which is not
where a question the user has to answer belongs.
