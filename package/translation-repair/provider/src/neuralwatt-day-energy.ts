import { neuralWattDayAfter, } from './neuralwatt-day.ts';
import type {
  NeuralWattDay,
  NeuralWattEnergy,
} from './neuralwatt-energy.ts';
import type { NeuralWattLedgerEntry, } from './neuralwatt-ledger-entry.ts';

//region NeuralWatt day energy
// WHAT A DAY HAS SPENT AND WHAT IS LEFT OF IT, drawn from the ledger's lines inside the day, the account's figure and
// the holds in flight (`neuralwatt-energy.ts` says which figure is the larger and why). Split out of
// `neuralwatt-ledger.ts`, which keeps the file, the holds and the reservations.

/**
 Whether the day takes no call at all, however small: its start is unknown, or the key's cap is blocked.

 @param day - the day a call is made in

 @returns Whether every reservation is refused

 @example
 ```ts
 if (dayIsClosed({ day, },)) return { granted: false, energy, estimateWh, };
 ```
 */
export function dayIsClosed({ day, }: { readonly day: NeuralWattDay; }): boolean {
  return (day.kind === 'unanchored') || (day.keyBlockedUntil !== undefined);
}

/**
 Whether a ledger line falls in the day: from its start up to but not including 24 hours after it.

 @param entry - the line

 @param day - the day, anchored

 @returns Whether the line counts

 @example
 ```ts
 const todays = entries.filter((entry) => entryIsInDay({ entry, day, }));
 ```
 */
export function entryIsInDay(
  {
    entry,
    day,
  }: {
    readonly entry: NeuralWattLedgerEntry;
    readonly day: Extract<NeuralWattDay, { readonly kind: 'anchored'; }>;
  },
): boolean {
  // THE STAMP IS COMPARED AS TEXT: a ledger line's `at` is read only in the spelling `toISOString` writes
  // (`neuralwatt-ledger-entry.ts`), which orders as the instants do.
  return (entry.at
    >= day.start
      .toISOString())
    && (entry.at
      < neuralWattDayAfter({ start: day.start, },)
        .toISOString());
}

/**
 The day's reading from the lines inside it, the account's figure and the holds as they stand NOW.

 @param entries - the ledger lines inside the day

 @param reservedWh - watt-hours held by calls in flight

 @param dailyWh - free energy the day holds

 @param day - the day

 @returns Reading; for a day with no known start, one with nothing left and no start named

 @example
 ```ts
 const energy = energyOfDay({ entries, reservedWh: 0.1, dailyWh: 550, day, },);
 ```
 */
export function energyOfDay(
  {
    entries,
    reservedWh,
    dailyWh,
    day,
  }: {
    readonly entries: readonly NeuralWattLedgerEntry[];
    readonly reservedWh: number;
    readonly dailyWh: number;
    readonly day: NeuralWattDay;
  },
): NeuralWattEnergy {
  if (day.kind === 'unanchored')
    return {
      dailyWh,
      spentWh: 0,
      reckonedWh: 0,
      reservedWh,
      remainingWh: 0,
      calls: 0,
    };

  /**
   Everything the lines say was spent.
   */
  const spentWh = entries.reduce(
    function addWh(
      running,
      entry,
    ): number {
      return running + entry.wh;
    },
    0,
  );

  /**
   The part of that which is an estimate.
   */
  const reckonedWh = entries.reduce(
    function addReckoned(
      running,
      entry,
    ): number {
      return (entry.estimated === undefined) ? running : (running + entry.wh);
    },
    0,
  );
  return {
    dayStart: day.start
      .toISOString(),
    dailyWh,
    spentWh,
    reckonedWh,
    reservedWh,
    // THE LARGER OF THE TWO, NEVER THEIR SUM: both count the calls this pipeline made.
    remainingWh: dailyWh - Math.max(
      spentWh,
      day.accountWh ?? 0,
    )
      - reservedWh,
    calls: entries.length,
    // Conditional spreads keep an absent field absent instead of undefined.
    ...((day.accountWh === undefined) ? {} : { accountWh: day.accountWh, }),
    ...((day.keyAllowance === undefined) ? {} : { keyAllowance: day.keyAllowance, }),
    ...((day.keyBlockedUntil === undefined) ? {} : { keyBlockedUntil: day.keyBlockedUntil
      .toISOString(), }),
  };
}

//endregion NeuralWatt day energy
