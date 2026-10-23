import {
  type BudgetView,
  durationWordsOf,
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
  unreachableWritingBenches,
} from './run-seats-floor.ts';
import { judgeSeatsFor, } from './run-seats.ts';
import {
  type BenchName,
  benchShortfalls,
  reachableSeats,
} from './run-seats-wait.ts';

//region Holds that cure a short bench
// A HOLD IS WAITED FOR ONLY WHEN THE PROVIDER, BACK, WOULD HELP CURE A BENCH THAT STANDS SHORT. The seat reader waits
// while a bench the phase leans on stands short and a hold runs; a hold on a provider whose return leaves every such
// bench as short as it is (it serves no seat of the bench, or only seats the wet providers already reach) buys
// nothing, however long it is waited for, so it is left out of what the reader waits on. The question is asked of the
// roster itself: the seats are derived again with the providers back (`judgeSeatsFor`, which reads each seat's reach
// through `reachOf`), never from a list of provider names, so a roster change moves the answer without an edit here.
//
// THE TEST IS "WOULD HELP", NOT "IS NECESSARY" (round 5, the ruling open to the owner's veto). The held providers are
// judged together: the benches they cure between them are the ones that matter, and a held provider counts when its
// own return adds a reachable seat to one of those benches. Two holds that only cure a bench jointly are both waited
// for, and so are two holds that each cure it alone; a necessity test (the bench stays short without this provider)
// counted neither of the second pair and seated the bench short without a wait. The cost of the looser test is a
// counted hold that turns out not to be needed: the loop reads the benches again after every wait, so that costs one
// extra read and no extra wall time, since the wait was for the shortest counted hold and the loop ends as soon as
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
function viewWithBack(
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
 Whether some providers' return lifts every writing bench the phase leans on to its floor, derived from the roster.

 @param dry - dryness per provider the reading stopped on

 @param back - providers whose return is asked about

 @param names - benches the phase leans on

 @returns Whether no writing bench is below its floor once they are back

 @example
 ```ts
 const lifts = returnLiftsWritingBenches({ dry, back: ['neuralwatt',], names: phaseBenches({ phase, },), },);
 ```
 */
export function returnLiftsWritingBenches(
  {
    dry,
    back,
    names,
  }: {
    readonly dry: BudgetView;
    readonly back: readonly ProviderName[];
    readonly names: readonly BenchName[];
  },
): boolean {
  /**
   The view after their return.
   */
  const after = viewWithBack({
    dry,
    back,
  },);
  return unreachableWritingBenches({
    benches: benchesOf({ seats: judgeSeatsFor({ dry: after, }), },),
    names,
    dry: after,
  },)
    .length
    === 0;
}

/**
 How long each provider's hold counts for the seat reader's wait: the hold itself where the held providers cure a bench
 between them and this provider's return adds a reachable seat to it, none otherwise.

 @param holds - how long each provider's refusal still holds it out

 @param dry - dryness per provider, holds folded in, the reading the benches were read from

 @param names - benches the phase leans on

 @returns Holds worth waiting for, in the shape of the holds given

 @example
 ```ts
 const waitFor = holdsThatCureShortBenches({ holds, dry, names: phaseBenches({ phase, },), },);
 ```
 */
export function holdsThatCureShortBenches(
  {
    holds,
    dry,
    names,
  }: {
    readonly holds: ProviderRecord<number>;
    readonly dry: BudgetView;
    readonly names: readonly BenchName[];
  },
): ProviderRecord<number> {
  /**
   Providers held out for some time.
   */
  const held = PROVIDER_ORDER.filter(function isHeld(provider,): boolean {
    return holds[provider] > 0;
  },);
  /**
   Benches that stand unmet now.
   */
  const unmet = unmetBenchNames({
    dry,
    names,
  },);
  /**
   Benches still unmet once every held provider is back.
   */
  const unmetAfterAll = unmetBenchNames({
    dry: viewWithBack({
      dry,
      back: held,
    },),
    names,
  },);
  /**
   Benches the held providers cure between them.
   */
  const cured = [...unmet,].filter(function isCured(name,): boolean {
    return !unmetAfterAll.has(name,);
  },);
  return providerRecord({
    of: function holdWorthWaiting(provider,): number {
      if (!held.includes(provider,))
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
 The words a reading that does not wait says in place of a wait: the running holds it skipped and why, with NeuralWatt's
 return when it is dry and none lies inside what the pass has left.

 @param running - how long each provider's refusal still holds it out, as the reader read it

 @param counted - holds the reader waits for, a subset of the running ones

 @param dry - dryness per provider, holds folded in

 @param nothingHeld - words for a reading where no hold is skipped (none runs, or every one is waited for)

 @returns Words

 @example
 ```ts
 // => 'held: xiaomi (2 h), deepseek (40 min); neither would help cure a short bench'
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
    ? 'it would not help cure a short bench'
    : `${(skipped.length === 2) ? 'neither' : 'none'} would help cure a short bench`;
  /**
   NeuralWatt's part, said only where it is dry with no hold of its own inside what the pass has left.
   */
  const neuralwatt = (dry.neuralwatt && (running.neuralwatt === 0))
    ? '; no NeuralWatt return inside what the pass has left'
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
 const said = shortBenchesWithCure({ dry, names: phaseBenches({ phase, },), },);
 // => ['wide 3 of 7 reachable, quorum 4 (reached again with the return of synthetic, hyper, openrouter)',]
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

//endregion Holds that cure a short bench
