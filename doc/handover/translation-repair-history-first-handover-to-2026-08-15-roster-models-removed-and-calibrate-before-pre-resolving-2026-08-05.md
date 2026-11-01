# Translation repair history, first handover document: the roster models removed on 2026-08-05 and the instruction to calibrate before pre-resolving grading items

The two roster models that stopped existing,
the alias trap that left six distinct models,
and the user instruction to pre-resolve unambiguous grading items only after measuring agreement against the 100 items already graded.
The alias trap is also recorded in `translation-repair-history-2026-08-05-to-07-roster-repair-quorum-rule-and-concurrency.md`.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The roster models removed on 2026-08-05 and the instruction to calibrate before pre-resolving

- SYNTHETIC ROSTER CHANGED UNDER US (measured 2026-08-05 against the live
  `GET https://api.synthetic.new/openai/v1/models`,
  not from memory).
  TWO OF THE SEVEN ROSTER MODELS ARE GONE:
  `hf:moonshotai/Kimi-K2.7-Code` and
  `hf:MiniMaxAI/MiniMax-M3` both return HTTP 404
  `"... is no longer supported. Try using a different model, like
  hf:zai-org/GLM-5.2"`.
  Both were live during run 017,
  so this landed in the
  days since.
  404 is NOT in `transient-retry.ts`'s retry set (408,
  429,
  500,
  502,
  503,
  504),
  so each is a non-transient throw:
  the stage loses that voice immediately with
  no retry and the run continues DEGRADED.
  Unchanged,
  every stage would now run
  at 5 of 7 voices,
  permanently and quietly.
  THE ALIAS TRAP,
  which is the part worth reading twice:
  the endpoint lists 10
  ids but only SIX are distinct models.
  `syn:large:text` is GLM-5.2,
  `syn:large:vision` is Kimi-K3,
  `syn:small:text` is GLM-4.7-Flash,
  and
  `syn:small:vision` is Qwen3.6-27B,
  each confirmed by the `hugging_face_id`
  field.
  Restoring a 7-voice panel by adding a `syn:` alias would put the SAME
  model on the panel twice,
  and the voting stages would count one model's
  opinion as two independent confirmations.
  Never select roster members by id
  alone;
  dedupe on `hugging_face_id`.
  Distinct models now available,
  with context lengths:
  GLM-5.2 (524288),
  Kimi-K3 (524288,
  NEW,
  text+image),
  Qwen3.6-27B (262144,
  text+image),
  Nemotron-3-Super-120B (262144),
  GLM-4.7-Flash (196608),
  gpt-oss-120b (131072).
  So the panel can hold at most SIX independent voices,
  down from seven,
  unless
  Kimi-K3 counts as the replacement for Kimi-K2.7-Code and MiniMax-M3 goes
  unreplaced.
  Quorum thresholds and cross-round comparability both depend on
  this;
  decide it explicitly rather than letting the roster silently shrink.
- NEW USER INSTRUCTION (2026-08-05):
  before handing over a grading sheet,
  pre-resolve the unambiguous Y/N items and hand over only genuinely contested
  ones.
  This revives task 31 (judge crosscheck) as the mechanism.
  MUST be calibrated before it is trusted:
  if the agent resolves items,
  the
  reported precision partly reflects the agent's judgement rather than the
  user's,
  which is the measurement the gate exists to protect.
  The user has now
  graded 100 items across two rounds,
  which is exactly the calibration set.
  Measure agreement against those 100 BEFORE pre-resolving anything,
  and report
  the agreement rate alongside the next sheet.
