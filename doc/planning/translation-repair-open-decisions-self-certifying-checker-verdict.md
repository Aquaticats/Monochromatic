# Translation repair open decisions: what a self-certifying checker's verdict is worth (question 4)

Question 4 on the weight of a checker's verdict about text it helped write,
with the three weighting options and the ranking.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

## Question 4: what a self-certifying checker's verdict is worth

BLOCKS `#91` alongside Question 1.

You ruled that checkers may certify text they helped write,
at lower weight.
The selection discount is a half,
and that number rests on an argument that
does NOT transfer:
in selection a winner needs weight 2,
so half-weight
self-votes cannot carry a candidate.
Resolution checking tallies verdicts about
one claim rather than ranking candidates,
so nothing in the arithmetic picks a
number.

### Options

A.
 A half,
matching the selection discount.
    Pros:
one number to explain,
one to tune;
visibly consistent.
    Cons:
consistency is the only argument for it.

B.
 Zero weight,
meaning a self-certifier's verdict is recorded but not counted.
    Pros:
keeps the record while never letting a model certify itself;
closest
    to the old exclusion without refusing the roster.
    Cons:
with every model producing,
a claim could end up with no counted
    verdict at all,
which reads as unproven rather than as unchecked.

C.
 Weighted by measured agreement:
a checker's verdict on its own work counts
    at the rate its verdicts agree with disinterested checkers elsewhere.
    Pros:
the only option grounded in evidence.
    Cons:
needs a measurement nobody has taken,
and it cannot be taken until
    the new shape has run.

RANKING:
A > B > C.

A over B because B's failure mode is silent:
a claim with no counted verdict
looks identical to one nobody could prove,
and this pipeline has been bitten by
that shape repeatedly.
B over C because C is right and unavailable:
it needs a corpus run under the new
shape to produce the agreement rates it weighs by,
so A or B has to hold the
seat until then anyway.

A
