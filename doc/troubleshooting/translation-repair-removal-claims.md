# Removal claims in the introduced-defect probe

Why `removalCorroborated` fell from 159 to 0,
what that turned out to mean,
and the correction to commit `ce130535d`'s message.

## The measurement that started it

Census across every run directory,
counting screened probe claims:

```text
runs         56 entries  2755 regions  corrob 354  REMOVAL 159  unanch 28  preExist  0
runs-pass10   3 entries   105 regions  corrob   3  REMOVAL   0  unanch  0  preExist  0
runs-pass12   1 entry      75 regions  corrob   0  REMOVAL   0  unanch  0  preExist  3
runs-pass13  22 entries   915 regions  corrob  84  REMOVAL   0  unanch 34  preExist 21
```

A count landing exactly on zero after a refactor is the shape this package has been bitten by before,
so it was treated as a suspected regression.

## What the zero actually means

Splitting every claim by whether it anchored on dropped wording:

```text
runs     removal claims: 159 corroborated,  25 contradicted,  1 unanchored
pass13   removal claims:   0 corroborated,   0 contradicted, 24 unanchored, 21 pre-existing
```

So probers still raise removal claims,
45 of them in pass13,
and none survives.
Both exits turned out to be correct:

-   The 24 unanchored claims quote wording absent from their region.
    Every one of those quotes is present in the entry's final `repairedText`,
    so the content was never dropped and the claim is false.
    `screenEvidence` refused them correctly.
-   The 21 pre-existing claims all quote wording that sits INSIDE a prior critic quote.
    That is a licensed removal:
    deleting the phrase the critic objected to is what the repair was for.

The likeliest explanation for the fall from 159 is therefore that the editor stopped dropping content
between 2026-08-07 and 2026-08-13,
which makes the zero a good result rather than blindness.

## The latent defect that was fixed anyway

`restatesPriorIssue` discounted a claim restating an accepted issue,
testing containment both ways.
That is right for added wording and wrong for removals.

A removal claim quotes what disappeared,
drawn from the before text,
which is the side the critic quoted,
on a region that exists precisely because the critic quoted something in it.
Containment is therefore close to guaranteed and its direction carries the whole signal:

-   Dropped wording INSIDE the prior quote was licensed to go.
-   Dropped wording CONTAINING the prior quote means the edit took the objected-to phrase
    and unrelated content with it,
    which is new damage nobody asked for.

The old rule read the second as a restatement,
so an over-deletion was structurally unreportable:
to report one a prober must quote the deleted span,
and that span contains the phrase the critic complained about,
which is why the editor was there.

## Correction to `ce130535d`

That commit's message credits the 159 to 0 fall to this reclassification.
That claim is retracted.
Applying the new rule to the 21 reclassified claims in pass13 recovers ZERO of them:
all 21 are licensed removals and stay suppressed.
No genuine over-deletion was being suppressed in the measured data.

What stands:
the old rule would suppress a real over-deletion,
proven on a synthetic case and by removing the guard and watching only that test fail.
It is a latent defect worth closing.
Only the claim that it explained the measured drop was wrong.

The per-region corroborated rates quoted in that message,
0.128 then 0.092,
describe added-wording claims and are unaffected.

## Method note

Three harness faults nearly produced false findings here,
each caught by a control rather than by inspection:

-   A fixture keyed on `id` where `collectPriorQuotes` reads `issueId`,
    so the issues argument went unread and four rows came back identical.
    Fixed by adding a control claim that must reclassify.
-   A claim-to-region join that silently dropped 20 of 84 claims.
    Caught by checking that corroborated claims match their region's after text,
    which they must by construction.
-   A haystack built from every string in the artifact,
    including other claims' own quote fields,
    so a duplicate claim would have vouched for itself.
    Rebuilt from document text only.

An unchanged result proves nothing until the probe is shown able to move.
