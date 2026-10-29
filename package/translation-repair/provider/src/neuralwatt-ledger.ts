import { homedir, } from 'node:os';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  contextRoot,
  readTextOrEmptyIfMissing,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { appendLedgerLine, } from './ledger-line-append.ts';
import {
  dayIsClosed,
  energyOfDay,
  entryIsInDay,
} from './neuralwatt-day-energy.ts';
import type {
  NeuralWattDay,
  NeuralWattEnergy,
} from './neuralwatt-energy.ts';
import {
  neuralWattDailyWhFrom,
  neuralWattLedgerPathFrom,
} from './neuralwatt-ledger-config.ts';
import {
  type NeuralWattLedgerEntry,
  neuralWattEntryOf,
  type NeuralWattReckoning,
} from './neuralwatt-ledger-entry.ts';

//region NeuralWatt ledger
// THE METER FOR THE DAILY FREE ENERGY, kept here because nobody else keeps it.
//
// The owner, 2026-10-20: "I just added TRANSLATION_REPAIR_NEURALWATT_API_KEY .
// It includes 550Wh of energy on DeepSeek V4.1 Flash and GLM 5.3 Flash per
// day. And we need to meter (using estimations) ourselves because overage eats
// into my normal plan which I use for other stuff." No API reports what is
// left of the free energy (`https://docs.neuralwatt.com/billing/daily-free-energy`,
// fetched 2026-10-20: "There's no API that reports how much free energy you
// have left today"), so every settled call appends one line to a durable file
// and the meter is the day's allowance minus the larger of the lines stamped
// inside the day and the whole account's covered energy, minus the holds of
// calls still in flight.
//
// TWO KINDS OF LINE. A MEASURED line carries the `: energy` figure the stream
// reported. An ESTIMATED line carries the estimate, marked, for a call whose
// stream reported none, or an attempt that failed after delivering something.
// A cost the stream reported is NOT in the line: NeuralWatt reports a nominal
// `request_cost_usd` on free-energy requests too (the owner's dashboard,
// 2026-10-20, counted a 0.0167 Wh request that reported 0.000107 USD wholly
// against the free energy), so it is not a billing signal and nothing here
// ends the day for it. A line an earlier build wrote may still carry
// `billedUsd`; the reader ignores it (`neuralwatt-ledger-entry.ts`).
//
// THE IN-FLIGHT HOLD. Calls run many at a time and a call's true energy is
// known only when its stream ends. A call therefore holds its estimate from
// before it is sent until it settles; the check against the day and the taking
// of the hold happen with no await between them, so concurrent calls cannot all
// read the same remainder. THE HOLD LIVES IN THIS PROCESS ONLY. A second
// process on the same ledger (a budget sample beside a pass) sees the file's
// lines and not this process's holds, so two processes can each spend what
// the other holds; the margin bounds that, nothing else does, and the record
// says so.
//
// THE DAY IS GIVEN TO EVERY READ AND EVERY RESERVATION, not kept here: it
// starts at an anchor the client reads off the account (`neuralwatt-day.ts`),
// the owner's "Every neuralwatt day starts at `15:15`" (19:15 UTC), and the
// ledger counts the lines whose `at` falls from that start up to but not
// including 24 hours after it. `at` is written with `toISOString` and read
// back only in that spelling. The ledger has no day of its own to fall back on,
// since a midnight it assumed would count the wrong day's lines.

/**
 Logger root for the ledger.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Decimals the day's figures are logged to: a call is a few thousandths of a
 watt-hour.
 */
const LOG_DECIMALS = 4;

/**
 Raised when a reservation is settled twice: the call it stood for is over.

 @example
 ```ts
 throw new NeuralWattReservationSettledError();
 ```
 */
export class NeuralWattReservationSettledError extends Error {
  /**
   Declares this message safe to forward: it is one authored sentence.
   */
  readonly messageNamesOnly: true = true;

  /**
   Says the reservation was already settled.

   @example
   ```ts
   new NeuralWattReservationSettledError();
   ```
   */
  public constructor() {
    super('a NeuralWatt energy reservation was settled twice; the call it stood for is already in the ledger',);
    this.name = 'NeuralWattReservationSettledError';
  }
}

/**
 What a call reports when it settles.

 @example
 ```ts
 const settlement: NeuralWattSettlement = { measuredWh: 0.015, };
 ```
 */
export type NeuralWattSettlement = {
  /**
   Energy the stream reported, in watt-hours; absent when it reported none,
   and the estimate then stands, marked.
   */
  readonly measuredWh?: number;
};

/**
 The energy one call holds while in flight.

 @example
 ```ts
 await reservation.settle({ measuredWh: 0.015, },);
 ```
 */
export type NeuralWattReservation = {
  /**
   Watt-hours the call holds.
   */
  readonly estimateWh: number;

  /**
   Appends the call's line, then releases the hold, even when the append
   fails.

   @throws {@link NeuralWattReservationSettledError} when already settled
   */
  readonly settle: (settlement: NeuralWattSettlement,) => Promise<void>;

  /**
   Appends a line at the estimate for an attempt that delivered something and
   failed; the hold stays for the attempt that follows.
   */
  readonly noteAbandoned: () => Promise<void>;

  /**
   Releases the hold without a line; harmless a second time.
   */
  readonly release: () => void;

  /**
   Releases the hold when the reservation goes out of scope, so a call that
   threw before settling gives its energy back.
   */
  readonly [Symbol.dispose]: () => void;
};

/**
 Outcome of asking to send a call.

 @example
 ```ts
 const outcome: NeuralWattReserveOutcome = { granted: false, energy, estimateWh: 0.3, };
 ```
 */
export type NeuralWattReserveOutcome =
  | {
    readonly granted: true;
    readonly reservation: NeuralWattReservation;
  }
    | {
      readonly granted: false;
      readonly energy: NeuralWattEnergy;
      readonly estimateWh: number;
    };

/**
 The ledger surface a client and a meter hold.

 @example
 ```ts
 const energy = await ledger.read();
 ```
 */
export type NeuralWattLedger = {
  /**
   File the lines are appended to.
   */
  readonly path: string;

  /**
   Reads the energy of the given day, holds in flight included.
   */
  readonly read: (request: { readonly day: NeuralWattDay; },) => Promise<NeuralWattEnergy>;

  /**
   Asks to send a call in the given day, holding its estimate when it fits.
   */
  readonly reserve: (
    request: {
      readonly model: string;
      readonly promptTokens: number;
      readonly estimateWh: number;
      readonly day: NeuralWattDay;
    },
  ) => Promise<NeuralWattReserveOutcome>;
};

/**
 The system clock, the ledger's time outside a test.

 @returns Current instant

 @example
 ```ts
 const now = systemClock();
 ```
 */
function systemClock(): Date {
  return new Date();
}

/**
 Builds the ledger over one file and one allowance.

 @param path - ledger file, created on the first line

 @param dailyWh - free energy a day holds

 @param now - clock, injectable so a test moves the day by hand

 @returns Ledger surface

 @example
 ```ts
 const ledger = createNeuralWattLedger({ path, dailyWh: 550, now: () => new Date(), },);
 ```
 */
export function createNeuralWattLedger(
  {
    path,
    dailyWh,
    now,
  }: {
    readonly path: string;
    readonly dailyWh: number;
    readonly now: () => Date;
  },
): NeuralWattLedger {
  /**
   Watt-hours held by calls in flight, by hold.
   */
  const holds = new Set<{ readonly wh: number; }>();

  /**
   Appends one line, creating the directory on the way.

   @param entry - call's line
   */
  async function note(entry: NeuralWattLedgerEntry,): Promise<void> {
    await appendLedgerLine({
      path,
      entry,
    },);
  }

  /**
   The file's lines inside the day, in order.

   @param day - day the lines count in

   @returns Entries stamped from the day's start up to but not including 24 hours after it; none for a day with no
   start
   */
  async function entriesOfTheDay(
    { day, }: { readonly day: NeuralWattDay; },
  ): Promise<readonly NeuralWattLedgerEntry[]> {
    if (day.kind === 'unanchored')
      return [];

    /**
     Whole file, or nothing where none exists yet.
     */
    const text = await readTextOrEmptyIfMissing({ path, },);
    return text
      .split('\n',)
      .flatMap(function toEntry(
        lineText,
        index,
      ): readonly NeuralWattLedgerEntry[] {
        if (lineText.trim() === '')
          return [];
        return [neuralWattEntryOf({
          path,
          text: lineText.trim(),
          line: index + 1,
        },),];
      },)
      .filter(function isInDay(entry,): boolean {
        return entryIsInDay({
          entry,
          day,
        },);
      },);
  }

  /**
   The holds as they stand NOW.

   @returns Watt-hours held by calls in flight
   */
  function reservedNow(): number {
    return [...holds,].reduce(
      function addHold(
        running,
        hold,
      ): number {
        return running + hold.wh;
      },
      0,
    );
  }

  /**
   Reads the energy of a day.

   @param day - day to read

   @returns Reading, holds in flight included
   */
  async function read({ day, }: { readonly day: NeuralWattDay; },): Promise<NeuralWattEnergy> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: read.name,
      l,
    },);

    /**
     The day's lines.
     */
    const entries = await entriesOfTheDay({ day, },);

    /**
     The reading.
     */
    const energy = energyOfDay({
      entries,
      reservedWh: reservedNow(),
      dailyWh,
      day,
    },);
    rl.debug(
      `${path}: ${String(energy.calls,)} ${
        wordForCount({
          count: energy.calls,
          one: 'call',
          many: 'calls',
        },)
      } in the day from ${energy.dayStart ?? 'an unknown start'} (UTC), ${energy.spentWh
        .toFixed(LOG_DECIMALS,)} Wh `
        + `spent of ${String(dailyWh,)} (${energy.reckonedWh
          .toFixed(LOG_DECIMALS,)} of it estimated), ${energy.reservedWh
            .toFixed(LOG_DECIMALS,)} Wh held, ${energy.remainingWh
              .toFixed(LOG_DECIMALS,)} Wh left`,
    );
    return energy;
  }

  /**
   Asks to send a call.

   @param model - model the call is for

   @param promptTokens - prompt tokens the estimate was drawn for

   @param estimateWh - energy the call would hold

   @param day - day the call is made in

   @returns Reservation when the call fits, else the reading it was refused on
   */
  async function reserve(
    {
      model,
      promptTokens,
      estimateWh,
      day,
    }: {
      readonly model: string;
      readonly promptTokens: number;
      readonly estimateWh: number;
      readonly day: NeuralWattDay;
    },
  ): Promise<NeuralWattReserveOutcome> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: reserve.name,
      l,
    },);

    /**
     The day's lines, read BEFORE the check so the check that follows has no await
     between it and the taking of the hold.
     */
    const entries = await entriesOfTheDay({ day, },);

    // NO AWAIT FROM HERE TO THE HOLD BEING TAKEN: the `energyOfDay` reading counts the
    // holds as they stand at this instant, so a call that read the file at
    // the same time as this one and was decided first has its hold counted.
    /**
     The day's reading with the holds as they stand NOW.
     */
    const energy = energyOfDay({
      entries,
      reservedWh: reservedNow(),
      dailyWh,
      day,
    },);
    if (dayIsClosed({ day, },) || (energy.remainingWh < estimateWh)) {
      rl.info(
        `${model}: refused before sending, estimate ${estimateWh.toFixed(LOG_DECIMALS,)} Wh, ${energy.remainingWh
          .toFixed(LOG_DECIMALS,)} Wh left in the day from ${energy.dayStart ?? 'an unknown start'} (UTC)${
            dayIsClosed({ day, },) ? ', and the day is closed to calls' : ''
          }`,
      );
      return {
        granted: false,
        energy,
        estimateWh,
      };
    }

    /**
     This call's hold.
     */
    const hold = { wh: estimateWh, };
    holds.add(hold,);

    /**
     Whether the call has been settled.
     */
    const state = { settled: false, };

    /**
     Line for this call at a given reckoning.

     @param wh - watt-hours the line carries

     @param estimated - why they are an estimate, absent for a measurement

     @returns Entry
     */
    function entryFor(
      {
        wh,
        estimated,
      }: {
        readonly wh: number;
        readonly estimated?: NeuralWattReckoning;
      },
    ): NeuralWattLedgerEntry {
      return {
        at: now()
          .toISOString(),
        model,
        wh,
        promptTokens,
        ...((estimated === undefined) ? {} : { estimated, }),
      };
    }

    /**
     Gives the hold back; harmless a second time.
     */
    function release(): void {
      state.settled = true;
      holds.delete(hold,);
    }

    /**
     Appends the call's line at its settlement, then releases the hold.

     @param measuredWh - energy the stream reported, absent when it reported none

     @throws {@link NeuralWattReservationSettledError} when already settled
     */
    async function settle({ measuredWh, }: NeuralWattSettlement,): Promise<void> {
      if (state.settled)
        throw new NeuralWattReservationSettledError();
      state.settled = true;

      // THE HOLD IS RELEASED WHETHER OR NOT THE LINE WAS WRITTEN: a failed write
      // must leave no phantom spend in flight.
      /**
       Releases the hold when this function ends.
       */
      using releasing = { [Symbol.dispose]: release, };
      void releasing;
      await note(entryFor({
        wh: measuredWh ?? estimateWh,
        ...((measuredWh === undefined) ? { estimated: 'unreported-estimate' as const, } : {}),
      },),);
    }

    /**
     Appends a line at the estimate for an attempt that delivered something
     and failed; the hold stays for the attempt that follows.
     */
    async function noteAbandoned(): Promise<void> {
      await note(entryFor({
        wh: estimateWh,
        estimated: 'abandoned-estimate',
      },),);
    }

    return {
      granted: true,
      reservation: {
        estimateWh,
        settle,
        noteAbandoned,
        release,
        [Symbol.dispose]: release,
      },
    };
  }

  return {
    path,
    read,
    reserve,
  };
}

/**
 Ledger at the path and allowance the environment names, on the real clock.

 @param env - process environment

 @returns Ledger surface

 @example
 ```ts
 const ledger = neuralWattLedgerFromEnv({ env: process.env, },);
 ```
 */
export function neuralWattLedgerFromEnv({ env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },): NeuralWattLedger {
  return createNeuralWattLedger({
    path: neuralWattLedgerPathFrom({
      env,
      home: homedir(),
    },),
    dailyWh: neuralWattDailyWhFrom({ env, },),
    now: systemClock,
  },);
}

//endregion NeuralWatt ledger
