# The settled artifact stores the archive text whole

## Decision

A version 2 artifact stores the archive English of the entry whole,
alongside the preparation hash it already carries.
Stored text without a hash is a record nobody can check,
so the two travel together or neither is worth writing.

Decided 2026-08-19 by the agent under the owner's standing instruction of 2026-08-18:
"I saw your open decisions and I think they're not worth asking to me,
again,
under the principle of 'always pick whatever yields the best quality,
if you don't know which will,
prototype and measure'."

OPEN TO VETO.
This is recorded rather than asked because the measurement below removed the
only thing that made it a question.
If the owner wants the other answer,
say so and it
reverses at the cost of one field.

## Why it stopped being a decision

Task `#96` carried it as the owner's call on one premise:
"Storing it doubles artifact size,
which is a decision rather than an oversight."

That premise was measured on 2026-08-17 over the archived version 2 artifacts and it is wrong
by two orders of magnitude:

```text
Aniloviraw    artifact 271,360 B    whole English document 1,668 B    cost 0.6%
zheermao101   artifact 659,456 B    whole English document 5,779 B    cost 0.9%
```

Under one percent,
not one hundred.
An artifact is not mostly text.
Judge exchanges,
findings,
ledger rows and the derived comparison dominate it,
and `zheermao101` is 644 KiB of them against an English document under 6 KiB.

With cost removed,
nothing is traded off,
and a choice with nothing traded off is not a
question for the owner.

## What this replaces

STORING A HASH PLUS THE CORPUS COMMIT,
which was the third option `#96` named.
It saves under a percent of a file and charges every future reader a checkout pinned to the
right commit.
That is exactly the dependency the generation-identity work exists to remove,
and `probe-store.ts` says so in as many words.
Buying back 0.9 percent by reintroducing it is a bad trade in the direction the codebase has
already decided to move.

STORING NOTHING,
the status quo,
fails the same way and harder.

## What the evidence already supports

`#115` found the rendering audit reads `sourceText` and `incumbentText` straight out of the
delivery ledger and never re-slices,
deliberately,
because auditing re-sliced text would audit a different input than the judges saw.
The artifact is already the unit of record for what was judged;
this makes it the unit of record for what the archive said.

Reconstruction from the corpus does work,
and that was measured too:
all four artifacts verify against a preparation recomputed at `HEAD`,
with slice source text character-identical.
So the rejected option is viable.
It is simply not cheaper in any way that matters.

## What argues the other way, kept rather than buried

AN ENTRY WITH FEW SLICES AND A LONG UNTOUCHED DOCUMENT would shift the ratio.
No such entry exists in this corpus,
so that is unmeasured rather than dismissed.
If one appears,
re-measure before extending this beyond the pinned corpus.

STORING MORE LICENSED MATERIAL in a file copied between machines is a policy question rather
than a size one.
It is a weak objection here only because the artifact already carries the same material
per slice,
so the whole-document field adds no category of content that is not already there.

## What is still owed

Writing the field is `settleEntry`,
on the producing path.
It lands after the running verification and alongside the other producing-path work,
not beside a pass in flight.
Tracked as its own item rather than as part of this decision.
