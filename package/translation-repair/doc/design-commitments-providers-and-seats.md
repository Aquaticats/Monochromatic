# Design commitments: providers and seats

The commitments on which provider serves which seat,
how unreadable and cut replies are re-asked,
what goes on the wire for NeuralWatt picture calls,
and when a picture reader joins a route.

Part of [Design commitments](design-commitments.md),
which lists every file of these commitments.

- **Judge seats follow the provider that would serve them.**
  `readJudgeSeats` reads the router's own dryness view (every provider's meter,
  holds folded in) before each phase of an entry (lanes,
  lane contest,
  consolidation;
  once per entry until 2026-09-03,
  when XIEPT2 ran Synthetic dry seven minutes into 219) and asks,
  per seat,
  which provider would take its calls:
  the first in `PROVIDER_ORDER` that serves the model and reads wet (`providerServing`).
  Where that is Hyper,
  the judges Hyper serves too slowly for the round window are withheld (`HYPER_SLOW_JUDGES`,
  Qwen3.8-27B:
  cut in 30 of 34 translate-lane select rounds with Hyper alone,
  answering 25 of 28 when Synthetic served it),
  and `HYPER_SLOW_SELECT_JUDGES` (Kimi-K3:
  cut in 0 of 69 select rounds on Synthetic,
  43 of 83 and 38 of 101 with Hyper serving most or all of them,
  almost never in any other judge role) leaves both
  lanes' slate select seats and the consolidation slate while keeping every other seat.
  Where that is OpenRouter,
  `OPENROUTER_WITHHELD` (Kimi-K3,
  by the owner's cost decision of 2026-09-03:
  3 and 15 USD per million tokens,
  52 to 61 percent of an entry's all-OpenRouter cost) leaves every seat,
  the translator and picture-reader seats and every roster-wide stage (block pairing,
  archive review,
  insertion admission,
  consolidation writing) included:
  the first OpenRouter-only pass,
  keyword233 on 2026-09-03 at 18:15 UTC,
  bought six translations from it while every judge bench had it out,
  a quarter of that pass's bill,
  and the next one bought its first call from the pairing round before any bench was read.
  Each stage reads its own `JUDGE SEATS` line (`phase=preparation`,
  `pictures`,
  `lanes`,
  `lane contest`,
  `consolidation`),
  and the checker bench is the first three of `RUN_CHECKER_ORDER`
  (`package/translation-repair/run/src/corpus-run/run-config.ts`,
  ranked by the checker benchmark of 2026-09-27)
  that a wet provider serves,
  padded with ranked seats no provider serves only while fewer than three are served,
  so the bench keeps the floor the checker contract holds;
  both checker assertions run on the derived roster before each phase.
  An unreadable view seats the full bench.
  The static benches in `run-config.ts` are the Synthetic-wet ones,
  seven wide seats and eight late judges;
  the `JUDGE SEATS` line names every provider's state,
  every bench size,
  the translator,
  reader and roster counts,
  and every withheld model.
  Decisions:
  `doc/decision/translation-repair-provider-aware-judge-seat.md`,
  `doc/decision/translation-repair-openrouter-fallback.md`.
  Shadow mode is a recorded decision rather than an unfinished edge,
  with the
  rejected gating designs and the condition that reopens it in `doc/decision/introduced-defect-probe-gating.md`.
  Claims are screened deterministically rather than believed:
  a quote must be new in the replacement,
  or gone from it for dropped content,
  and `screenNonTranslationVotes` is the precedent for evidence that dismisses
  an impossible claim without having to prove a possible one.
- **A reply no one could read is re-asked on another provider,
  nudged.**
  Owner,
  2026-09-28,
  asked what becomes of the router's cross-provider re-ask,
  which never ran because the prompt-uniqueness wrapper bypasses it:
  "Enable with the nudge".
  The re-ask carries the recovery nudge,
  so its prompt digest differs from the first ask's
  and prompt uniqueness holds (ledger P9).
  Refined for quality,
  2026-09-28,
  under the owner's standing directive that the quality of the end result decides:
  the re-ask's nudge is worded apart from the stage recovery round's,
  since the round re-asks every seat still unreadable and a shared wording would make its prompt
  this re-ask's digest,
  answered from the claims with the reply that already failed;
  and a refusal-shaped reply is re-asked elsewhere too,
  with a nudge neutral on why the reply could not be used.
- **A reply the length limit cut is re-asked as cut,
  not as off the shape.**
  Decided for quality,
  2026-09-28,
  under the owner's standing directive that the quality of the end result decides (ledger P10):
  the recovery round told every unreadable seat its reply "did not match the required response shape",
  while 74 of the 75 cap-cut deepseek-v4.1-flash replies on Hyper had streamed no content at all.
  Each cause now has its own wording (`package/translation-repair/stage/src/recovery-nudge.ts`);
  the cut one names the length limit and asks for the answer within it,
  keeping any working brief,
  which is prose in the prompt and never a reasoning parameter on the wire.
- **A reasoning parameter goes on the wire for NeuralWatt calls only.**
  Decided by the owner,
  2026-10-21,
  after the pass for `XingZ6016` lost a picture reading to `glm-5.3-flash` spending its whole completion cap reasoning at
  NeuralWatt's default effort (ledger B718 and B719):
  (two sentences of one message,
  the dash between them left out here):
  "NeuralWatt is a special case of all the providers:
  It implements what it documents correctly,
  almost always"
  and
  "I'm lifting the ban on passing params,
  but only for NeuralWatt."
  The owner's instruction of 2026-08-25 (no thinking,
  budget or effort key in any body) holds for every other provider.
  The lift was first built for picture calls only (ledger B719),
  and widened to the text calls of the same seat on 2026-10-21 (ledger B807),
  because the words name the provider and not the kind of call.
  A NeuralWatt card side may name a `pictureReasoningEffort` and a `textReasoningEffort` among the efforts the provider documents for the model
  (`low`,
  `high`,
  `max`),
  and only that card type has the field,
  so no other client can send it.
  `glm-5.3-flash` names `low` for both,
  unmeasured:
  the first keyed run at that effort is the measurement.
  `deepseek-v4.1-flash` names none,
  since the repo records no supported efforts for it.
  `thinking_token_budget` is documented but its support table does not name this model,
  so it is not sent.
- **A picture reader joins a route only after a keyed measurement of that route.**
  Decided by the owner,
  2026-10-21 ("Yes try gemma"),
  on the rule of 2026-09-08 that a reader's transcription is measured against the seated readers before it counts:
  `gemma-4-26b-a4b-it` is listed as reading pictures on OpenRouter (zero data retention on every request),
  held shut by `openrouter-pictures-unmeasured` until `reader-measure` in the bench package reports both clauses met,
  and the bench of readers is counted by the providers that can show a picture (ledger B721 to B723).
  The measurement met both clauses on 2026-10-21 (UTC) and the hold was lifted from that card
  (`roster-changes.md` has the three runs and their limit).
- **Only dryness of every provider ends the run.**
  Ruled by the lead on 2026-10-22 (UTC),
  open to the owner's veto,
  and queued as D4:
  a model whose own providers are all dry while others are wet is a lost seat that the bench reseats,
  not the end of the run.
  The owner's ruling that the every-provider-dry refusal ends the run rests on "this one leaves nothing,
  so no seat's voice can be heard",
  which holds for run-wide dryness only.
  The code still raises the refusal per model until D4 lands.
  The full record is in
  [the rulings of 2026-10-22](design-commitments-rulings-of-2026-10-22.md) (ledger B822).
