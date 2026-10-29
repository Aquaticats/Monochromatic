import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { BudgetView, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
  type ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { benchesOf, } from './run-seats-floor.ts';
import { judgeSeatsFor, } from './run-seats.ts';
import {
  type BenchLift,
  benchFloorOf,
  liftOf,
} from './run-seats-bench-floor.ts';
import {
  returningProvidersOf,
  viewWithBack,
} from './run-seats-hold-need.ts';
import {
  type BenchName,
  type BenchReading,
  benchReadings,
} from './run-seats-wait.ts';

//region Ceiling of a short bench
// WHAT A SHORT BENCH COULD READ ONCE THE RUNNING HOLDS HAD ENDED, said beside what it reads now, so a reader of the run
// sees why the reader waited or did not: against the floor the stages need to decide anything and against the quorum
// they settle on (`run-seats-bench-floor.ts` says which stage code makes each matter).

/**
 The words for what a return does to one bench, from the thresholds it lifts the bench across.

 @param lift - thresholds the return lifts the bench across

 @param underFloor - whether the bench stands under its floor now

 @returns Words after "so"

 @example
 ```ts
 // => 'the return lifts it to the floor but not to the quorum'
 const verdict = liftWordsOf({ lift: { toFloor: true, toQuorum: false, }, underFloor: true, },);
 ```
 */
function liftWordsOf(
  {
    lift,
    underFloor,
  }: {
    readonly lift: BenchLift;
    readonly underFloor: boolean;
  },
): string {
  if (lift.toFloor && lift.toQuorum)
    return 'the return lifts it to the floor and to the quorum';
  if (lift.toFloor)
    return 'the return lifts it to the floor but not to the quorum';
  if (lift.toQuorum)
    return 'the return lifts it to the quorum';
  return underFloor
    ? 'no return lifts it to the floor or to the quorum'
    : 'no return lifts it to the quorum';
}

/**
 The clause for one short bench with a hold running.

 @param now - bench as it reads now

 @param after - same bench read with the returning providers back

 @param returning - providers whose return a wait can bring back

 @param meterHeld - held providers that stay dry by their meter or by a payment refusal that stands until it moves

 @returns Clause naming the ceiling, both thresholds, what the return lifts and the seats it brings back

 @example
 ```ts
 const clause = ceilingClauseOf({ now, after, returning, meterHeld, },);
 ```
 */
function ceilingClauseOf(
  {
    now,
    after,
    returning,
    meterHeld,
  }: {
    readonly now: BenchReading;
    readonly after: BenchReading;
    readonly returning: readonly ProviderName[];
    readonly meterHeld: readonly ProviderName[];
  },
): string {
  /**
   Seats the bench reads now and with the providers back.
   */
  const {
    reachable: reachedNow,
  } = now;
  /**
   Seats the bench reads with the providers back, and the quorum it would have.
   */
  const {
    seats: seatsAfter,
    reachable: reachedAfter,
    quorum: quorumAfter,
  } = after;
  /**
   Seats the return would bring back.
   */
  const brought = reachedAfter.filter(function isNotReachableNow(modelId,): boolean {
    return !reachedNow.includes(modelId,);
  },);
  /**
   Voices the stages leaning on the bench need before they decide anything.
   */
  const floor = benchFloorOf({ name: now.name, },);
  /**
   Why a held provider is not counted as coming back, where one is not.
   */
  const meterWords = (meterHeld.length === 0)
    ? ''
    : `; ${meterHeld.join(', ',)} ${(meterHeld.length === 1) ? 'reads' : 'read'} dry by its meter or by a payment `
      + 'refusal that stands until the meter moves, which a hold ending does not change';
  /**
   What the return lifts the bench across.
   */
  const verdict = liftWordsOf({
    lift: liftOf({
      now,
      after,
    },),
    underFloor: reachedNow.length < floor,
  },);
  return `${now.name} ceiling ${String(reachedAfter.length,)} of ${String(seatsAfter.length,)} with ${
    (returning.length === 0) ? 'no held provider' : returning.join(', ',)
  } back, floor ${String(floor,)} and quorum ${String(quorumAfter,)}, so ${verdict} (the return brings back ${
    (brought.length === 0) ? 'no seat' : brought.join(', ',)
  }${meterWords})`;
}

/**
 What each bench that stands short could read once every running hold had ended: the seats the return of the held
 providers would bring back, and whether that lifts the bench to its floor and to its quorum.

 THE CEILING OF A BENCH IS WHAT IT READS WITH THE RETURNING PROVIDERS BACK AND EVERY OTHER PROVIDER AS IT IS NOW, so a
 provider dry by its meter stays dry, held or not. `wide 3 of 7, quorum 4` under a Synthetic hold means Synthetic's
 return cannot lift the wide bench to its quorum, which is what the corpus pass `XingZ6020` waited 228 times to find
 out; its floor is a second reading, since the same return lifts the bench from 1 over the floor of 2.

 WITH NO HOLD RUNNING THERE IS NO CEILING TO STATE, the bench's ceiling being what it reads, which its own clause says.
 A bench under its floor is the one thing that clause leaves out, so it is said to stand under the floor and to have
 no hold that could lift it.

 @param running - how long each provider's refusal still holds it out, as the reader read it

 @param dry - dryness per provider, holds folded in

 @param meterDry - providers dry by their meter, whose holds ending bring no return

 @param names - benches the clauses are for

 @returns One clause per short bench, in the order the benches are named

 @example
 ```ts
 const said = ceilingWordsOf({ running, dry, meterDry, names: phaseBenches({ phase, },), },);
 // => ['wide ceiling 2 of 7 with synthetic back, floor 2 and quorum 4, so the return lifts it to the floor but not
 // to the quorum (the return brings back hf:moonshotai/Kimi-K3)',]
 ```
 */
export function ceilingWordsOf(
  {
    running,
    dry,
    meterDry,
    names,
  }: {
    readonly running: ProviderRecord<number>;
    readonly dry: BudgetView;
    readonly meterDry: ReadonlySet<ProviderName>;
    readonly names: readonly BenchName[];
  },
): readonly string[] {
  /**
   Each bench as it reads now.
   */
  const reading = benchReadings({
    benches: benchesOf({ seats: judgeSeatsFor({ dry, }), },),
    names,
    dry,
  },)
    .filter(function isShort({
      quorum,
      reachable,
    },): boolean {
      return reachable.length < quorum;
    },);
  /**
   Providers held out for some time.
   */
  const held = PROVIDER_ORDER.filter(function isHeld(provider,): boolean {
    return running[provider] > 0;
  },);
  if (held.length === 0) {
    return reading
      .filter(function isUnderFloor({
        name,
        reachable,
      },): boolean {
        return reachable.length < benchFloorOf({ name, },);
      },)
      .map(function floorWordsOf({ name, },): string {
        return `${name} reads under its floor of ${
          String(benchFloorOf({ name, },),)
        } and no hold runs that could lift it`;
      },);
  }
  /**
   Providers whose return a wait can bring back.
   */
  const returning = returningProvidersOf({
    holds: running,
    meterDry,
  },);
  /**
   The view once every returning provider is back.
   */
  const after = viewWithBack({
    dry,
    back: returning,
  },);
  /**
   Each bench as it reads once they are back.
   */
  const ceilings = benchReadings({
    benches: benchesOf({ seats: judgeSeatsFor({ dry: after, }), },),
    names,
    dry: after,
  },);
  /**
   Held providers that stay dry by their meter or by a payment refusal that stands until it moves.
   */
  const meterHeld = held.filter(function staysDry(provider,): boolean {
    return meterDry.has(provider,);
  },);
  return reading.map(function ceilingWordsOfBench(now,): string {
    return ceilingClauseOf({
      now,
      after: nonNullishOrThrow(ceilings.find(function isThisBench(candidate,): boolean {
        return candidate.name === now.name;
      },),),
      returning,
      meterHeld,
    },);
  },);
}

//endregion Ceiling of a short bench
