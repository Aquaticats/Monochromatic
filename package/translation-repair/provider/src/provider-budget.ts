import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { monotonicMs, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { BedrockClient, } from './bedrock-client.ts';
import type { DeepSeekClient, } from './deepseek-client.ts';
import type { HyperClient, } from './hyper-client.ts';
import type { NeuralWattClient, } from './neuralwatt-client.ts';
import type { OpenRouterClient, } from './openrouter-client.ts';
import type { ClaudeClient, } from './claude-client.ts';
import type { XiaomiClient, } from './xiaomi-client.ts';
import {
  otherProviders,
  PROVIDER_ORDER,
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ModelClient, } from './chat-contract.ts';
import {
  EVERY_PROVIDER_COUNTS,
  type NeuralWattReturn,
  type WritingReach,
} from './budget-routing.ts';
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
import {
  beforeAnyReading,
  createWaitCap,
  HoldBoundError,
  meterLevelsOf,
  meterStatesOf,
  neuralwattReturnOf,
  notHeldOut,
  systemClock,
} from './provider-budget-defaults.ts';
import { createPaymentMarks, } from './provider-budget-payment-marks.ts';
import {
  type UnmetFloors,
  writingProviderFloor,
} from './provider-budget-give-up-aside.ts';
import { createRefusalHolds, } from './provider-budget-refusal-holds.ts';

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
 const budgets: ProviderBudgets = createProviderBudgets({ synthetic, hyper, openrouter, maxHoldMs: 25_200_000, },);
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
   hold rather than declare the run over. NEURALWATT'S SPENT DAY IS NOT A HOLD: its day start is stated by the refusal
   that ends the run and never waited for (the owner's answer of 2026-10-29, "Never wait, relaunch").
   */
  readonly holds: () => ProviderRecord<number>;

  /**
   Records that a provider served a call, which is what tells a refusal right after a cooldown ran out (the provider
   did not come back) from one long after a provider that came back. ABSENT ON A VIEW THAT SEES NO SERVICE, whose
   providers are then never given up on.
   */
  readonly markServed?: (args: { readonly provider: ProviderName; },) => void;

  /**
   Starts an entry with no provider given up on, so a provider an entry that ended had stopped waiting for is waited for
   and asked again, and writes the entry's name where a provider is given up on. ABSENT ON A VIEW THAT HAS NO ENTRIES,
   which keeps one mark set for its whole life.
   */
  readonly beginEntry?: (args: { readonly entryId: string; },) => void;

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
};

/**
 The budget view with the meter states beside it, which is what the run
 builds: the router needs only the view, the holds and the correction
 (`ProviderBudgets`), and a seat reading that names how each provider reads
 needs the states too.

 @example
 ```ts
 const budgets: ProviderBudgetsWithStates = createProviderBudgets({
   synthetic,
   hyper,
   openrouter,
   maxHoldMs: 25_200_000,
 },);
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
   hold changes the routed bit and never this state.

   A PAYMENT REFUSAL READS `dry` HERE until the meter moves, as it does in
   the routed bit: that refusal keeps the provider dry however wet the meter
   reads and however long the hold it stated, so a reader that takes the
   states for the meters alone, and counts a provider whose hold ends as
   returning only when its meter is not dry, would otherwise wait out a hold
   for a provider that stays dry. The reading a refusal is judged against
   (`markRefused`) keeps what the meter said.
   */
  readonly readStates: (args: { readonly signal: AbortSignal; },) => Promise<ProviderRecord<MeterState>>;

  /**
   What the run knows of NeuralWatt's day: off with no NeuralWatt client, unanchored while its client has decided no
   day start, and the decided time of day once it has. READ OFF THE CLIENT'S DECISION, not off the `METERS` text.
   */
  readonly neuralwattReturn: () => NeuralWattReturn;

  /**
   The layer the run builds sees the service and the entries, which the narrower view leaves absent.
   */
  readonly markServed: NonNullable<ProviderBudgets['markServed']>;

  /**
   Starts an entry (`ProviderBudgets`).
   */
  readonly beginEntry: NonNullable<ProviderBudgets['beginEntry']>;
};

/**
 Builds the cached budget view every provider is routed by.

 @param neuralwatt - first provider's energy meter, which is all this reads

 @param synthetic - second provider's quota meter, which is all this reads

 @param hyper - third provider's balance meter, which is all this reads

 @param bedrock - Bedrock's ledger meter, which is all this reads

 @param deepseek - DeepSeek's balance meter, which is all this reads

 @param xiaomi - Xiaomi's spend meter, which is all this reads

 @param anthropic - the Claude key's spend meter, which is all this reads

 @param openrouter - fourth provider's credits meter, which is all this reads

 @param freshForMs - how long one reading is trusted

 @param cooldownMs - how long a refusal holds a provider out

 @param rateLimitBackoffMs - how long a refusal on a wet meter holds a
 provider out while another provider can take the traffic

 @param maxHoldMs - longest a wait a refusal states for itself may hold a provider out, which a run sets to its hard
 cap or, where its command has none, to a named bound with a reason (REQUIRED, with no unbounded default: a provider
 whose `Retry-After` states 99,999,999,999 seconds held that provider out for about 3,170 years): a hold past it can
 never be waited out, so a longer stated wait is cut to it and logged once for each provider with the
 value the provider stated

 @param heldOut - whether the run holds a provider out for a reason its meter cannot show, such as a refused
 credential; the `METERS` line prints such a provider as dry. It changes only the printed word, never the routed view
 or the states the budgets report (default: no provider is held out)

 @param writing - the providers whose wet reading keeps the run going (`WritingReach`): an entry's give-up of a provider
 is set aside when that provider is the only one of them not dry, since reading it dry would end the run, and they are
 the floor of the give-up where no `unmetFloors` is given (default: every provider counts)

 @param unmetFloors - the bench floors the entry's stages need: an entry's give-up of a provider is set aside when with
 that provider back no floor would be left unmet while without it one is, since waiting is then what lets the entry
 finish, and holds otherwise (default: some writing provider of `writing` can be asked)

 @param now - clock, injectable so tests do not wait; `monotonicMs` by
 default, since a hold read on the system clock lasted an hour longer when
 it was set back (ledger B78)

 @param wallNow - wall clock, read only for the time of day a spent NeuralWatt day starts again, since that day
 starts at a time of day in UTC and not after a duration; the system clock by default

 @param returnMarginMs - milliseconds added to a NeuralWatt return so the meter is read after the day has started
 (`NEURALWATT_RETURN_MARGIN_MS`)

 @returns Budget view plus the correction a refused call feeds back, and the states the meters gave

 @throws {@link HoldBoundError} when `maxHoldMs` is not a finite whole number of milliseconds greater than zero

 @example
 ```ts
 const budgets = createProviderBudgets({ neuralwatt, synthetic, hyper, bedrock, openrouter, maxHoldMs: 25_200_000, },);
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
    anthropic,
    openrouter,
    freshForMs = BUDGET_FRESH_MS,
    cooldownMs = REFUSAL_COOLDOWN_MS,
    rateLimitBackoffMs = RATE_LIMIT_BACKOFF_MS,
    maxHoldMs,
    heldOut = notHeldOut,
    writing = EVERY_PROVIDER_COUNTS,
    unmetFloors = writingProviderFloor({ writing, },),
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
    readonly anthropic?: Pick<ClaudeClient, 'spend'>;
    readonly openrouter?: Pick<OpenRouterClient, 'credits'>;
    readonly freshForMs?: number;
    readonly cooldownMs?: number;
    readonly rateLimitBackoffMs?: number;
    readonly maxHoldMs: number;
    readonly heldOut?: (provider: ProviderName,) => boolean;
    readonly writing?: WritingReach;
    readonly unmetFloors?: UnmetFloors;
    readonly now?: () => number;
    readonly wallNow?: () => Date;
    readonly returnMarginMs?: number;
  },
): ProviderBudgetsWithStates {
  // `isInteger` is false for NaN and both infinities, so one test covers every value that is not a whole number.
  if ((!Number.isInteger(maxHoldMs,)) || (maxHoldMs <= 0))
    throw new HoldBoundError({ maxHoldMs, },);

  /**
   Last meter reading and when it was taken (`beforeAnyReading`).
   */
  const cache = beforeAnyReading();

  /**
   The payment refusals of this layer: which providers last refused us for payment and have not moved since, and
   whether the meter of each still reads the level it read then.
   */
  const {
    paidOut,
    markPaid,
  } = createPaymentMarks();

  /**
   The holds refusals set, and the providers the entry in flight has given up waiting for because each refused right
   after a refusal cooldown ran out with nothing served in between (`provider-budget-refusal-holds.ts`).
   */
  const refusals = createRefusalHolds({
    now,
    l,
    writing,
    unmetFloors,
  },);

  /**
   When NeuralWatt's spent day starts again, as the last meter reading named it.
   */
  const returns = createNeuralWattReturnMark({ now, },);

  /**
   Cuts a wait a refusal states for itself to the longest hold the run can use, logging the first cut of each provider.
   */
  const cappedWait = createWaitCap({ maxHoldMs, },);

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
      ...((anthropic === undefined) ? {} : { anthropic, }),
      ...((openrouter === undefined) ? {} : { openrouter, }),
      wall: {
        now: wallNow,
        marginMs: returnMarginMs,
      },
      signal,
    },);

    // NEURALWATT'S SPENT DAY NAMES ITS RETURN (XingZ6019, 2026-10-22: dry 48 s before its day started, the entry stopped
    // short of its bench floor). The return is stated by the refusal that ends the run and is no hold, so nothing waits
    // for it (the owner's answer of 2026-10-29, "Never wait, relaunch"); the cached reading is still read again at it
    // (`currentReading`), whatever its age, because it says dry and the day is no longer the one it read.
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
    const levelsOf = meterLevelsOf({ meters, },);
    /**
     Whether a refused credential or a payment refusal keeps a provider out, whatever its meter says.

     @param provider - provider to check

     @returns Whether the provider is held out for a reason its meter cannot show
     */
    function heldApart(provider: ProviderName,): boolean {
      return heldOut(provider,) || paidOut({
        provider,
        levels: levelsOf,
      },);
    }
    /**
     Each meter's state, as the reading keeps it.
     */
    const states = meterStatesOf({ meters, },);
    /**
     What the meters alone say is dry, as the routed view starts from it.
     */
    const view = providerRecord({
      of: function dryOf(provider,): boolean {
        return routesAsDry({ state: states[provider], },);
      },
    },);
    // THE GIVE-UP DECISION IS TAKEN FROM EVERY READING: which providers the meters, a refused credential or a payment
    // refusal keep dry, apart from the holds and the give-ups, decides whether a given-up provider is set aside.
    refusals.noteReading({
      dry: providerRecord({
        of: function dryApartFromRefusals(provider,): boolean {
          return view[provider] || heldApart(provider,);
        },
      },),
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
      // it. Four things hold a provider out beyond its meter, and the router
      // and the seat readers fold all four in: a refused credential, a timed
      // hold after a refusal, a refusal for payment and a provider the entry
      // gave up on after it refused right after a cooldown ran out. ONLY THE
      // PRINTED WORD MOVES in the reading: the state it keeps stays what the
      // meter said, because `markRefused` judges a refusal against it (a wet
      // meter makes it a rate limit). `readStates` folds the payment refusal
      // in on the way out and nothing else.
      if (heldApart(provider,) || refusals.keepsOut({ provider, },))
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
      view,
      states,
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
    // NeuralWatt's day may have started again since the reading that named its return, which the cached reading does not know.
    if (((now() - cache.startedAt) >= freshForMs) || returns.passed())
      await readNow({ signal, },);

    return await cache.reading;
  }

  return {
    neuralwattReturn: function neuralwattReturn(): NeuralWattReturn {
      return neuralwattReturnOf((neuralwatt === undefined) ? {} : { neuralwatt, },);
    },

    readStates: async function readStates({ signal, },): Promise<ProviderRecord<MeterState>> {
      /**
       Meter reading this call reports the states of.
       */
      const {
        states,
        levels,
      } = await currentReading({ signal, },);

      // A PAYMENT REFUSAL IS DRY UNTIL THE METER MOVES, whatever the meter read when it was refused.
      return providerRecord({
        of: function stateOrPaidOut(provider,): MeterState {
          return paidOut({
            provider,
            levels,
          },)
            ? 'dry'
            : states[provider];
        },
      },);
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

      // A hold can only keep a provider OUT, never bring one back in. So can the entry's giving a provider up, which
      // ends with the entry and lets the router and the seat reader move on to the providers that answer, and which is set
      // aside where waiting for the provider is what lets the entry finish (`provider-budget-give-up-aside.ts`).
      return providerRecord({
        of: function dryOrHeld(provider,): boolean {
          return view[provider]
            || refusals.keepsOut({ provider, },)
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
      statedWaitMs: statedByProvider = 0,
      paymentRequired = false,
    },): Promise<void> {
      /**
       Logger pre-tagged with this function's name.
       */
      const rl = tagged({
        tag: markRefused.name,
        l,
      },);

      /**
       Wait the refusal stated, cut to the longest hold the run can use.
       */
      const statedWaitMs = cappedWait({
        provider,
        statedWaitMs: statedByProvider,
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
        markPaid({
          provider,
          level: levels[provider],
        },);
        // A PAYMENT REFUSAL IS NOT A COOLDOWN, so a cooldown after it is not judged against one before it.
        refusals.hold({
          provider,
          holdMs: statedWaitMs,
          cooldown: false,
        },);
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
      // A COOLDOWN IS THE HOLD THE LAYER GUESSED ALONE: a meter that did not say wet and a refusal that named no return.
      // A refusal right after one ran out, with nothing served in between, is the answer the wait bought, so the entry
      // stops waiting for this provider's cooldowns and treats it as unreachable until the entry ends, unless the provider
      // is the only writing provider not dry or its return would leave no bench floor unmet where without it one is
      // (`provider-budget-give-up-aside.ts`).
      refusals.hold({
        provider,
        holdMs,
        cooldown: (state !== 'wet') && (statedWaitMs === 0),
      },);
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

    // The refusal hold alone: NeuralWatt's spent day is no hold, since nothing waits for its return.
    holds: function holds(): ProviderRecord<number> {
      return providerRecord({
        of: function holdOf(provider: ProviderName,): number {
          return refusals.waitedLeft({ provider, },);
        },
      },);
    },

    markServed: refusals.noteServed,

    beginEntry: refusals.beginEntry,

    neuralwattDryness: returns.dryness,
  };
}

//endregion Provider budget
