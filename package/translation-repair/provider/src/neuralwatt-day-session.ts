import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  exchangeFailureLogText,
  type ModelTransport,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  readAccountCoveredWh,
  readAliasMap,
} from './neuralwatt-account-read.ts';
import type { NeuralWattAliasMap, } from './neuralwatt-aliases.ts';
import type { NeuralWattAnchorKeeper, } from './neuralwatt-day-anchor-keeper.ts';
import { neuralWattDayStartAt, } from './neuralwatt-day.ts';
import type { NeuralWattDay, } from './neuralwatt-energy.ts';
import {
  blockedUntilOf,
  type NeuralWattKeyAllowance,
} from './neuralwatt-key-allowance.ts';
import type { NeuralWattQuota, } from './neuralwatt-quota.ts';
import type { NeuralWattQuotaSession, } from './neuralwatt-quota-session.ts';
import type { RetryPolicy, } from './transient-retry.ts';

//region NeuralWatt day session
// THE DAY A RUN COUNTS IN, learned once and refreshed at every meter reading. The owner, 2026-10-20: "The stop logic
// should obviously count total free energy used per day. Every neuralwatt day starts at `15:15`" (19:15 UTC for an owner
// in America/New_York, EDT), and `neuralwatt-day.ts` says why the anchor is the quota's `current_period_start`.
//
// THREE THINGS ARE LEARNED, each with its own failure rule:
//
//   1. THE ANCHOR, once per run, from the baseline quota reading (`neuralwatt-quota-session.ts`), else the
//      `TRANSLATION_REPAIR_NEURALWATT_DAY_START` setting, else not at all: NeuralWatt is then refused for the run,
//      dry on the meter and refused at a call, with one warning that names the missing field and both remedies.
//      There is no midnight fallback.
//   2. THE ACCOUNT FIGURE, at run start and at each meter reading, from `GET /v1/usage/by-model`
//      (`neuralwatt-usage.ts`). Between readings the last figure stands. An unreadable route keeps the last figure if
//      it belongs to the current day, else the ledger counts alone, with one warning per outage.
//   3. THE KEY'S CAP, at the same readings, from `key.allowance` of the quota (`neuralwatt-key-allowance.ts`). A
//      blocked cap holds NeuralWatt dry until its `reset_at`, or until the next day's anchor where it names none. A
//      cap that will not read is warned and ignored; a quota that cannot be read keeps the last cap.
//
// THE ANCHOR IS A TIME OF DAY, so the day is recomputed from the clock at every use: a run that crosses the anchor sees
// the allowance come back, and an account figure taken for the previous day is not carried into the new one.

/**
 Logger root for the day session.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Decimals the account figure is logged to: a call is a few thousandths of a watt-hour.
 */
const LOG_DECIMALS = 4;

/**
 Day reads of one client.

 @example
 ```ts
 const days = createNeuralWattDaySession({ headers, baseUrl, transport, retryPolicy, quota, now, anchors, },);
 ```
 */
export type NeuralWattDaySession = {
  /**
   The day as it stands now.

   REFRESHING reads the account figure and the key's cap first, which a meter reading does; a call's reservation
   does not, and the first of either takes the first figures.
   */
  readonly dayFor: (args: {
    readonly signal: AbortSignal;
    readonly refresh: boolean
  },) => Promise<NeuralWattDay>;
};

/**
 Builds the day session of one client.

 @param headers - request headers, auth included

 @param baseUrl - API base address

 @param transport - HTTP the reads go over

 @param retryPolicy - transient-retry pacing

 @param quota - client's quota session, whose baseline carries the anchor

 @param now - clock

 @param anchors - client's keeper of the day anchor, which learns it again while it is unknown; the client keeps
 the keeper itself so that it can report what was decided without reading anything

 @returns Session surface

 @example
 ```ts
 const days = createNeuralWattDaySession({ headers, baseUrl, transport, retryPolicy, quota, now, anchors, },);
 ```
 */
export function createNeuralWattDaySession(
  {
    headers,
    baseUrl,
    transport,
    retryPolicy,
    quota,
    now,
    anchors,
  }: {
    readonly headers: Readonly<Record<string, string>>;
    readonly baseUrl: string;
    readonly transport: ModelTransport;
    readonly retryPolicy: RetryPolicy;
    readonly quota: NeuralWattQuotaSession;
    readonly now: () => Date;
    readonly anchors: NeuralWattAnchorKeeper;
  },
): NeuralWattDaySession {
  /**
   What the session holds between reads.
   */
  const state: {
    learnedQuota?: NeuralWattQuota;
    account?: {
      readonly startIso: string;
      readonly wh: number
    };
    readOnce: boolean;
    refreshing?: Promise<void>;
    accountOutage: boolean;
    aliasOutage: boolean;
    allowanceOutage: boolean;
    allowanceMalformed: boolean;
    blockedWarned: boolean;
    allowance?: NeuralWattKeyAllowance;
    blockedUntil?: Date;
  } = {
    readOnce: false,
    accountOutage: false,
    aliasOutage: false,
    allowanceOutage: false,
    allowanceMalformed: false,
    blockedWarned: false,
  };

  /**
   Reads which names are tracking aliases of a covered model, falling back to the usage rows' own served model.

   Never throws for a failed read: the count then goes by the served model, with one warning per outage.

   @param signal - abort signal for the read

   @returns Aliases, or that they could not be told
   */
  async function readAliases({ signal, }: { readonly signal: AbortSignal; },): Promise<NeuralWattAliasMap> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: readAliases.name,
      l,
    },);

    /**
     What the models route gave, or why it gave none.
     */
    const map = await readAliasMap({
      headers,
      baseUrl,
      transport,
      retryPolicy,
      signal,
    },);
    if (map.known) {
      state.aliasOutage = false;
      return map;
    }
    if (!state.aliasOutage)
      rl.warn(
        `NeuralWatt GET /models could not be used (${map.why}); the tracking aliases are not told apart, so every nw- `
          + 'row that a covered model served is counted in the account figure instead',
      );
    state.aliasOutage = true;
    return map;
  }

  /**
   Reads the account's covered energy since a day's start, keeping the figure for that day.

   Never throws for a failed read: the last figure stands, or the ledger alone.

   @param signal - abort signal for the read

   @param start - first instant of the day
   */
  async function readAccount(
    {
      signal,
      start,
    }: {
      readonly signal: AbortSignal;
      readonly start: Date;
    },
  ): Promise<void> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: readAccount.name,
      l,
    },);
    try {
      /**
       The tracking aliases whose calls count too.
       */
      const aliases = await readAliases({ signal, },);

      /**
       Covered energy of the whole account from the day's start to now.
       */
      const wh = await readAccountCoveredWh({
        headers,
        baseUrl,
        transport,
        retryPolicy,
        asked: {
          start,
          end: now(),
        },
        aliases,
        signal,
      },);
      state.account = {
        startIso: start.toISOString(),
        wh,
      };
      state.accountOutage = false;
      rl.debug(`NeuralWatt account used ${wh.toFixed(LOG_DECIMALS,)} Wh of the covered models since ${start.toISOString()}`,);
    }
    catch (error) {
      signal.throwIfAborted();
      if (!state.accountOutage)
        rl.warn(
          `NeuralWatt GET /usage/by-model could not be used (${exchangeFailureLogText({ error, },)}); the last account `
            + 'figure stands while it belongs to the current day, and the ledger counts alone otherwise',
        );
      state.accountOutage = true;
    }
  }

  /**
   Takes what the quota says about the key's cap into the session.

   @param reading - quota reading

   @param start - first instant of the day, the next day's anchor being a day after it
   */
  function takeAllowance(
    {
      reading,
      start,
    }: {
      readonly reading: NeuralWattQuota;
      readonly start: Date
    },
  ): void {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: takeAllowance.name,
      l,
    },);
    if (reading.keyAllowanceProblem !== undefined) {
      if (!state.allowanceMalformed)
        rl.warn(
          `NeuralWatt key allowance in the quota reply is malformed (${reading.keyAllowanceProblem}); it is ignored `
            + 'and the last good reading, if any, stands',
        );
      state.allowanceMalformed = true;
      return;
    }
    state.allowanceMalformed = false;
    if (reading.keyAllowance === undefined) {
      delete state.allowance;
      delete state.blockedUntil;
      state.blockedWarned = false;
      return;
    }
    state.allowance = reading.keyAllowance;
    if (!reading.keyAllowance
      .blocked) {
      delete state.blockedUntil;
      state.blockedWarned = false;
      return;
    }

    /**
     When the block lifts: the cap's own reset, else the next day's anchor.
     */
    const until = blockedUntilOf({
      allowance: reading.keyAllowance,
      start,
    },);
    state.blockedUntil = until;
    if (!state.blockedWarned)
      rl.warn(
        `NeuralWatt key allowance is blocked (${String(reading.keyAllowance
          .remainingUsd,)} USD left); NeuralWatt `
          + `is held dry until ${until.toISOString()} (UTC)`,
      );
    state.blockedWarned = true;
  }

  /**
   Reads the key's cap, from the baseline on the first reading and from a fresh quota read after.

   Never throws for a failed read: the last cap stands.

   @param signal - abort signal for the read

   @param start - first instant of the day
   */
  async function readAllowance(
    {
      signal,
      start,
    }: {
      readonly signal: AbortSignal;
      readonly start: Date
    },
  ): Promise<void> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: readAllowance.name,
      l,
    },);
    try {
      if ((!state.readOnce) && (state.learnedQuota !== undefined)) {
        // THE RETRY THAT LEARNED THE DAY READ THE QUOTA JUST NOW, so the cap is taken from that reading and not asked again.
        takeAllowance({
          reading: state.learnedQuota,
          start,
        },);
        state.allowanceOutage = false;
        return;
      }
      if (!state.readOnce) {
        /**
         The starting quota reading, which the first reading of the run is: it was warned about if it failed, and is not
         read a second time for the cap.
         */
        const baseline = await quota.ensureBaseline({ signal, },);
        if (baseline.taken)
          takeAllowance({
            reading: baseline.quota,
            start,
          },);
        state.allowanceOutage = !baseline.taken;
        return;
      }
      takeAllowance({
        reading: await quota.readNow({ signal, },),
        start,
      },);
      state.allowanceOutage = false;
    }
    catch (error) {
      signal.throwIfAborted();
      if (!state.allowanceOutage)
        rl.warn(
          `NeuralWatt GET /quota could not be read for the key allowance (${errorName({ error, },)}); the last `
            + 'reading of the allowance stands',
        );
      state.allowanceOutage = true;
    }
  }

  /**
   Takes the account figure and the key's cap together, sharing one read among callers that arrive during it.

   @param signal - abort signal for the reads

   @param start - first instant of the day
   */
  async function takeReadings(
    {
      signal,
      start,
    }: {
      readonly signal: AbortSignal;
      readonly start: Date
    },
  ): Promise<void> {
    if (state.refreshing !== undefined) {
      await state.refreshing;
      return;
    }

    /**
     Both reads, then the mark that the first readings were taken.
     */
    async function readBoth(): Promise<void> {
      await allInInputOrder({
        members: [
          readAccount({
            signal,
            start,
          },),
          readAllowance({
            signal,
            start,
          },),
        ],
      },);
      state.readOnce = true;
    }
    state.refreshing = readBoth();

    /**
     Clears the shared read when this function ends, however it ends.
     */
    using clearing = {
      [Symbol.dispose]: function clear(): void {
        delete state.refreshing;
      },
    };
    void clearing;
    await state.refreshing;
  }

  /**
   The day as it stands now.

   @param signal - abort signal for the reads

   @param refresh - whether to read the account figure and the cap first

   @returns Day
   */
  async function dayFor(
    {
      signal,
      refresh,
    }: {
      readonly signal: AbortSignal;
      readonly refresh: boolean
    },
  ): Promise<NeuralWattDay> {
    /**
     Where the day starts.
     */
    const anchor = await anchors.anchorFor({ signal, },);
    if (!anchor.known)
      return { kind: 'unanchored', };
    if ((!state.readOnce) && (anchor.quota !== undefined))
      state.learnedQuota = anchor.quota;
    if (refresh || (!state.readOnce))
      await takeReadings({
        signal,
        start: neuralWattDayStartAt({
          now: now(),
          msOfDay: anchor.msOfDay,
        },),
      },);

    /**
     The day as the clock says it is after the reads.
     */
    const current = now();

    /**
     First instant of that day.
     */
    const start = neuralWattDayStartAt({
      now: current,
      msOfDay: anchor.msOfDay,
    },);
    return {
      kind: 'anchored',
      start,
      // Conditional spreads keep an absent field absent instead of undefined.
      ...(((state.account === undefined) || (state.account
        .startIso
        !== start.toISOString()))
        ? {}
        : { accountWh: state.account
          .wh, }),
      ...((state.allowance === undefined) ? {} : { keyAllowance: state.allowance, }),
      ...(((state.blockedUntil === undefined) || (state.blockedUntil <= current))
        ? {}
        : { keyBlockedUntil: state.blockedUntil, }),
    };
  }

  return { dayFor, };
}

//endregion NeuralWatt day session
