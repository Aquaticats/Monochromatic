import type { CompletionCapPool, } from './model-card.ts';
import { MODEL_CARDS, } from './model-cards.ts';
import { recordOver, } from './model-card-derive.ts';
import {
  ROSTER_MODEL_IDS,
  type RosterModelId,
} from './roster-id.ts';

//region Completion cap
// EVERY CALL ON EVERY PROVIDER CARRIES `max_tokens` SINCE 2026-09-09. The
// per-token provider got it first that morning, at a table keyed by its own
// served ids: between the top-up of 2026-09-08 11:27 UTC and the payment
// refusal of 2026-09-09 12:22 UTC the meter spent 199.92 USD, of which the
// `SPEND` lines accounted for 141.62, and the rest was streams the rounds had
// abandoned 120 s after quorum that the endpoints kept generating and billing
// to their own end. The same afternoon the owner said not to ignore that
// whatever is sent to Hyper and Synthetic more efficiently makes those last
// longer, so fewer calls fall back to the per-token provider at all: Synthetic
// meters a weekly token allowance (`synthetic-quota.ts`), Hyper a daily
// limit, Bedrock a credit the owner will never top up, and a runaway reply
// spends every one of them. So the table is keyed by roster id and every
// client sends it.
//
// MEASURED, NOT CHOSEN. Over every `SPEND` line in the pass logs as of
// 2026-09-09 16:20 UTC (142,437 completed calls across the four providers;
// `package/translation-repair/doc/status.md` records the figures, and ledger P10 re-reads them), each cap is the highest 99th percentile of `completion_tokens`
// that any provider with at least 100 calls of the model recorded, so a
// model is never cut on one provider at a length another provider's
// tokenizer counts higher; floored at the pooled 90th percentile (3,831) so a
// seat whose replies are mostly short verdicts is never cut on the long
// answer a writer seat asks of it. Every cap cuts under one percent of that
// model's completed calls; `minimax-m3` the most, 274 of 34,018, since its
// Hyper replies ran to the 32,000 ceiling that provider already enforced.
//
// NOT A THINKING PARAMETER. The owner's standing instruction of 2026-08-25
// ("don't set any thinking parameter or budget tokens") is about reasoning
// controls, and since 2026-10-22 it holds for third tier providers alone
// ("I'm only against thinking params on 3rd tier providers"); `max_tokens` is
// the ordinary output ceiling every endpoint takes, whatever its tier, and the owner's instruction of 2026-09-09,
// "do everything in our power to NOT bleed", is what puts it on. A reply
// that meets the cap ends `finish_reason=length`, which
// `chat-json-outcome.ts` already reads as a truncated voice. Hyper keeps its
// own per-model ceiling (`answerCeilingFor`) and takes the lower of the two.
//
// A MODEL THE MEASUREMENT HOLDS NO COMPLETED CALL FOR takes the pooled 99th
// percentile (13,082) until its own calls are read, as Mercury 2.5 did
// between its seating and 20:05 UTC on 2026-09-09. THE EIGHT CLAUDE CARDS ARE THE ONE EXCEPTION: thinking counts toward
// `max_tokens` on both Claude models, so they carry a larger starting cap of their own (`claude-card-sides.ts`).
//
// RE-READ ON 2026-09-28 (ledger P10) over the pass-run logs alone, 499,820
// completed calls; the caps and the pooled numbers stand. Every call since the
// caps went on the wire carries its cap, so a re-read can confirm a cap or
// lower it and never shows a longer answer. The pool it gives (p90 2,607, p99
// 10,822) is mostly such capped calls, nearly a third of them from the two
// Bedrock Gemma seats (147,775 calls, p50 near 75), and lowering the floor to
// it would cut writer answers for nothing. Seven seat and provider pairs now
// run to their cap on more than one percent of calls since then, against
// "under one percent" in the table: `hf:zai-org/GLM-5.3-Flash` on Synthetic
// 4.85 (97 of its 100 with no content), `minimax-m3` on Hyper 4.06 (157 of
// 162), `google.gemma-4-31b` on Bedrock 2.34 (all 5 with content, over 214
// calls), `deepseek-v4.1-flash` on Hyper 1.85 (74 of 75 with none; read on the
// pre-1017 Hyper id, which the seat left on 2026-10-20),
// Qwen3.8-27B on Synthetic 1.75 (185 of 197), `glm-5.3` on Hyper 1.46 (23 of
// 25), and GLM-5.3-Flash on OpenRouter 1.15 (233 of its 239 with no stream
// line to pair). Where a cut call pairs with its stream, nearly every one
// streamed no content: a reasoning runaway, which is what the cap is for. A
// longer cap buys a longer runaway, and the recovery round re-asks them; the
// Gemma 31B cuts are the exception, five answers over a thin sample.

/**
 Pooled 90th percentile of completion tokens over every completed call in
 the measurement, the floor under a model's own cap. Exported for the census
 that re-reads the rule over later runs (`corpus-run/cap-census.ts`).

 @example
 ```ts
 const floored = Math.max(p99, POOLED_P90,);
 ```
 */
export const POOLED_P90 = 3_831;

/**
 Fewest completed calls a provider must hold of a model before its 99th
 percentile counts toward that model's cap, as the 2026-09-09 table read it.

 @example
 ```ts
 const counts = calls >= MIN_PROVIDER_CALLS;
 ```
 */
export const MIN_PROVIDER_CALLS = 100;

/**
 Existing pooled 99th percentile for a new model without its own completed-call distribution.
 Measured over 142437 completed samples as of 2026-09-09, p99 13082 (`package/translation-repair/doc/status.md` records the sample count, and ledger P10 re-reads the figure).
 */
const POOLED_P99 = 13_082;

/**
 Pooled percentile a card names in place of a measured cap.
 */
const POOLED: Readonly<Record<CompletionCapPool, number>> = {
  'pooled-p90': POOLED_P90,
  'pooled-p99': POOLED_P99,
};

/**
 Completion token ceiling per roster model, sent as `max_tokens` by every
 client, read off the cards.

 @example
 ```ts
 const cap = COMPLETION_CAP['deepseek-v4.1-flash'];
 ```
 */
export const COMPLETION_CAP: Readonly<Record<RosterModelId, number>> = recordOver({
  keys: ROSTER_MODEL_IDS,
  of: function capOf(modelId,): number {
    /**
     Cap the card carries: a measured number or a pooled percentile's name.
     */
    const { completionCap, } = MODEL_CARDS[modelId];
    return ((typeof completionCap) === 'number') ? completionCap : POOLED[completionCap];
  },
},);

/**
 Ceiling one call carries: the measured cap, or a caller's own when lower.

 @param modelId - roster model the call is for

 @param requested - caller's ceiling, which only ever lowers the cap

 @returns Value for the request body's `max_tokens`

 @example
 ```ts
 const maxTokens = completionCapFor({ modelId: 'minimax-m3', requested: 500, },);
 ```
 */
export function completionCapFor(
  {
    modelId,
    requested,
  }: {
    readonly modelId: RosterModelId;
    readonly requested?: number;
  },
): number {
  /**
   Measured ceiling for this model.
   */
  const cap = COMPLETION_CAP[modelId];
  if (requested === undefined)
    return cap;
  return Math.min(
    requested,
    cap,
  );
}

//endregion Completion cap
