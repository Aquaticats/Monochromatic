import type { BedrockCredits, } from './bedrock-ledger.ts';
import type { DeepSeekBalance, } from './deepseek-balance.ts';
import type { OpenRouterCredits, } from './openrouter-credits.ts';
import {
  type HyperCredits,
  PROVIDER_ORDER,
  type ProviderName,
  type ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { QuotaSnapshot, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { isIsoStampText, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { XiaomiSpend, } from './xiaomi-spend.ts';
import { modelDarkReason, } from './model-dark-reason.ts';
import {
  namedDrynessOf,
  type NeuralWattReturn,
  openingClauseOf,
  returnClauseOf,
  returnTimeOf,
} from './neuralwatt-return-words.ts';
import type { NeuralWattDryness, } from './neuralwatt-return-hold.ts';

export type { NeuralWattReturn, } from './neuralwatt-return-words.ts';

//region Budget routing
// WHICH PROVIDER SERVES ONE CALL, decided from budget and saturation rather
// than from a fixed split.
//
// The owner's policy, in the owner's order: send everything to Synthetic, and when a provider's per-model concurrency
// limit is reached, overflow to the next one with budget; if Synthetic has run out of quota, use Hyper; when Hyper is
// dry too, buy on OpenRouter (2026-09-03); if every provider is out at once, throw an error saying so and end the run.
// No provider states a per-model ceiling today (Synthetic dropped its own on 2026-10-19), so no live call overflows
// for saturation; the walk still honours a ceiling a caller injects.
//
// THE POLICY IS A WALK DOWN `PROVIDER_ORDER`. The first provider that serves the model and has budget takes the call,
// unless its per-model slots are all taken and a later provider with budget can take it now; a saturated provider
// with nobody behind it still gets the call, since a queue behind its limit is slower than the split and still buys
// the answer.
//
// SYNTHETIC HAS A WEEKLY LIMIT, AND A REPLY MAY STILL CARRY A FIVE-HOUR WINDOW. Synthetic's rate-limit page of 2026-10-19
// describes only a weekly credit budget, and its quota reply of that day had no `rollingFiveHourLimit`, so the weekly
// budget decides every reading. An older reply, or an account still on the old terms, may carry the five-hour window, and
// that window is honoured when present: either being empty is a reason to fail over. A second provider was
// added because the weekly one emptied and 866 of 875 lost voices carried a
// single HTTP 429, while the reader that would have seen it coming had been
// parsing `weekly.percentRemaining` and discarding it since 2026-07-16.
//
// DRYNESS ARRIVES AS A RECORD OF BOOLEANS rather than being read off the
// snapshots here, so a caller can OR in what it just learned at the wire. A
// budget reading is minutes old and a 429 is now; a router that could only see
// the reading would keep sending to a provider that has already started
// refusing.
//
// A MODEL THAT NO LIVE PROVIDER SERVES IS AN OUTCOME, NOT A THROW. Part of the
// roster is served by fewer than all providers, and one of those providers
// being dry costs that panelist its voice, which the pipeline already records
// and proceeds without. Only every budget being empty ends a run, because at
// that point nothing can be bought at all. A provider that reads wet but does not serve the model asked still leaves
// the run something to buy for its other seats, so it keeps its own reading (owner's ruling of 2026-10-22, open to the
// owner's veto). A WET PROVIDER COUNTS AGAINST "EVERY PROVIDER DRY" ONLY IF IT SERVES A MODEL SEATED IN A WRITING BENCH
// (`WritingReach`): Xiaomi's wet meter, whose one seat is a checker, no longer hid a fleet that could write nothing
// (the lead's ruling of 2026-10-22 on the review's M3, open to the owner's veto).

/**
 Which providers keep the run going while they read wet: those that serve a model seated in a writing bench, so a
 writing bench could still reach its floor from them.

 DERIVED FROM THE ROSTER BY THE RUN, NOT LISTED HERE: this package knows no roster. A provider that serves only seats
 that judge or read (Xiaomi, whose one seat is a checker, in the roster of 2026-10-22) cannot keep a run going that
 writes nothing, so its wet meter does not count against "every provider dry" (owner's ruling of 2026-10-22 on the
 review's M3, open to the owner's veto). Per-call routing does not read this: a call whose model has a wet provider
 still goes there.

 @example
 ```ts
 const writing: WritingReach = {
   neuralwatt: true, synthetic: true, hyper: true, bedrock: true, deepseek: true, xiaomi: false, openrouter: true,
 };
 ```
 */
export type WritingReach = ProviderRecord<boolean>;

/**
 The record of a caller that knows no roster: every provider counts, so only every meter reading dry ends the run.
 */
export const EVERY_PROVIDER_COUNTS: WritingReach = {
  neuralwatt: true,
  synthetic: true,
  hyper: true,
  bedrock: true,
  deepseek: true,
  xiaomi: true,
  openrouter: true,
};

/**
 Whether every provider that keeps the run going reads dry.

 @param view - dryness per provider, holds and refusals folded in

 @param writing - providers whose wet reading keeps the run going

 @returns Whether nothing that writes a page can be bought anywhere

 @example
 ```ts
 const over = everyWritingProviderDry({ view, writing, },);
 ```
 */
export function everyWritingProviderDry(
  {
    view,
    writing,
  }: {
    readonly view: ProviderRecord<boolean>;
    readonly writing: WritingReach;
  },
): boolean {
  return PROVIDER_ORDER.every(function dryOrIdle(provider,): boolean {
    return view[provider] || (!writing[provider]);
  },);
}

/**
 The providers that read wet but serve no seat that writes, which a refusal that ends the run names.

 @param view - dryness per provider

 @param writing - providers whose wet reading keeps the run going

 @returns Idle providers in spending order

 @example
 ```ts
 const idle = idleWetProviders({ view, writing, },);
 ```
 */
export function idleWetProviders(
  {
    view,
    writing,
  }: {
    readonly view: ProviderRecord<boolean>;
    readonly writing: WritingReach;
  },
): readonly ProviderName[] {
  return PROVIDER_ORDER.filter(function isIdle(provider,): boolean {
    return (!view[provider]) && (!writing[provider]);
  },);
}

/**
 How the refusal that ends the run reads one meter, `dry` or a wet provider with the reason it does not count.

 @param view - dryness per provider

 @param provider - provider read

 @returns Word for the measured clause

 @example
 ```ts
 meterWord({ view, provider: 'xiaomi', },); // => 'wet (serves no seat that writes)'
 ```
 */
export function meterWord(
  {
    view,
    provider,
  }: {
    readonly view: ProviderRecord<boolean>;
    readonly provider: ProviderName;
  },
): string {
  return view[provider] ? 'dry' : 'wet (serves no seat that writes)';
}

/**
 What each provider's dry clause says in the refusal, in the order the refusal lists them.
 */
const DRY_CLAUSES: readonly (readonly [
  Exclude<ProviderName, 'neuralwatt'>,
  string,
  string
])[] = [
  [
    'synthetic',
    'Synthetic',
    'Synthetic has no weekly credit left (or no five-hour request left, on a reply that still carries that window)',
  ],
  [
    'hyper',
    'Charm Hyper',
    'Charm Hyper has no balance left',
  ],
  [
    'bedrock',
    'Amazon Bedrock',
    'Amazon Bedrock has no credit left short of the owner\'s card',
  ],
  [
    'deepseek',
    'DeepSeek',
    'DeepSeek has no balance left (or says its account is unavailable)',
  ],
  [
    'xiaomi',
    'Xiaomi',
    'Xiaomi has refused a call for payment',
  ],
  [
    'openrouter',
    'OpenRouter',
    'OpenRouter has no credit left',
  ],
];

/**
 The sentence naming what each provider is out of, with a provider that reads wet but serves no seat that writes named
 for that instead.

 @param idle - providers that read wet but serve no seat that writes

 @returns Clauses joined as a list ending in "and", without the closing period

 @example
 ```ts
 providerClausesOf({ idle: ['xiaomi',], },);
 ```
 */
function providerClausesOf({ idle, }: { readonly idle: readonly ProviderName[]; },): string {
  /**
   One clause per provider.
   */
  const clauses = DRY_CLAUSES.map(function clauseOf([provider, name, dry,],): string {
    return idle.includes(provider,) ? `${name} reads wet but serves no seat that writes` : dry;
  },);
  /**
   The last clause, which the list joins with "and".
   */
  const last = clauses.at(-1,);
  return `${clauses.slice(
    0,
    -1,
  )
    .join(', ',)}, and ${last ?? ''}`;
}

/**
 Refusal raised when no provider has budget left.

 ENDS THE RUN, at the owner's instruction. Every other budget state leaves
 something buyable, and this one leaves nothing.

 RENAMED 2026-09-03 from the two-provider name when OpenRouter joined; the
 old name is in the local forbidden-strings appendix so it cannot come back.

 @example
 ```ts
 throw new EveryProviderDryError();
 ```
 */
export class EveryProviderDryError extends Error {
  /**
   Declares this message safe to forward: fixed sentences plus, at most, the
   meter states and hold lengths the decision was made on and NeuralWatt's day-start time of day, never content.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure stating that no provider can serve any call.

   @param measured - meter states and holds at the decision, stated so a
   reader can tell exhaustion from refusal holds;
   composed by the caller from the dryness record and the holds, and "no
   reading cited" where the caller has none

   @param neuralwatt - what the run knows of NeuralWatt's day, so the sentence about its return states the account's
   own time of day and never midnight; "not decided" where the caller has none

   @param idle - providers that read wet but serve no seat that writes, which the refusal names in place of an
   exhaustion they do not have; none where every provider reads dry

   @param neuralwattDryness - why NeuralWatt reads dry as the reading found it, so the refusal states only that cause;
   absent where the caller knows none

   @param neuralwattReturnsInMs - milliseconds until NeuralWatt's day starts again, set WHILE THE RETURN
   STILL LIES AHEAD, whether or not the call waited out another hold first: the pass reads it to park until the return
   instead of ending the run, and a return that has passed (the account lagging) names none

   @param neuralwattReadAtMs - instant of the reading that named the return, on the monotonic clock, given by a
   caller that makes the refusal some time before it reaches the pass (the seat reader), so a park counts its wait from
   the read; absent for the router's own refusal, which names the distance as it is raised

   @param afterWaiting - whether the call waited for a hold before this refusal

   @example
   ```ts
   new EveryProviderDryError({ measured: 'meters read synthetic dry, hyper dry, openrouter dry; holds synthetic 0ms, hyper 0ms, openrouter 0ms', },);
   ```
   */
  public constructor(
    {
      measured = 'no reading cited',
      neuralwatt = { kind: 'unanchored', },
      idle = [],
      neuralwattDryness,
      neuralwattReturnsInMs,
      neuralwattReadAtMs,
      afterWaiting = false,
    }: {
      readonly measured?: string;
      readonly neuralwatt?: NeuralWattReturn;
      readonly idle?: readonly ProviderName[];
      readonly neuralwattDryness?: NeuralWattDryness;
      readonly neuralwattReturnsInMs?: number;
      readonly neuralwattReadAtMs?: number;
      readonly afterWaiting?: boolean;
    } = {},
  ) {
    /**
     Why NeuralWatt reads dry, as the refusal states it.
     */
    const named = namedDrynessOf({
      ...((neuralwattDryness === undefined) ? {} : { neuralwattDryness, }),
      ...((neuralwattReturnsInMs === undefined) ? {} : { neuralwattReturnsInMs, }),
    },);
    super(
      `Every provider is out of budget at once: ${
        openingClauseOf({
          day: neuralwatt,
          ...named,
        },)
      }`
        + `${providerClausesOf({ idle, },)}. `
        + `${
          (idle.length === 0)
            ? 'Nothing further can be bought'
            : 'Nothing further that writes a page can be bought'
        }, `
        + 'so this run ends. '
        + `${
          returnClauseOf({
            day: neuralwatt,
            ...named,
            afterWaiting,
          },)
        }Synthetic regenerates on its own schedule; Hyper, `
        + 'DeepSeek and OpenRouter refill on purchase; Xiaomi, once refused for payment, is read again only after one '
        + 'of its calls settles, so a top-up is seen on the next launch; '
        + 'Bedrock\'s credit is never topped up. '
        + `Measured at the decision: ${measured}.`,
    );
    this.name = 'EveryProviderDryError';
    if (neuralwattDryness !== undefined)
      this.neuralwattDryness = neuralwattDryness;
    if (neuralwattReadAtMs !== undefined)
      this.neuralwattReadAtMs = neuralwattReadAtMs;
    if (neuralwattReturnsInMs !== undefined) {
      this.neuralwattReturnsInMs = neuralwattReturnsInMs;
      /**
       The time of day the return falls at, for the line that parks the pass.
       */
      const { text, } = returnTimeOf({
        day: neuralwatt,
        ...named,
      },);
      if (text !== undefined)
        this.neuralwattReturnsAt = text;
    }
  }

  /**
   Why NeuralWatt read dry in the reading the refusal was made on, absent where the caller named none.
   */
  readonly neuralwattDryness?: NeuralWattDryness;

  /**
   Milliseconds until NeuralWatt's spent day starts again, margin included, present while the return still lies ahead,
   whether or not the call waited out another hold: the pass parks for it when it has the time.
   */
  readonly neuralwattReturnsInMs?: number;

  /**
   The time of day the return falls at, with its zone, present with the distance.
   */
  readonly neuralwattReturnsAt?: string;

  /**
   The instant of the reading that named the return, on the monotonic clock, present where the seat reader made the
   refusal.
   */
  readonly neuralwattReadAtMs?: number;
}

/**
 Which providers can serve one model at all, before budget is considered.

 STATED RATHER THAN DERIVED, so this decision is testable without a roster and
 keeps answering correctly while the roster is being widened.

 @example
 ```ts
 const reach: ModelReach = { synthetic: true, hyper: true, openrouter: true, };
 ```
 */
export type ModelReach = ProviderRecord<boolean>;

/**
 Reading given when no provider in order both serves a model and has budget.
 */
export const NO_PROVIDER = 'none';

/**
 Where one call goes, or why it can go nowhere.

 @example
 ```ts
 const choice: ProviderChoice = { kind: 'hyper', };
 ```
 */
export type ProviderChoice =
  | {
    /**
     Provider that takes the call.
     */
    readonly kind: ProviderName;
  }
    | {
    /**
     Discriminator marking a call no live provider can take.
     */
      readonly kind: 'unreachable';

    /**
     Why, in terms a voice-loss record can carry.
     */
      readonly reason: string;
    };

/**
 Whether Synthetic's budget reading says nothing more can be bought there.

 THE WEEKLY BUDGET ALWAYS DECIDES, and a five-hour window is read only when the snapshot has one. Synthetic's reply of 2026-10-19
 carried no such window, so an empty weekly budget is the one way to be out; where an older reply still carries the window,
 the provider throttling the account and an empty window are two more, and the one that actually emptied was the weekly budget.

 @param quota - most recent budget reading

 @returns Whether that reading leaves nothing buyable

 @example
 ```ts
 const dry = syntheticIsDry({ quota, },);
 ```
 */
export function syntheticIsDry(
  { quota, }: { readonly quota: QuotaSnapshot; },
): boolean {
  /**
   Weekly budget, the limit that actually emptied and the one every reading has.
   */
  const { percentRemaining, } = quota.weekly;

  if (percentRemaining <= 0)
    return true;

  /**
   Rolling window, when the reply carried one.
   */
  const { fiveHour, } = quota;

  // No window in the reply: the weekly budget has already decided.
  if (fiveHour === undefined)
    return false;

  /**
   Rolling window, whose emptiness is stated two ways.
   */
  const {
    limited,
    remaining,
  } = fiveHour;

  if (limited)
    return true;

  return remaining <= 0;
}

/**
 Whether Hyper's balance says nothing more can be bought there.

 NO MARGIN ABOVE ZERO. What one call costs has not been measured, so any
 cushion would be a number nobody established. A balance too small for the
 next call surfaces as a refusal at the wire, which the caller ORs into the
 dryness it passes back in.

 @param credits - most recent balance reading

 @returns Whether that reading leaves nothing buyable

 @example
 ```ts
 const dry = hyperIsDry({ credits, },);
 ```
 */
export function hyperIsDry(
  { credits, }: { readonly credits: HyperCredits; },
): boolean {
  return credits.balance <= 0;
}

/**
 Whether OpenRouter's credits say nothing more can be bought there.

 THE SAME RULE AS HYPER'S, for the same reason: a balance too small for the
 next call answers `402` at the wire, and that refusal holds the provider
 out through the budget layer.

 @param credits - most recent credits reading

 @returns Whether that reading leaves nothing buyable

 @example
 ```ts
 const dry = openRouterIsDry({ credits, },);
 ```
 */
export function openRouterIsDry(
  { credits, }: { readonly credits: OpenRouterCredits; },
): boolean {
  return credits.remainingUsd <= 0;
}

/**
 Balance at or under which Bedrock reads dry, in USD: what calls already
 started can still spend once the reading says the credit is gone.

 MEASURED, NOT PICKED (ledger P1, 2026-09-28). The meter is read once a
 60 s freshness window, and a call started inside one lands after it, as
 late as its stream runs. The most Bedrock spend any span of one window plus
 the longest Bedrock stream on record (363,790 ms) held, over every pass-run
 log (`p1-window-measure2-runs.mjs`), is 1.3245 USD over 822 calls
 (XingZ628). Both seated Gemma models now carry a 60 s stream bound, so that
 span over-covers them; the owner's card is the side an under-read reaches,
 and a margin left unspent only sends the last calls to OpenRouter.
 */
export const BEDROCK_DRY_MARGIN_USD = 1.33;

/**
 Whether Bedrock's ledger says nothing more can be bought there.
 THE SAME RULE AS THE OTHER TWO BALANCES, on a balance this package keeps
 itself: past the owner's credit the account would bill the owner's card,
 which the owner said never to reach ("I will NEVER top it up"). DRY WITH
 {@link BEDROCK_DRY_MARGIN_USD} STILL LEFT, which the calls in flight when
 the reading went stale can spend.

 @param credits - most recent ledger reading

 @returns Whether that reading leaves nothing buyable

 @example
 ```ts
 const dry = bedrockIsDry({ credits, },);
 ```
 */
export function bedrockIsDry(
  { credits, }: { readonly credits: BedrockCredits; },
): boolean {
  return credits.remainingUsd <= BEDROCK_DRY_MARGIN_USD;
}

/**
 Balance at or under which DeepSeek reads dry, in USD.

 NO MARGIN ABOVE ZERO, THE REASONING OF HYPER'S AND OPENROUTER'S FLOORS. What
 one call costs on this route has not been measured in a keyed run, so any
 cushion would be a number nobody established. A call costs at most a few
 cents at the peak rates the card prices (the measured cap of the card is
 13,082 completion tokens, 0.016 USD at 1.2 USD per million), against a balance
 given to use "as you wish", and a balance too small for the next call
 surfaces as `402 Insufficient Balance` at the wire
 (`https://api-docs.deepseek.com/quick_start/error_codes`, read 2026-10-22),
 which the router passes to `markRefused` as a payment refusal, and the budget layer then holds the provider dry until
 the
 meter's level moves (`provider-budget.ts`). Bedrock alone
 carries a margin, because its overshoot reaches the owner's card; DeepSeek's
 money is prepaid.
 */
export const DEEPSEEK_DRY_FLOOR_USD = 0;

/**
 Decimal places the Xiaomi spend field is printed to.
 */
const XIAOMI_SPEND_PLACES = 4;

/**
 Whether DeepSeek's balance says nothing more can be bought there.

 DRY WHEN DEEPSEEK SAYS IT CANNOT SERVE (`is_available` false) OR THE BALANCE
 IS AT THE FLOOR, whichever reads first: the flag is DeepSeek's own verdict on
 the account and the figure is the money behind it.

 @param balance - most recent balance reading

 @returns Whether that reading leaves nothing buyable

 @example
 ```ts
 const dry = deepSeekIsDry({ balance, },);
 ```
 */
export function deepSeekIsDry(
  { balance, }: { readonly balance: DeepSeekBalance; },
): boolean {
  return (!balance.isAvailable) || (balance.totalUsd <= DEEPSEEK_DRY_FLOOR_USD);
}

/**
 Whether Xiaomi's meter says nothing more can be bought there: never.

 XIAOMI DOCUMENTS NO BALANCE ENDPOINT, so its meter has nothing to be dry on
 and answers wet always (`xiaomi-spend.ts`). Exhaustion is learned from a
 payment refusal at the wire, which `markRefused` holds dry.

 @param spend - most recent spend reading, which does not decide

 @returns False

 @example
 ```ts
 const dry = xiaomiIsDry({ spend, },);
 ```
 */
export function xiaomiIsDry(
  { spend, }: { readonly spend: XiaomiSpend; },
): boolean {
  // The reading is carried so the verdict and the field come off one read,
  // like every other meter; it decides nothing.
  void spend;
  return false;
}

/**
 First provider in spending order that serves a model and has budget.

 THE SEAT READER'S QUESTION AS WELL AS THE ROUTER'S: `run-seats.ts` asks
 where each judge would be served before a phase starts, because a seat that
 one provider serves too slowly for the round window is withheld only while
 that provider is the one that would take its calls.

 @param reach - providers that serve this model at all

 @param dry - which providers have nothing buyable right now

 @returns Provider that would take a call with a free slot, or none

 @example
 ```ts
 const provider = providerServing({ reach, dry, },);
 ```
 */
export function providerServing(
  {
    reach,
    dry,
  }: {
    readonly reach: ModelReach;
    readonly dry: ProviderRecord<boolean>;
  },
): ProviderName | typeof NO_PROVIDER {
  return PROVIDER_ORDER.find(function usable(provider,): boolean {
    return reach[provider] && (!dry[provider]);
  },) ?? NO_PROVIDER;
}

/**
 Decides which provider serves one call.

 @param reach - providers that serve this model at all

 @param dry - which providers have nothing buyable, budget reading and
 anything just learned at the wire taken together

 @param saturated - which providers have this model's per-model concurrency
 limit already taken, which is the overflow trigger; a provider with no such
 limit is never saturated

 @param neuralwatt - what the run knows of NeuralWatt's day, which the refusal that ends the run states the return
 time from; absent where the caller has none

 @param writing - providers whose wet reading keeps the run going (`WritingReach`); every provider where the caller
 knows no roster

 @returns Provider to call, or why none can be

 @throws {@link EveryProviderDryError} when no provider that serves a seat that writes has budget left, which ends the
 run

 @example
 ```ts
 const choice = routeProviderFor({ reach, dry, saturated, },);
 ```
 */
export function routeProviderFor(
  {
    reach,
    dry,
    saturated,
    neuralwatt,
    writing = EVERY_PROVIDER_COUNTS,
  }: {
    readonly reach: ModelReach;
    readonly dry: ProviderRecord<boolean>;
    readonly saturated: ProviderRecord<boolean>;
    readonly neuralwatt?: NeuralWattReturn;
    readonly writing?: WritingReach;
  },
): ProviderChoice {
  if (everyWritingProviderDry({
    view: dry,
    writing,
  },)) {
    // THE READING IT DECIDED ON, cited (ledger P12): this refusal said "no
    // reading cited" where the router had the whole dryness record in hand.
    // EVERY PROVIDER THAT KEEPS THE RUN GOING READS DRY HERE, whether or not it serves the model, so each clause says
    // "dry"; a wet provider that serves no seat that writes says why it does not count.
    throw new EveryProviderDryError({
      measured: `meters read ${
        PROVIDER_ORDER.map(function stateOf(provider,): string {
          return `${provider} ${meterWord({
            view: dry,
            provider,
          },)}`;
        },)
          .join(', ',)
      }; no holds read at this decision`,
      idle: idleWetProviders({
        view: dry,
        writing,
      },),
      // Conditional spread keeps the day absent where the caller has none.
      ...((neuralwatt === undefined) ? {} : { neuralwatt, }),
    },);
  }

  /**
   Providers that both serve this model and have budget, in spending order.
   */
  const usable = PROVIDER_ORDER.filter(function serves(provider,): boolean {
    return reach[provider] && (!dry[provider]);
  },);

  /**
   First usable provider, or nothing when every serving provider is dry.
   */
  const [preferred,] = usable;

  if (preferred === undefined)
    return {
      kind: 'unreachable',
      reason: modelDarkReason({
        reach,
        dry,
      },),
    };

  /**
   First usable provider with a free slot, which overflow prefers.
   */
  const unsaturated = usable.find(function hasRoom(provider,): boolean {
    return !saturated[provider];
  },);

  // A saturated provider with nobody usable behind it still takes the call:
  // a queue behind its limit is slower than the split and still buys the answer.
  return { kind: unsaturated ?? preferred, };
}

/**
 Field value naming a meter the reply did not carry, so a record shows an absent window rather than dropping its fields.
 */
const NO_WINDOW = 'none';

/**
 Renders what the first provider's meter actually said, as record fields.

 A VERDICT ALONE CANNOT BE DIAGNOSED. `wet` and `dry` say what routing did,
 not what was read: a dry reading could be an emptied weekly budget, an
 emptied rolling window, an account this provider is actively throttling, or
 a threshold in this file being wrong about a budget that was fine. Only a
 second live call separates those, and once the moment has passed there is no
 second call to make.

 EVERY LIMIT THE REPLY CARRIED, EVERY TIME, including whichever one is full. A record naming
 only the limit that emptied would leave a reader unable to watch the other
 one approach. A reply with no five-hour window (Synthetic's of 2026-10-19) prints
 `syntheticFiveHour=none` and `syntheticThrottled=none`, so the fields keep their places and a reader can tell
 an absent window from one that read zero; the readers of these records (`meter-sample-read.ts`, `meter-report-level.ts`)
 carry fields verbatim, so the older `48/50` form and `none` both read.

 A SPENT WEEKLY BUDGET STATES ITS RETURN: `syntheticWeeklyReturn=` the instant the reply names for the next regeneration
 (`nextRegenAt`, 2026-10-23T05:03:47.000Z in the reply of that day, where the next regeneration returns 2% of the limit), so a dry
 record says when it can end instead of leaving a reader to ask the provider. A reading with weekly budget left prints no such field.
 ONLY AN INSTANT AS THE LOGGER WRITES ONE IS PRINTED (`isIsoStampText`); the reader drops any other value and keeps no raw string,
 and a window with no `nextRegenAt`, or a snapshot built another way whose value is not one, prints `syntheticWeeklyReturn=unreadable`,
 so no string can add a field or a line to the record. THE PRINTED INSTANT PASSES A SPELLING CHECK, NOT A PLAUSIBILITY CHECK: a past or
 far-future instant (`1970-01-01T00:00:00.000Z`, `+275760-09-13T00:00:00.000Z`) prints as given.

 @param quota - snapshot the dryness verdict was read from

 @returns `key=value` tokens, no value carrying a space

 @example
 ```ts
 syntheticMeterLevel({ quota, },);
 // => ['syntheticWeekly=97%', 'syntheticFiveHour=48/50', 'syntheticThrottled=no',]
 // => ['syntheticWeekly=97%', 'syntheticFiveHour=none', 'syntheticThrottled=none',] (no five-hour window)
 // => ['syntheticWeekly=0%', 'syntheticFiveHour=none', 'syntheticThrottled=none', 'syntheticWeeklyReturn=2026-10-23T05:03:47.000Z',]
 ```
 */
export function syntheticMeterLevel(
  { quota, }: { readonly quota: QuotaSnapshot; },
): readonly string[] {
  /**
   Rolling window, when the reply carried one, and weekly budget, either of which emptying is a dry meter.
   */
  const {
    fiveHour,
    weekly,
  } = quota;

  /**
   The weekly field, which every reply has.
   */
  const weeklyField = `syntheticWeekly=${String(weekly.percentRemaining,)}%`;

  /**
   The weekly return, stated only where the weekly budget is spent.
   */
  const returnFields = (weekly.percentRemaining <= 0)
    ? [
      `syntheticWeeklyReturn=${
        ((weekly.nextRegenAt !== undefined) && isIsoStampText({ text: weekly.nextRegenAt, },))
          ? weekly.nextRegenAt
          : 'unreadable'
      }`,
    ]
    : [];

  if (fiveHour === undefined)
    return [
      weeklyField,
      `syntheticFiveHour=${NO_WINDOW}`,
      `syntheticThrottled=${NO_WINDOW}`,
      ...returnFields,
    ];

  return [
    weeklyField,
    `syntheticFiveHour=${String(fiveHour.remaining,)}/${String(fiveHour.max,)}`,
    `syntheticThrottled=${fiveHour.limited ? 'yes' : 'no'}`,
    ...returnFields,
  ];
}

/**
 Renders what the second provider's meter actually said, as record fields.

 ONE NUMBER, because this provider reports one. Read back later, a recorded
 balance of zero is what separates a provider that was genuinely empty from a
 threshold here that was wrong about a balance that was not.

 @param credits - balance the dryness verdict was read from

 @returns `key=value` tokens, no value carrying a space

 @example
 ```ts
 hyperMeterLevel({ credits, },);
 // => ['hyperBalance=0',]
 ```
 */
export function hyperMeterLevel(
  { credits, }: { readonly credits: HyperCredits; },
): readonly string[] {
  return [`hyperBalance=${String(credits.balance,)}`,];
}

/**
 Renders what the third provider's meter actually said, as record fields.

 WHAT IS LEFT, IN USD, TO TWO PLACES. The provider reports purchased and used
 to nine decimals; a record field is for watching a balance approach zero
 across readings, and cents are the unit the owner tops up in.

 @param credits - credits the dryness verdict was read from

 @returns `key=value` tokens, no value carrying a space

 @example
 ```ts
 openRouterMeterLevel({ credits, },);
 // => ['openrouterUsd=57.62',]
 ```
 */
export function openRouterMeterLevel(
  { credits, }: { readonly credits: OpenRouterCredits; },
): readonly string[] {
  /**
   What is left, which is the one number the record watches.
   */
  const { remainingUsd, } = credits;

  return [`openrouterUsd=${remainingUsd.toFixed(2,)}`,];
}

/**
 Renders what the fourth provider's meter actually said, as record fields.
 WHAT IS LEFT, IN USD, TO TWO PLACES, as for OpenRouter; the figure is the
 ledger's, not the account's, which is worth remembering when reading one.

 @param credits - ledger reading the dryness verdict was read from

 @returns `key=value` tokens, no value carrying a space

 @example
 ```ts
 bedrockMeterLevel({ credits, },);
 // => ['bedrockUsd=198.50', 'bedrockReckonedUsd=0.20',]
 ```
 */
export function bedrockMeterLevel(
  { credits, }: { readonly credits: BedrockCredits; },
): readonly string[] {
  /**
   What is left, which is the one number the record watches.
   */
  const {
    remainingUsd,
    reckonedUsd,
  } = credits;

  // THE RECKONED SHARE BESIDE IT (ledger P1): how much of the spend behind
  // that number rests on bounds and estimates rather than reported usage.
  return [
    `bedrockUsd=${remainingUsd.toFixed(2,)}`,
    `bedrockReckonedUsd=${reckonedUsd.toFixed(2,)}`,
  ];
}

/**
 Renders what the DeepSeek meter actually said, as record fields.

 WHAT IS LEFT, IN USD, TO TWO PLACES, as for OpenRouter: the unit the owner
 tops up in. The availability flag is not a field of its own; a false flag
 reads dry and the balance beside it shows what the account held.

 @param balance - balance the dryness verdict was read from

 @returns `key=value` tokens, no value carrying a space

 @example
 ```ts
 deepSeekMeterLevel({ balance, },);
 // => ['deepseekUsd=9.96',]
 ```
 */
export function deepSeekMeterLevel(
  { balance, }: { readonly balance: DeepSeekBalance; },
): readonly string[] {
  return [`deepseekUsd=${balance.totalUsd
    .toFixed(2,)}`,];
}

/**
 Renders what the Xiaomi meter actually said, as record fields.

 THE RUN'S OWN PRICED SPEND, TO FOUR PLACES: a call on this model costs
 hundredths of a cent, so two places would read zero for a long time. It is
 what this process spent, not what the account holds, which is worth
 remembering when reading one.

 @param spend - spend the verdict was read from

 @returns `key=value` tokens, no value carrying a space

 @example
 ```ts
 xiaomiMeterLevel({ spend, },);
 // => ['xiaomiUsdSpent=0.0042',]
 ```
 */
export function xiaomiMeterLevel(
  { spend, }: { readonly spend: XiaomiSpend; },
): readonly string[] {
  return [`xiaomiUsdSpent=${spend.spentUsd
    .toFixed(XIAOMI_SPEND_PLACES,)}`,];
}

//endregion Budget routing
