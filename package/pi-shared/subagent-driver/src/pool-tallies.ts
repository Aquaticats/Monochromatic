/**
 What the runs going now hold of each provider pool, the line that says it, and which pool a run may take.

 Both are read from the runs, never from a clock or a file of the launcher's own: a run that ends frees its slot the moment its
 launcher or its unit is gone.

 @module
 */

import type { GoingRun, } from './going-runs.ts';
import {
  type PoolMembership,
  type ProviderPool,
  PROVIDER_POOLS,
} from './pool-limits.ts';

/**
 No pool has the room a run asked for, as a symbol so that no pool can be mistaken for it.
 */
export const NO_POOL_HAS_ROOM: unique symbol = Symbol('no provider pool holds room for another Pi run',);

/**
 How many of each pool's runs are going now.
 */
export type PoolTallies = {
  /**
   The count each pool is at, by the pool's name. A pool with no run going is at zero.
   */
  readonly counts: ReadonlyMap<string, number>;
};

/**
 Reads the pools one membership holds a slot in.

 @param membership - a run's membership

 @returns one entry per pool it holds, and nothing for a run no pool holds

 @example
 ```ts
 const pools = poolsHeldBy({ membership: poolNamed({ provider: 'neuralwatt', model: 'deepseek-v4.1-flash', },), },);
 ```
 */
export function poolsHeldBy({
  membership,
}: {
  readonly membership: PoolMembership;
},): readonly ProviderPool[] {
  if (membership.holds === 'a-pool') {
    return [membership.pool,];
  }
  return (membership.holds === 'some-pools') ? membership.pools : [];
}

/**
 Counts the runs going now by the pool each holds a slot in.

 A run whose pool could not be read holds a slot in every pool it could be running on, so each of those is one closer to full:
 under-counting a pool is what let a run be started into room it did not have.

 @param runs - the runs going now

 @returns how many runs each pool holds

 @example
 ```ts
 const tallies = talliesOf({ runs: [], },);
 ```
 */
export function talliesOf({ runs, }: { readonly runs: readonly GoingRun[] },): PoolTallies {
  /**
   The count each pool is at, which starts at nothing for every pool the table ships.
   */
  const counts = new Map<string, number>(PROVIDER_POOLS.map(function atNothing(pool,) {
    return [
      pool.name,
      0,
    ] as const;
  },),);
  for (const run of runs) {
    for (const pool of poolsHeldBy({ membership: run.membership, },)) {
      counts.set(
        pool.name,
        (counts.get(pool.name,) ?? 0) + 1,
      );
    }
  }
  return { counts, };
}

/**
 Says how full every pool is, as the line a caller reads.

 @param counts - the count each pool is at, by the pool's name

 @returns one entry per pool, in the owner's order of preference

 @example
 ```ts
 const summary = summaryTextOf(talliesOf({ runs: [], },),);
 ```
 */
export function summaryTextOf({ counts, }: PoolTallies,): string {
  return PROVIDER_POOLS.map(function atItsLimit(pool,) {
    return `${pool.name} ${String(counts.get(pool.name,) ?? 0,)}/${String(pool.limit,)}`;
  },)
    .join(', ',);
}

/**
 Says whether a pool has room for one more run.

 @param tallies - how many runs each pool holds

 @param pool - the pool to ask about

 @returns whether the pool is below its limit

 @example
 ```ts
 const room = poolHasRoom({ tallies, pool: NEURALWATT_STANDARD, },);
 ```
 */
export function poolHasRoom({
  tallies,
  pool,
}: {
  readonly tallies: PoolTallies;
  readonly pool: ProviderPool;
},): boolean {
  return (tallies.counts
    .get(pool.name,)
    ?? 0) < pool.limit;
}

/**
 Reads the first pool with room for a run that named no model of its own: the owner's order is standard Neuralwatt, then its
 flex pool, then `opencode-go`.

 @param tallies - how many runs each pool holds

 @returns the pool to take, or {@link NO_POOL_HAS_ROOM}

 @example
 ```ts
 const pool = firstDefaultPoolWithRoom({ tallies, },);
 ```
 */
export function firstDefaultPoolWithRoom({ tallies, }: { readonly tallies: PoolTallies },): ProviderPool | typeof NO_POOL_HAS_ROOM {
  /**
   The first pool of the owner's order that takes default runs and has room.
   */
  const found = PROVIDER_POOLS.filter(function takesDefaultRuns(pool,) {
    return pool.takesDefaultRuns;
  },)
    .find(function hasRoom(pool,) {
      return poolHasRoom({
        tallies,
        pool,
      },);
    },);
  return found ?? NO_POOL_HAS_ROOM;
}
