# Translation repair history 2026-08-05 to 2026-08-07: round two graded, and the terms set for round three

Round two's precision gate failed and why,
and the grading process and attribution warning agreed for round three.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Session 2026-08-05/06: round two graded, roster repaired, branch rebased

### Round two graded: the precision gate failed, and why

- ROUND TWO FAILED at 0.740 / 0.787 / 0.800 (strict / partials excluded /
  ceiling) against the 0.9 bar:
  37 clear Y,
  10 clear N,
  3 ungradable of 50.
  Round one was 0.56 / 0.64 / 0.68,
  so fixes A-F moved precision a long way and
  halved clear false positives (16 -> 10),
  and the gate still is not close.
  Band-balanced 0.788 and pool-weighted 0.794 agree within a point,
  so the
  weighting question raised before the draw did not need the user's decision.
  Full analysis:
  `node_modules/.monochromatic/translation-repair-runs/gate-verdict-round-two.md`
  (outside git;
  quotes UNLICENSED corpus).
- ROOT CAUSES of the 10 false positives:
  literalism fighting fluency (3:
  poetry judged as prose,
  总是 forced to "always",
  a fluency-serving conjunction
  counted as an addition),
  anchoring/alignment (3),
  context insufficiency at
  judgement (2),
  a doubted domain fact (1),
  token degeneration (1).
- PRECISION CANNOT SEE REPAIR QUALITY,
  which the grading exposed and the metric
  hides:
  4 of the 37 TRUE positives carry notes saying detection was right and
  the repair was poor ("is there a better way?").
  Those score as successes.
  Milestone four needs a repair-quality metric separate from detection
  precision;
  task 47.
- FIX H LANDED:
  the critic prompt carried non-literal translation policy and the
  adjudication prompt carried NONE of it (3 matches versus 0 for
  literal/synonym/poetry/fluency/natural).
  The panel's quoted-evidence check
  cannot catch that class,
  because such claims are accurate about the text and
  wrong about what counts as a defect.
  Directly targets FPs 26,
  37, 42.

## Grading process and attribution for round three

- GRADING PROCESS CHANGES for the next round (user instruction):
  pre-resolve the
  unambiguous Y/N items and hand over only genuinely contested ones.
  This CANNOT be honestly calibrated against the existing 100 graded items,
  because the agent has read all of them including the rationale,
  and round one
  came from a pre-fix pipeline.
  Plan:
  pre-grade round three BLIND,
  hand over
  every item with the agent's grade marked,
  let the user correct,
  and derive the
  agreement rate from that round.
  Only filter on the round after.
  Revives
  task 31.
  Say plainly that the instruction takes effect one round later than it
  sounds.
- ATTRIBUTION WARNING for round three,
  accepted by the user ("Bundle all the
  improvements that could be made,
  in"):
  the roster,
  the editor,
  the checker
  set,
  the quorum rule,
  the adjudication policy,
  and the house policy all
  changed at once,
  and the naturalness lane is still to come.
  A precision delta
  will not be attributable to any single change.
  Say so in the verdict rather
  than implying otherwise.
