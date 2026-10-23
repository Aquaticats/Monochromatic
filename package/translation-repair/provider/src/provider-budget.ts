import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { monotonicMs, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { BedrockClient, } from './bedrock-client.ts';
import type { DeepSeekClient, } from './deepseek-client.ts';
import type { HyperClient, } from './hyper-client.ts';
import type { NeuralWattClient, } from './neuralwatt-client.ts';
import type { OpenRouterClient, } from './openrouter-client.ts';
import type { XiaomiClient, } from './xiaomi-client.ts';
import {
  otherProviders,
  PROVIDER_ORDER,
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ModelClient, } from './chat-contract.ts';
import type { NeuralWattReturn, } from './budget-routing.ts';
import {
  NEURALWATT_RETURN_MARGIN_MS,
  type NeuralWattCause,
} from './neuralwatt-return-hold.ts';
import { createNeuralWattReturnMark, } from './neuralwatt-return-mark.ts';
import {
  type MeterState,
  readEveryMeter,
  routesAsDry,
} from './provider-meters.ts';

//region Provider budget
// ASKS FOR METERS, NOT CLIENTS. Every parameter is narrowed to the one method
// this file calls, so a caller can hand it a meter without a client and a test
// does not have to fake a chat surface to check an arithmetic.

// WHETHER EACH PROVIDER STILL HAS MONEY, cached, and correctable by what the
// wire actually says.
//
// Routing exists because a pass exhausted one provider's weekly credit and 866
// of 875 lost voices carried a single HTTP 429. Routing away from an exhausted
// provider needs to know it is exhausted, and there are two ways to learn that:
// ask the meter, or be refused. This reads both.
//
// AN UNREADABLE METER READS AS WET, NOT DRY, and that is the decision this file
// turns on. A budget endpoint that times out is a monitoring failure; treating
// it as exhaustion would convert it into an outage, stopping calls that would
// have succeeded. The owner's instruction is explicit that provider trouble is
// normal and the pipeline must not fail on it. Being wrong the other way costs
// one refused call, which the router already recovers from by failing over.
//
// ONE READING IS SHARED BY EVERY CALLER WAITING ON IT, which the first shape
// got wrong. The staleness check and the stamp sat on either side of the await,
// so every call arriving while a read was in flight saw the old stamp and
// started its own. MEASURED on the 2026-08-24 producer calibration: 158 quota
// reads and 158 credit reads in 46.5 minutes, against a 60-second window that
// should allow about 46. That is the same shape as the saturation race in
// `provider-router.ts`: state checked before an await and written after it.
//
// The fix collapses "fresh" and "in flight" into one idea. A reading STARTED
// inside the freshness window is the reading everyone uses, so the stamp goes
// on before the await rather than after and there is nothing left to race.
//
// A REFUSAL IS STICKIER THAN A METER READING. The meter can lag behind a 429 by
// its own refresh interval, so a provider that just refused us is held dry for
// a cooldown rather than until the next read, which could clear it immediately
// and walk straight back into the same wall.

/**
 How long a budget reading is trusted before it is taken again.

 NOT A MEASUREMENT of any provider's meter latency. It is short enough that
 an exhausted budget is noticed within a stage rather than a whole pass, and
 long enough that a fan-out of slices does not spend a meter read per call.
 */
const BUDGET_FRESH_MS = 60_000;

/**
 How long a provider stays dry after refusing us, whatever its meter says.

 ONE-DIRECTIONAL: it can only hold a provider OUT, never bring one back in. A
 meter reporting exhaustion keeps it out past the cooldown on its own.
 */
const REFUSAL_COOLDOWN_MS = 300_000;

/**
 How long a provider stays out after refusing us while its meter reads wet.

 A 429 FROM A WET PROVIDER IS A CONCURRENCY LIMIT, NOT EXHAUSTION. The pin
 pass of 2026-09-02 held Synthetic out for the whole cooldown on a
 burst of 429s while its meter read 2729 of 2750, and two such holds ended the
 pass for every remaining entry. The bursts measured there lasted 31 s
 (01:39:20 to 01:39:51 UTC), 3 s (01:40:32 to 01:40:35 UTC) and 2 s
 (01:54:06 to 01:54:08 UTC).

 TIED TO THE FRESHNESS WINDOW rather than picked: the reading that excused
 the refusal is trusted for this long, so the hold expires with it, and the
 window is longer than every burst measured.

 UNLESS THE REFUSAL NAMES ITS RETURN: a wait the refusal itself states holds
 the provider out past this backoff, whatever the meter reads (`markRefused`).
 */
const RATE_LIMIT_BACKOFF_MS = BUDGET_FRESH_MS;

/**
 Logger root for the budget layer.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Which providers currently have NO budget to spend, keyed by name.

 @example
 ```ts
 const view: BudgetView = { synthetic: false, hyper: true, openrouter: false, };
 ```
 */
export type BudgetView = ProviderRecord<boolean>;

/**
 One reading of every meter: the routed bits and the states they came from.

 BOTH KEPT, because a refusal needs the state (a wet meter makes the refusal
 a rate limit; an unreadable one keeps it sticky) while routing needs only
 the bit.

 @internal
 */
type MeterReading = {
  /**
   Routed dryness of each provider.
   */
  readonly view: BudgetView;

  /**
   What each meter said, or that it said nothing.
   */
  readonly states: ProviderRecord<MeterState>;

  /**
   Every number each meter reported, as the `METERS` line prints them,
   empty for a meter that reported none.
   WHAT A PAYMENT REFUSAL IS JUDGED AGAINST: a provider that answered 402
   reads dry until this text changes, since the balance that refused is the
   balance still there until someone adds to it.
   */
  readonly levels: ProviderRecord<string>;
};

/**
 Cached budget state, correctable by what a refused call reported.

 @example
 ```ts
 const budgets: ProviderBudgets = createProviderBudgets({ synthetic, hyper, openrouter, },);
 ```
 */
export type ProviderBudgets = {
  /**
   Current view, re-reading the meters when the cached one has aged out.
   */
  readonly read: (args: { readonly signal: AbortSignal; },) => Promise<BudgetView>;

  /**
   Records that a provider refused us, re-reading its meter at once and
   holding it out for the cooldown when the meter agrees or cannot be read;
   when the meter still reads wet, for the rate-limit backoff if another
   provider reads wet too, and for no time if none does.
   */
  readonly markRefused: (args: {
    readonly provider: ProviderName;
    readonly signal: AbortSignal;

    /**
     Wait the refusal named for the provider's return, zero or absent when
     it named none; the hold lasts at least this long whatever the meter
     reads.
     */
    readonly statedWaitMs?: number;

    /**
     Whether the refusal said the balance cannot pay for the call (HTTP
     402); the provider then reads dry until its meter moves, however wet
     the meter reads, and no timed hold is placed for it.
     */
    readonly paymentRequired?: boolean;
  },) => Promise<void>;

  /**
   How much longer each provider is held out by a refusal, zero when it is
   not, so a caller facing only held providers can wait out the shortest
   hold rather than declare the run over.
   */
  readonly holds: (args?: { readonly returnBoundMs?: number; },) => ProviderRecord<number>;

  /**
   What the run knows of NeuralWatt's day, which the refusal that ends the run states when the free energy returns
   from: the time of day the run decided its day starts at, that the client never learned one, or that NeuralWatt is
   off. ABSENT ON A VIEW THAT KNOWS NOTHING OF NEURALWATT, which the refusal then words as "not decided".
   */
  readonly neuralwattReturn?: () => NeuralWattReturn;

  /**
   Why NeuralWatt reads dry as the last reading found it (`ProviderBudgetsWithStates`); ABSENT ON A VIEW THAT KNOWS
   NOTHING OF NEURALWATT, whose refusal then states no cause.
   */
  readonly neuralwattDryness?: NeuralWattCause['neuralwattDryness'];

  /**
   Whether the hold that keeps NeuralWatt out is its day start and not a refusal's; absent on a view that knows nothing
   of NeuralWatt, which then names the day start as before.
   */
  readonly neuralwattHeldByReturn?: NeuralWattCause['neuralwattHeldByReturn'];
};

/**
 The budget view with the meter states beside it, which is what the run
 builds: the router needs only the view, the holds and the correction
 (`ProviderBudgets`), and a seat reading that names how each provider reads
 needs the states too.

 @example
 ```ts
 const budgets: ProviderBudgetsWithStates = createProviderBudgets({ synthetic, hyper, openrouter, },);
 ```
 */
export type ProviderBudgetsWithStates = ProviderBudgets & NeuralWattCause & {
  /**
   What each meter said in the reading `read` decides on, re-reading the
   meters when the cached one has aged out: `wet` and `dry` for a meter that
   answered, `unreadable` for one that did not.

   BESIDE THE VIEW, NOT IN IT: the view is one routed bit per provider, in
   which an unreadable meter already counts as spendable, so a seat reading
   that prints the bit alone prints `wet` for a meter that said nothing. A
   hold or a payment refusal changes the routed bit and never this state.
   */
  readonly readStates: (args: { readonly signal: AbortSignal; },) => Promise<ProviderRecord<MeterState>>;

  /**
   What the run knows of NeuralWatt's day: off with no NeuralWatt client, unanchored while its client has decided no
   day start, and the decided time of day once it has. READ OFF THE CLIENT'S DECISION, not off the `METERS` text.
   */
  readonly neuralwattReturn: () => NeuralWattReturn;
};

/**
 The reading before any meter has answered: nothing is dry.

 @returns Spendable

 @example
 ```ts
 const view = providerRecord({ of: spendableBeforeReading, },);
 ```
 */
function spendableBeforeReading(): boolean {
  return false;
}

/**
 The level before any meter has answered: no number at all.

 @returns Empty

 @example
 ```ts
 const levels = providerRecord({ of: noLevelBeforeReading, },);
 ```
 */
function noLevelBeforeReading(): string {
  return '';
}

/**
 Whether a provider last refused us for payment, and the meter level it
 read then.

 @example
 ```ts
 const mark: PaymentMark = { marked: true, level: 'openrouterUsd=0.01', };
 ```
 */
type PaymentMark = {
  marked: boolean;
  level: string;
};

/**
 The mark before any provider has refused us for payment: none.

 @returns Unmarked

 @example
 ```ts
 const marks = providerRecord({ of: unmarkedPayment, },);
 ```
 */
function unmarkedPayment(): PaymentMark {
  return {
    marked: false,
    level: '',
  };
}

/**
 The state before any meter has answered: nothing was read.

 @returns Unreadable

 @example
 ```ts
 const states = providerRecord({ of: unreadBeforeReading, },);
 ```
 */
function unreadBeforeReading(): MeterState {
  return 'unreadable';
}

/**
 The hold of a run that holds no provider out for a reason its meter cannot show.

 @returns Not held

 @example
 ```ts
 const held = notHeldOut();
 ```
 */
function notHeldOut(): boolean {
  return false;
}

/**
 The system clock, the wall clock outside a test.

 @returns The current instant

 @example
 ```ts
 const at = systemClock();
 ```
 */
function systemClock(): Date {
  return new Date();
}

/**
 Builds the cached budget view every provider is routed by.

 @param neuralwatt - first provider's energy meter, which is all this reads

 @param synthetic - second provider's quota meter, which is all this reads

 @param hyper - third provider's balance meter, which is all this reads

 @param bedrock - Bedrock's ledger meter, which is all this reads

 @param deepseek - DeepSeek's balance meter, which is all this reads

 @param xiaomi - Xiaomi's spend meter, which is all this reads

 @param openrouter - fourth provider's credits meter, which is all this reads

 @param freshForMs - how long one reading is trusted

 @param cooldownMs - how long a refusal holds a provider out

 @param rateLimitBackoffMs - how long a refusal on a wet meter holds a
 provider out while another provider can take the traffic

 @param heldOut - whether the run holds a provider out for a reason its meter cannot show, such as a refused
 credential; the `METERS` line prints such a provider as dry. It changes only the printed word, never the routed view
 or the states the budgets report (default: no provider is held out)

 @param now - clock, injectable so tests do not wait; `monotonicMs` by
 default, since a hold read on the system clock lasted an hour longer when
 it was set back (ledger B78)

 @param wallNow - the wall clock, read only for the time of day a spent NeuralWatt day starts again, since that day
 starts at a time of day in UTC and not after a duration; the system clock by default

 @param returnMarginMs - milliseconds added to a NeuralWatt return so the meter is read after the day has started
 (`NEURALWATT_RETURN_MARGIN_MS`)

 @returns Budget view plus the correction a refused call feeds back, and the states the meters gave

 @example
 ```ts
 const budgets = createProviderBudgets({ neuralwatt, synthetic, hyper, bedrock, openrouter, },);
 ```
 */
export function createProviderBudgets(
  {
    neuralwatt,
    synthetic,
    hyper,
    bedrock,
    deepseek,
    xiaomi,
    openrouter,
    freshForMs = BUDGET_FRESH_MS,
    cooldownMs = REFUSAL_COOLDOWN_MS,
    rateLimitBackoffMs = RATE_LIMIT_BACKOFF_MS,
    heldOut = notHeldOut,
    now = monotonicMs,
    wallNow = systemClock,
    returnMarginMs = NEURALWATT_RETURN_MARGIN_MS,
  }: {
    readonly neuralwatt?: Pick<NeuralWattClient, 'energy'> & { readonly dayAnchor?: NeuralWattClient['dayAnchor']; };
    readonly synthetic?: Pick<ModelClient, 'quotas'>;
    readonly hyper?: Pick<HyperClient, 'credits'>;
    readonly bedrock?: Pick<BedrockClient, 'credits'>;
    readonly deepseek?: Pick<DeepSeekClient, 'credits'>;
    readonly xiaomi?: Pick<XiaomiClient, 'spend'>;
    readonly openrouter?: Pick<OpenRouterClient, 'credits'>;
    readonly freshForMs?: number;
    readonly cooldownMs?: number;
    readonly rateLimitBackoffMs?: number;
    readonly heldOut?: (provider: ProviderName,) => boolean;
    readonly now?: () => number;
    readonly wallNow?: () => Date;
    readonly returnMarginMs?: number;
  },
): ProviderBudgetsWithStates {
  /**
   Last meter reading and when it was taken, stamped infinitely long ago until
   the first.

   THE PRE-READ VIEW IS NOT A PLACEHOLDER LIE. Before any meter has answered,
   nothing is known about any budget, and this file's policy for an unknown
   budget is already that it counts as spendable. A stamp infinitely long ago
   is stale on any clock, so the first call reads every meter regardless.

   NOT ZERO. The stamp was zero for never, which an epoch clock never reads;
   `monotonicMs` counts from near the process's start and reads zero in its
   first millisecond, so a reading taken then passed for no reading and every
   meter was read again inside the window (ledger B78). Nor can zero stand as
   an ordinary stamp: on this clock it is the process's start, so every call
   in the first `freshForMs` would take the pre-read view as fresh and read no
   meter, which failed nine of this file's cases when tried.
   */
  const cache: {
    startedAt: number;
    inFlight: boolean;
    reading: Promise<MeterReading>;
  } = {
    startedAt: Number.NEGATIVE_INFINITY,
    inFlight: false,
    reading: Promise.resolve({
      view: providerRecord({ of: spendableBeforeReading, },),
      states: providerRecord({ of: unreadBeforeReading, },),
      levels: providerRecord({ of: noLevelBeforeReading, },),
    },),
  };

  /**
   Which providers last refused us for payment and have not moved since,
   beside the meter level each read at that refusal; a provider reads dry
   while it is marked and its meter still reads that level.
   */
  const paidOutAt: Record<ProviderName, PaymentMark> = providerRecord({ of: unmarkedPayment, },);

  /**
   Whether a payment refusal still holds a provider dry: its meter has not
   moved since. A meter that has moved clears the mark, whichever way it
   moved, since the refusal was about the balance it read then.

   DOWNWARD IS A CLEARING MOVE, NOT A MISTAKE (checked for the ledger's P13,
   2026-09-28). Of 230 payment refusals in the run logs, 198 came from
   OpenRouter while its meter read wet, at 0.01 to 1.94 USD, and they refuse
   what the balance leaves once the calls in flight have reserved their
   share: 8 bodies name `in_flight_budget_exhausted` ("Retry after in-flight
   requests settle"), and the rest ask for up to 131,072 tokens and are told
   1,844 to 84,651 are affordable. A settling call lowers the balance by what
   it cost and frees the larger reservation it held, so holding the mark
   until the balance rises would keep OpenRouter dry for the rest of a run
   that can still afford smaller calls. The cost measured the same day: 28
   refusals came back after a downward move, each a refused call that bought
   nothing.

   @param provider - provider to check

   @param levels - what every meter reads now

   @returns Whether the provider reads dry for payment

   @example
   ```ts
   const dry = paidOut({ provider: 'openrouter', levels, },);
   ```
   */
  function paidOut(
    {
      provider,
      levels,
    }: {
      readonly provider: ProviderName;
      readonly levels: ProviderRecord<string>;
    },
  ): boolean {
    /**
     The mark and the level at the refusal.
     */
    const refused = paidOutAt[provider];
    if (!refused.marked)
      return false;
    if (refused.level === levels[provider])
      return true;
    refused.marked = false;
    return false;
  }

  /**
   Until when each provider is held out by a refusal, zero for never.
   */
  const heldUntil: Record<ProviderName, number> = {
    neuralwatt: 0,
    synthetic: 0,
    hyper: 0,
    bedrock: 0,
    deepseek: 0,
    xiaomi: 0,
    openrouter: 0,
  };

  /**
   When NeuralWatt's spent day starts again, as the last meter reading named it.
   */
  const returns = createNeuralWattReturnMark({ now, },);

  /**
   How much longer a provider's refusal holds it out, zero when it does not.

   @param provider - provider to check

   @returns Milliseconds of hold left

   @example
   ```ts
   const left = holdLeft({ provider: 'hyper', },);
   ```
   */
  function holdLeft(
    { provider, }: { readonly provider: ProviderName; },
  ): number {
    /**
     When its hold ends.
     */
    const until = heldUntil[provider];

    if (until === 0)
      return 0;
    return Math.max(
      0,
      until - now(),
    );
  }

  /**
   The hold left on one provider, in the shape a provider record is built from.

   @param provider - provider to check

   @returns Milliseconds of hold left

   @example
   ```ts
   const holds = providerRecord({ of: holdLeftOn, },);
   ```
   */
  function holdLeftOn(provider: ProviderName,): number {
    return holdLeft({ provider, },);
  }

  /**
   Reads every meter once, for everyone waiting on this reading.

   THE FIRST CALLER'S SIGNAL GOVERNS, which is a real consequence worth
   naming rather than hiding. If that caller aborts, every read rejects,
   each provider reports as spendable, and the reading resolves WET for
   every sharer. That is this file's answer for an unreadable meter anyway,
   and the router still recovers a real refusal through failover.

   @param signal - abort signal of whichever call started this reading

   @returns Every provider's dryness, unreadable meters counting as spendable

   @example
   ```ts
   cache.reading = takeReading({ signal, },);
   ```
   */
  async function takeReading(
    { signal, }: { readonly signal: AbortSignal; },
  ): Promise<MeterReading> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: takeReading.name,
      l,
    },);

    /**
     Every meter's record, keyed by provider, read together.
     */
    const meters = await readEveryMeter({
      ...((neuralwatt === undefined) ? {} : { neuralwatt, }),
      ...((synthetic === undefined) ? {} : { synthetic, }),
      ...((hyper === undefined) ? {} : { hyper, }),
      ...((bedrock === undefined) ? {} : { bedrock, }),
      ...((deepseek === undefined) ? {} : { deepseek, }),
      ...((xiaomi === undefined) ? {} : { xiaomi, }),
      ...((openrouter === undefined) ? {} : { openrouter, }),
      wall: {
        now: wallNow,
        marginMs: returnMarginMs,
      },
      signal,
    },);

    // NEURALWATT'S SPENT DAY NAMES ITS RETURN (XingZ6019, 2026-10-22: dry 48 s before its day started, the entry stopped
    // short of its bench floor). The return is a hold the waits already wait out (`holds`), and the cached reading is
    // read again at it (`currentReading`), whatever its age, because it says dry and the day is no longer the one it read.
    returns.replace({ dryness: meters.neuralwatt
      .dryness
      ?? { kind: 'wet', }, },);

    // INFO RATHER THAN DEBUG, because this line is the only record that a
    // provider was AVAILABLE at a given moment, and a run does not record debug.
    // It carries the meter state rather than the routed bit, so an unreadable
    // meter cannot be read back later as a provider that was up.
    // Refusals alone cannot measure a duty cycle: `NoProviderForModelError`
    // appears only where something happened to ask for a model that provider
    // serves, so a quiet period reads identically to a healthy one. This is
    // bounded by the freshness window rather than by call volume, so it costs
    // about one line a minute however busy the run is.
    //
    // IT CARRIES THE NUMBERS TOO, added after reading one back. `hyper=dry`
    // alone could not be told from a threshold in `budget-routing.ts` being
    // wrong, and answering that took a live call to the provider, which is not
    // available for a moment that has already passed.
    //
    // STATES FIRST, IN PROVIDER ORDER, THEN EVERY NUMBER, so the reader in
    // `meter-sample-read.ts` finds each provider's state by name and older
    // lines without the third state still read.
    /**
     What every meter reported, joined as `paidOut` compares it with the level at a payment refusal.
     */
    const levelsOf = providerRecord({
      of: function levelOf(provider,): string {
        /**
         This provider's meter record.
         */
        const meter = meters[provider];
        return meter.fields
          .join(' ',);
      },
    },);
    /**
     Each provider's state, as `name=state`.
     */
    const stateFields = PROVIDER_ORDER.map(function stateOf(provider,): string {
      /**
       This provider's meter record.
       */
      const meter = meters[provider];
      // A PROVIDER THE RUN HAS HELD OUT PRINTS DRY, whatever its meter says,
      // because the line is the record of when a provider was AVAILABLE and a
      // held provider was not. CuspariaKLSY22 printed `bedrock=wet` on all 27
      // lines while its `JUDGE SEATS` lines read `bedrock=dry` from the second:
      // the key was refused on the first call, the meter is the configured
      // credit less the package's own ledger, and a refused key cannot move
      // it. Three things hold a provider out beyond its meter, and the router
      // and the seat readers fold all three in: a refused credential, a timed
      // hold after a refusal and a refusal for payment. ONLY THE PRINTED WORD
      // MOVES: the state `readStates` returns stays what the meter said,
      // because `markRefused` judges a refusal against it (a wet meter makes
      // it a rate limit).
      if (heldOut(provider,) || (holdLeft({ provider, },) > 0)
        || paidOut({
          provider,
          levels: levelsOf,
        },))
        return `${provider}=dry`;
      return `${provider}=${meter.state}`;
    },);
    /**
     Every number every meter reported, in provider order.
     */
    const levelFields = PROVIDER_ORDER.flatMap(function fieldsOf(provider,): readonly string[] {
      /**
       This provider's meter record.
       */
      const meter = meters[provider];
      return meter.fields;
    },);
    rl.info(`METERS ${[
      ...stateFields,
      ...levelFields,
    ].join(' ',)}`,);
    return {
      view: providerRecord({
        of: function dryOf(provider,): boolean {
          /**
           This provider's meter record.
           */
          const meter = meters[provider];
          return routesAsDry({ state: meter.state, },);
        },
      },),
      states: providerRecord({
        of: function stateOf(provider,): MeterState {
          /**
           This provider's meter record.
           */
          const meter = meters[provider];
          return meter.state;
        },
      },),
      levels: levelsOf,
    };
  }

  /**
   Starts a reading now, whatever the cache's age, for everyone who reads
   after it.

   @param signal - abort signal of the call forcing the read

   @returns The fresh reading

   @example
   ```ts
   const reading = await readNow({ signal, },);
   ```
   */
  async function readNow(
    { signal, }: { readonly signal: AbortSignal; },
  ): Promise<MeterReading> {
    // A reading still in flight is younger than any refusal arriving now, so a
    // burst of refusals shares it rather than starting one meter call each.
    if (cache.inFlight)
      return await cache.reading;
    /**
     The in-flight mark, cleared when this read's scope ends however it ends.
     */
    using flight = markInFlight();
    cache.startedAt = now();
    cache.reading = takeReading({ signal, },);
    return await cache.reading;
  }

  /**
   Marks a reading as in flight until the scope that took it ends, however
   that scope ends.

   @returns Disposable that clears the mark

   @example
   ```ts
   using flight = markInFlight();
   ```
   */
  function markInFlight(): Disposable {
    cache.inFlight = true;
    return {
      [Symbol.dispose]() {
        cache.inFlight = false;
      },
    };
  }

  /**
   Whether NeuralWatt's day has started again since the reading that named its return, which the cached reading does not
   know.

   @returns Whether the return has passed

   @example
   ```ts
   if (neuralwattHasReturned())
     await readNow({ signal, },);
   ```
   */
  function neuralwattHasReturned(): boolean {
    return returns.passed();
  }

  /**
   The meter reading the cache holds, taken afresh when it has aged out.

   @param signal - abort signal of the call asking

   @returns Reading shared with every other call that arrived inside the same window

   @example
   ```ts
   const { states, } = await currentReading({ signal, },);
   ```
   */
  async function currentReading(
    { signal, }: { readonly signal: AbortSignal; },
  ): Promise<MeterReading> {
    // STAMPED BEFORE THE READ IS STARTED, never after it returns. The stamp
    // says when this reading BEGAN, so a call arriving while it is still in
    // flight sees a fresh stamp and waits on the same promise instead of
    // starting a second read of every meter.
    if (((now() - cache.startedAt) >= freshForMs) || neuralwattHasReturned())
      await readNow({ signal, },);

    return await cache.reading;
  }

  return {
    neuralwattReturn: function neuralwattReturn(): NeuralWattReturn {
      if (neuralwatt === undefined)
        return { kind: 'off', };
      // A stub client that carries no anchor decided none.
      /**
       What the client has decided, without asking it to decide.
       */
      const anchor = neuralwatt.dayAnchor?.() ?? { known: false, };
      if (!anchor.known)
        return { kind: 'unanchored', };
      return {
        kind: 'at',
        msOfDay: anchor.msOfDay,
      };
    },

    readStates: async function readStates({ signal, },): Promise<ProviderRecord<MeterState>> {
      /**
       Meter reading this call reports the states of.
       */
      const { states, } = await currentReading({ signal, },);
      return states;
    },

    read: async function read({ signal, },): Promise<BudgetView> {
      /**
       Meter reading this call is decided on, shared with every other call
       that arrived inside the same window.
       */
      const {
        view,
        levels,
      } = await currentReading({ signal, },);

      // A hold can only keep a provider OUT, never bring one back in.
      return providerRecord({
        of: function dryOrHeld(provider,): boolean {
          return view[provider]
            || (holdLeft({ provider, },) > 0)
            || paidOut({
              provider,
              levels,
            },);
        },
      },);
    },

    markRefused: async function markRefused({
      provider,
      signal,
      statedWaitMs = 0,
      paymentRequired = false,
    },): Promise<void> {
      /**
       Logger pre-tagged with this function's name.
       */
      const rl = tagged({
        tag: markRefused.name,
        l,
      },);

      // THE METER IS ASKED AT ONCE, freshness window or not. A refusal from a
      // provider whose meter still reads wet is its concurrency limit, which
      // clears in seconds, and holding it out for the exhaustion cooldown is
      // what ended the pin pass of 2026-09-02. A meter that agrees, or
      // one that cannot be read, keeps the refusal stickier than the reading.
      /**
       What every meter said just now.
       */
      const {
        states,
        levels,
      } = await readNow({ signal, },);
      /**
       What that provider's meter said just now.
       */
      const state = states[provider];
      if (paymentRequired) {
        paidOutAt[provider] = {
          marked: true,
          level: levels[provider],
        };
        heldUntil[provider] = now() + statedWaitMs;
        rl.info(
          `${provider}: refused us for payment while its meter reads ${state} (${levels[provider]}); `
            + `reads dry until that meter moves`,
        );
        return;
      }
      /**
       The other providers, in spending order.
       */
      const others = otherProviders({ provider, },);
      /**
       Whether some other provider reads wet, which decides whether a hold
       moves traffic anywhere.
       */
      const anotherIsWet = others.some(function isWet(other,): boolean {
        return states[other] === 'wet';
      },);
      // A HOLD MOVES TRAFFIC; WITH NOWHERE TO MOVE IT, IT ONLY HERDS. A refusal
      // on a wet meter is a concurrency limit, and holding the provider out for
      // the backoff sends the next calls to another provider, which is what
      // the hold is for. When no other provider is wet there is nothing to
      // send them to: every call waits the same hold and fires together when it
      // ends, into the same limit. Measured on XIEPT2, 2026-09-03 00:46 to
      // 00:51 UTC, Synthetic at 0% weekly and Hyper alone: 429 bursts of 80 to
      // 131 hits every 80 seconds, 95 calls waiting 60-second holds, 44 lost as
      // both-dry inside the next hold, and no slice settled in five minutes.
      // With no hold, each call keeps its own jittered retry ladder in the
      // transport layer, which is the pacing a concurrency limit wants.
      /**
       How long this refusal would hold the provider out on its meter alone.
       */
      const meterHoldMs = (state === 'wet')
        ? (anotherIsWet ? rateLimitBackoffMs : 0)
        : cooldownMs;

      // A REFUSAL THAT NAMES ITS RETURN IS NOT A CONCURRENCY LIMIT, whatever
      // the meter reads. Hyper's daily limit on Huasheng, 2026-09-07, 17:00 to
      // 19:54 UTC: "You've hit your daily rate limit. Please try again in
      // 2h25m18s" on a meter reading wet at 909 credits, held out 60 s at a
      // time: 831 holds and 2,693 refused attempts before the named instant,
      // and four consolidation chunks of 75 min each settling on nobody.
      /**
       How long this refusal holds the provider out: the meter's hold or the
       wait the refusal named, whichever is longer.
       */
      const holdMs = Math.max(
        meterHoldMs,
        statedWaitMs,
      );
      heldUntil[provider] = now() + holdMs;
      /**
       The other meters' states, for the line.
       */
      const otherStates = others.map(function stateOf(other,): string {
        return `${other} ${states[other]}`;
      },);
      /**
       Clause naming the refusal's own wait, empty when it named none.
       */
      const naming = (statedWaitMs > 0)
        ? `, naming its return in ${String(statedWaitMs,)}ms`
        : '';
      rl.info(
        `${provider}: refused us while its meter reads ${state} and ${otherStates.join(', ',)}`
          + `${naming}; held out for ${String(holdMs,)}ms`,
      );
    },

    // The refusal hold, and for NeuralWatt also the time until its spent day starts again, which the cached reading
    // does not know and which only a return within the caller's bound counts for.
    holds: function holds({ returnBoundMs = Number.POSITIVE_INFINITY, }: { readonly returnBoundMs?: number; } = {},
    ): ProviderRecord<number> {
      return providerRecord({
        of: function holdOrReturn(provider: ProviderName,): number {
          return Math.max(
            holdLeft({ provider, },),
            (provider === 'neuralwatt') ? returns.leftWithinMs({ boundMs: returnBoundMs, },) : 0,
          );
        },
      },);
    },

    neuralwattDryness: returns.dryness,

    neuralwattHeldByReturn: function neuralwattHeldByReturn(
      { returnBoundMs = Number.POSITIVE_INFINITY, }: { readonly returnBoundMs?: number; } = {},
    ): boolean {
      /**
       The day start's hold, and the longest hold on NeuralWatt of either kind.
       */
      const byReturn = returns.leftWithinMs({ boundMs: returnBoundMs, },);
      return (byReturn > 0) && (byReturn >= holdLeft({ provider: 'neuralwatt', },));
    },
  };
}

//endregion Provider budget
