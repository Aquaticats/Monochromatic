//region Hyper price
// WHAT A CHARM HYPER TOKEN COSTS, as of the date this table was read.
//
// PRICES ARE AN OBSERVATION, NOT A CONSTANT. The 2026-09-11 public model-list
// read verified every previous row and added the newly listed models. The
// 2026-10-11 read (keyless, after TianqiChen66622's spend report disagreed with
// Hyper's balance) found three rows moved and re-priced them; the rows of
// models the list no longer carries keep their 2026-09-11 rates. Hyper's
// models documentation and FAQ define one hypercredit as USD 0.05, so these
// per-million-token credit quotes equal the API's USD rates multiplied by 20.
// The dated snapshot prices usage; it is not a wire-reported debit.
//
// CREDITS PER MILLION TOKENS, which is how the provider quotes them. Output is
// two to five times input across this roster and `completion_tokens` counts
// thinking, which dominates output on these models, so the answer half is the
// expensive half and it is not the visible half.
//
// CACHE RATES REMAIN RECORDED, but this estimator has only prompt and completion
// totals. It applies ordinary input pricing rather than inventing a cache-token
// breakdown. A server-side cache discount is not ruled out by the absence of
// caller-supplied cache_control; computed credits are quoted-rate estimates.
//
// SYNTHETIC IS NOT PRICED HERE AND MUST NOT BE. It is a flat subscription whose
// meter is a percentage of a weekly allowance, already carried on the `METERS`
// line. Converting its tokens to credits would invent a currency it does not
// bill in and would inflate any total that mixed the two.

/**
 Date every rate of a model the public model-list API still lists was last
 verified against it.

 SHIPPED WITH THE RATES rather than left to a comment, because every report
 that prints a credit figure has to be able to say how old it is.
 */
export const HYPER_PRICE_READ_ON = '2026-10-11';

/**
 What one model costs, in credits per million tokens.

 @example
 ```ts
 const rates: CreditRates = { input: 40, output: 120, cacheCreate: 0, cacheHit: 5, };
 ```
 */
export type CreditRates = {
  /**
   Credits per million prompt tokens that were not served from cache.
   */
  readonly input: number;

  /**
   Credits per million completion tokens, thinking included.
   */
  readonly output: number;

  /**
   Credits per million tokens written into the cache.
   Retained for usage records that explicitly distinguish cache tokens.
   */
  readonly cacheCreate: number;

  /**
   Credits per million prompt tokens served from the cache.
   Retained for usage records that explicitly distinguish cache tokens.
   */
  readonly cacheHit: number;
};

/**
 Tokens one quoted rate covers, since the provider quotes per million.
 */
const RATE_UNIT_TOKENS = 1_000_000;

/**
 Every model the provider listed, by the id it serves the model under.

 THE WHOLE PAGE RATHER THAN THE EIGHT THIS PIPELINE SEATS, so that changing
 the roster does not silently drop a seat into the unpriced bucket, and so a
 reader comparing seats can see what an unseated model would have cost.
 */
const HYPER_CREDIT_RATES = {
  // Read 2026-10-20, separately from `HYPER_PRICE_READ_ON`: USD 0.33 in, 1.31 out and
  // 0.03 cache hit, the same as `deepseek-v4.1-flash`. The seat sends this id
  // since 2026-10-20, and the SPEND line labels a Hyper call by the id on the
  // wire, so this is the row that prices its spend.
  'deepseek-v4.1-flash-1017': {
    input: 6.6,
    output: 26.2,
    cacheCreate: 0,
    cacheHit: 0.6,
  },
  // Read 2026-10-11: USD 0.33 in and 1.31 out (was 0.30 and 1.20 on 2026-09-11).
  // Kept for the logs written before 2026-10-20, which label the seat by this id.
  'deepseek-v4.1-flash': {
    input: 6.6,
    output: 26.2,
    cacheCreate: 0,
    cacheHit: 0.6,
  },
  // Pricing observations do not add unapproved models to the serving catalog.
  'inkling': {
    input: 21.776,
    output: 88.1928,
    cacheCreate: 0,
    cacheHit: 3.70192,
  },
  'kimi-k2-thinking': {
    input: 12,
    output: 50,
    cacheCreate: 0,
    cacheHit: 6,
  },
  'deepseek-v4-flash': {
    input: 4,
    output: 8,
    cacheCreate: 0,
    cacheHit: 0.8,
  },
  'deepseek-v4-flash-0731': {
    input: 8.8,
    output: 26.4,
    cacheCreate: 0,
    cacheHit: 0.88,
  },
  'deepseek-v4-pro': {
    input: 48,
    output: 96,
    cacheCreate: 0,
    cacheHit: 4,
  },
  'deepseek-v4-pro-0813': {
    input: 28.74432,
    output: 86.23296,
    cacheCreate: 0,
    cacheHit: 0.958144,
  },
  // Read 2026-10-11: USD 0.098 in, 0.334 out, 0.049 cache hit (was 0.122, 0.42
  // and 0.061 on 2026-09-11).
  'gemma-4-26b-a4b-it': {
    input: 1.96,
    output: 6.68,
    cacheCreate: 0,
    cacheHit: 0.98,
  },
  'glm-5': {
    input: 17.2,
    output: 55.68,
    cacheCreate: 0,
    cacheHit: 8.6,
  },
  'glm-5.1': {
    input: 26.52,
    output: 84.4,
    cacheCreate: 0,
    cacheHit: 13.26,
  },
  'glm-5.2': {
    input: 30.4864,
    output: 95.8144,
    cacheCreate: 0,
    cacheHit: 3.04864,
  },
  'glm-5.3': {
    input: 30.4864,
    output: 95.8144,
    cacheCreate: 0,
    cacheHit: 5.66176,
  },
  'glm-5.3-flash': {
    input: 3.2664,
    output: 10.888,
    cacheCreate: 0,
    cacheHit: 0.6315040000000001,
  },
  'gpt-oss-120b': {
    input: 3.56,
    output: 13.6,
    cacheCreate: 0,
    cacheHit: 1.78,
  },
  'kimi-k2.5': {
    input: 11.168,
    output: 58.7,
    cacheCreate: 0,
    cacheHit: 5.584,
  },
  'kimi-k2.6': {
    input: 20.6872,
    output: 87.104,
    cacheCreate: 0,
    cacheHit: 3.48416,
  },
  'kimi-k2.7-code': {
    input: 20.6872,
    output: 87.104,
    cacheCreate: 0,
    cacheHit: 4.13744,
  },
  'kimi-k3': {
    input: 65.328,
    output: 326.64,
    cacheCreate: 0,
    cacheHit: 6.5328,
  },
  'llama-3.3-70b-instruct': {
    input: 12.132,
    output: 20.772,
    cacheCreate: 0,
    cacheHit: 6.066,
  },
  'llama-4-maverick-17b-128e-instruct-fp8': {
    input: 5.48,
    output: 17.984,
    cacheCreate: 0,
    cacheHit: 2.74,
  },
  // Read 2026-10-11: USD 0.484 in, 1.852 out, 0.242 cache hit (was 0.404, 1.496
  // and 0.202 on 2026-09-11).
  'minimax-m2.7': {
    input: 9.68,
    output: 37.04,
    cacheCreate: 0,
    cacheHit: 4.84,
  },
  'minimax-m3': {
    input: 6.5328,
    output: 26.1312,
    cacheCreate: 0,
    cacheHit: 1.284784,
  },
  'qwen3-coder-480b-a35b-instruct-int4-mixed-ar': {
    input: 8.9,
    output: 42.9,
    cacheCreate: 0,
    cacheHit: 4.45,
  },
  'qwen3-next-80b-a3b-instruct': {
    input: 2.35,
    output: 22.72,
    cacheCreate: 0,
    cacheHit: 1.175,
  },
  'qwen3.6-flash': {
    input: 20,
    output: 80,
    cacheCreate: 25,
    cacheHit: 2,
  },
  'qwen3.6-max': {
    input: 40,
    output: 240,
    cacheCreate: 50,
    cacheHit: 4,
  },
  'qwen3.6-plus': {
    input: 40,
    output: 120,
    cacheCreate: 50,
    cacheHit: 4,
  },
  'qwen3.7-flash': {
    input: 4,
    output: 16,
    cacheCreate: 0,
    cacheHit: 0.8,
  },
  'qwen3.7-max': {
    input: 50,
    output: 150,
    cacheCreate: 0,
    cacheHit: 10,
  },
  'qwen3.7-plus': {
    input: 24,
    output: 96,
    cacheCreate: 0,
    cacheHit: 4.8,
  },
  'qwen3.8-2.4t-a95b': {
    input: 40,
    output: 120,
    cacheCreate: 0,
    cacheHit: 5,
  },
  'qwen3.8-27b': {
    input: 10,
    output: 60,
    cacheCreate: 0,
    cacheHit: 2,
  },
  'qwen3.8-flash': {
    input: 3,
    output: 9.4,
    cacheCreate: 0,
    cacheHit: 0.32,
  },
  'qwen3.8-max': {
    input: 40,
    output: 120,
    cacheCreate: 0,
    cacheHit: 5,
  },
} satisfies Readonly<Record<string, CreditRates>>;

/**
 Rates keyed for lookup by a model id that came off a log line.

 A `Map` RATHER THAN `HYPER_CREDIT_RATES` ITSELF, for the reason `spend-read.ts` gives
 about its own field table: the key arrives from a run log, and an object
 lookup would answer `__proto__` and `constructor` with something that is not
 a rate. The literal stays an object because it is written here and reads
 better as one, checked rather than typed as a record of strings so its type
 keeps the ids and a lookup on it by any other text does not compile
 (ledger B77).
 */
const RATES: ReadonlyMap<string, CreditRates> = new Map(Object.entries(HYPER_CREDIT_RATES,),);

/**
 What this model costs, or that the table has never heard of it.

 NAMED ABSENCE RATHER THAN A ZERO RATE. A model the provider added after this
 table was read is not free, and a total that quietly billed it at nothing
 would read as a cheaper run rather than an incomplete one.

 @param model - id as the provider serves it, exactly as the `SPEND` line
 recorded it

 @returns Rates for this model, or that it is not in the table

 @example
 ```ts
 const rates = ratesFor({ model: 'qwen3.8-max', },);
 ```
 */
export function ratesFor(
  { model, }: { readonly model: string; },
): CreditRates | 'unpriced' {
  /**
   Row this model has, absent where the table predates it.
   */
  const found = RATES.get(model,);

  if (found === undefined)
    return 'unpriced';

  return found;
}

/**
 Prices one seat's tokens, keeping the two halves apart.

 BOTH HALVES RETURNED, not just their sum, because the output half is where
 this roster's cost actually lives and a single total hides which seat's
 thinking bought it.

 @param model - id as the provider serves it

 @param promptTokens - prompt tokens summed over this seat's reported calls

 @param completionTokens - completion tokens summed over the same calls,
 thinking included

 @returns Credits for each half, or that this model is not in the table

 @example
 ```ts
 const cost = creditsFor({ model: 'kimi-k3', promptTokens: 84_000, completionTokens: 51_065, },);
 ```
 */
export function creditsFor(
  {
    model,
    promptTokens,
    completionTokens,
  }: {
    readonly model: string;
    readonly promptTokens: number;
    readonly completionTokens: number;
  },
): {
  readonly inputCredits: number;
  readonly outputCredits: number;
} | 'unpriced' {
  /**
   Rates this model bills at, absent where the table has no row.
   */
  const rates = ratesFor({ model, },);

  if (rates === 'unpriced')
    return 'unpriced';

  return {
    inputCredits: (promptTokens * rates.input) / RATE_UNIT_TOKENS,
    outputCredits: (completionTokens * rates.output) / RATE_UNIT_TOKENS,
  };
}

//endregion Hyper price
