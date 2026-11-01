# Translation repair history 2026-08-05 to 2026-08-07: the roster repaired, the quorum rule, and concurrency

The synthetic roster repaired after two models were withdrawn,
the quorum rule changed to half the roster,
and per-model concurrency measured.
The alias trap is also recorded in `translation-repair-history-first-handover-to-2026-08-15-roster-models-removed-and-calibrate-before-pre-resolving-2026-08-05.md`,
in the first handover group.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The synthetic roster repaired, the quorum rule, and per-model concurrency

- SYNTHETIC ROSTER REPAIRED.
  `Kimi-K2.7-Code` and `MiniMax-M3` now answer HTTP
  404 "no longer supported",
  and 404 is not in the transient retry set,
  so every
  stage was about to lose two of seven voices silently with no retry.
  THE ALIAS TRAP:
  the models endpoint lists ten ids but only SIX are distinct.
  `syn:large:text` is GLM-5.2,
  `syn:large:vision` is Kimi-K3,
  `syn:small:text`
  is GLM-4.7-Flash,
  `syn:small:vision` is Qwen3.6-27B,
  each stated by the
  endpoint's own `hugging_face_id`.
  Restoring a seventh voice with an alias
  would seat one model twice on a voting panel and count one opinion as two
  confirmations.
  Dedupe roster edits on `hugging_face_id`,
  never on id.
  Kimi-K3 now EDITS (user:
  much stronger than anything else offered),
  and
  checkers exclude the editor,
  ending the old GLM-5.2 self-check.
- QUORUM is now `voices >= ceil(roster / 2)` (user decision).
  The old "strictly
  more than half" demanded more than a majority on EVEN rosters:
  at six it
  wanted 4,
  so a stage sitting at exactly 3 burned every retry round.
  Odd
  rosters unchanged.
  `minBallotWeight` stays the absolute 3,
  so the share of the
  panel needed for any decision rose from 3-of-7 to 3-of-6;
  user accepted that
  explicitly ("50% is okay here").
- PER-MODEL CONCURRENCY MEASURED,
  and it does not help:
  throughput is flat at
  0.32 to 0.42 req/s across concurrency 1,
  2,
  4,
  and 8,
  while wall time scales
  nearly linearly (3.1 s at n=1 to 19.0 s at n=8,
  against 24.8 s if perfectly
  serial),
  with zero 429s.
  The provider QUEUES per model rather than throttling,
  so `perModelConcurrency: 1` is correct and the available parallelism is across
  models,
  which the pipeline already uses.
