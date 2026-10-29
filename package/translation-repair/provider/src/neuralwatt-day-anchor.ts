import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  NEURALWATT_DAY_START_VAR,
  type NeuralWattDayStart,
  neuralWattDayStartAt,
  neuralWattDayStartOfInstant,
  neuralWattTimeOfDayUtc,
} from './neuralwatt-day.ts';
import type { NeuralWattQuota, } from './neuralwatt-quota.ts';
import type {
  BaselineOutcome,
  NeuralWattQuotaSession,
} from './neuralwatt-quota-session.ts';

//region NeuralWatt day anchor
// WHERE THIS RUN'S DAY STARTS, decided once. The owner, 2026-10-20: "Every neuralwatt day starts at `15:15`" (19:15 UTC
// for an owner in America/New_York, EDT). The time of day comes from the quota's `current_period_start` (the baseline
// reading of the run, `neuralwatt-quota-session.ts`), else from the `TRANSLATION_REPAIR_NEURALWATT_DAY_START` setting,
// else not at all: NeuralWatt is then refused for the run, with one warning that names the missing field and both
// remedies. THERE IS NO MIDNIGHT FALLBACK (`neuralwatt-day.ts`).

/**
 Logger root for the anchor decision.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Where the run's day anchor came from, or that it came from nowhere.

 @example
 ```ts
 const anchor: NeuralWattDayAnchor = { known: false, };
 ```
 */
export type NeuralWattDayAnchor =
  | {
    readonly known: true;

    /**
     Milliseconds after 00:00 UTC the day starts at.
     */
    readonly msOfDay: number;

    /**
     The fresh quota reading the anchor was learned from on a retry, which also carries the key's cap; absent for the
     first attempt, whose reading is the starting one.
     */
    readonly quota?: NeuralWattQuota;
  }
    | {
      readonly known: false;
    };

/**
 Reads the quota afresh for a later attempt, saying nothing of a failure: the outage was warned about at the first one.

 @param quota - client's quota session

 @param signal - abort signal for the read

 @returns Reading, or that none was taken

 @example
 ```ts
 const reading = await freshReading({ quota, signal, },);
 ```
 */
async function freshReading(
  {
    quota,
    signal,
  }: {
    readonly quota: NeuralWattQuotaSession;
    readonly signal: AbortSignal;
  },
): Promise<BaselineOutcome> {
  try {
    return {
      taken: true,
      quota: await quota.readNow({ signal, },),
    };
  }
  catch (error) {
    signal.throwIfAborted();
    tagged({
      tag: freshReading.name,
      l,
    },)
      .debug(`NeuralWatt quota could not be read again for the day start (${errorName({ error, },)})`,);
    return { taken: false, };
  }
}

/**
 Decides where the run's day starts.

 @param quota - client's quota session, whose baseline carries `current_period_start`

 @param setting - day-start setting, empty where the owner set none

 @param now - clock, for naming the day the anchor places now

 @param signal - abort signal for the baseline read

 @param retry - whether this is a later attempt, which reads the quota afresh instead of taking the starting reading

 @param warn - whether to warn that the day start is unknown, which is done once for an outage

 @returns Anchor, or that none could be learned

 @example
 ```ts
 const anchor = await decideNeuralWattDayAnchor({ quota, setting, now, signal, retry: false, warn: true, },);
 ```
 */
export async function decideNeuralWattDayAnchor(
  {
    quota,
    setting,
    now,
    signal,
    retry,
    warn,
  }: {
    readonly quota: NeuralWattQuotaSession;
    readonly setting: NeuralWattDayStart;
    readonly now: () => Date;
    readonly signal: AbortSignal;
    readonly retry: boolean;
    readonly warn: boolean;
  },
): Promise<NeuralWattDayAnchor> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: decideNeuralWattDayAnchor.name,
    l,
  },);

  /**
   The quota reading to learn the anchor from: the starting one on the first attempt, a fresh one after.
   */
  const baseline = retry ? await freshReading({
    quota,
    signal,
  },) : await quota.ensureBaseline({ signal, },);

  /**
   The quota's period start, if the reading gave one.
   */
  const periodStart = baseline.taken
    ? baseline.quota
      .subscription
      ?.periodStart
    : undefined;

  /**
   Its time of day, empty where the reading gave no instant.
   */
  const fromQuota = (periodStart === undefined) ? {} : neuralWattDayStartOfInstant({ instant: periodStart, },);

  /**
   Where the anchor came from, and its time of day.
   */
  const chosen = (fromQuota.msOfDay !== undefined)
    ? {
      msOfDay: fromQuota.msOfDay,
      source: `the quota's subscription.current_period_start (${String(periodStart,)})`,
    }
    : ((setting.msOfDay === undefined)
      ? undefined
      : {
        msOfDay: setting.msOfDay,
        source: `the ${NEURALWATT_DAY_START_VAR} setting`,
      });
  if (chosen === undefined) {
    if (warn)
      rl.warn(
        `NeuralWatt is refused until its day start is learned: the day start is unknown, because ${
        baseline.taken
          ? 'the quota reply carries no usable subscription.current_period_start'
          : 'the quota could not be read for subscription.current_period_start'
      } and ${NEURALWATT_DAY_START_VAR} is not set. Either make GET /v1/quota readable with `
          + `subscription.current_period_start, or set ${NEURALWATT_DAY_START_VAR}=HH:MM (UTC; 19:15 for a 15:15 `
          + 'day start in America/New_York during daylight saving time). Until then the quota is read again, no sooner '
          + 'than a minute after each failed attempt, and this is not warned about again',
      );
    return { known: false, };
  }

  /**
   First instant of the day the anchor places now.
   */
  const today = neuralWattDayStartAt({
    now: now(),
    msOfDay: chosen.msOfDay,
  },);
  rl.info(
    `NeuralWatt day starts at ${neuralWattTimeOfDayUtc({ msOfDay: chosen.msOfDay, },)} UTC every day, from ${chosen.source}${retry ? ' (learned on a retry)' : ''}; the current day began ${
      today.toISOString()
    }`,
  );
  return {
    known: true,
    msOfDay: chosen.msOfDay,
    ...((retry && baseline.taken) ? { quota: baseline.quota, } : {}),
  };
}

//endregion NeuralWatt day anchor
