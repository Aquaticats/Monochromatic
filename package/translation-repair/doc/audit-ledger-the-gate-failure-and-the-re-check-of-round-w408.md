# Audit ledger: the gate failure and the re-check of round w408

Round w409 (2026-11-01 UTC) answers the one case that gate g406 failed
and the findings of the cross-vendor reviews of rounds w407 and w408
on the integration tree `tr-i406` (detached at `dffa3ac22f`,
the head of round w408).
The owner wants every mistake fixed:
"no mistake is too small or low impact or ambiguious to fix",
and recorded in the package docs so it does not recur.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
The rules the entries leave behind are in
[Preventing mistakes: tasks, builds and bulk output][tasks-rules]
and [Preventing mistakes: test cases that hold a seam or a count][seam-rules].

Ids B1370 to B1383 are used in this file only.
The logs are under `/var/home/user/temp/agent/tr-fan/w409-fix/`.
The owner's words in this file are those the brief of the round gave:
"Honour holds (Recommended)",
"Keep exempt (Recommended)",
"no matter what,
we must ship"
and "Count self-votes (Recommended)".

## B1370: the repair lane asks its hook for each slice twice, and the case counted one

The failure:
gate g406 at `1dd1e1af3f` failed one case of `translate-lane`,
"ASKS BEFORE EVERY SLICE OF EITHER LANE" in `document-lanes.unit.test.ts`,
with `expected 4 to equal 2` on the count of repair-lane asks
(`gate-g406/translate-lane.test-unit.log`).
The rotation sweep found it red with no shift too,
so it was deterministic and not a matter of order.

The cause:
commit `36b94b6d0f` of round w406 made `runRefinePhase` ask the re-seat hook once for each slice (B1332),
and `repairPreparedDocument` hands the refine phase the same hook the accuracy loop asks.
The lane now asks once before the accuracy settlement of a slice and once before its refinement.
The case was written when the lane asked once,
and `git log e9b966af55..HEAD` on `document-lanes.unit.test.ts` and `document-lanes.ts` lists no commit:
the case was never run after `36b94b6d0f`.

Whether the second ask is a defect or the expectation is out of date:
the second ask is the design of B1332 and not a defect.
Its purpose is that a hold which began between the two phases is honoured by the refinement of the slice,
and the case of `corpus-pass` (`repair-translation.unit.test.ts`) already says "at least once per slice".
The cost of the second ask was read in the production hook (`pass-reseat-hook.ts`):
with nothing held it is one synchronous read of the holds and a return of the seating last read;
under a hold it reads the seats again from the cached budget reading and writes one `JUDGE SEATS` line.
It calls no model,
so it spends no quota.
The expectation was out of date.

The fix:
the case holds the driver to its contract,
which is that it forwards every ask of the repair lane and adds or drops none.
It runs `repairPreparedDocument` alone over the same preparation with the same scripted client and a counting hook,
and asserts that the driver's repair count equals that count and is at least the number of slices.
A first form named the figure two (`REPAIR_ASKS_PER_SLICE`) and was replaced before the next step:
a named literal is the shape that B1377 fixes in another test,
and it would have failed again the day a phase added an ask.
Commits `1572a64b91` (the first form) and `8024b87261` (the derived count).
The green logs are `green-lanes-asks.txt` and `green-lanes-asks-2.txt`.

What let it through:
rounds w406 and w407 ran the `test:unit` of the packages they had changed and of the ones they thought of,
and `translate-lane` depends on `repair`.
Neither round ran its suite.
The rule that closes it is B1378.

[tasks-rules]: mistake-prevention-tasks-builds-and-bulk-output.md
[seam-rules]: mistake-prevention-test-cases-that-hold-a-seam-or-a-count.md
