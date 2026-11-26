/**
 What the runs going now hold of each provider pool and of each virtual selection's members together, the line that says it, and
 which pool a run may take.

 Both are read from the runs, never from a clock or a file of the launcher's own: a run that ends frees its slot the moment its
 launcher or its unit is gone.

 @module
 */

import type { GoingRun, } from './going-runs.ts';
import {
  type PoolMembership,
  type ProviderPool,
  PROVIDER_POOLS,
  VIRTUAL_POOLS,
  type VirtualPool,
  poolOfThatNameIn,
  virtualTextOf,
} from './pool-limits.ts';

/**
 No pool has the room a run asked for, as a symbol so that no pool can be mistaken for it.
 */
export const NO_POOL_HAS_ROOM: unique symbol = Symbol('no provider pool holds room for another Pi run',);

/**
 How full one virtual selection's members are together.

 A run on the selection is one run against this room, never one run in each member: the router starts every request at the first
 member and moves the request on when a member refuses, so what bounds these runs is how many the members take between them.
 */
export type VirtualRoom = {
  /**
   The selection the counts are of.
   */
  readonly virtual: VirtualPool;

  /**
   The pools the selection's router may take, as the counted table holds them, in the router's order.
   */
  readonly pools: readonly ProviderPool[];

  /**
   How many runs are going now on any of those members, each counted once however many of them it may take.
   */
  readonly going: number;

  /**
   How many runs those members take together now: a closed member adds no room.
   */
  readonly room: number;
};

/**
 How many of each pool's runs are going now, and how full each virtual selection's members are together.
 */
export type PoolTallies = {
  /**
   The count each pool is at, by the pool's name. A pool with no run going is at zero. Only a run pinned to a pool is counted
   here: a run on a virtual selection holds the room its members have together instead.
   */
  readonly counts: ReadonlyMap<string, number>;

  /**
   The pools the counts are of, in the owner's order of preference, each with the limit it was counted against: a closed pool is
   one of them, with no room.
   */
  readonly pools: readonly ProviderPool[];

  /**
   The virtual selections counted, each with its member pools and their room together, in the order the table lists them.
   */
  readonly virtualRooms: readonly VirtualRoom[];
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
 Reads the pools one membership holds a slot in, each against that pool's own room.

 A run on a virtual selection holds none of them individually, because it is counted once against their room together: counting it
 in each member is what capped these runs at the smallest member's room.

 @param membership - a run's membership

 @returns one entry per pool it holds, and nothing for a run no pool holds or for a run on a virtual selection

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
 Reads every pool a run may land in, whether it holds that pool's own room or the room its members have together.

 @param membership - a run's membership

 @returns one entry per pool it may use, and nothing for a run no pool holds

 @example
 ```ts
 const pools = poolsARunMayTake({ membership: membershipOfSelection({ provider: 'rate-limit-fallback', model: 'writer', },), },);
 ```
 */
export function poolsARunMayTake({
  membership,
}: {
  readonly membership: PoolMembership;
},): readonly ProviderPool[] {
  if (membership.holds === 'a-pool') {
    return [membership.pool,];
  }
  return ((membership.holds === 'some-pools') || (membership.holds === 'a-virtual-selection'))
    ? membership.pools
    : [];
}

/**
 Counts the runs going now by the pool each holds a slot in, and by the members each virtual selection's runs share.

 A run whose pool could not be read holds a slot in every pool it could be running on, so each of those is one closer to full:
 under-counting a pool is what let a run be started into room it did not have. Such a run is also counted once in the combined
 room of every selection it could be on, for the same reason.

 @param runs - the runs going now

 @param pools - the pools to count against, which are the shipped ones unless a caller set their limits

 @param virtuals - the virtual selections to count, which are the shipped ones unless a test hands over others

 @returns how many runs each pool holds, and how full each selection's members are together

 @example
 ```ts
 const tallies = talliesOf({ runs: [], },);
 ```
 */
export function talliesOf({
  runs,
  pools = PROVIDER_POOLS,
  virtuals = VIRTUAL_POOLS,
}: {
  readonly runs: readonly GoingRun[];
  readonly pools?: readonly ProviderPool[];
  readonly virtuals?: readonly VirtualPool[];
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
  /**
   The members of each selection and the runs that may take them, counted once per run.
   */
  const virtualRooms = virtuals.map(function countedTogether(virtual,) {
    /**
     The selection's members, as the counted table holds them, so a caller's limits decide their room.
     */
    const members = virtual.memberPools
      .map(function asTheCountedTableHoldsIt(pool,) {
        return poolOfThatNameIn({
          pool,
          pools,
        },);
      },);
    /**
     The names of those members, which is how a run is read as being on this selection.
     */
    const memberNames = new Set(members.map(function nameOf(pool,) {
      return pool.name;
    },),);
    /**
     The runs going now that may take one of those members, each counted once.
     */
    const going = runs
      .filter(function mayTakeAMember(run,) {
        return poolsARunMayTake({ membership: run.membership, },)
          .some(function isOneOfTheMembers(pool,) {
            return memberNames.has(pool.name,);
          },);
      },)
      .length;
    /**
     What those members take together, which a closed member adds nothing to.
     */
    const room = members.reduce(
      function sumOf(
        sum,
        pool,
      ) {
        return sum + roomIn(pool,);
      },
      0,
    );
    return {
      virtual,
      pools: members,
      going,
      room,
    };
  },);
  return {
    counts,
    pools,
    virtualRooms,
  };
}

/**
 Says how full every pool is, and how full each virtual selection's members are together, as the line a caller reads.

 A pool's entry counts the runs pinned to it, which is what its own room bounds. A virtual selection's entry counts every run on
 its members once, with the room those members have together, so a reader sees the number that decides whether another run on the
 selection may start.

 @param counts - how many runs each pool holds, by the pool's name

 @param pools - the pools that were counted, in the owner's order of preference

 @param virtualRooms - how full each virtual selection's members are together

 @returns one entry per pool and one per selection, each with the room it has now

 @example
 ```ts
 const summary = summaryTextOf(talliesOf({ runs: [], },),);
 ```
 */
export function summaryTextOf({
  counts,
  pools,
  virtualRooms,
}: PoolTallies,): string {
  return [
    ...pools.map(function atItsLimit(pool,) {
      return `${pool.name} ${String(counts.get(pool.name,) ?? 0,)}/${String(roomIn(pool,),)}`;
    },),
    ...virtualRooms.map(function together(selection,) {
      return `${virtualTextOf({ virtual: selection.virtual, },)} ${String(selection.going,)}/${String(selection.room,)} together`;
    },),
  ]
    .join(', ',);
}

/**
 Reads whether a pool has room for one more run.

 A run pinned to a pool needs that pool's own room, and also leaves the room its members have together a slot: it is one more run
 on every virtual selection that pool belongs to, so it may not take the last of that room.

 @param tallies - how many runs each pool holds

 @param pool - the pool to ask about

 @returns whether the pool is below the room it has now, and every selection it belongs to is below theirs

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
  /**
   The pool as the counted table holds it, so a limit a caller set decides its room here too.
   */
  const counted = poolOfThatNameIn({
    pool,
    pools: tallies.pools,
  },);
  if ((tallies.counts
    .get(counted.name,)
    ?? 0) >= roomIn(counted,)) {
    return false;
  }
  return tallies.virtualRooms
    .filter(function takesThisPool(selection,) {
      return selection.pools
        .some(function isThisPool(member,) {
          return member.name === counted.name;
        },);
    },)
    .every(function hasItsRoomTogether(selection,) {
      return selection.going < selection.room;
    },);
}

/**
 Reads the first pool with room for a run that named no model of its own, in the owner's order of {@link PROVIDER_POOLS}: the
 qwen plan's DeepSeek pool, then `opencode-go`, then Neuralwatt's DeepSeek flex pool, since the pools between and after them are
 closed or take only their own model.

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

/**
 Reads how full one virtual selection's members are together.

 @param tallies - how many runs each pool holds, and the selections that were counted

 @param virtual - the selection to read

 @returns the selection's member pools, the runs on them, and their room together

 @throws Error when the tallies were not counted for this selection, which is a caller that handed over a table the count does not
 describe

 @example
 ```ts
 const room = virtualRoomOf({ tallies, virtual: VIRTUAL_WRITER, },);
 ```
 */
export function virtualRoomOf({
  tallies,
  virtual,
}: {
  readonly tallies: PoolTallies;
  readonly virtual: VirtualPool;
},): VirtualRoom {
  /**
   How this selection is named, which is how the tallies key it.
   */
  const named = virtualTextOf({ virtual, },);
  /**
   The count of this selection, when the tallies hold one.
   */
  const found = tallies.virtualRooms
    .find(function isThisSelection(selection,) {
      return virtualTextOf({ virtual: selection.virtual, },) === named;
    },);
  if (found === undefined) {
    throw new Error(`${named} was not counted: the tallies describe other selections, so no room of its members can be read from them`,);
  }
  return found;
}

/**
 Says whether one more run may start on a virtual selection: the runs on its members are fewer than the room those members have
 together.

 @param tallies - how many runs each pool holds, and the selections that were counted

 @param virtual - the selection to ask about

 @returns whether its members have room together for another run

 @example
 ```ts
 const room = virtualSelectionHasRoom({ tallies, virtual: VIRTUAL_WRITER, },);
 ```
 */
export function virtualSelectionHasRoom({
  tallies,
  virtual,
}: {
  readonly tallies: PoolTallies;
  readonly virtual: VirtualPool;
},): boolean {
  /**
   How full this selection's members are together.
   */
  const room = virtualRoomOf({
    tallies,
    virtual,
  },);
  return room.going < room.room;
}
