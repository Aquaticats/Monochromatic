import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { BudgetView, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
  type ProviderRecord,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { ceilingWordsOf, } from './run-seats-ceiling.ts';
import {
  noWaitWordsOf,
  shortBenchesWithCure,
  unmetBenchNames,
} from './run-seats-hold-need.ts';
import { belowFloorWritingBenches, } from './run-seats-floor.ts';
import {
  withNeuralWattReturn,
  writtenRefusal,
} from './run-seats-refusal.ts';
import type {
  SeatReadingClient,
  WarnedState,
} from './run-seats-short-bench.ts';
import {
  type BenchName,
  type JudgeSeatPhase,
  shortBenches,
} from './run-seats-wait.ts';

//region Reading that takes no wait
// THE HALF OF THE SHORT-BENCH READING THAT SEATS WHAT IS REACHABLE, split out of `run-seats-short-bench.ts` at its line
// budget: a bench the phase leans on stands short, and no hold the reader could wait for would lift it over its floor
// or to its quorum, so the reading says why and goes on, or stops the entry for a writing bench below its floor.

/**
 What a reading says of the returns when it stops waiting: no provider has one the reader waits for, NeuralWatt's day
 return never being one.
 */
const NO_WAITABLE_RETURN = 'no provider has named a return the pass can wait for';

/**
 Seats what is reachable on the dryness first read, saying why no wait is taken: a writing bench below its floor stops
 the entry unless the phase goes on without its writers, and a bench short of its quorum is said once per state, with
 the ceiling the running holds could lift it to.

 @param client - run client whose holds and NeuralWatt return the lines read

 @param phase - phase about to start or continue, which the lines name

 @param l - entry logger, which records the shortfall

 @param warned - memory of the no-wait line last written, which the reading reads and updates; absent for a caller that
 reads once

 @param dry - dryness as first read, holds folded in

 @param running - how long each provider's refusal still holds it out, as the pass allows it to be waited for

 @param holds - holds among them that the reader waits for, none here

 @param meterDry - providers dry by their meter, which no hold ending brings back

 @param names - benches the phase leans on, in the order the line prints them

 @param benches - each bench as first read, keyed by name

 @param unreachable - writing benches below their floor as first read

 @param goesOn - whether the phase goes on when a writing bench is below its floor

 @mutates warned - holds the signature of the state this reading found

 @returns Dryness unchanged, no wait, and the writing benches below the floor

 @throws {@link WritingBenchUnreachableError} when a writing bench is below its floor in a phase that stops for it

 @example
 ```ts
 return seatWithoutWaiting({
   client, phase, l, warned, dry: first, running, holds, meterDry, names, benches, unreachable, goesOn,
 },);
 ```
 */
export function seatWithoutWaiting(
  {
    client,
    phase,
    l,
    warned,
    dry,
    running,
    holds,
    meterDry,
    names,
    benches,
    unreachable,
    goesOn,
  }: {
    readonly client: SeatReadingClient;
    readonly phase: JudgeSeatPhase;
    readonly l: Logger;
    readonly warned?: WarnedState;
    readonly dry: BudgetView;
    readonly running: ProviderRecord<number>;
    readonly holds: ProviderRecord<number>;
    readonly meterDry: ReadonlySet<ProviderName>;
    readonly names: readonly BenchName[];
    readonly benches: Readonly<Record<BenchName, readonly RosterModelId[]>>;
    readonly unreachable: readonly string[];
    readonly goesOn: boolean;
  },
): {
  readonly dry: BudgetView;
  readonly waitMs: number;
  readonly unreachableWriters: readonly string[];
} {
  /**
   What the reading says in place of a wait: the running holds it skipped and why, or that no return was named.
   */
  const noWait = noWaitWordsOf({
    running,
    counted: holds,
    dry,
    nothingHeld: NO_WAITABLE_RETURN,
  },);
  /**
   What this state is: which providers are held and which benches stand unmet.
   */
  const signature = `${
    PROVIDER_ORDER.filter(function isHeld(provider,): boolean {
      return running[provider] > 0;
    },)
      .join(',',)
  }|${
    [
      ...unmetBenchNames({
        dry,
        names,
      },),
    ].join(',',)
  }`;
  /**
   Whether the last line this caller let through was about the same state, so it is not said again.
   */
  const repeated = warned?.signature === signature;
  if (warned !== undefined)
    warned.signature = signature;
  /**
   The unreachable benches with why NeuralWatt reads dry, so a refusal states the true cause.
   */
  const stated = withNeuralWattReturn({
    clauses: unreachable,
    dry,
    client,
    refusalFollows: !goesOn,
  },);
  if (unreachable.length > 0) {
    // THE FIFTEENTH CLASS: a writing bench below its floor with nothing to wait for stops the entry rather than
    // writing a one-writer page. A PHASE THAT GOES ON WITHOUT ITS WRITERS says so once, naming the bench, and never
    // says the entry stops, since it does not.
    if (!(goesOn && repeated)) {
      l.error(
        `JUDGE SEATS phase=${phase} writing bench unreachable: ${stated.join('; ',)}; ${
          goesOn
            ? `${noWait}, so the phase goes on without its writers`
            : 'stopping the entry'
        }`,
      );
    }
    if (!goesOn) {
      throw writtenRefusal({
        phase,
        clauses: stated,
        dry,
        client,
      },);
    }
  }
  /**
   Writing benches below the floor, which the error line of a phase that goes on has named.
   */
  const belowFloor = new Set(belowFloorWritingBenches({
    benches,
    names,
    dry,
  },)
    .map(function nameOf({ name, },): BenchName {
      return name;
    },),);
  /**
   Benches whose shortfall no error line has named.
   */
  const namesElsewhere = names.filter(function isNotNamedAbove(name,): boolean {
    return !(goesOn && belowFloor.has(name,));
  },);
  /**
   Benches short of quorum that no error line has named.
   */
  const shortElsewhere = shortBenches({
    benches,
    names: namesElsewhere,
    dry,
  },);
  if ((shortElsewhere.length > 0) && (!repeated)) {
    // SAID EVEN WHEN THERE IS NOTHING TO WAIT FOR. The seventh hakureico
    // launch (2026-09-08, Bedrock alone) printed `readers=4 roster=10
    // withheld=none` at the pictures with three providers dry, since
    // `withheld=` names only the seats the slowness and cost rules take, and
    // stopped INCOMPLETE twenty seconds later with no reader reachable. A
    // spent balance names no return, so the phase runs on what is reachable;
    // the line says so before it starts.
    l.warn(
      `JUDGE SEATS phase=${phase} short of quorum: ${
        withNeuralWattReturn({
          clauses: shortBenchesWithCure({
            dry,
            names: namesElsewhere,
          },),
          dry,
          client,
          refusalFollows: false,
        },)
          .join('; ',)
      }; ${
        [
          ...ceilingWordsOf({
            running,
            dry,
            meterDry,
            names: namesElsewhere,
          },),
          noWait,
        ].join('; ',)
      }, so the phase runs on what is reachable`,
    );
  }
  return {
    dry,
    waitMs: 0,
    unreachableWriters: unreachable,
  };
}

//endregion Reading that takes no wait
