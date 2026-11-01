# Translation repair history 2026-08-05 to 2026-08-07: house policy, and its conflict with the recall benchmark

The corpus's editorial rules reaching the models,
and how they contradict a benchmark that counts every omission as a defect.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## House policy reaches the models

- HOUSE POLICY NOW REACHES THE MODELS (`house-policy.ts`,
  spliced into the critic
  and adjudicator prompts).
  The corpus documents its editorial rules in
  `CODE_OF_CONDUCT.md` (编写原则) in the one-among-us/data repo,
  NOT in
  `CONTRIBUTING.md`,
  and no stage had ever been told them.
  The consequential rule:
  when a death was by suicide the method is deliberately
  vague,
  and drug names and dosages are deliberately absent.
  A page obeying that
  rule looks like `accuracy/omission` to an uninformed critic,
  and the editor
  then RESTORES the detail the rule exists to remove,
  so acting on the finding
  makes the shipped translation violate the corpus's reader-protection policy.
  That is worse than a false positive.
  Also carried:
  third person,
  neutral pronouns preserved rather than resolved to
  he/she,
  and a memorial tone rule rejecting both overwrought and clinical
  writing.
  `house-policy.ts` PARAPHRASES rather than copies,
  because the corpus
  repo is unlicensed;
  never paste from it.
  Open interaction:
  the recall benchmark treats every omission as a defect,
  so
  policy-protected omissions and seeded omissions need reconciling.

## House policy and the recall benchmark contradict each other

- HOUSE POLICY AND THE RECALL BENCHMARK CONTRADICT EACH OTHER,
  and the
  contradiction is structural rather than a bug in either (task 49,
  still OPEN
  because it is the user's call).
  `HOUSE_POLICY_BLOCK` tells critics and the panel,
  verbatim:
  "Never report that
  as an omission,
  and never restore the detail,
  even when the ORIGINAL states it
  plainly",
  for suicide method and for drug names and dosages.
  `deriveOmissionSeeds` plants seeds by DELETING the longest sentences from the
  clean English translation,
  and the benchmark scores whether the pipeline
  restored them.
  When a deleted sentence's Chinese counterpart carries protected
  content,
  the policy instructs the pipeline not to restore it and the benchmark
  records a recall MISS.
  Correct behavior scores as failure.
  The ground truths genuinely differ,
  and neither is wrong.
  The benchmark's is
  "this sentence was in the published English,
  so it belongs",
  which is sound
  because the community wrote that English under its own rules.
  The critic
  cannot see that:
  it sees only the mutilated English and the Chinese,
  so
  Chinese-only sensitive detail reads to it as a deliberate omission.
  MAGNITUDE IS UNMEASURED.
  Establishing it means reading corpus content to
  classify 27 seeded sentences,
  which is possible (the clone is readable) but
  has not been done,
  so do not describe the effect as small or large.
  RUN 001 IS UNAFFECTED,
  verified rather than assumed:
  the run's own START line
  records `tip=2cf7fd453bb3a20b889b9c01d5640dd7fe81e858`,
  committed 23:19:51,
  and the process began 23:20:01,
  while `house-policy.ts` and its splice into
  the critic prompt landed in `5daf7b853` at 23:42:53,
  23 minutes later.
  The
  process resolved its module graph at startup and has no runtime dynamic
  imports,
  so run 001 measures a policy-free pipeline.
  That makes run 001 a clean POLICY-FREE RECALL BASELINE,
  and it also means
  round-three recall will not be comparable to it on this axis.
  RESOLVED by user decision 2026-08-06 ("I'll go with your recommendation"):
  ATTRIBUTE rather than exclude,
  shipped in `363d4649e`.
  `gradeSeedDetection` returns a `SeedDetectionVerdict` instead of a boolean:
  `accepted`,
  `declined-protective` (the panel landed `source-defect` at the
  seed's region),
  `declined-other`,
  or `undetected`.
  The scorecard reports
  `policyDeclinedSeeds` and `seedDetectionRateExcludingPolicy` BESIDE the raw
  `seedDetectionRate` rather than replacing it,
  because both numbers are true
  and a verdict has to say which it cites.
  Rejected alternatives,
  with reasons:
  excluding protected sentences from
  seeding needs a classifier over suicide and medication topics whose misfires
  are their own harm;
  running the benchmark with the policy disabled would
  measure a pipeline that is not the one shipping.
  Attribution also costs no
  extra model calls.
