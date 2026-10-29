import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { NeuralWattServedId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { PROMPT_BYTES_PER_TOKEN, } from './openrouter-abandoned-spend.ts';

//region NeuralWatt energy estimate
// WHAT A CALL IS EXPECTED TO COST IN ENERGY BEFORE IT IS SENT, for the meter
// that holds it against the day's free allowance.
//
// WHY AN ESTIMATE AT ALL. No endpoint reports what is left of the free daily
// energy, and overage bills the owner's normal plan ("we need to meter (using
// estimations) ourselves because overage eats into my normal plan which I use
// for other stuff", 2026-10-20). A call's true energy is known only when its
// stream ends, in the `: energy` comment (`neuralwatt-comments.ts`). Calls run
// many at a time, so each must hold its expected share of the day the moment
// it is sent, or every call that starts together reads the same remainder and
// the day is spent past its end. The measured figure replaces the estimate at
// settlement (`neuralwatt-ledger.ts`), so this table decides how much is held
// in flight and never how much was spent.
//
// THE FIGURES ARE NEURALWATT'S OWN ENERGY MATRIX, public JSON at
// `https://portal.neuralwatt.com/api/energy-pricing/matrix`, read 2026-10-20
// (`generated_at` 2026-10-20T20:04:27Z, a 7-day window, `Wh_per_request`,
// statistic `mean`, basis `charged`, the standard lane, a minimum of 30
// requests per bucket). The matrix buckets a request by its prompt tokens;
// the bucket edges are the matrix's `min_prompt_tokens` and `max_prompt_tokens`.
//
// TWO BUCKETS ARE THE PIPELINE'S OWN, the 4,096 to 16,384 and the 16,384 to
// 65,536 token buckets, measured over the 1,779 settled calls of the corpus
// pass `XingZ6020` (build `1d46232457`, 2026-10-22T19:32Z to 2026-10-23T06:05Z,
// all in the NeuralWatt day that began 2026-10-22T19:15Z; the calls measured
// 451.711 Wh, the 451.712 Wh of the day's account on the last meter line). Each bucket's mean is
// the measured sum over its calls: 4,096 to 16,384 tokens, DeepSeek V4.1 Flash
// 248.687 Wh over 817 calls (0.3044 Wh) and GLM-5.3-Flash 25.577 Wh over 358
// calls (0.0714 Wh); 16,384 to 65,536 tokens, DeepSeek 174.478 Wh over 539
// calls (0.3237 Wh) and GLM 2.392 Wh over 43 calls (0.0556 Wh). Replayed over
// that run the table holds 451.697 Wh against 451.711 measured, which is a
// check that the means were computed and typed correctly and not evidence the
// next run will match.
//
// THE TWO MODELS MOVED IN OPPOSITE DIRECTIONS, and for different reasons.
// GLM-5.3-Flash text calls run at the `low` reasoning effort since ledger B807
// (2026-10-21), so their reasoning, which the energy follows, is a fraction of
// what the earlier rotation measured at the default `max`: the rotation's
// 0.9577 Wh mean was 13 times the XingZ6020 mean, and the old table held 395.852
// Wh against 28.006 Wh used over the run's 412 GLM calls. The rotation's build
// (`2a336603bf`) does not contain the effort change, so its GLM figures are not
// pooled: they describe a configuration that no longer runs. DeepSeek V4.1
// Flash sends no effort in either, so its calls are comparable, and its
// measured energy sat above the old hold (552 of 1,367 calls, the largest
// 21.27 times the hold). THE HOLD STAYS ON THE SAFE SIDE FOR DEEPSEEK: its
// means are those of XingZ6020 alone, the higher of that run and the call-
// weighted pool with the rotation (4,096 to 16,384 tokens, 0.3044 against
// 0.2603; 16,384 to 65,536 tokens, 0.3237 against 0.3224). A larger hold
// refuses the last calls of a day a little earlier, the cost B808 accepted for
// a hold right on average.
//
// THE LIMITS OF THIS RUN'S MEASUREMENT. It is one entry, so the per-entry
// spread seen in the rotation (0.0936 to 0.2347 Wh across entries for
// DeepSeek) applies to it as well. The GLM means describe text at `low`: the
// one GLM picture read of the run (0.0221 Wh; the provider reported 8,188 prompt
// tokens and the energy line held it for 8,264) fell in the 4,096 to 16,384
// token bucket too and is held at the same mean, a single call that says
// nothing of a picture's cost. Were the `low` effort lifted,
// the GLM hold would be too small, by about 13 and 22 times in the two main
// buckets, until a run measured it again, and the settlement would correct
// each call. The other buckets keep the matrix's figures: this run held at
// most 9 calls of either model in each of them (DeepSeek 3 and 8 calls from
// 256 to 4,096 tokens, GLM 2 and 9), too few for a mean (the matrix's own
// minimum is 30), and their measured means sit under the matrix's except
// DeepSeek's from 1,024 to 4,096 tokens (0.0665 Wh over 8 calls against
// 0.0233), which is too few calls to move.
// Every settled call prints its measured figure beside its estimate
// (`neuralwatt-settlement.ts`), so a later run can replace them the same way.
//
// WHAT THE TABLE HELD BEFORE 2026-10-23 (ledger B808), kept for the record.
// The two buckets held the means of the three-run rotation `zheermao22`,
// `CuspariaKLSY22` and `XingZ6017` (2026-10-21T21:05Z to 2026-10-22T03:34Z, all
// in the NeuralWatt day that began 2026-10-21T19:15Z; the 1,297 settled calls
// measured 555.188 Wh, the day's account on the last meter line of
// `XingZ6017`): 4,096 to 16,384 tokens, DeepSeek 109.253 Wh over 558 calls
// (0.1958 Wh) and GLM 285.402 Wh over 298 calls (0.9577 Wh); 16,384 to 65,536
// tokens, DeepSeek 123.392 Wh over 385 calls (0.3205 Wh) and GLM 36.716 Wh over
// 30 calls (1.2239 Wh). Before 2026-10-22 they held the means of run
// `zheermao19` (build `5b5dca49a9`, 2026-10-20) and the matrix's 0.1395 and
// 0.2009. Matrix means come from production traffic whose prompts were largely
// served from the prefix cache, and this pipeline's sheets are fresh.
//
// THE ENTRIES DIFFER IN COST, AND THE POOL WAS CALL-WEIGHTED. Per-entry means in
// the 4,096 to 16,384 token bucket, DeepSeek then GLM (calls in brackets):
// `zheermao22` 0.0936 (129) and 0.5528 (52); `CuspariaKLSY22` 0.1699 (54) and
// 0.6626 (18); `XingZ6017` 0.2347 (375) and 1.0734 (228). `XingZ6017` supplies
// most of the pool, and the whole of the 16,384 to 65,536 token buckets. Held
// at that table, `zheermao22` was held at 1.8 times its use, `CuspariaKLSY22`
// at 1.3 times and `XingZ6017` at 0.92 times; weighted by calls the rotation was
// held at its use. A pass that runs lighter entries over-holds a little, which
// refuses the last calls of a day a little early, and a pass of heavy entries
// under-holds a little, which the settlement corrects call by call.
//
// THE DRIFT SERIES IS ONE PER ENTRY, NOT A CLIMB. The measured-to-held ratio
// over the settled calls of the six runs of 2026-10-21 and 2026-10-22, in the
// order they ran: `zheermao21` 1.23, `CuspariaKLSY21` 2.00, `XingZ6016` 0.87
// (31 calls), `zheermao22` 1.39, `CuspariaKLSY22` 1.94, `XingZ6017` 2.84.
// The same entry read 1.23 then 1.39 and 2.00 then 1.94, so the figure moved
// with the entry run and not with the date. What `XingZ6017` did differently
// is measured: it sent 385 DeepSeek and 30 GLM calls in the 16,384 to 65,536
// token bucket (the other five runs sent 8 in all, held at the matrix's
// figures), and its calls in the 4,096 to 16,384 token bucket had longer
// prompts (mean 14.5k estimated tokens against 7.4k to 10.3k) and replies
// between the other two entries' (mean completion tokens 6,047 for GLM against
// 3,860 and 6,490; 2,042 for DeepSeek against 1,364 and 2,492), and spent more
// energy per thousand completion tokens (GLM 0.178 Wh against 0.143 and 0.102;
// DeepSeek 0.115 against 0.069 and 0.068). Why that entry costs more per
// completion token is cause not traced; its longer prompts are one candidate.
//
// THE SPREAD IS IN THE OUTPUT, NOT THE PROMPT. A call's energy tracked its
// completion tokens (read off the `SPEND` line of the same stamp): over the
// rotation the correlation was 0.76 for DeepSeek and 0.85 for GLM, at about
// 0.12 and 0.18 Wh per thousand completion tokens, against 0.19 and 0.26 for
// the prompt tokens the provider reported. In the 4,096 to 16,384 token bucket
// the DeepSeek median is 0.0856 Wh, the 90th percentile 0.4529 and the largest
// call 3.109 (the mean 0.1958); for GLM 0.6595, 2.2332 and 5.939 (the mean
// 0.9577); in the 16,384 to 65,536 token bucket the largest calls were 3.572 Wh
// for DeepSeek and 7.704 for GLM. These reasoning models spend most of a call
// on decoding, which a prompt-size bucket cannot see before the call is sent;
// the completion cap (`completion-cap.ts` in the shared contract) bounds the
// worst call. A hold at the mean therefore reads a typical call high and a
// tail call low, and the settlement sums that out. OPEN: an estimate shaped by
// the completion a call is expected to produce would fit far better than the
// prompt bucket, and no such figure is known before the send; it is a redesign
// of this table, not a change to its figures, and has not been made.
//
// THE MARGIN IS 1, now that the buckets that matter hold the measured mean.
// It only sizes the hold a call has while in flight, because the measured
// figure replaces it at settlement, so a hold at the mean is right on average.
// What a short hold can cost is bounded by the calls in flight at the moment
// the day runs out. The busiest minute of the rotation settled 24 calls (4 of
// them GLM, 2.5 Wh); up to 30 calls may be in flight, and 30 GLM calls each
// landing at the 90th percentile (2.23 Wh against the 0.96 Wh held) would
// overrun the day by about 38 Wh (7.0 percent of the 550 Wh), a bound no
// measured minute came near; calls spread about their mean add as the square
// root of their number, about 5 Wh for 30 GLM calls. The day of the rotation
// ended 5.188 Wh past zero (0.94 percent) with the old holds at about a third
// of the use. At the XingZ6020 means the same bound reads smaller for GLM and
// near the same for DeepSeek: 30 GLM calls each at the 90th percentile (0.16 Wh
// against 0.07 held) would overrun by about 2.7 Wh, and 30 DeepSeek calls each
// at theirs (0.75 Wh against 0.30) by about 13.4 Wh (2.4 percent of the 550
// Wh). The old margin of 2 was a start and not a measurement, and a
// larger margin sends the last calls of every day to a paid provider while
// the free energy still covers them. What the margin cannot do is stop plan
// spend by itself: a cost a stream reports is nominal on free-energy requests
// too, so it ends nothing (`neuralwatt-settlement.ts`), and an overrun shows in
// the plan-energy difference a run prints at its end
// (`neuralwatt-quota-session.ts`). The owner, 2026-10-20: "From here on just use
// NeuralWatt normally, I can tolerate some potential losses. If it works
// wrongly I'll tell you." and the rule that the hold be "as small as can be
// justified, so free energy is not left unspent".

/**
 Factor every bucket mean is multiplied by to give the energy a call holds
 while it is in flight; see the module note for why 1.
 */
export const NEURALWATT_ENERGY_MARGIN = 1;

/**
 One bucket of the matrix: the prompt sizes it covers and what a request in it
 used on average, by served model.

 @example
 ```ts
 const bucket: EnergyBucket = { belowTokens: 256, meanWh: { 'deepseek-v4.1-flash': 0.0011364, 'glm-5.3-flash': 0.0030471, }, };
 ```
 */
type EnergyBucket = {
  /**
   Prompt tokens below which a request falls in this bucket (and not in an
   earlier one): the matrix's `max_prompt_tokens`. A prompt at an edge
   belongs to the bucket above it, as the matrix's `min_prompt_tokens` reads.
   */
  readonly belowTokens: number;

  /**
   Mean watt-hours per request, by served model: the matrix's, or the
   pipeline's own where a run measured enough calls in the bucket.
   */
  readonly meanWh: Readonly<Record<NeuralWattServedId, number>>;
};

/**
 The matrix's first seven buckets, in order, two of them holding the measured
 means (the module note says which). Its eighth, a million prompt
 tokens and more, has no figure; a prompt past the last edge is estimated at
 the last bucket's mean.
 */
const ENERGY_BUCKETS: readonly EnergyBucket[] = [
  {
    belowTokens: 256,
    meanWh: {
      'deepseek-v4.1-flash': 0.0011364,
      'glm-5.3-flash': 0.0030471,
    },
  },
  {
    belowTokens: 1_024,
    meanWh: {
      'deepseek-v4.1-flash': 0.0073264,
      'glm-5.3-flash': 0.0088908,
    },
  },
  {
    belowTokens: 4_096,
    meanWh: {
      'deepseek-v4.1-flash': 0.0233012,
      'glm-5.3-flash': 0.0389276,
    },
  },
  {
    belowTokens: 16_384,
    // MEASURED, NOT THE MATRIX'S: the pipeline's own means over the settled
    // calls of `XingZ6020` (the module note has the counts, the earlier table
    // and why the GLM figure fell).
    meanWh: {
      'deepseek-v4.1-flash': 0.3044,
      // MEASURED AT THE `low` EFFORT: were it lifted to `max` this hold would fall short by about 13 times.
      'glm-5.3-flash': 0.0714,
    },
  },
  {
    belowTokens: 65_536,
    // MEASURED, NOT THE MATRIX'S: the same run.
    meanWh: {
      'deepseek-v4.1-flash': 0.3237,
      // MEASURED AT THE `low` EFFORT: were it lifted to `max` this hold would fall short by about 22 times.
      'glm-5.3-flash': 0.0556,
    },
  },
  {
    belowTokens: 262_144,
    meanWh: {
      'deepseek-v4.1-flash': 0.2784429,
      'glm-5.3-flash': 0.3999211,
    },
  },
  {
    belowTokens: 1_048_576,
    meanWh: {
      'deepseek-v4.1-flash': 0.6078422,
      'glm-5.3-flash': 0.9089305,
    },
  },
];

/**
 Energy one call is expected to use, and the bucket reading it came from.

 @example
 ```ts
 const estimate: NeuralWattEstimate = { bucketMeanWh: 0.0233012, estimateWh: 0.0233012, };
 ```
 */
export type NeuralWattEstimate = {
  /**
   The table's mean for the prompt's bucket, in watt-hours.
   */
  readonly bucketMeanWh: number;

  /**
   What the call holds while in flight: the mean times the margin.
   */
  readonly estimateWh: number;
};

/**
 The least any call can hold: the cheapest bucket of the cheapest model, with
 the margin. A day with less than this left cannot take any call, which is
 where the meter reads dry.
 */
export const NEURALWATT_SMALLEST_ESTIMATE_WH: number = Math.min(
  ...ENERGY_BUCKETS.flatMap(function firstBucketMeans(
    bucket,
    index,
  ): readonly number[] {
    return (index === 0) ? Object.values(bucket.meanWh,) : [];
  },),
) * NEURALWATT_ENERGY_MARGIN;

/**
 Prompt tokens a request body stands for, read off its bytes.

 THE REPOSITORY'S ONE FIGURE, `PROMPT_BYTES_PER_TOKEN` of the abandoned-spend
 estimate: no tokenizer runs before a call, the body is mostly UTF-8 Chinese
 at three bytes a character and English sheet text at one, and the first run
 supplies the real counts (`usage.prompt_tokens`) to compare against.

 FOR A BODY THAT CARRIES NO PICTURE. A picture travels as base64 inside the
 body and would be counted as text, about fifty times what the provider
 reports for it, so the client counts a request through
 `promptTokensOfRequest` (`neuralwatt-picture-tokens.ts`), which gives this
 function the text bytes and counts each picture by its patches.

 @param bodyBytes - size of the JSON body about to be sent

 @returns Prompt tokens, rounded up so a body never reads as lighter than it is

 @example
 ```ts
 promptTokensOfBody({ bodyBytes: 4_001, },); // 1001
 ```
 */
export function promptTokensOfBody({ bodyBytes, }: { readonly bodyBytes: number; },): number {
  return Math.ceil(bodyBytes / PROMPT_BYTES_PER_TOKEN,);
}

/**
 Energy a call is expected to use, by the model and the prompt's size.

 @param servedId - model the call goes to

 @param promptTokens - prompt tokens the call carries

 @returns Bucket's mean and estimate held against the day

 @throws {@link RangeError} when the count is negative or not a finite number

 @example
 ```ts
 const { estimateWh, } = estimateNeuralWattWh({ servedId: 'glm-5.3-flash', promptTokens: 2_000, },);
 ```
 */
export function estimateNeuralWattWh(
  {
    servedId,
    promptTokens,
  }: {
    readonly servedId: NeuralWattServedId;
    readonly promptTokens: number;
  },
): NeuralWattEstimate {
  if ((!Number.isFinite(promptTokens,)) || (promptTokens < 0))
    throw new RangeError(`a prompt of ${String(promptTokens,)} ${
      wordForCount({
        count: promptTokens,
        one: 'token',
        many: 'tokens',
      },)
    } falls in no energy bucket`,);

  /**
   Bucket the prompt falls in: the first whose edge is above it, else the last
   one measured.
   */
  const bucket = ENERGY_BUCKETS.find(function edgeExceedsPrompt(candidate,): boolean {
    return promptTokens < candidate.belowTokens;
  },) ?? nonNullishOrThrow(ENERGY_BUCKETS.at(-1,),);

  /**
   The bucket's mean for this model.
   */
  const bucketMeanWh = bucket.meanWh[servedId];
  return {
    bucketMeanWh,
    estimateWh: bucketMeanWh * NEURALWATT_ENERGY_MARGIN,
  };
}

//endregion NeuralWatt energy estimate
