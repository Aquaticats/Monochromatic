import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  isSuccessStatus,
  errorName,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type ModelTransport,
  ProviderHttpError,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { neuralWattQuotaUrlFor, } from './neuralwatt-catalog.ts';
import {
  NEURALWATT_QUOTA_RETRY_SECONDS,
  neuralWattSecondsAfter,
} from './neuralwatt-day.ts';
import {
  type NeuralWattPlanSpend,
  type NeuralWattQuota,
  parseNeuralWattQuota,
  planSpendBetween,
} from './neuralwatt-quota.ts';
import {
  exchangeWithRetry,
  type RetryPolicy,
} from './transient-retry.ts';

//region NeuralWatt quota session
// THE TWO QUOTA READINGS OF ONE RUN: a baseline before the first NeuralWatt
// call and the reading at the end, whose difference is the plan energy spent
// (`neuralwatt-quota.ts` says what that figure can and cannot see).
//
// THE BASELINE IS TAKEN LAZILY, the first time the run needs to know the day:
// a meter reading or a call, whichever comes first, since the day's anchor is
// the time of day of the baseline's `current_period_start`
// (`neuralwatt-day-session.ts`). A run that never reads the NeuralWatt meter
// or reaches it (a start guard, a held model) reads the quota zero times, and
// the end-of-run report says it has no starting reading rather than inventing
// one. A READING THAT FAILS NEVER BLOCKS A CALL: the quota is a report on the
// owner's plan, and the ledger and the cost-comment backstop are what keep
// calls off it. The failure is warned, with the endpoint's status or the shape
// complaint. (A day whose start then cannot be learned does stop NeuralWatt,
// but that is the day's rule and not this session's.)
//
// A FAILED BASELINE IS TRIED AGAIN, no sooner than a minute after the last
// failure on the injected clock, the pace of the day anchor keeper
// (`neuralwatt-day-anchor-keeper.ts`), and the outage is warned about once. A
// later read of the quota that the run makes for its own reasons (the anchor
// keeper's retry, the key allowance's refresh) takes the baseline's place the
// same way, since a reading is a reading and asking twice would only spend the
// endpoint's patience.
//
// A BASELINE TAKEN AFTER A FAILURE IS NOT THE RUN'S START. The comparison then
// begins at that later reading, and energy spent before it is not in the
// difference: the end report says so, with the reading's time
// (`laterStartAt` of `NeuralWattPlanSpend`), rather than present the figure as
// covering the run. A run whose every read fails says how many attempts failed
// and when the last was.

/**
 Logger root for the quota session.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Whether the starting reading was taken, and what it said.

 @example
 ```ts
 const outcome: BaselineOutcome = { taken: false, };
 ```
 */
export type BaselineOutcome =
  | {
    readonly taken: true;
    readonly quota: NeuralWattQuota;
  }
    | {
      readonly taken: false;
    };

/**
 Quota reads of one client.

 @example
 ```ts
 const session = createNeuralWattQuotaSession({ headers, baseUrl, transport, retryPolicy, now, },);
 ```
 */
export type NeuralWattQuotaSession = {
  /**
   Takes the starting reading if it has not been taken, trying again no sooner
   than a minute after a failed attempt; never throws for a failed read.
   */
  readonly ensureBaseline: (args: { readonly signal: AbortSignal; },) => Promise<BaselineOutcome>;

  /**
   Reads the quota now, for the figures that move (the key's cap). It leaves a
   taken starting reading alone; where the starting reading failed earlier and
   none has been taken since, this reading becomes the starting one.

   @throws {@link ProviderHttpError} on a non-success status

   @throws {@link import('./neuralwatt-quota.ts').NeuralWattQuotaShapeError} on a body that is not the documented shape
   */
  readonly readNow: (args: { readonly signal: AbortSignal; },) => Promise<NeuralWattQuota>;

  /**
   Reads the quota again and compares it with the starting reading, saying where
   the comparison starts where that reading was taken after a failure.
   */
  readonly planSpend: (args: { readonly signal: AbortSignal; },) => Promise<NeuralWattPlanSpend>;
};

/**
 Says what the plan had charged in a reading, for the line that names the baseline.

 @param quota - reading

 @returns Plain phrase
 */
function chargedText({ quota, }: { readonly quota: NeuralWattQuota; },): string {
  return (quota.subscription
    ?.kwhUsed
    === undefined)
    ? 'no energy charged to a plan is reported'
    : `${String(quota.subscription
      .kwhUsed,)} kWh charged this period`;
}

/**
 Builds the quota session of one client.

 @param headers - request headers, auth included

 @param baseUrl - API base address

 @param transport - HTTP the reads go over

 @param retryPolicy - transient-retry pacing

 @param now - clock the time of a reading and the spacing of baseline attempts are read from, injected so that tests
 wait for nothing

 @returns Session surface

 @example
 ```ts
 const session = createNeuralWattQuotaSession({ headers, baseUrl, transport, retryPolicy, now, },);
 ```
 */
export function createNeuralWattQuotaSession(
  {
    headers,
    baseUrl,
    transport,
    retryPolicy,
    now,
  }: {
    readonly headers: Readonly<Record<string, string>>;
    readonly baseUrl: string;
    readonly transport: ModelTransport;
    readonly retryPolicy: RetryPolicy;
    readonly now: () => Date;
  },
): NeuralWattQuotaSession {
  /**
   What the session holds: the starting reading once taken and whether it was
   taken after a failure, the attempt in flight, and the failed attempts so far.
   */
  const state: {
    baseline?: {
      readonly quota: NeuralWattQuota;
      readonly at: Date;
      readonly late: boolean;
    };
    attempting?: Promise<BaselineOutcome>;
    begun: boolean;
    failed?: {
      readonly count: number;
      readonly lastAt: Date;
    };
  } = { begun: false, };

  /**
   Reads the quota once.

   @param signal - abort signal for the read

   @returns Reading

   @throws {@link ProviderHttpError} on a non-success status

   @throws {@link import('./neuralwatt-quota.ts').NeuralWattQuotaShapeError} on a body that is not the documented shape
   */
  async function readQuota({ signal, }: { readonly signal: AbortSignal; },): Promise<NeuralWattQuota> {
    /**
     Reply from the quota endpoint.
     */
    const reply = await exchangeWithRetry({
      provider: 'neuralwatt',
      transport,
      exchange: {
        url: neuralWattQuotaUrlFor({ baseUrl, },),
        label: 'quota',
        method: 'GET',
        headers,
        signal,
      },
      policy: retryPolicy,
    },);
    if (!isSuccessStatus({ status: reply.status, },))
      throw new ProviderHttpError({
        provider: 'neuralwatt',
        status: reply.status,
        bodyText: reply.bodyText,
        summary: `NeuralWatt /quota returned HTTP ${String(reply.status,)}:`,
      },);
    return parseNeuralWattQuota({ bodyText: reply.bodyText, },);
  }

  /**
   Keeps a reading as the starting one if none is kept yet, saying where the comparison starts.

   A reading taken after a failed attempt is a late start, and is logged as one.

   @param quota - reading

   @returns Reading kept as the starting one, which is the first kept where two arrive together
   */
  function takeBaseline({ quota, }: { readonly quota: NeuralWattQuota; },): NeuralWattQuota {
    if (state.baseline !== undefined)
      return state.baseline
        .quota;

    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: takeBaseline.name,
      l,
    },);

    /**
     When the reading was taken.
     */
    const at = now();

    /**
     Whether an attempt to take the starting reading failed before this one.
     */
    const late = state.failed !== undefined;
    state.baseline = {
      quota,
      at,
      late,
    };
    rl.info(
      late
        ? `NeuralWatt quota read at ${at.toISOString()}, after the first read of the run failed: `
          + `${chargedText({ quota, },)}; the plan energy comparison starts at this later reading, and energy spent `
          + 'before it is not counted'
        : `NeuralWatt quota at the start of the run: ${chargedText({ quota, },)}`,
    );
    return quota;
  }

  /**
   Reads the quota now, for the figures that move, and keeps the reading as the starting one when the baseline failed
   earlier and none has been taken since.

   @param signal - abort signal for the read

   @returns Reading

   @throws {@link ProviderHttpError} on a non-success status

   @throws {@link import('./neuralwatt-quota.ts').NeuralWattQuotaShapeError} on a body that is not the documented shape
   */
  async function readNow({ signal, }: { readonly signal: AbortSignal; },): Promise<NeuralWattQuota> {
    /**
     The reading.
     */
    const quota = await readQuota({ signal, },);
    if (state.failed !== undefined)
      takeBaseline({ quota, },);
    return quota;
  }

  /**
   Reads the quota for the baseline, saying so when it cannot.

   @param signal - abort signal for the read

   @returns Reading, or that none was taken
   */
  async function tryBaseline({ signal, }: { readonly signal: AbortSignal; },): Promise<BaselineOutcome> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: tryBaseline.name,
      l,
    },);
    try {
      /**
       The starting reading.
       */
      const quota = await readQuota({ signal, },);
      return {
        taken: true,
        quota: takeBaseline({ quota, },),
      };
    }
    catch (error) {
      /**
       Failed attempts, this one included.
       */
      const count = (state.failed === undefined) ? 1 : state.failed
        .count
        + 1;
      state.failed = {
        count,
        lastAt: now(),
      };
      if (count === 1)
        rl.warn(
          `NeuralWatt quota could not be read before the first call (${errorName({ error, },)}); the run goes on, `
            + `the quota is read again no sooner than ${String(NEURALWATT_QUOTA_RETRY_SECONDS,)} seconds after this `
            + 'failure, and the end report has no starting reading unless one of those reads succeeds',
        );
      else
        rl.debug(
          `NeuralWatt quota could not be read again for the starting reading (${errorName({ error, },)}); `
            + `attempts so far: ${String(count,)}`,
        );
      return { taken: false, };
    }
  }

  /**
   Takes the starting reading if it has not been taken, trying again after a failure once the spacing has passed.

   @param signal - abort signal for the read

   @returns What the reading found, or that none is held
   */
  async function ensureBaseline({ signal, }: { readonly signal: AbortSignal; },): Promise<BaselineOutcome> {
    if (state.baseline !== undefined)
      return {
        taken: true,
        quota: state.baseline
          .quota,
      };
    if (state.attempting !== undefined)
      return await state.attempting;
    if ((state.failed !== undefined)
      && (now()
        < neuralWattSecondsAfter({
          from: state.failed
            .lastAt,
          seconds: NEURALWATT_QUOTA_RETRY_SECONDS,
        },)))
      return { taken: false, };
    state.begun = true;
    state.attempting = tryBaseline({ signal, },);

    /**
     Clears the attempt in flight when this function ends, however it ends.
     */
    using clearing = {
      [Symbol.dispose]: function clear(): void {
        delete state.attempting;
      },
    };
    void clearing;
    return await state.attempting;
  }

  /**
   Reads the quota again and compares it with the starting reading.

   @param signal - abort signal for the read

   @returns Difference, or why there is none
   */
  async function planSpend({ signal, }: { readonly signal: AbortSignal; },): Promise<NeuralWattPlanSpend> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: planSpend.name,
      l,
    },);
    if (!state.begun)
      return {
        kind: 'not-comparable',
        reason: 'the run made no NeuralWatt call, so no starting reading was taken',
      };
    if (state.attempting !== undefined)
      await state.attempting;

    /**
     The starting reading, if one was taken.
     */
    const { baseline, } = state;
    if (baseline === undefined) {
      /**
       The failed attempts, of which there is at least one since an attempt that took no reading failed.
       */
      const { failed, } = state;
      if (failed === undefined)
        throw new Error('a baseline attempt ended with neither a reading nor a failure',);
      return {
        kind: 'not-comparable',
        reason: 'the starting reading could not be taken: every attempt to read the quota for it failed '
          + `(${String(failed.count,)} ${(failed.count === 1) ? 'attempt' : 'attempts'}, the last at ${
            failed.lastAt
              .toISOString()
          })`,
      };
    }
    try {
      /**
       What the two readings say.
       */
      const spend = planSpendBetween({
        before: baseline.quota,
        after: await readQuota({ signal, },),
      },);
      return ((spend.kind === 'measured') && baseline.late)
        ? {
          ...spend,
          laterStartAt: baseline.at
            .toISOString(),
        }
        : spend;
    }
    catch (error) {
      rl.warn(`NeuralWatt quota could not be read at the end of the run (${errorName({ error, },)})`,);
      return {
        kind: 'not-comparable',
        reason: 'the reading at the end of the run could not be taken',
      };
    }
  }

  return {
    ensureBaseline,
    readNow,
    planSpend,
  };
}

//endregion NeuralWatt quota session
