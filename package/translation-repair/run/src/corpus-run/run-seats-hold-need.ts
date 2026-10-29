import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type BudgetView,
  durationWordsOf,
  type MeterState,
  routesAsDry,
} from '@monochromatic-dev/translation-repair-provider/ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
  providerRecord,
  type ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  belowFloorWritingBenches,
  benchesOf,
} from './run-seats-floor.ts';
import { judgeSeatsFor, } from './run-seats.ts';
import { liftOf, } from './run-seats-bench-floor.ts';
import {
  type BenchName,
  benchReadings,
  benchShortfalls,
  reachableSeats,
} from './run-seats-wait.ts';

//region Holds whose return changes a short bench
// A HOLD IS WAITED FOR ONLY WHEN THE PROVIDER, BACK, WOULD LIFT A SHORT BENCH ACROSS A THRESHOLD THAT CHANGES WHAT THE
// STAGES DO. The seat reader waits while a bench the phase leans on stands short and a hold runs; a hold on a provider
// whose return leaves every such bench on the same side of its floor and of its quorum (it serves no seat of the bench,
// or only seats the wet providers already reach, or too few to reach either) buys nothing, however long it is waited
// for, so it is left out of what the reader waits on. The two thresholds and the stage code that makes each matter are
// in `run-seats-bench-floor.ts`. The question is asked of the roster itself: the seats are derived again with the
// providers back (`judgeSeatsFor`, which reads each seat's reach through `reachOf`), never from a list of provider
// names, so a roster change moves the answer without an edit here.
//
// A PROVIDER DRY BY ITS METER IS NOT BROUGHT BACK BY ITS HOLD ENDING. The dryness view folds holds into the meters and
// cannot tell the two apart, so the meter states are read beside it and a provider whose meter says dry is left out of
// the returning providers, in the wait and in the ceiling alike.
//
// THE TEST IS "WOULD HELP", NOT "IS NECESSARY". The returning providers
// are judged together: the benches they lift between them are the ones that matter, and a returning provider counts
// when its own return adds a reachable seat to one of those benches. Two holds that only lift a bench jointly are both
// waited for, and so are two holds that each lift it alone; a necessity test (the bench stays short without this
// provider) counted neither of the second pair and seated the bench short without a wait. The cost of the looser test
// is a counted hold that turns out not to be needed: the loop reads the benches again after every wait, so that costs
// one extra read and no extra wall time, since the wait was for the shortest counted hold and the loop ends as soon as
// the benches are met.

/**
 The view with some providers back: dry for none of them, as before for the rest.

 @param dry - dryness per provider, holds folded in

 @param back - providers whose return is asked about

 @returns View after their return

 @example
 ```ts
 const after = viewWithBack({ dry, back: ['hyper',], },);
 ```
 */
export function viewWithBack(
  {
    dry,
    back,
  }: {
    readonly dry: BudgetView;
    readonly back: readonly ProviderName[];
  },
): BudgetView {
  return providerRecord({
    of: function dryAfter(provider,): boolean {
      return dry[provider] && (!back.includes(provider,));
    },
  },);
}

/**
 The providers whose meter says dry, read from the meter states and not from the dryness view, which folds a hold in
 and so cannot tell a provider held out from one the meter keeps out. A provider refused for payment counts: the
 states report it dry until its meter moves, so its stated hold ending brings nothing back.

 @param meters - what each provider's meter said

 @returns Providers the router holds out for their meter alone, whatever their holds

 @example
 ```ts
 const meterDry = meterDryOf({ meters, });
 ```
 */
export function meterDryOf(
  { meters, }: { readonly meters: ProviderRecord<MeterState>; },
): ReadonlySet<ProviderName> {
  return new Set(PROVIDER_ORDER.filter(function isDryByMeter(provider,): boolean {
    return routesAsDry({ state: meters[provider], },);
  },),);
}

/**
 The providers whose return a wait can bring back: held out for some time, and not kept out by their meter, since a
 provider whose meter reads dry stays dry however its hold ends.

 @param holds - how long each provider's refusal still holds it out

 @param meterDry - providers dry by their meter

 @returns Providers a hold running on them can end in a return for, in provider order

 @example
 ```ts
 const back = returningProvidersOf({ holds: running, meterDry, });
 ```
 */
export function returningProvidersOf(
  {
    holds,
    meterDry,
  }: {
    readonly holds: ProviderRecord<number>;
    readonly meterDry: ReadonlySet<ProviderName>;
  },
): readonly ProviderName[] {
  return PROVIDER_ORDER.filter(function returns(provider,): boolean {
    return (holds[provider] > 0) && (!meterDry.has(provider,));
  },);
}

/**
 Benches the phase leans on that cannot do their work in a view: short of their quorum, or a
 writing bench below its floor.

 @param dry - dryness per provider the benches are derived for

 @param names - benches the phase leans on

 @returns Names of the unmet benches

 @example
 ```ts
 const unmet = unmetBenchNames({ dry, names: phaseBenches({ phase, },), },);
 ```
 */
export function unmetBenchNames(
  {
    dry,
    names,
  }: {
    readonly dry: BudgetView;
    readonly names: readonly BenchName[];
  },
): ReadonlySet<BenchName> {
  /**
   Benches as derived for this view, whose seats the dryness itself withholds.
   */
  const benches = benchesOf({ seats: judgeSeatsFor({ dry, }), },);
  return new Set([
    ...benchShortfalls({
      benches,
      names,
      dry,
    },)
      .map(function nameOfShortfall({ name, },): BenchName {
        return name;
      },),
    ...belowFloorWritingBenches({
      benches,
      names,
      dry,
    },)
      .map(function nameBelowFloor({ name, },): BenchName {
        return name;
      },),
  ],);
}

/**
 Seats of a bench some wet provider serves in a view, the bench derived again for that view.

 @param dry - dryness per provider the bench is derived for

 @param name - bench asked about

 @returns How many of its seats the router could send somewhere

 @example
 ```ts
 const reachable = reachableCountOf({ dry, name: 'wide', },);
 ```
 */
function reachableCountOf(
  {
    dry,
    name,
  }: {
    readonly dry: BudgetView;
    readonly name: BenchName;
  },
): number {
  return reachableSeats({
    seats: benchesOf({ seats: judgeSeatsFor({ dry, }), },)[name],
    dry,
    reach: (name === 'readers') ? 'vision' : 'text',
  },)
    .length;
}

/**
 How long each provider's hold counts for the seat reader's wait: the hold itself where the returning providers lift a
 bench across its floor or its quorum between them and this provider's return adds a reachable seat to it, none
 otherwise.

 @param holds - how long each provider's refusal still holds it out

 @param dry - dryness per provider, holds folded in, the reading the benches were read from

 @param meterDry - providers dry by their meter, whose return no hold brings about

 @param names - benches the phase leans on

 @returns Holds worth waiting for, in the shape of the holds given

 @example
 ```ts
 const waitFor = holdsThatCureShortBenches({ holds, dry, meterDry, names: phaseBenches({ phase, },), },);
 ```
 */
export function holdsThatCureShortBenches(
  {
    holds,
    dry,
    meterDry,
    names,
  }: {
    readonly holds: ProviderRecord<number>;
    readonly dry: BudgetView;
    readonly meterDry: ReadonlySet<ProviderName>;
    readonly names: readonly BenchName[];
  },
): ProviderRecord<number> {
  /**
   Providers held out for some time whose return a wait can bring back.
   */
  const returning = returningProvidersOf({
    holds,
    meterDry,
  },);
  /**
   Each bench as it reads now.
   */
  const reading = benchReadings({
    benches: benchesOf({ seats: judgeSeatsFor({ dry, }), },),
    names,
    dry,
  },);
  /**
   The view once every returning provider is back.
   */
  const afterAll = viewWithBack({
    dry,
    back: returning,
  },);
  /**
   Each bench as it reads once they are back.
   */
  const readingAfter = benchReadings({
    benches: benchesOf({ seats: judgeSeatsFor({ dry: afterAll, }), },),
    names,
    dry: afterAll,
  },);
  /**
   Benches the returning providers lift across a threshold between them: their floor, or their quorum.
   */
  const cured = reading
    .filter(function isCrossed(now,): boolean {
      /**
       The same bench read with the providers back.
       */
      const after = nonNullishOrThrow(readingAfter.find(function isThisBench(candidate,): boolean {
        return candidate.name === now.name;
      },),);
      /**
       What the return lifts the bench across.
       */
      const lift = liftOf({
        now,
        after,
      },);
      return lift.toFloor || lift.toQuorum;
    },)
    .map(function nameOf({ name, },): BenchName {
      return name;
    },);
  return providerRecord({
    of: function holdWorthWaiting(provider,): number {
      if (!returning.includes(provider,))
        return 0;
      /**
       The view with this provider alone back.
       */
      const backAlone = viewWithBack({
        dry,
        back: [provider,],
      },);
      return cured.some(function addsSeat(name,): boolean {
        return reachableCountOf({
          dry: backAlone,
          name,
        }) > reachableCountOf({
          dry,
          name,
        });
      },)
        ? holds[provider]
        : 0;
    },
  },);
}

/**
 The words a reading that does not wait says in place of a wait: the running holds it skipped and why, with a note that
 NeuralWatt's day return is not waited for when it is dry.

 @param running - how long each provider's refusal still holds it out, as the reader read it

 @param counted - holds the reader waits for, a subset of the running ones

 @param dry - dryness per provider, holds folded in

 @param nothingHeld - words for a reading where no hold is skipped (none runs, or every one is waited for)

 @returns Words

 @example
 ```ts
 // => 'held: xiaomi (2 h), deepseek (40 min); neither would lift a short bench over its floor or to its quorum'
 const said = noWaitWordsOf({ running, counted, dry, nothingHeld: 'no provider has named a return', },);
 ```
 */
export function noWaitWordsOf(
  {
    running,
    counted,
    dry,
    nothingHeld,
  }: {
    readonly running: ProviderRecord<number>;
    readonly counted: ProviderRecord<number>;
    readonly dry: BudgetView;
    readonly nothingHeld: string;
  },
): string {
  /**
   Providers with a hold running that the reader does not wait for.
   */
  const skipped = PROVIDER_ORDER.filter(function isSkipped(provider,): boolean {
    return (running[provider] > 0) && (counted[provider] === 0);
  },);
  if (skipped.length === 0)
    return nothingHeld;
  /**
   Why none of them is waited for, in the count of the list.
   */
  const why = (skipped.length === 1)
    ? 'it would lift no short bench over its floor or to its quorum'
    : `${(skipped.length === 2) ? 'neither' : 'none'} would lift a short bench over its floor or to its quorum`;
  /**
   NeuralWatt's part, said only where it is dry with no hold of its own: its day return is never waited for.
   */
  const neuralwatt = (dry.neuralwatt && (running.neuralwatt === 0))
    ? '; NeuralWatt\'s day return is not waited for'
    : '';
  return `held: ${
    skipped.map(function holdWords(provider,): string {
      return `${provider} (${durationWordsOf({ ms: running[provider], },)})`;
    },)
      .join(', ',)
  }; ${why}${neuralwatt}`;
}

/**
 The clauses of the benches that stand short, each with the providers whose return alone would reach the quorum.

 @param dry - dryness per provider, holds folded in

 @param names - benches the clauses are for

 @returns One clause per short bench, naming the providers that could cure it, or saying that no single provider could

 @example
 ```ts
 const said = shortBenchesWithCure({ dry: everyProviderDry, names: ['late',], },);
 // => ['late 0 of 7 reachable, quorum 4 (reached again with the return of hyper, openrouter)',]
 ```
 */
export function shortBenchesWithCure(
  {
    dry,
    names,
  }: {
    readonly dry: BudgetView;
    readonly names: readonly BenchName[];
  },
): readonly string[] {
  return benchShortfalls({
    benches: benchesOf({ seats: judgeSeatsFor({ dry, }), },),
    names,
    dry,
  },)
    .map(function clauseWithCure({
      name,
      clause,
    },): string {
      /**
       Providers whose return alone leaves the bench met.
       */
      const curing = PROVIDER_ORDER.filter(function cures(provider,): boolean {
        return dry[provider] && (!unmetBenchNames({
          dry: viewWithBack({
            dry,
            back: [provider,],
          },),
          names,
        },)
          .has(name,));
      },);
      return (curing.length === 0)
        ? `${clause} (no single provider's return reaches it)`
        : `${clause} (reached again with the return of ${curing.join(', ',)})`;
    },);
}

//endregion Holds whose return changes a short bench
