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

  /**
   The pools the counts are of, in the owner's order of preference, each with the limit it was counted against: a closed pool is
   one of them, with no room.
   */
  readonly pools: readonly ProviderPool[];
};

/**
 Reads the room a pool has for another run now: none at all while it is closed, however much room it has when it is open.

 A pool the owner set aside keeps the room it returns to, so the reason it is closed and the number it comes back at stay readable,
 and every count and every question about room goes through this one reading. It lives here, beside the counts that ask it, because
 a reader of a count and a reader of a pool's room must not be able to disagree.

 @param pool - the pool to read

 @returns how many runs it takes at once now

 @example
 ```ts
 const room = roomIn(OPENCODE_GO,);
 ```
 */
export function roomIn(pool: ProviderPool,): number {
  return (pool.closed === undefined) ? pool.limit : 0;
}

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

 @param pools - the pools to count against, which are the shipped ones unless a caller set their limits

 @returns how many runs each pool holds

 @example
 ```ts
 const tallies = talliesOf({ runs: [], },);
 ```
 */
export function talliesOf({
  runs,
  pools = PROVIDER_POOLS,
}: {
  readonly runs: readonly GoingRun[];
  readonly pools?: readonly ProviderPool[];
},): PoolTallies {
  /**
   The count each pool is at, which starts at nothing for every pool the caller's table holds.
   */
  const counts = new Map<string, number>(pools.map(function atNothing(pool,) {
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
  return {
    counts,
    pools,
  };
}

/**
 Says how full every pool is, as the line a caller reads.

 @param counts - how many runs each pool holds, by the pool's name

 @param pools - the pools that were counted, in the owner's order of preference

 @returns one entry per pool, each with the room it has now

 @example
 ```ts
 const summary = summaryTextOf(talliesOf({ runs: [], },),);
 ```
 */
export function summaryTextOf({
  counts,
  pools,
}: PoolTallies,): string {
  return pools.map(function atItsLimit(pool,) {
    return `${pool.name} ${String(counts.get(pool.name,) ?? 0,)}/${String(roomIn(pool,),)}`;
  },)
    .join(', ',);
}

/**
 Says whether a pool has room for one more run.

 @param tallies - how many runs each pool holds

 @param pool - the pool to ask about

 @returns whether the pool is below the room it has now, which is none at all while it is closed

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
    ?? 0) < roomIn(pool,);
}

/**
 Reads the first pool with room for a run that named no model of its own, in the owner's order of {@link PROVIDER_POOLS}: the
 qwen plan's DeepSeek pool, then Neuralwatt's flex pool, since the pools before it are closed or take only their own model.

 @param tallies - how many runs each pool holds, and the pools that were counted

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
  const found = tallies.pools
    .filter(function takesDefaultRuns(pool,) {
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
