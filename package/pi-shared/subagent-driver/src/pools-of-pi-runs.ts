/**
 The provider pools a run may be started in, how many runs each one takes, and which pool a new run goes to.

 Every limit and every ordering here is the owner's, quoted beside the value it decides. Counting is the union of the units
 systemd is running and the starts still in flight, because a new run's unit takes minutes to appear, and a count of units alone
 once allowed a twenty-first run on a pool of twenty.

 @module
 */

/**
 One provider pool: a provider and model pair whose concurrency the provider bounds.
 */
export type SlotPool = {
  /**
   The pool's name, as a line about slots prints it.
   */
  readonly name: string;

  /**
   What Pi is asked to run on.
   */
  readonly provider: string;

  /**
   The model the pool is counted for.
   */
  readonly model: string;

  /**
   How many runs the pool takes at once.
   */
  readonly limit: number;

  /**
   What a wall clock given by a caller is multiplied by in this pool, which is how a slower pool gets longer to do one job.
   */
  readonly wallFactor: number;

  /**
   Whether a run that names the default model may be moved into this pool. A run naming a pool of its own stays in it.
   */
  readonly takesDefaultRuns: boolean;
};

/**
 No pool of this table takes the run, as a symbol so no pool's name can be mistaken for it.
 */
export const NO_POOL: unique symbol = Symbol('absent: no provider pool of this table takes the run',);

/**
 No start of the run is in flight, as a symbol so no pool can be mistaken for it.
 */
export const NO_LAUNCHER: unique symbol = Symbol('absent: no start of this run is in flight',);

/**
 The owner set the first limit: "we're permanently raising the practical limit to 20."
 */
const NEURALWATT_LIMIT = 20;

/**
 The owner on the pools, 2026-11-10: "You can use -flex postfix on neuralwatt models. They have their own separate pool of 22
 slots. They're around 100% slower than standard in practice."
 */
const NEURALWATT_FLEX_LIMIT = 22;

/**
 A flex run gets twice the wall clock, because the owner measured the pool at around 100% slower than standard.
 */
const FLEX_WALL_FACTOR = 2;

/**
 The standard pool's wall clock is what the caller asked for.
 */
const STANDARD_WALL_FACTOR = 1;

/**
 The owner on `opencode-go`, 2026-11-10: "In practice, opencode-go may have worse inference quality and worse speed than
 neuralwatt's flex, so prefer neuralwatt." It is still a pool of its own, so it counts against its own limit only.
 */
const OPENCODE_GO_LIMIT = 20;

/**
 The owner, 2026-11-10: "Use 22 against Kimi K3.1 Pro flex from neuralwatt for now, until upstream clarifies or we hit
 something."
 */
const NEURALWATT_KIMI_FLEX_LIMIT = 22;

/**
 The pools, in the owner's order: a run that may move takes the first pool with room, standard Neuralwatt before flex before
 `opencode-go`.

 A run on Synthetic has no pool of this table, so it is outside every Neuralwatt limit; the owner, 2026-11-10: "Kimi trials
 don't count within 20 slots because 20 is neuralwatt's effective concurrency and kimi runs on Synthetic."
 */
export const PI_POOLS: readonly SlotPool[] = [
  {
    name: 'neuralwatt',
    provider: 'neuralwatt',
    model: 'deepseek-v4.1-flash',
    limit: NEURALWATT_LIMIT,
    wallFactor: STANDARD_WALL_FACTOR,
    takesDefaultRuns: true,
  },
  {
    name: 'neuralwatt-flex',
    provider: 'neuralwatt',
    model: 'deepseek-v4.1-flash-flex',
    limit: NEURALWATT_FLEX_LIMIT,
    wallFactor: FLEX_WALL_FACTOR,
    takesDefaultRuns: true,
  },
  {
    name: 'opencode-go',
    provider: 'opencode-go',
    model: 'deepseek-v4.1-flash',
    limit: OPENCODE_GO_LIMIT,
    wallFactor: STANDARD_WALL_FACTOR,
    takesDefaultRuns: true,
  },
  {
    name: 'neuralwatt-kimi-flex',
    provider: 'neuralwatt',
    model: 'kimi-k3.1-pro-flex',
    limit: NEURALWATT_KIMI_FLEX_LIMIT,
    wallFactor: FLEX_WALL_FACTOR,
    takesDefaultRuns: false,
  },
];

/**
 The pool a run's unit counts in when no start of it is in flight any more: the pool whose limit it was started under is
 forgotten once its unit is up, and `opencode-go` is the one pool the coordinator fills by hand.
 */
export const POOL_OF_A_RUN_WITH_NO_LAUNCHER = 'opencode-go';

/**
 The start of a run's name that puts it outside every Neuralwatt limit: an audit, which runs on another provider.
 */
export const AUDIT_RUN_PREFIX = 'sol-';

/**
 The start of a run's name that puts it outside every Neuralwatt limit when no launcher names a pool: a Kimi run on Synthetic.
 */
export const KIMI_RUN_PREFIX = 'kimi-';

/**
 The status the start command answers with when the run's pool has no room, which a caller waiting for a slot reads.
 */
export const NO_SLOT_EXIT_STATUS = 3;

/**
 The status the start command answers with when a run of that name is already going, and this one is not started.
 */
export const ALREADY_RUNNING_EXIT_STATUS = 4;

/**
 The status a start that ran to an exit without one of its own is reported as.
 */
export const START_FAILED_EXIT_STATUS = 1;

/**
 One run that counts against a pool, and what its start named.
 */
export type RunUnderAPool = {
  /**
   The run's name, which is its unit's name without the prefix and suffix.
   */
  readonly name: string;

  /**
   The pool the run's start named, {@link NO_POOL} when that start names no pool of this table, or {@link NO_LAUNCHER} when no
   start of it is in flight.
   */
  readonly launcherPool: SlotPool | typeof NO_POOL | typeof NO_LAUNCHER;
};

/**
 Reads the pool a provider and model pair belongs to.

 @param provider - what the run asked Pi to use

 @param model - what the run asked Pi to use

 @returns the pool, or {@link NO_POOL}

 @example
 ```ts
 const pool = poolNamed({ provider: 'neuralwatt', model: 'deepseek-v4.1-flash', });
 ```
 */
export function poolNamed({
  provider,
  model,
}: {
  readonly provider: string;
  readonly model: string;
},): SlotPool | typeof NO_POOL {
  return PI_POOLS.find(function ofThisPool(pool,) {
    return (pool.provider === provider) && (pool.model === model);
  },) ?? NO_POOL;
}

/**
 Counts the runs each pool holds.

 @param runs - every run that is going, with what its start named

 @returns how many runs each pool holds, by the pool's name

 @example
 ```ts
 const counts = countsOfPools({ runs: [{ name: 'a', launcherPool: NO_LAUNCHER, },], });
 ```
 */
export function countsOfPools({
  runs,
}: {
  readonly runs: readonly RunUnderAPool[];
},): ReadonlyMap<string, number> {
  /**
   What every pool holds, so a pool nothing counts still prints its zero.
   */
  const counts = new Map<string, number>(PI_POOLS.map(function withZero(pool,) {
    return [
      pool.name,
      0,
    ] as const;
  },),);
  for (const run of runs) {
    if (run.name
      .startsWith(AUDIT_RUN_PREFIX,)) {
      continue;
    }
    if ((run.name
      .startsWith(KIMI_RUN_PREFIX,))
      && (run.launcherPool === NO_LAUNCHER)) {
      continue;
    }
    if (run.launcherPool === NO_POOL) {
      continue;
    }
    /**
     The pool this run counts in.
     */
    const poolName = (run.launcherPool === NO_LAUNCHER)
      ? POOL_OF_A_RUN_WITH_NO_LAUNCHER
      : run.launcherPool
        .name;
    counts.set(
      poolName,
      (counts.get(poolName,) ?? 0) + 1,
    );
  }
  return counts;
}

/**
 Reads whether a pool has room for one more run.

 @param pool - the pool to read

 @param counts - how many runs each pool holds

 @returns whether the pool holds fewer runs than its limit

 @example
 ```ts
 const room = hasRoom({ pool: PI_POOLS[0] ?? PI_POOLS[0], counts: new Map(), });
 ```
 */
export function hasRoom({
  pool,
  counts,
}: {
  readonly pool: SlotPool;
  readonly counts: ReadonlyMap<string, number>;
},): boolean {
  return (counts.get(pool.name,) ?? 0) < pool.limit;
}

/**
 Writes the counts as one line, every pool with its limit.

 @param counts - how many runs each pool holds

 @returns the counts in the owner's pool order

 @example
 ```ts
 const summary = countsSummaryOf({ counts: new Map(), });
 ```
 */
export function countsSummaryOf({
  counts,
}: {
  readonly counts: ReadonlyMap<string, number>;
},): string {
  return PI_POOLS.map(function asCountOfLimit(pool,) {
    return `${pool.name} ${String(counts.get(pool.name,) ?? 0,)}/${String(pool.limit,)}`;
  },)
    .join(', ',);
}

/**
 Reads whether a run stands in the way of a second run of the same name. An audit may be repeated under one name, so it is
 never the one that stands in the way.

 @param name - the name the caller asked for

 @param going - every run that is going

 @returns whether a run of that name is already going

 @example
 ```ts
 const going = runIsAlreadyGoing({ name: 'copy-route', going: new Set(['copy-route',],), });
 ```
 */
export function runIsAlreadyGoing({
  name,
  going,
}: {
  readonly name: string;
  readonly going: ReadonlySet<string>;
},): boolean {
  return going.has(name,) && (!name.startsWith(AUDIT_RUN_PREFIX,));
}

/**
 Chooses the pool a new run goes to.

 @param requested - the pool the caller's provider and model name, or {@link NO_POOL}

 @param resumes - whether the run resumes a session, which stays in the pool that session was started in

 @param counts - how many runs each pool holds

 @returns the chosen pool, or {@link NO_POOL} when none has room

 @example
 ```ts
 const chosen = poolWithRoom({ requested: PI_POOLS[1] ?? PI_POOLS[0], resumes: false, counts: new Map(), });
 ```
 */
export function poolWithRoom({
  requested,
  resumes,
  counts,
}: {
  readonly requested: SlotPool | typeof NO_POOL;
  readonly resumes: boolean;
  readonly counts: ReadonlyMap<string, number>;
},): SlotPool | typeof NO_POOL {
  /**
   Whether the run may move to another pool: only a run on a default-run pool, and only when it starts a new session.
   */
  const mayMove = (requested !== NO_POOL) && requested.takesDefaultRuns
    && (!resumes);
  if (!mayMove) {
    return requested;
  }
  return PI_POOLS.filter(function takesDefaultRuns(pool,) {
    return pool.takesDefaultRuns;
  },)
    .find(function hasRoomForOne(pool,) {
      return hasRoom({
        pool,
        counts,
      },);
    },)
    ?? NO_POOL;
}

/**
 Reads the wall clock a run gets in the pool it went to, which is longer in a slower pool.

 @param pool - the pool the run goes to

 @param wallSeconds - the wall clock the caller asked for

 @returns the wall clock the run is given

 @example
 ```ts
 const wall = wallSecondsInPool({ pool: PI_POOLS[0], wallSeconds: 600, });
 ```
 */
export function wallSecondsInPool({
  pool,
  wallSeconds,
}: {
  readonly pool: SlotPool;
  readonly wallSeconds: number;
},): number {
  return wallSeconds * pool.wallFactor;
}
