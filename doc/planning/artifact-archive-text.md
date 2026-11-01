# Should the settled artifact store the archive text

Task `#96`,
item 7,
the last thing left in it.
This is a recommendation rather than a decision:
the task records it as the user's call,
so the measurement is here and the choice is not made.

## The premise the question rests on is wrong

`#96` says,
verbatim:

> `targetText` is measured and not stored,
> so comparing the archive against either lane's output after the fact is impossible.
> Storing it doubles artifact size,
> which is a decision rather than an oversight.

The doubling is what made it a decision.
It was never measured.
Measured now,
over all four archived version 2 artifacts:

```text
                artifact        whole target text     cost of storing it
Aniloviraw      271,360 B                 1,668 B                  0.6%
zheermao101     659,456 B                 5,779 B                  0.9%
```

Under one percent,
not one hundred.

The reason is that an artifact is not mostly text.
It carries every judge exchange,
every finding,
every per-slice ledger row
and the derived comparison,
and those dominate it by two orders of magnitude over the document itself.
`zheermao101` is 15 slices and 644 KiB;
the English document it was settled against is under 6 KiB.

So the cost argument that made this a decision does not exist.

## What that leaves

The three options `#96` names,
re-read against the measurement:

-   PER-SLICE `incumbentText` IS ALREADY THERE and is already enough for
    every analysis run so far.
    `#115`'s rendering audit reads `sourceText` and `incumbentText` straight out
    of the delivery ledger and never re-slices,
    deliberately:
    auditing re-sliced text would audit a different input than the judges saw.
-   STORE `targetText` WHOLE.
    Costs under one percent,
    which is the finding above.
-   STORE A HASH plus the corpus commit,
    and accept that reconstruction needs the checkout.
    `#96` calls this the cheapest and says it is "the one to argue against rather than for".

The measurement removes the only argument for the third option over the second.
The third is cheaper by under a percent of a file,
and it buys that by making every future reader need a corpus checkout
pinned to the right commit.

## The argument for storing it whole

SELF-SUFFICIENCY IS SOMETHING THIS PROJECT HAS REPEATEDLY PAID FOR.
The whole generation-identity effort exists so an artifact answers for itself.
`probe-store.ts` says it in as many words:
a result is worth keeping only if a later reader can say what produced it,
so a file found on disk months later answers for itself
"instead of needing the transcript that this module exists to stop depending on".
A hash plus a commit reintroduces exactly that dependency,
one layer out.

NO NEW CLASS OF CONTENT IS ADDED.
The artifact already stores `sourceText` and `incumbentText` per slice,
so corpus text is already on disk in it.
Storing the whole target adds the unsliced regions and nothing else in kind.
This is unlike the audit RUN files,
where `#115` chose digests over text:
those are read,
grepped and quoted into documents,
whereas an artifact is a record nobody pastes.

RECONSTRUCTION FROM THE CORPUS DOES WORK,
and that is measured rather than assumed:
`#115` verified all four artifacts against a preparation recomputed at HEAD,
and slice source text at HEAD is character-identical to what they carry.
So the third option is viable.
It is simply not cheaper in any way that matters.

## What would argue against it

Two things,
and both are worth stating because neither is refuted here.

A LARGER CORPUS CHANGES THE RATIO,
but in the direction that helps.
Artifact size grows with slices and judge exchanges;
document text grows with the document.
An entry with few slices and a long untouched document would be the case where
storing it costs a larger share.
Nothing like that exists in this corpus,
and the four measured entries are the only version 2 artifacts there are,
so this is unmeasured rather than dismissed.

THE LICENCE QUESTION IS UNCHANGED BUT WORTH RE-ASKING DELIBERATELY.
Storing more licensed material in a file that gets copied between machines
is a policy question and not a size question,
and it is the only remaining reason to prefer a hash.
The answer today is that the artifact already carries the same material per slice.

## Recommendation

Store `targetText` whole,
beside the preparation identity that already covers it by hash.
The hash stays:
it is what proves the stored text is the text the run measured,
and storing text without it would be a record nobody could check.

## Implementable now, and still undecided

The freeze that blocked this has lifted and returned.
The 92-entry pass stopped at six entries on
2026-08-17,
and a second,
targeted pass over five flagged entries started at 01:29Z and holds the
producing path again until it finishes.

So the obstacle is no longer capability,
it is that this is the user's call and they have not made
it.
`settleEntry` is the producing path,
and writing a new artifact field means rebuilding,
which a
running pass makes unresumable.
Between passes it is a small change.

WHAT IS STILL WAITING ON A PERSON:
the licence question.
Storing more licensed material in a file
that gets copied between machines is a policy question rather than a size question,
and it is the
only remaining argument for a hash over the text.
Everything measurable has been measured:
the
premise that storing it doubles artifact size is false,
at 0.6 to 0.9 percent,
and reconstruction
from the corpus was verified against a preparation recomputed at HEAD.
