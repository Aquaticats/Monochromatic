# cli-git sibling shadow control: its guard costs a second per gate, and stays

## Status

Answered in the question tool on 2026-11-23,
recorded 2026-11-23T17:47:21-05:00.
It confirms the option the coordinating session had adopted without asking,
on the integration pass of issue 6344's fix
(`~/temp/agent/integration-pass-shadow-sibling-control-two-timing-windows-2026-11-22/keep/review.md`,
"Decision 3:
committed guards for the first-attempt-only plant and the cleared plant (L1,
L2)").
It is built on `fix/git-policy-cli-shadow-sibling-control-two-timing-windows` at `48346ef6c2`.

## What was asked

Issue 6344 is a test of the wrapper's shadow repositories that failed once in a gate for no product fault.
Its fix closed two timing windows,
and its Sonnet pass found the fix of the second window held only by a check someone has to run by hand.
The coordinating session had adopted the pass's first-ranked option:
commit a one-second delay inside the test's stand-in Git,
so the regular test suite catches a regression.
Measured since:
the test takes 1.87 to 1.95 s with the delay against 0.91 to 0.95 s without,
so about one second more per whole gate run of roughly 17 to 29 minutes.

## What was chosen

The project chose "1:
keep the 1 s guard (Recommended)":
"Commit the one-second delay on the sibling test's second start,
plus the free leak test.
Pros:
both fixes are held by the suite;
a regression turns a test red in every gate.
Cons:
about one second more per gate run."

Not chosen:
"2:
leak test only",
which drops the delay;
and "3:
neither".
