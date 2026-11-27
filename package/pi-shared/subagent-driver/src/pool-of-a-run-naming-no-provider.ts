/**
 The pool a run whose command line names no provider takes, and what a run that no pool takes is refused with.

 The project decided on 2026-11-23 that such a launch still starts, taking the first open pool in the owner's order
 (`qwen-plan-deepseek`, then `opencode-go`, then Neuralwatt's flex pool), never a closed pool and never one that takes only runs
 naming it, and accepted the cost of the run not being counted against that pool's limit
 (`doc/decision/pi-driver-closed-pools-and-the-worktree-launchers-default.md`).

 The choice is the pool gate's own, so a run this launcher places is placed where the gate would place it: one rule of the owner's
 order, read in one place.

 @module
 */

import {
  CLOSED_POOL_LIMIT,
  POOL_LIMITS_VARIABLE,
  type PoolMembership,
  type ProviderPool,
  poolTakesNoNewRun,
} from './pool-limits.ts';
import { poolChoiceFor, } from './pool-gate.ts';
import {
  type PoolTallies,
  roomIn,
} from './pool-tallies.ts';

/**
 What a run whose command line names no provider takes: the pool it goes on, or the refusal of a run no pool takes.
 */
export type PoolOfARunNamingNoProvider =
  | {
    /**
     The pool the run goes on.
     */
    readonly took: 'a-pool';

    /**
     The pool, with the provider, model and wall clock the run takes from it.
     */
    readonly pool: ProviderPool;

    /**
     Whether this command chose the pool rather than keeping the one the run's own model names, which a resumed run keeps while
     that pool takes new runs.
     */
    readonly movesTheRun: boolean;
  }
    | {
    /**
     Whether a pool was found at all.
     */
      readonly took: 'no-pool';

    /**
     What the run is refused with: one line per pool a run naming no provider may take, each with why it takes no run now, and
     the setting that gives a pool room again.
     */
      readonly refusal: string;
    };

/**
 Reads the pool a run whose command line names no provider takes.

 The run's own pool is what its model names, read with the provider this package gives a run that names none: a pair that names no
 pool, or one a run naming no provider may not take, is not this function's, and its caller keeps the pair the command line named.

 @param requested - the pool the run's provider and model name, as the table ships it and before any limit a caller set

 @param resumes - whether the run resumes a session, which keeps it on the pool its model names while that pool takes new runs

 @param tallies - how many runs each pool holds, and the pools that were counted, which carry the limits a caller set

 @returns the pool the run goes on, or the refusal of a run no pool takes

 @throws Error when the pair given names no pool a run naming no provider may take, which is a caller that skipped the check this
 function documents

 @example
 ```ts
 const taken = poolOfARunNamingNoProvider({ requested, resumes: false, tallies: talliesOf({ runs: [], },), },);
 ```
 */
export function poolOfARunNamingNoProvider({
  requested,
  resumes,
  tallies,
}: {
  readonly requested: PoolMembership;
  readonly resumes: boolean;
  readonly tallies: PoolTallies;
}): PoolOfARunNamingNoProvider {
  /**
   Which pool the run may take, as the pool gate reads it: the first pool of the owner's order with room, or a pool its own model
   names and keeps, or none at all.
   */
  const choice = poolChoiceFor({
    requested,
    tallies,
    resumes,
  },);
  if (choice.destination === 'a-pool') {
    return {
      took: 'a-pool',
      pool: choice.pool,
      movesTheRun: choice.movesTheRun,
    };
  }
  if (choice.destination === 'its-own-provider') {
    /**
     What the pair given names, for the error a caller that skipped the check above gets.
     */
    const named = (requested.holds === 'a-pool') ? requested.pool
      .name : 'a pair no pool holds';
    throw new Error(
      `poolOfARunNamingNoProvider was given ${named}, which no run that names no provider may take: its caller keeps the pair its command line named instead.`,
    );
  }
  if (choice.destination === 'a-virtual-pool') {
    throw new Error(
      'poolOfARunNamingNoProvider was given a virtual selection, which keeps the provider and model its run named: its caller places that run by the pools the selection\'s router may take instead.',
    );
  }
  return {
    took: 'no-pool',
    refusal: noPoolTakesItTextOf({ tallies, },),
  };
}

/**
 Says what a run that no pool takes is refused with.

 @param tallies - how many runs each pool holds, and the pools that were counted

 @returns the refusal, one line per pool a run naming no provider may take, and the setting that gives a pool room again

 @example
 ```ts
 const refusal = noPoolTakesItTextOf({ tallies, },);
 ```
 */
function noPoolTakesItTextOf({ tallies, }: { readonly tallies: PoolTallies },): string {
  /**
   The lines the refusal is made of, which start with what this command could not find.
   */
  const lines: string[] = ['No pool a run that names no provider may take has room now:'];
  /**
   The pools such a run may take, in the owner's order.
   */
  const defaultPools = tallies.pools
    .filter(function takesDefaultRuns(candidate,) {
      return candidate.takesDefaultRuns;
    },);
  for (const pool of defaultPools) {
    lines.push(`  ${pool.name}: ${whyItTakesNoRun({
      tallies,
      pool,
    },)}`,);
  }
  lines.push(
    `A pool takes new runs again when ${POOL_LIMITS_VARIABLE} gives it a limit above zero, as ${POOL_LIMITS_VARIABLE}=opencode-go=<n> does for one command.`,
  );
  return lines.join('\n',);
}

/**
 Says why one pool takes no run now: the owner's or the caller's closing of it, or the runs already going filling it.

 @param tallies - how many runs each pool holds, and the pools that were counted

 @param pool - the pool to read, as the table with the caller's limits holds it

 @returns the reason, as one phrase a refusal carries beside the pool's name

 @example
 ```ts
 const why = whyItTakesNoRun({ tallies, pool: OPENCODE_GO, },);
 ```
 */
export function whyItTakesNoRun({
  tallies,
  pool,
}: {
  readonly tallies: PoolTallies;
  readonly pool: ProviderPool;
},): string {
  if (poolTakesNoNewRun({ pool, },)) {
    /**
     The closing of the pool, when the owner set it aside: the caller's zero says the pool is closed now, and the owner's words
     say why it was set aside before that.
     */
    const { closed, } = pool;
    /**
     Why the pool takes no new run: the reason its `closed` field gives, or its limit when nothing else was said.
     */
    const reason = (closed === undefined) ? `its limit is ${String(CLOSED_POOL_LIMIT,)}` : closed.reason;
    return `takes no new run: ${reason}`;
  }
  /**
   How many of this pool's runs are going now.
   */
  const going = tallies.counts
    .get(pool.name,);
  return `full: ${String(going ?? 0,)} going of the ${String(roomIn(pool,),)} it takes`;
}
