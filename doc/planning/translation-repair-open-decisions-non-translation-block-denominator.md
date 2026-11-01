# Translation repair open decisions: what the non-translation block counts against (question 7)

Question 7 on the denominator of the one whole-document refusal,
the coverage census over the pinned corpus,
and the three options.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

## Question 7: what the non-translation block counts against

BLOCKS NOTHING TODAY,
and nothing changed behaviourally overnight.
It decides
what one whole-document refusal MEANS,
and that refusal is terminal:
it returns
the archive untouched and stops the entry from inside the slice loop.

WHAT THE CODE DOES.
`assessNonTranslationDominance` sums characters over the
SLICES it was handed,
on both sides of the comparison,
and blocks when the
slices standing as non-translation are more than half of them with no anchor
in sight.
Anything the aligner refused to pair,
and anything slicing left
whole,
is in neither term.
So the sentence the block currently supports is
"most of what we EXAMINED is not a translation",
while the parameter
documentation until last night said "archive characters in total".

MEASURED,
at zero quota,
over the pinned corpus of 92 prepared pairs.
Each
pair was prepared,
its slice characters summed,
and the total compared against
the whole English document:

-   Mean slice coverage,
    over entries with no alignment finding,
    is 92.5%.
-   14 entries fall under 90% covered,
    2 under 50%.
-   Only 2 entries carry any alignment finding at all.

The routine gap is front matter and the separators between chunks,
which
belong in no slice by construction and are not what this question is about.
The tail is,
and its two ends name the two causes.
`XIEPT2` produces ZERO
slices from 17 alignment refusals,
so 0 of its 1218 characters are examined:
both terms are zero,
the block cannot fire,
and the entry settles as a clean
unchanged document having looked at nothing.
`ArtsEpiphany` at 27.5% and
`windward0032` at 60.9% carry no alignment finding at all,
so their shortfall
is slicing rather than pairing,
which is `#90`'s territory rather than this
question's.

### Options

A.
 Read the ratio over the DOCUMENT,
so the block means "most of this
    translation is not a translation".
    Pros:
matches what every reader of a whole-document refusal will assume it
    measured,
and stops an entry whose examined part is all non-translation but
    whose bulk was never sliced from blocking the whole document.
    Cons:
an entry can then hide non-translation behind unsliced bulk,
since
    the majority it needs is now over text nothing inspected.
It also makes the
    block quieter the worse the slicing gets,
which is the wrong direction for
    a signal.

B.
 Read it over the SLICES,
as today,
and say so in the contract.
    Pros:
every character in the ratio was actually examined,
so the block
    never rests on text no model read;
costs one documentation pass,
which is
    already half done.
    Cons:
the refusal's name promises more than it measures,
and a document
    that is mostly unsliced cannot reach the block however bad the part that
    was examined.

C.
 Keep B's denominator and add a coverage floor:
refuse to DECIDE on a
    document whose sliced fraction falls below it.
    Pros:
the only option that gives `XIEPT2` an honest outcome,
which is
    neither clean nor blocked but unexaminable;
the floor is one number and the
    measurement above says where it would sit.
    Cons:
a third terminal outcome to represent,
and `#96` says the artifact
    cannot express "unknown" yet,
so this one waits on that.
It also needs a
    number picked from 92 entries,
and 2 of them are the whole population it
    would catch.

RANKING:
C > B > A.

C over B because B leaves `XIEPT2` settling as a clean unchanged document on
the strength of having examined nothing,
which is the one outcome in this
question that is actively wrong rather than merely narrow,
and C fixes it
without changing what the block itself counts.
B over A because A's failure
mode grows with the very defect it is meant to survive:
the less of a document
gets sliced,
the harder its examined non-translation is to see,
whereas B's
narrowness is stated and constant.

WHAT I WOULD DO IF YOU DELEGATE THIS:
take B now,
since it is documentation of
what already happens,
and hold C until `#96` can carry an unexaminable verdict.
That ordering costs nothing,
because C keeps B's denominator.

DELEGATE
