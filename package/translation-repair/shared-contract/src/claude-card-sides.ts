import type {
  AnthropicCard,
  AnthropicPrices,
  ModelCard,
} from './model-card.ts';
import type {
  AnthropicEffort,
  AnthropicServedId,
  AnthropicWireModel,
} from './roster-id.ts';

//region Claude card sides
// THE EIGHT CARDS OF THE OWNER'S CLAUDE KEY, 2026-10-22, kept beside `model-cards.ts` for the reason
// `direct-card-sides.ts` gives: that table sits at the package's line budget.
//
// THE OWNER'S GRANT: "I've added TRANSLATION_REPAIR_CLAUDE_API_KEY . You may use latest Haiku and latest Sonnet with
// this key. This key would have $200 every month so be careful what you use." The key is not model-scoped (the
// parent session read it listing Opus, Fable and older models), so the lock is in our code three ways: the wire
// model of every card is one of two literals (`roster-id.ts`), `claude-cards.unit.test.ts` fails on a third, and the
// client throws for any roster model that is not one of these eight before a request leaves (`claude-client.ts`).
//
// ONE CARD PER MODEL AND EFFORT. The owner declined to choose an effort ("Why don't you bench?"), and the bench seats
// a roster identity, so each level is a card of its own: `<model>-effort-low`, `-effort-medium` and `-effort-high`,
// each sending that level as `output_config.effort`. The card named by the model alone sends no effort, which is the
// model's own default (Sonnet `high`, Haiku `medium`), the neutral reading of no choice. THE SIX EFFORT CARDS ARE
// FOR THE BENCH. They carry `judge-unmeasured`, so no derived bench seats them, and a run seats one only after the
// bench names its level and the owner's seat table lists it. They share a wire model with the card of the same model,
// so a bench arm must never seat two cards of one model together: one model in two seats would overstate agreement
// (`roster-id.ts`, `synthetic-catalog.ts`).
//
// WHICH CARD HOLDS WHICH SEAT, from the owner's "Sonnet edits, Haiku wide":
//
//   - `claude-sonnet-5-5` (no effort) is the third editor. It holds `judge-unmeasured`, which keeps it off every
//     derived bench (the wide seats, the late judges, the checkers, the translators), and the run's editor list
//     names it directly. `writer-unmeasured` and `reader-unmeasured` say what is true: nothing has measured its
//     writing for the translate lane or a transcription from a picture.
//   - `claude-haiku-5-5` (no effort) is a wide seat (critics, adjudication panel, select judges) and takes a place on
//     the checker order. `late-judge-dropped` keeps it off the roster-wide rounds after the lanes (lane contest,
//     consolidation), which the owner did not name; `writer-unmeasured` and `reader-unmeasured` for the reason given for
//     Sonnet.
//
// THE PRICES ARE THE ANTHROPIC PRICING PAGE (`https://platform.claude.com/docs/en/about-claude/pricing`, fetched
// 2026-10-22). Its footnote 2 prices Sonnet 5.5 cache hits at 0.05 times base input, 0.10 USD a million. The bundled
// API skill's migration guide of the same day still says 0.20 for Sonnet 5.5 cache reads; the live page wins.
//
// NO ZERO DATA RETENTION is promised by this route. The owner's ruling of 2026-10-05 (the corpus is public, PII is
// allowed) and the owner's grant quoted in this note are what permit it.

/**
 Sonnet 5.5's prices: 2 USD in, 10 out, 2.50 for a five-minute cache write and 0.10 for a cache hit, per million
 tokens, at every prompt size.

 @example
 ```ts
 const prices = SONNET_5_5_PRICES;
 ```
 */
export const SONNET_5_5_PRICES: AnthropicPrices = {
  inputUsdPerMillion: 2,
  outputUsdPerMillion: 10,
  cacheWriteUsdPerMillion: 2.5,
  cacheHitUsdPerMillion: 0.1,
};

/**
 Haiku 5.5's prices: up to 100,000 prompt tokens 0.10 in, 0.50 out, 0.125 cache write and 0.01 cache hit; past them
 0.50, 2.50, 0.625 and 0.05, per million tokens, the longer tier pricing the whole call.

 @example
 ```ts
 const prices = HAIKU_5_5_PRICES;
 ```
 */
export const HAIKU_5_5_PRICES: AnthropicPrices = {
  inputUsdPerMillion: 0.1,
  outputUsdPerMillion: 0.5,
  cacheWriteUsdPerMillion: 0.125,
  cacheHitUsdPerMillion: 0.01,
  longPrompt: {
    aboveTokens: 100_000,
    inputUsdPerMillion: 0.5,
    outputUsdPerMillion: 2.5,
    cacheWriteUsdPerMillion: 0.625,
    cacheHitUsdPerMillion: 0.05,
  },
};

/**
 Output ceiling both models list: 128,000 tokens (the bundled API skill's model table, 2026-10-22), thinking counted.
 */
const MAX_OUTPUT_TOKENS = 128_000;

/**
 Completion ceiling of every Claude card, in tokens, thinking included.

 NOT THE POOLED P99 OF 13,082 (`completion-cap.ts`), which was measured on providers whose reasoning does not count the
 way Claude's does: on both models adaptive thinking counts toward `max_tokens` even though its text is not the answer,
 and a call that spends the cap thinking stops with `stop_reason: "max_tokens"` after a `thinking` block and before any
 text, is billed in full and is asked again (the bundled API skill, `shared/model-migration.md`: "Thinking counts toward
 `max_tokens`", and for Haiku 5.5 "Raise the cap to leave room for thinking, or choose a lower effort level"). The skill
 starts long agentic turns at 64,000; these calls answer a structured sheet and not a long turn, so the start here is
 32,000, which costs at most 0.32 USD for a call that runs to it on Sonnet (10 USD a million output tokens) and 0.016 on
 Haiku. A STARTING VALUE UNTIL THE FIRST CALLS ARE MEASURED: the cap census (`run/src/corpus-run/cap-census.ts`) reads
 the `SPEND` lines of the Claude calls like any other model's and flags the card when its own 99th percentile disagrees.

 THE DEADLINE THIS CAP NEEDS IS `claudeCallDeadlineMs`, 460,000 ms: the run's 360,000 ms reaches the cap only at 89
 tokens a second or better.
 */
export const CLAUDE_COMPLETION_CAP = 32_000;

/**
 Slowest whole-call output rate a Claude call is sized for, in tokens a second, thinking counted.

 MEASURED, 2026-10-22 22:02 to 22:03 UTC, by the lead's live probe (the figures are kept in
 `package/translation-repair/doc/claude-route-probe-figures-and-review-handles.md`): Haiku 5.5 170.2 and 152.6 tokens a
 second, Sonnet 5.5 113.8 and 101.3, two short-prompt samples each with no spread measured. 80 sits about a fifth under
 the slowest measured call, Sonnet's 101.3. A FLOOR TO SIZE A DEADLINE BY, NOT A PREDICTION: the first corpus pass's
 `SPEND` lines and elapsed times say whether a longer prompt or a loaded hour runs slower.
 */
export const CLAUDE_FLOOR_TOKENS_PER_SECOND = 80;

/**
 Time a Claude call is allowed before its first output token, in milliseconds: the prompt is read and the first byte
 arrives inside it. A SIZED ALLOWANCE, NOT A MEASUREMENT: the six probe calls of 2026-10-22 were short prompts, and
 the stage sheets are long ones.
 */
export const CLAUDE_FIRST_BYTE_ALLOWANCE_MS = 60_000;

/**
 Milliseconds in a second.
 */
const MS_PER_SECOND = 1_000;

/**
 Deadline for one Claude call, in milliseconds: the completion ceiling at the floor rate, plus the allowance for the
 prompt and the first byte. The cap of 32,000 tokens is 400 s at 80 tokens a second, plus 60 s, so 460,000 ms.

 WHY THE RUN'S 360,000 MS (`RUN_PER_CALL_TIMEOUT_MS`) DID NOT SUFFICE: 32,000 tokens inside 360 s is 88.9 tokens a
 second, so a call that thinks and answers up to the cap below that speed would be cut by its deadline, billed for
 what it generated and return no answer. The client takes the larger of the run's deadline and this one
 (`claude-client.ts`), so a Claude call is never given less than its ceiling needs and a caller's longer bound stands.

 @param maxTokens - ceiling the call sends as `max_tokens`

 @returns Milliseconds the call may run

 @example
 ```ts
 claudeCallDeadlineMs({ maxTokens: CLAUDE_COMPLETION_CAP, },);
 // => 460_000
 ```
 */
export function claudeCallDeadlineMs({ maxTokens, }: { readonly maxTokens: number; },): number {
  return Math.ceil((maxTokens / CLAUDE_FLOOR_TOKENS_PER_SECOND) * MS_PER_SECOND,)
    + CLAUDE_FIRST_BYTE_ALLOWANCE_MS;
}

/**
 Builds one card side.

 @param id - roster spelling of the card

 @param wireModel - model id the request carries

 @param prices - the model's prices

 @param effort - effort the card sends, absent for the model's own default

 @returns The side

 @example
 ```ts
 const side = sideOf({ id: 'claude-haiku-5-5', wireModel: 'claude-haiku-5-5', prices: HAIKU_5_5_PRICES, },);
 ```
 */
function sideOf(
  {
    id,
    wireModel,
    prices,
    effort,
  }: {
    readonly id: AnthropicServedId;
    readonly wireModel: AnthropicWireModel;
    readonly prices: AnthropicPrices;
    readonly effort?: AnthropicEffort;
  },
): AnthropicCard {
  return {
    id,
    wireModel,
    // Conditional spread keeps the member absent instead of undefined.
    ...((effort === undefined) ? {} : { effort, }),
    readsImages: true,
    maxOutputLength: MAX_OUTPUT_TOKENS,
    prices,
  };
}

/**
 Holds of every card that sits on no derived bench: the six effort cards and Sonnet's.
 */
const NO_DERIVED_BENCH = [
  'judge-unmeasured',
  'writer-unmeasured',
  'reader-unmeasured',
] as const;

/**
 The eight cards of the Claude key, keyed by roster id.

 @example
 ```ts
 const card = CLAUDE_CARDS['claude-sonnet-5-5'];
 ```
 */
export const CLAUDE_CARDS: Readonly<Record<AnthropicServedId, ModelCard>> = {
  'claude-sonnet-5-5': {
    anthropic: sideOf({
      id: 'claude-sonnet-5-5',
      wireModel: 'claude-sonnet-5-5',
      prices: SONNET_5_5_PRICES,
    },),
    completionCap: CLAUDE_COMPLETION_CAP,
    holds: [...NO_DERIVED_BENCH,],
  },
  'claude-sonnet-5-5-effort-low': {
    anthropic: sideOf({
      id: 'claude-sonnet-5-5-effort-low',
      wireModel: 'claude-sonnet-5-5',
      prices: SONNET_5_5_PRICES,
      effort: 'low',
    },),
    completionCap: CLAUDE_COMPLETION_CAP,
    holds: [...NO_DERIVED_BENCH,],
  },
  'claude-sonnet-5-5-effort-medium': {
    anthropic: sideOf({
      id: 'claude-sonnet-5-5-effort-medium',
      wireModel: 'claude-sonnet-5-5',
      prices: SONNET_5_5_PRICES,
      effort: 'medium',
    },),
    completionCap: CLAUDE_COMPLETION_CAP,
    holds: [...NO_DERIVED_BENCH,],
  },
  'claude-sonnet-5-5-effort-high': {
    anthropic: sideOf({
      id: 'claude-sonnet-5-5-effort-high',
      wireModel: 'claude-sonnet-5-5',
      prices: SONNET_5_5_PRICES,
      effort: 'high',
    },),
    completionCap: CLAUDE_COMPLETION_CAP,
    holds: [...NO_DERIVED_BENCH,],
  },
  'claude-haiku-5-5': {
    anthropic: sideOf({
      id: 'claude-haiku-5-5',
      wireModel: 'claude-haiku-5-5',
      prices: HAIKU_5_5_PRICES,
    },),
    completionCap: CLAUDE_COMPLETION_CAP,
    holds: [
      'writer-unmeasured',
      'reader-unmeasured',
      'late-judge-dropped',
    ],
  },
  'claude-haiku-5-5-effort-low': {
    anthropic: sideOf({
      id: 'claude-haiku-5-5-effort-low',
      wireModel: 'claude-haiku-5-5',
      prices: HAIKU_5_5_PRICES,
      effort: 'low',
    },),
    completionCap: CLAUDE_COMPLETION_CAP,
    holds: [...NO_DERIVED_BENCH,],
  },
  'claude-haiku-5-5-effort-medium': {
    anthropic: sideOf({
      id: 'claude-haiku-5-5-effort-medium',
      wireModel: 'claude-haiku-5-5',
      prices: HAIKU_5_5_PRICES,
      effort: 'medium',
    },),
    completionCap: CLAUDE_COMPLETION_CAP,
    holds: [...NO_DERIVED_BENCH,],
  },
  'claude-haiku-5-5-effort-high': {
    anthropic: sideOf({
      id: 'claude-haiku-5-5-effort-high',
      wireModel: 'claude-haiku-5-5',
      prices: HAIKU_5_5_PRICES,
      effort: 'high',
    },),
    completionCap: CLAUDE_COMPLETION_CAP,
    holds: [...NO_DERIVED_BENCH,],
  },
};

//endregion Claude card sides
