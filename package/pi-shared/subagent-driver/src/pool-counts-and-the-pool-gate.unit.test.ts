/**
 What the runs going now hold of each provider pool, which pool a run may take, and the room the build volume has: the counts,
 the summary line, and the answer the pool gate gives. Everything here is read from the runs, the pools and the room a test
 hands over, so no unit, no process, no systemd and no disk of this machine is involved.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  ALREADY_RUNNING_EXIT_STATUS,
  BYTES_IN_A_GIBIBYTE,
  CLOSED_POOL_LIMIT,
  type FreeSpace,
  FREE_SPACE_FLOOR_BYTES,
  type GateOutcome,
  type GoingRun,
  type LauncherArguments,
  MODEL_OF_A_RUN_NAMING_NONE,
  MODEL_OPTION,
  NEURALWATT_DEEPSEEK_FLEX,
  NEURALWATT_KIMI_FLEX,
  NEURALWATT_STANDARD,
  NO_POOL_HAS_ROOM,
  NO_SLOT_EXIT_STATUS,
  OPENCODE_GO,
  POOL_LIMITS_VARIABLE,
  PROVIDER_OF_A_RUN_NAMING_NONE,
  PROVIDER_OPTION,
  PROVIDER_POOLS,
  PoolGateOptionsError,
  type PoolGateOptions,
  type PoolMembership,
  PoolLimitsError,
  type ProviderPool,
  QWEN_PLAN_DEEPSEEK,
  QWEN_PLAN_QWEN4_MAX_PREVIEW,
  WALL_OPTION,
  firstDefaultPoolWithRoom,
  freeSpaceFloorTextOf,
  freeSpaceTextOf,
  gateOutcomeOf,
  launcherArgumentsOf,
  limitsIn,
  membershipOf,
  poolChoiceFor,
  poolHasRoom,
  poolNamed,
  poolTakesNoNewRun,
  poolsHeldBy,
  poolsWithTheseLimits,
  roomIn,
  summaryTextOf,
  talliesOf,
} from '../dist/final/node/index.mjs';

/**
 The volume every room in this file is on, which is no volume of this machine.

 */
const TEST_VOLUME = '/a-build-volume-of-the-test';

/**
 One run going now, in the pool a test names.

 @param name - the run's name

 @param pool - the membership to count it with, which is outside every pool when absent

 @returns the run

 @example
 ```ts
 const run = aRun('copy-route');
 ```
 */
function aRun(name: string, pool?: PoolMembership,): GoingRun {
  return {
    name,
    membership: pool ?? { holds: 'no-pool', },
  };
}

/**
 A membership in one pool.

 @param pool - the pool

 @returns the membership

 @example
 ```ts
 const inTheFlexPool = inPool(NEURALWATT_DEEPSEEK_FLEX);
 ```
 */
function inPool(pool: ProviderPool,): PoolMembership {
  return {
    holds: 'a-pool',
    pool,
  };
}

/**
 A membership a command line could not place: every pool such a run could be running on.

 @param pools - the pools the run could hold

 @returns the membership

 @example
 ```ts
 const unknown = couldBeIn(PROVIDER_POOLS);
 ```
 */
function couldBeIn(pools: readonly ProviderPool[],): PoolMembership {
  return {
    holds: 'some-pools',
    pools,
  };
}

/**
 Runs of one pool, named after their pool so a failure prints which count is wrong.

 @param pool - the pool to fill

 @param count - how many runs to make

 @returns the runs

 @example
 ```ts
 const runs = runsIn(QWEN_PLAN_DEEPSEEK, QWEN_PLAN_DEEPSEEK.limit);
 ```
 */
function runsIn(pool: ProviderPool, count: number,): readonly GoingRun[] {
  return Array.from(
    { length: count, },
    function named(_unused, index,) {
      return aRun(
        `${pool.name}-${String(index,)}`,
        inPool(pool,),
      );
    },
  );
}

/**
 Every pool a run that names no model of its own may take, which is every pool that takes default runs and is open.

 @returns the pools, in the order such a run takes them

 @example
 ```ts
 const pools = openDefaultPools();
 ```
 */
function openDefaultPools(): readonly ProviderPool[] {
  return PROVIDER_POOLS.filter(function takesDefaultRunsAndIsOpen(pool,) {
    return pool.takesDefaultRuns && (!poolTakesNoNewRun({ pool, },));
  },);
}

/**
 One run in every slot of every pool the test hands over.

 @param pools - the pools to fill

 @returns the runs

 @example
 ```ts
 const runs = runsFilling(openDefaultPools());
 ```
 */
function runsFilling(pools: readonly ProviderPool[],): readonly GoingRun[] {
  return pools.flatMap(function everySlot(pool,) {
    return runsIn(
      pool,
      pool.limit,
    );
  },);
}

/**
 The pools with the Kimi pool reopened at its own limit, the way a caller reopens it for one command with the limits variable,
 because the shipped table keeps that pool closed.

 @returns the pools, with the Kimi pool open

 @example
 ```ts
 const pools = reopenedKimiPool();
 ```
 */
function reopenedKimiPool(): readonly ProviderPool[] {
  // The shipped table closes this pool, which is what makes reopening it a caller's act: were it open, no test here could tell a
  // caller's room from the pool's own, and the closure could be taken out without one of them failing.
  expect(poolTakesNoNewRun({ pool: NEURALWATT_KIMI_FLEX, },),).toBe(true,);
  return poolsWithTheseLimits({
    limits: limitsIn({ listed: `${NEURALWATT_KIMI_FLEX.name}=${String(NEURALWATT_KIMI_FLEX.limit,)}`, },),
  },);
}

/**
 The pool a run that names no model of its own takes while every pool is empty, read from the pool table rather than written
 down here.

 @returns the pool

 @throws Error when the table takes no default run at all, so expectations built from it could not hold

 @example
 ```ts
 const pool = theFirstDefaultPool();
 ```
 */
function theFirstDefaultPool(): ProviderPool {
  /**
   What the pool table answers about a machine on which nothing is going.
   */
  const pool = firstDefaultPoolWithRoom({ tallies: talliesOf({ runs: [], },), },);
  if (pool === NO_POOL_HAS_ROOM) {
    throw new Error('no pool takes a default run, so this test cannot expect a pool to take one',);
  }
  return pool;
}

/**
 How many of the runs hold a slot in one pool, counting a run whose pool could not be read in every pool it could be in.

 @param runs - the runs going now

 @param pool - the pool to count

 @returns how many runs of those hold a slot in the pool

 @example
 ```ts
 const count = countOf({ runs, pool: QWEN_PLAN_DEEPSEEK });
 ```
 */
function countOf({
  runs,
  pool,
}: {
  readonly runs: readonly GoingRun[];
  readonly pool: ProviderPool;
},): number {
  return runs.filter(function isInThisPool(run,) {
    return poolsHeldBy({ membership: run.membership, },)
      .some(function isThePool(held,) {
        return held.name === pool.name;
      },);
  },).length;
}

/**
 Says the counts line names every pool the table ships, in the table's order, each with the count these runs give it and its own
 limit.

 @param line - the counts line, without the room the volume has

 @param runs - the runs the count is read from

 @example
 ```ts
 expectCountsLineNamesEachPool({ line: summaryTextOf(tallies), runs });
 ```
 */
function expectCountsLineNamesEachPool({
  line,
  runs,
}: {
  readonly line: string;
  readonly runs: readonly GoingRun[];
},): void {
  /**
   One entry per pool, in the order the line prints them.
   */
  const entries = line.split(', ',);
  expect(entries.length,).toBe(PROVIDER_POOLS.length,);
  for (const [index, pool,] of PROVIDER_POOLS.entries()) {
    expect(entries[index],).toBe(`${pool.name} ${String(countOf({
      runs,
      pool,
    },),)}/${String(roomIn(pool,),)}`,);
  }
}

/**
 Room on the build volume well above the floor, so a run can start in a test.

 @returns the room, and the volume it is on

 @example
 ```ts
 const room = roomToSpare();
 ```
 */
function roomToSpare(): FreeSpace {
  return {
    volume: TEST_VOLUME,
    freeBytes: FREE_SPACE_FLOOR_BYTES + BYTES_IN_A_GIBIBYTE,
  };
}

/**
 Room on the build volume under the floor, which is the fill of 2026-11-11 that this floor came from.

 @returns the room, and the volume it is on

 @example
 ```ts
 const room = roomUnderTheFloor();
 ```
 */
function roomUnderTheFloor(): FreeSpace {
  return {
    volume: TEST_VOLUME,
    freeBytes: FREE_SPACE_FLOOR_BYTES - 1,
  };
}

/**
 The pool gate's options, as a test writes them.

 @param options - what the test cares about

 @returns the options, with the folder the runs live in filled in

 @example
 ```ts
 const options = gateOptions({ name: 'copy-route', });
 ```
 */
function gateOptions(options: {
  readonly name?: string;
  readonly countsOnly?: boolean;
  readonly provider?: string;
  readonly model?: string;
  readonly wallSeconds?: number;
  readonly sessionOfRunFolder?: string;
  readonly commandLine?: readonly string[];
},): PoolGateOptions {
  return {
    runsFolder: '/runs',
    countsOnly: options.countsOnly ?? false,
    ...(options.name === undefined ? {} : { name: options.name, }),
    ...(options.provider === undefined ? {} : { provider: options.provider, }),
    ...(options.model === undefined ? {} : { model: options.model, }),
    ...(options.wallSeconds === undefined ? {} : { wallSeconds: options.wallSeconds, }),
    ...(options.sessionOfRunFolder === undefined ? {} : { sessionOfRunFolder: options.sessionOfRunFolder, }),
    commandLine: options.commandLine ?? [],
  };
}

/**
 A session file reader that answers with a fixed file.

 @returns the file

 @example
 ```ts
 const outcome = await gateOutcomeOf({ options, runs: [], tallies, sessionFileOf: aSessionFile, freeSpace: roomToSpare(), });
 ```
 */
async function aSessionFile(): Promise<string> {
  return '/sessions/a.jsonl';
}

await describe({
  name: '',
  children: [
    describe({
      name: poolNamed.name,
      children: [
        it({
          name: 'reads the pool a provider and model share, and answers that they share none when they do not',
          fn: async () => {
            expect(poolNamed({
              provider: QWEN_PLAN_DEEPSEEK.provider,
              model: QWEN_PLAN_DEEPSEEK.model,
            },),).toEqual(inPool(QWEN_PLAN_DEEPSEEK,),);
            expect(poolNamed({
              provider: NEURALWATT_KIMI_FLEX.provider,
              model: NEURALWATT_KIMI_FLEX.model,
            },),).toEqual(inPool(NEURALWATT_KIMI_FLEX,),);
            expect(poolNamed({
              provider: QWEN_PLAN_QWEN4_MAX_PREVIEW.provider,
              model: QWEN_PLAN_QWEN4_MAX_PREVIEW.model,
            },),).toEqual(inPool(QWEN_PLAN_QWEN4_MAX_PREVIEW,),);
            expect(poolNamed({
              provider: 'synthetic',
              model: 'moonshotai/Kimi-K3.1-Pro',
            },),).toEqual({ holds: 'no-pool', },);
          },
        },),
        it({
          name: 'reads the provider and model a unit of each pool carries on its own command line as that pool, which is the accounts\' own wording rather than a copy of the table',
          fn: async () => {
            /**
             The `--provider` and `--model` words each pool's units and launchers carry, as the provider accounts name them, so
             a typo in one changes both this list and the pool it names.
             */
            const wordsOfAUnit: readonly (readonly [string, string, ProviderPool])[] = [
              ['qwen-token-plan-individual', 'deepseek-v4.1-flash-1017', QWEN_PLAN_DEEPSEEK],
              ['neuralwatt', 'deepseek-v4.1-flash', NEURALWATT_STANDARD],
              ['neuralwatt', 'deepseek-v4.1-flash-flex', NEURALWATT_DEEPSEEK_FLEX],
              ['opencode-go', 'deepseek-v4.1-flash', OPENCODE_GO],
              ['neuralwatt', 'kimi-k3.1-pro-flex', NEURALWATT_KIMI_FLEX],
              ['qwen-token-plan-individual', 'qwen4-max-preview', QWEN_PLAN_QWEN4_MAX_PREVIEW],
            ];
            expect(wordsOfAUnit.length,).toBe(PROVIDER_POOLS.length,);
            for (const [provider, model, pool,] of wordsOfAUnit) {
              expect(poolNamed({
                provider,
                model,
              },),).toEqual(inPool(pool,),);
            }
          },
        },),
        it({
          name: 'reads the provider and model a run names nothing of as standard Neuralwatt, the pool the qwen plan replaced, so the run is placed on the first pool with room',
          fn: async () => {
            expect(poolNamed({
              provider: PROVIDER_OF_A_RUN_NAMING_NONE,
              model: MODEL_OF_A_RUN_NAMING_NONE,
            },),).toEqual(inPool(NEURALWATT_STANDARD,),);
            expect(poolTakesNoNewRun({ pool: NEURALWATT_STANDARD, },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: poolTakesNoNewRun.name,
      children: [
        it({
          name: 'says a pool is closed exactly when a run of its own cannot take it, and the table holds both a closed pool and an open one',
          fn: async () => {
            /**
             What the table holds of no pool at all.
             */
            const empty = talliesOf({ runs: [], },);
            for (const pool of PROVIDER_POOLS) {
              expect(poolTakesNoNewRun({ pool, },),).toBe(!poolHasRoom({
                tallies: empty,
                pool,
              },),);
            }
            expect(PROVIDER_POOLS.some(function isClosed(pool,) {
              return poolTakesNoNewRun({ pool, },);
            },),).toBe(true,);
            expect(PROVIDER_POOLS.some(function isOpen(pool,) {
              return !poolTakesNoNewRun({ pool, },);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'says the pool the qwen plan replaced, the qwen plan pool the owner took out and the Kimi pool the owner closed take no new run, and the pools that were given room do, the one whose week has reset among them',
          fn: async () => {
            expect(poolTakesNoNewRun({ pool: NEURALWATT_STANDARD, },),).toBe(true,);
            expect(poolTakesNoNewRun({ pool: QWEN_PLAN_DEEPSEEK, },),).toBe(true,);
            expect(poolTakesNoNewRun({ pool: NEURALWATT_KIMI_FLEX, },),).toBe(true,);
            expect(poolTakesNoNewRun({ pool: OPENCODE_GO, },),).toBe(false,);
            expect(poolTakesNoNewRun({ pool: NEURALWATT_DEEPSEEK_FLEX, },),).toBe(false,);
            expect(poolTakesNoNewRun({ pool: QWEN_PLAN_QWEN4_MAX_PREVIEW, },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: membershipOf.name,
      children: [
        it({
          name: 'counts a run in the pool its command line names, including a Kimi run that names the flex pool',
          fn: async () => {
            expect(membershipOf({
              runName: 'copy-route',
              pool: inPool(OPENCODE_GO,),
            },),).toEqual(inPool(OPENCODE_GO,),);
            expect(membershipOf({
              runName: 'kimi-review-round-9',
              pool: inPool(NEURALWATT_KIMI_FLEX,),
            },),).toEqual(inPool(NEURALWATT_KIMI_FLEX,),);
          },
        },),
        it({
          name: 'counts no slot for an audit, or for a Kimi run on a provider no pool holds',
          fn: async () => {
            expect(membershipOf({ runName: 'sol-audit-1', }),).toEqual({ holds: 'no-pool', },);
            expect(membershipOf({
              runName: 'sol-audit-1',
              pool: inPool(NEURALWATT_STANDARD,),
            },),).toEqual({ holds: 'no-pool', },);
          },
        },),
        it({
          name: 'holds a slot in every pool a run could be in when its command line could not be read',
          fn: async () => {
            expect(membershipOf({ runName: 'copy-route', }),).toEqual(couldBeIn(PROVIDER_POOLS,),);
            expect(membershipOf({ runName: 'kimi-review-a', }),).toEqual(couldBeIn([NEURALWATT_KIMI_FLEX,],),);
          },
        },),
      ],
    },),
    describe({
      name: talliesOf.name,
      children: [
        it({
          name: 'counts every pool, at nothing when no run holds it, and leaves a run in no pool out',
          fn: async () => {
            /**
             Two runs on the qwen plan's pool and one outside every pool.
             */
            const runs = [
              ...runsIn(QWEN_PLAN_DEEPSEEK, 2,),
              aRun('sol-audit-1',),
            ];
            /**
             What the runs hold of each pool.
             */
            const tallies = talliesOf({ runs, },);
            expect(tallies.counts.get(QWEN_PLAN_DEEPSEEK.name,),).toBe(2,);
            expect(tallies.counts.get(NEURALWATT_DEEPSEEK_FLEX.name,),).toBe(0,);
            expectCountsLineNamesEachPool({
              line: summaryTextOf(tallies,),
              runs,
            },);
          },
        },),
        it({
          name: 'counts a run whose pool could not be read in every pool it could be running on, and says so in the line',
          fn: async () => {
            /**
             One run whose command line could not be read, and one that named the Kimi pool and could not be read either.
             */
            const runs = [
              aRun('copy-route', couldBeIn(PROVIDER_POOLS,),),
              aRun('kimi-review-a', couldBeIn([NEURALWATT_KIMI_FLEX,],),),
            ];
            /**
             What the two unreadable runs hold of each pool.
             */
            const tallies = talliesOf({ runs, },);
            expect(tallies.counts.get(NEURALWATT_STANDARD.name,),).toBe(1,);
            expect(tallies.counts.get(NEURALWATT_DEEPSEEK_FLEX.name,),).toBe(1,);
            expect(tallies.counts.get(OPENCODE_GO.name,),).toBe(1,);
            expect(tallies.counts.get(NEURALWATT_KIMI_FLEX.name,),).toBe(2,);
            expectCountsLineNamesEachPool({
              line: summaryTextOf(tallies,),
              runs,
            },);
          },
        },),
      ],
    },),
    describe({
      name: summaryTextOf.name,
      children: [
        it({
          name: 'names every pool the table ships, in the order a run that names no model of its own takes them, each with its own limit',
          fn: async () => {
            expectCountsLineNamesEachPool({
              line: summaryTextOf(talliesOf({ runs: [], },),),
              runs: [],
            },);
          },
        },),
      ],
    },),
    describe({
      name: poolHasRoom.name,
      children: [
        it({
          name: 'says a pool with a slot left has room, one at its limit has none, and a closed pool never has any',
          fn: async () => {
            /**
             A pool an ordinary run goes on, which is open and takes default runs.
             */
            const pool = theFirstDefaultPool();
            expect(poolHasRoom({
              tallies: talliesOf({ runs: runsIn(pool, pool.limit - 1,), },),
              pool,
            },),).toBe(true,);
            expect(poolHasRoom({
              tallies: talliesOf({ runs: runsIn(pool, pool.limit,), },),
              pool,
            },),).toBe(false,);
            for (const closed of PROVIDER_POOLS.filter(function isClosed(candidate,) {
              return poolTakesNoNewRun({ pool: candidate, },);
            },)) {
              expect(poolHasRoom({
                tallies: talliesOf({ runs: [], },),
                pool: closed,
              },),).toBe(false,);
            }
          },
        },),
      ],
    },),
    describe({
      name: firstDefaultPoolWithRoom.name,
      children: [
        it({
          name: 'takes the owner order, which reaches opencode-go first because the qwen plan pool before it is closed, goes on to Neuralwatt flex once opencode-go is full, and skips a pool that takes only its own model',
          fn: async () => {
            expect(firstDefaultPoolWithRoom({ tallies: talliesOf({ runs: [], },), },),).toEqual(OPENCODE_GO,);
            // Runs still going on the pool the owner took out are counted there, and take no room from the pool a new run goes to.
            expect(firstDefaultPoolWithRoom({
              tallies: talliesOf({ runs: runsIn(QWEN_PLAN_DEEPSEEK, OPENCODE_GO.limit,), },),
            },),).toEqual(OPENCODE_GO,);
            // The pool the owner prefers is taken until it is full, and only then the next one.
            expect(firstDefaultPoolWithRoom({
              tallies: talliesOf({ runs: runsIn(OPENCODE_GO, OPENCODE_GO.limit - 1,), },),
            },),).toEqual(OPENCODE_GO,);
            expect(firstDefaultPoolWithRoom({
              tallies: talliesOf({ runs: runsIn(OPENCODE_GO, OPENCODE_GO.limit,), },),
            },),).toEqual(NEURALWATT_DEEPSEEK_FLEX,);
            // With the preferred pool closed by a caller, as when its week is spent, the next one is taken at once.
            expect(firstDefaultPoolWithRoom({
              tallies: talliesOf({
                runs: [],
                pools: poolsWithTheseLimits({ limits: new Map([[OPENCODE_GO.name, CLOSED_POOL_LIMIT],]), },),
              },),
            },),).toEqual(NEURALWATT_DEEPSEEK_FLEX,);
          },
        },),
        it({
          name: 'answers that no pool has room once every open pool that takes a default run is full, because the closed ones take no new run',
          fn: async () => {
            /**
             The pools a run that names no model of its own may take.
             */
            const open = openDefaultPools();
            expect(open.length,).toBeGreaterThan(0,);
            expect(firstDefaultPoolWithRoom({ tallies: talliesOf({ runs: runsFilling(open,), },), },),).toBe(NO_POOL_HAS_ROOM,);
          },
        },),
      ],
    },),
    describe({
      name: limitsIn.name,
      children: [
        it({
          name: 'reads the limits a caller set, and refuses a pair that names no pool, holds no whole number, or misses its equals sign',
          fn: async () => {
            expect(limitsIn({ listed: 'opencode-go=20, neuralwatt-kimi-flex=0', },),).toEqual(new Map([
              ['opencode-go', 20],
              ['neuralwatt-kimi-flex', 0],
            ],),);
            expect(limitsIn({ listed: '', },),).toEqual(new Map(),);
            expect(() => limitsIn({ listed: 'neuralwatt=nineteen', },),).toThrow(PoolLimitsError,);
            expect(() => limitsIn({ listed: 'no-such-pool=1', },),).toThrow('no-such-pool',);
            expect(() => limitsIn({ listed: 'opencode-go', },),).toThrow(PoolLimitsError,);
          },
        },),
      ],
    },),
    describe({
      name: poolsWithTheseLimits.name,
      children: [
        it({
          name: 'opens a pool the owner closed, closes an open one a caller set to zero, and leaves every other pool as it ships',
          fn: async () => {
            /**
             The shipped pool the owner closed.
             */
            const asItShips = PROVIDER_POOLS.find(function isKimiFlex(pool,) {
              return pool.name === NEURALWATT_KIMI_FLEX.name;
            },) ?? NEURALWATT_KIMI_FLEX;
            expect(roomIn(asItShips,),).toBe(CLOSED_POOL_LIMIT,);
            /**
             The same pool with the room a caller gives it, which is another open pool's own limit rather than a number written
             down here.
             */
            const opened = poolsWithTheseLimits({ limits: new Map([[NEURALWATT_KIMI_FLEX.name, NEURALWATT_DEEPSEEK_FLEX.limit],]), },)
              .find(function isKimiFlex(pool,) {
                return pool.name === NEURALWATT_KIMI_FLEX.name;
              },) ?? NEURALWATT_KIMI_FLEX;
            expect(opened.closed,).toBeUndefined();
            expect(roomIn(opened,),).toBe(NEURALWATT_DEEPSEEK_FLEX.limit,);
            /**
             The pool with a weekly limit, which ships open and which a caller closes when its week is spent.
             */
            const shippedOpen = PROVIDER_POOLS.find(function isOpencodeGo(pool,) {
              return pool.name === OPENCODE_GO.name;
            },) ?? OPENCODE_GO;
            expect(roomIn(shippedOpen,),).toBe(OPENCODE_GO.limit,);
            expect(shippedOpen.closed,).toBeUndefined();
            /**
             That pool with a caller's zero, which says who closed it.
             */
            const spent = poolsWithTheseLimits({ limits: new Map([[OPENCODE_GO.name, CLOSED_POOL_LIMIT],]), },)
              .find(function isOpencodeGo(pool,) {
                return pool.name === OPENCODE_GO.name;
              },) ?? OPENCODE_GO;
            expect(roomIn(spent,),).toBe(CLOSED_POOL_LIMIT,);
            expect(poolTakesNoNewRun({ pool: spent, },),).toBe(true,);
            expect(spent.closed?.reason,).toContain(POOL_LIMITS_VARIABLE,);
            /**
             A pool a caller closed, which keeps the room it returns to and says who closed it.
             */
            const closed = poolsWithTheseLimits({ limits: new Map([[NEURALWATT_STANDARD.name, CLOSED_POOL_LIMIT],]), },)
              .find(function isStandard(pool,) {
                return pool.name === NEURALWATT_STANDARD.name;
              },) ?? NEURALWATT_STANDARD;
            expect(roomIn(closed,),).toBe(CLOSED_POOL_LIMIT,);
            expect(closed.closed?.reason,).toContain(POOL_LIMITS_VARIABLE,);
            /**
             A pool the caller named nothing about.
             */
            const untouched = poolsWithTheseLimits({ limits: new Map(), },)
              .find(function isFlex(pool,) {
                return pool.name === NEURALWATT_DEEPSEEK_FLEX.name;
              },) ?? NEURALWATT_DEEPSEEK_FLEX;
            expect(untouched,).toEqual(NEURALWATT_DEEPSEEK_FLEX,);
          },
        },),
        it({
          name: 'gives a pool the caller\'s own number, which is no limit the table ships, so a caller\'s room is never read as the shipped one',
          fn: async () => {
            /**
             A room above every limit the table ships, so no pool's own number can be mistaken for it, and a table whose limits
             change moves it with them.
             */
            const roomAboveEveryShippedLimit = Math.max(...PROVIDER_POOLS.map(function limitOf(pool,) {
              return pool.limit;
            },),) + 1;
            expect(PROVIDER_POOLS.some(function shipsIt(pool,) {
              return pool.limit === roomAboveEveryShippedLimit;
            },),).toBe(false,);
            for (const pool of PROVIDER_POOLS) {
              /**
               The same pool with the caller's number, which is a room whether the pool ships open or closed.
               */
              const given = poolsWithTheseLimits({ limits: new Map([[pool.name, roomAboveEveryShippedLimit],]), },)
                .find(function isThatPool(candidate,) {
                  return candidate.name === pool.name;
                },) ?? pool;
              expect(roomIn(given,),).toBe(roomAboveEveryShippedLimit,);
              expect(poolTakesNoNewRun({ pool: given, },),).toBe(false,);
            }
          },
        },),
      ],
    },),
    describe({
      name: poolChoiceFor.name,
      children: [
        it({
          name: 'gives no room to a run naming a pool a caller closed through the variable, which is what the variable is for',
          fn: async () => {
            /**
             The pools with the qwen 4 pool closed by a caller, as a coordinator closes a pool whose week is spent.
             */
            const pools = poolsWithTheseLimits({
              limits: limitsIn({ listed: `${QWEN_PLAN_QWEN4_MAX_PREVIEW.name}=${String(CLOSED_POOL_LIMIT,)}`, },),
            },);
            /**
             The pool the run's own words name, which is the pool the caller closed.
             */
            const requested = poolNamed({
              provider: QWEN_PLAN_QWEN4_MAX_PREVIEW.provider,
              model: QWEN_PLAN_QWEN4_MAX_PREVIEW.model,
            },);
            expect(poolChoiceFor({
              requested,
              tallies: talliesOf({
                runs: [],
                pools,
              },),
              resumes: false,
            },),).toEqual({ destination: 'no-room', },);
          },
        },),
        it({
          name: 'moves a run that names a default model, here the closed pool the launcher defaults name, to the first pool with room, and says it was moved',
          fn: async () => {
            expect(poolChoiceFor({
              requested: inPool(NEURALWATT_STANDARD,),
              tallies: talliesOf({ runs: [], },),
              resumes: false,
            },),).toEqual({
              destination: 'a-pool',
              pool: theFirstDefaultPool(),
              movesTheRun: true,
            },);
          },
        },),
        it({
          name: 'keeps a run that resumes a session on the pool it named while that pool takes new runs, with no wall factor',
          fn: async () => {
            expect(poolChoiceFor({
              requested: inPool(NEURALWATT_DEEPSEEK_FLEX,),
              tallies: talliesOf({ runs: [], },),
              resumes: true,
            },),).toEqual({
              destination: 'a-pool',
              pool: NEURALWATT_DEEPSEEK_FLEX,
              movesTheRun: false,
            },);
          },
        },),
        it({
          name: 'moves a resume whose own pool is closed to the first pool with room, because staying on it would refuse the run for ever',
          fn: async () => {
            /**
             Every pool the owner closed, which a resumed run may name.
             */
            const closed = PROVIDER_POOLS.filter(function isClosed(pool,) {
              return pool.takesDefaultRuns && poolTakesNoNewRun({ pool, },);
            },);
            expect(closed.length,).toBeGreaterThan(0,);
            for (const pool of closed) {
              expect(poolChoiceFor({
                requested: inPool(pool,),
                tallies: talliesOf({ runs: [], },),
                resumes: true,
              },),).toEqual({
                destination: 'a-pool',
                pool: theFirstDefaultPool(),
                movesTheRun: true,
              },);
            }
          },
        },),
        it({
          name: 'keeps a run that names the Kimi model on the Kimi pool while it has room, never moving it to a default pool',
          fn: async () => {
            /**
             The Kimi pool as a caller reopens it, because the shipped table closes it and this test needs an open pool that only
             a run naming its own model takes.
             */
            const reopened = reopenedKimiPool();
            expect(poolChoiceFor({
              requested: inPool(NEURALWATT_KIMI_FLEX,),
              tallies: talliesOf({
                runs: [],
                pools: reopened,
              },),
              resumes: false,
            },),).toEqual({
              destination: 'a-pool',
              pool: reopened.find(function isTheKimiPool(pool,) {
                return pool.name === NEURALWATT_KIMI_FLEX.name;
              },),
              movesTheRun: false,
            },);
          },
        },),
        it({
          name: 'gives no room to a run naming the Kimi model from the shipped table, still counts the runs already on that pool, and gives it its room back when a caller names the limit',
          fn: async () => {
            /**
             The pool the run's own words name, read from the shipped table the way a launcher reads it.
             */
            const requested = poolNamed({
              provider: NEURALWATT_KIMI_FLEX.provider,
              model: NEURALWATT_KIMI_FLEX.model,
            },);
            /**
             Runs already going on the Kimi pool, as many as its own limit, so the count under test is the limit and no number
             written down here.
             */
            const going = runsIn(NEURALWATT_KIMI_FLEX, NEURALWATT_KIMI_FLEX.limit,);
            /**
             What the shipped table counts of those runs.
             */
            const shipped = talliesOf({ runs: going, },);
            expect(poolChoiceFor({
              requested,
              tallies: talliesOf({ runs: [], },),
              resumes: false,
            },),).toEqual({ destination: 'no-room', },);
            expect(shipped.counts.get(NEURALWATT_KIMI_FLEX.name,),).toBe(going.length,);
            expect(typeof NEURALWATT_KIMI_FLEX.closed?.reason,).toBe('string',);
            expect(NEURALWATT_KIMI_FLEX.limit,).toBeGreaterThan(0,);
            expect(roomIn(NEURALWATT_KIMI_FLEX,),).toBe(CLOSED_POOL_LIMIT,);
            /**
             The same table with the limit named by a caller through the variable, which is how one command reopens the pool.
             */
            const reopened = reopenedKimiPool();
            expect(poolChoiceFor({
              requested,
              tallies: talliesOf({
                runs: [],
                pools: reopened,
              },),
              resumes: false,
            },),).toEqual({
              destination: 'a-pool',
              pool: reopened.find(function isTheKimiPool(pool,) {
                return pool.name === NEURALWATT_KIMI_FLEX.name;
              },),
              movesTheRun: false,
            },);
          },
        },),
        it({
          name: 'sends a run whose provider no pool holds on its own provider',
          fn: async () => {
            expect(poolChoiceFor({
              requested: { holds: 'no-pool', },
              tallies: talliesOf({ runs: [], },),
              resumes: false,
            },),).toEqual({ destination: 'its-own-provider', },);
          },
        },),
        it({
          name: 'says no pool has room when every pool the run could take is full, including the Kimi pool it named',
          fn: async () => {
            /**
             Every pool a default run can take, filled.
             */
            const filled = talliesOf({ runs: runsFilling(openDefaultPools(),), },);
            expect(poolChoiceFor({
              requested: inPool(NEURALWATT_STANDARD,),
              tallies: filled,
              resumes: false,
            },),).toEqual({ destination: 'no-room', },);
            // The shipped table closes the Kimi pool, so the full pool this assertion needs is the one a caller reopens.
            expect(poolChoiceFor({
              requested: inPool(NEURALWATT_KIMI_FLEX,),
              tallies: talliesOf({
                runs: runsIn(NEURALWATT_KIMI_FLEX, NEURALWATT_KIMI_FLEX.limit,),
                pools: reopenedKimiPool(),
              },),
              resumes: false,
            },),).toEqual({ destination: 'no-room', },);
          },
        },),
      ],
    },),
    describe({
      name: launcherArgumentsOf.name,
      children: [
        it({
          name: 'writes the chosen pool into the options the caller gave, doubles the wall clock of a flex pool, and appends what the caller left out',
          fn: async () => {
            /**
             What the launcher is given for a run that named only itself.
             */
            const given: LauncherArguments = launcherArgumentsOf({
              commandLine: ['--name=copy-route', '--wall=600',],
              pool: NEURALWATT_DEEPSEEK_FLEX,
              movesTheRun: true,
              wallSeconds: 600,
              runsFolder: '/runs',
            },);
            expect(given.words,).toEqual([
              '--name=copy-route',
              `--wall=${String(600 * NEURALWATT_DEEPSEEK_FLEX.wallFactor,)}`,
              `${PROVIDER_OPTION}${NEURALWATT_DEEPSEEK_FLEX.provider}`,
              `${MODEL_OPTION}${NEURALWATT_DEEPSEEK_FLEX.model}`,
              '--runs-folder=/runs',
            ],);
            expect(given.addedOptions,).toContain(`${PROVIDER_OPTION}${NEURALWATT_DEEPSEEK_FLEX.provider}`,);
          },
        },),
        it({
          name: 'rewrites the provider and model the caller named when the run takes the first free pool',
          fn: async () => {
            expect(launcherArgumentsOf({
              commandLine: [`${PROVIDER_OPTION}${NEURALWATT_STANDARD.provider}`, `${MODEL_OPTION}${NEURALWATT_STANDARD.model}`, '--name=a',],
              pool: OPENCODE_GO,
              movesTheRun: true,
              runsFolder: '/runs',
            },).words,).toEqual([
              `${PROVIDER_OPTION}${OPENCODE_GO.provider}`,
              `${MODEL_OPTION}${OPENCODE_GO.model}`,
              '--name=a',
              '--runs-folder=/runs',
            ],);
          },
        },),
        it({
          name: 'leaves every word alone when the run keeps the pool it named',
          fn: async () => {
            expect(launcherArgumentsOf({
              commandLine: [`${PROVIDER_OPTION}${NEURALWATT_KIMI_FLEX.provider}`, '--name=a', '--wall=60',],
              pool: NEURALWATT_KIMI_FLEX,
              movesTheRun: false,
              wallSeconds: 60,
              runsFolder: '/runs',
            },).words,).toEqual([
              `${PROVIDER_OPTION}${NEURALWATT_KIMI_FLEX.provider}`,
              '--name=a',
              '--wall=60',
              '--runs-folder=/runs',
            ],);
          },
        },),
        it({
          name: 'turns a resumed run folder into its session file, and does not add the folder the caller already named',
          fn: async () => {
            expect(launcherArgumentsOf({
              commandLine: ['--name=a', '--session-of=copy-route-4', '--runs-folder=/elsewhere',],
              movesTheRun: false,
              sessionFile: '/sessions/4.jsonl',
              runsFolder: '/runs',
            },).words,).toEqual([
              '--name=a',
              '--session=/sessions/4.jsonl',
              '--runs-folder=/elsewhere',
            ],);
          },
        },),
        it({
          name: 'gives a run moved to the first pool with room the wall clock it asked for times that pool\'s own factor, and leaves the wall of a run it keeps',
          fn: async () => {
            /**
             The pool the gate moves a run that names no model of its own to while nothing is going.
             */
            const pool = theFirstDefaultPool();
            /**
             The wall clock the caller asked for, in seconds, which is the caller's own number and no limit of the table.
             */
            const askedFor = 600;
            expect(launcherArgumentsOf({
              commandLine: ['--name=copy-route', `${WALL_OPTION}${String(askedFor,)}`,],
              pool,
              movesTheRun: true,
              wallSeconds: askedFor,
              runsFolder: '/runs',
            },).words,).toContain(`${WALL_OPTION}${String(askedFor * pool.wallFactor,)}`,);
            // A run the gate leaves on its own pool keeps the wall clock it asked for, which is what makes the product above the
            // factor the pool adds rather than a second number of the gate's.
            expect(launcherArgumentsOf({
              commandLine: ['--name=copy-route', `${WALL_OPTION}${String(askedFor,)}`,],
              pool,
              movesTheRun: false,
              wallSeconds: askedFor,
              runsFolder: '/runs',
            },).words,).toContain(`${WALL_OPTION}${String(askedFor,)}`,);
          },
        },),
      ],
    },),
    describe({
      name: gateOutcomeOf.name,
      children: [
        it({
          name: 'answers the counts when asked for them alone, and needs no name for that',
          fn: async () => {
            /**
             What the runs going now hold of each pool, which is nothing.
             */
            const runs: readonly GoingRun[] = [];
            /**
             What the gate answered a caller that named no run.
             */
            const outcome: GateOutcome = await gateOutcomeOf({
              options: gateOptions({ countsOnly: true, }),
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('counts',);
            expect(outcome.lines,).toEqual([`${summaryTextOf(talliesOf({ runs, },),)}; ${freeSpaceTextOf(roomToSpare(),)}`,],);
            expectCountsLineNamesEachPool({
              line: (outcome.lines[0] ?? '').split('; ',)[0] ?? '',
              runs,
            },);
          },
        },),
        it({
          name: 'answers the counts even when a run of that name is going, because a count names no run',
          fn: async () => {
            /**
             A run of the name the gate was asked about.
             */
            const runs = [aRun('copy-route', inPool(QWEN_PLAN_DEEPSEEK,),),];
            /**
             What the gate answered while that run was going.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({
                countsOnly: true,
                name: 'copy-route',
              },),
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('counts',);
          },
        },),
        it({
          name: 'answers the counts with the room the build volume has, even while that room is under the floor, because reading the counts starts nothing',
          fn: async () => {
            /**
             What the volume holds while it is under the floor.
             */
            const room = roomUnderTheFloor();
            /**
             What the gate answered while the volume was that full.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ countsOnly: true, }),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: room,
            },);
            expect(outcome.action,).toBe('counts',);
            expect(outcome.lines,).toEqual([`${summaryTextOf(talliesOf({ runs: [], },),)}; ${freeSpaceTextOf(room,)}`,],);
          },
        },),
        it({
          name: 'refuses to start anything without a run to name',
          fn: async () => {
            try {
              await gateOutcomeOf({
                options: gateOptions({},),
                runs: [],
                tallies: talliesOf({ runs: [], },),
                sessionFileOf: aSessionFile,
                freeSpace: roomToSpare(),
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(PoolGateOptionsError,);
              expect(String(error,),).toContain('--name is required',);
              return;
            }
            throw new Error('a gate with no run name answered instead of refusing',);
          },
        },),
        it({
          name: 'answers that the run is already going, with the status the retrying start reads as no retry',
          fn: async () => {
            /**
             A run of the name the gate was asked about.
             */
            const runs = [aRun('copy-route', inPool(QWEN_PLAN_DEEPSEEK,),),];
            /**
             What the gate answered about that name.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'copy-route', }),
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('already-running',);
            expect((outcome.action === 'already-running') ? outcome.status : 0,).toBe(ALREADY_RUNNING_EXIT_STATUS,);
            expect(outcome.lines,).toEqual(['ALREADY RUNNING: copy-route',],);
          },
        },),
        it({
          name: 'answers that the run is already going even while the volume is under the floor, because that answer needs no room',
          fn: async () => {
            /**
             A run of the name the gate was asked about.
             */
            const runs = [aRun('copy-route', inPool(QWEN_PLAN_DEEPSEEK,),),];
            /**
             What the gate answered about that name while the volume was under the floor.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'copy-route', }),
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomUnderTheFloor(),
            },);
            expect(outcome.action,).toBe('already-running',);
            expect(outcome.lines,).toEqual(['ALREADY RUNNING: copy-route',],);
          },
        },),
        it({
          name: 'refuses to start a run while the volume that holds the repository is under the floor, saying how much room there is and how much a run needs',
          fn: async () => {
            /**
             What the volume holds while it is under the floor.
             */
            const room = roomUnderTheFloor();
            /**
             What the gate answered while the volume was that full.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'copy-route', }),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: room,
            },);
            expect(outcome.action,).toBe('no-room',);
            expect((outcome.action === 'no-room') ? outcome.status : 0,).toBe(NO_SLOT_EXIT_STATUS,);
            expect(outcome.lines,).toEqual([
              `NO SLOT: ${freeSpaceTextOf(room,)}, under the floor of ${freeSpaceFloorTextOf()}; not started: copy-route`,
            ],);
          },
        },),
        it({
          name: 'starts a second run of an audit name, because an audit holds no slot to conflict with',
          fn: async () => {
            /**
             An audit of the name the gate was asked about.
             */
            const runs = [aRun('sol-audit-1',),];
            /**
             What the gate answered about that audit name.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'sol-audit-1', }),
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
          },
        },),
        it({
          name: 'answers no slot with the status the retrying start waits on',
          fn: async () => {
            /**
             Every pool a run that names a default model could take, filled.
             */
            const filled = runsFilling(openDefaultPools(),);
            /**
             What the gate answered while every one of them was full.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'copy-route', }),
              runs: filled,
              tallies: talliesOf({ runs: filled, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('no-room',);
            expect((outcome.action === 'no-room') ? outcome.status : 0,).toBe(NO_SLOT_EXIT_STATUS,);
            expect(outcome.lines[0],).toContain(`NO SLOT: ${summaryTextOf(talliesOf({ runs: filled, },),)}`,);
            expect(outcome.lines[0],).toContain('not started: copy-route',);
          },
        },),
        it({
          name: 'starts a run on its own provider when no pool holds that provider, saying so',
          fn: async () => {
            /**
             What the gate answered for a Kimi run on Synthetic.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({
                name: 'kimi-review-round-9',
                provider: 'synthetic',
                model: 'moonshotai/Kimi-K3.1-Pro',
              },),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
            expect(outcome.lines[0],).toContain('goes on its own provider',);
          },
        },),
        it({
          name: 'reads the session file of the run a run resumes, and hands it to the launcher',
          fn: async () => {
            /**
             What the gate answered for a run that resumes another.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({
                name: 'copy-route-2',
                sessionOfRunFolder: 'copy-route-1',
                commandLine: ['--name=copy-route-2', '--session-of=copy-route-1',],
              },),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
            expect(outcome.action === 'start' ? outcome.arguments.words : [],).toContain('--session=/sessions/a.jsonl',);
          },
        },),
        it({
          name: 'starts a resume on the first pool with room when the pool the run resumes on is closed, naming that pool in the launcher words',
          fn: async () => {
            expect(poolTakesNoNewRun({ pool: NEURALWATT_STANDARD, },),).toBe(true,);
            /**
             What the gate answered for a run that resumes a run on the closed pool.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({
                name: 'copy-route-2',
                provider: NEURALWATT_STANDARD.provider,
                model: NEURALWATT_STANDARD.model,
                sessionOfRunFolder: 'copy-route-1',
                commandLine: [
                  '--name=copy-route-2',
                  `${PROVIDER_OPTION}${NEURALWATT_STANDARD.provider}`,
                  `${MODEL_OPTION}${NEURALWATT_STANDARD.model}`,
                  '--session-of=copy-route-1',
                ],
              },),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
            expect(outcome.action === 'start' ? outcome.arguments.words : [],).toContain(
              `${PROVIDER_OPTION}${theFirstDefaultPool().provider}`,
            );
            expect(outcome.action === 'start' ? outcome.arguments.words : [],).toContain(
              `${MODEL_OPTION}${theFirstDefaultPool().model}`,
            );
          },
        },),
        it({
          name: 'moves a resumed session that names no provider and no model to the first pool with room, and gives it that pool\'s wall factor',
          fn: async () => {
            /**
             The pool the gate moves a run that names nothing to while nothing is going, which is the pool a resumed session that
             names nothing lands on.
             */
            const pool = theFirstDefaultPool();
            /**
             The wall clock the caller asked for, in seconds.
             */
            const askedFor = 600;
            /**
             What the gate answered for a session resumed with no provider and no model of its own named.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({
                name: 'copy-route-2',
                sessionOfRunFolder: 'copy-route-1',
                wallSeconds: askedFor,
                commandLine: [
                  '--name=copy-route-2',
                  '--session-of=copy-route-1',
                  `${WALL_OPTION}${String(askedFor,)}`,
                ],
              },),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
            expect(outcome.lines[0],).toContain(`goes on ${pool.name}`,);
            /**
             The words the launcher is given.
             */
            const words = (outcome.action === 'start') ? outcome.arguments.words : [];
            expect(words,).toContain(`${PROVIDER_OPTION}${pool.provider}`,);
            expect(words,).toContain(`${MODEL_OPTION}${pool.model}`,);
            expect(words,).toContain(`${WALL_OPTION}${String(askedFor * pool.wallFactor,)}`,);
          },
        },),
        it({
          name: 'keeps a resumed session that names the flex pool on it, with the wall clock its caller asked for and no second factor',
          fn: async () => {
            /**
             The wall clock the caller asked for, in seconds.
             */
            const askedFor = 600;
            /**
             What the gate answered for a session resumed with the flex pool named.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({
                name: 'copy-route-2',
                provider: NEURALWATT_DEEPSEEK_FLEX.provider,
                model: NEURALWATT_DEEPSEEK_FLEX.model,
                sessionOfRunFolder: 'copy-route-1',
                wallSeconds: askedFor,
                commandLine: [
                  '--name=copy-route-2',
                  `${PROVIDER_OPTION}${NEURALWATT_DEEPSEEK_FLEX.provider}`,
                  `${MODEL_OPTION}${NEURALWATT_DEEPSEEK_FLEX.model}`,
                  '--session-of=copy-route-1',
                  `${WALL_OPTION}${String(askedFor,)}`,
                ],
              },),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
            expect(outcome.lines[0],).toContain(`goes on ${NEURALWATT_DEEPSEEK_FLEX.name}`,);
            /**
             The words the launcher is given.
             */
            const words = (outcome.action === 'start') ? outcome.arguments.words : [];
            expect(words,).toContain(`${PROVIDER_OPTION}${NEURALWATT_DEEPSEEK_FLEX.provider}`,);
            expect(words,).toContain(`${MODEL_OPTION}${NEURALWATT_DEEPSEEK_FLEX.model}`,);
            expect(words,).toContain(`${WALL_OPTION}${String(askedFor,)}`,);
          },
        },),
        it({
          name: 'moves a fresh run that names the flex pool to the first pool with room, because a pool that takes default runs is taken in the owner\'s order whoever names it',
          fn: async () => {
            /**
             The pool the gate moves a run whose own words name a default pool to.
             */
            const pool = theFirstDefaultPool();
            /**
             The wall clock the caller asked for, in seconds.
             */
            const askedFor = 600;
            /**
             What the gate answered for a fresh run that names the flex pool.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({
                name: 'copy-route',
                provider: NEURALWATT_DEEPSEEK_FLEX.provider,
                model: NEURALWATT_DEEPSEEK_FLEX.model,
                wallSeconds: askedFor,
                commandLine: [
                  '--name=copy-route',
                  `${PROVIDER_OPTION}${NEURALWATT_DEEPSEEK_FLEX.provider}`,
                  `${MODEL_OPTION}${NEURALWATT_DEEPSEEK_FLEX.model}`,
                  `${WALL_OPTION}${String(askedFor,)}`,
                ],
              },),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
            expect(outcome.lines[0],).toContain(`goes on ${pool.name}`,);
            /**
             The words the launcher is given.
             */
            const words = (outcome.action === 'start') ? outcome.arguments.words : [];
            expect(words,).toContain(`${PROVIDER_OPTION}${pool.provider}`,);
            expect(words,).toContain(`${MODEL_OPTION}${pool.model}`,);
            expect(words,).toContain(`${WALL_OPTION}${String(askedFor * pool.wallFactor,)}`,);
          },
        },),
        it({
          name: 'tells a run refused room because the pool it names is closed why, in that pool\'s own words',
          fn: async () => {
            /**
             What the gate answered for a run naming the model of the pool the owner closed.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({
                name: 'kimi-review-round-9',
                provider: NEURALWATT_KIMI_FLEX.provider,
                model: NEURALWATT_KIMI_FLEX.model,
              },),
              runs: [],
              tallies: talliesOf({ runs: [], },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('no-room',);
            /**
             The line the gate tells a reader with, on its own standard error.
             */
            const told = (outcome.action === 'no-room') ? outcome.closedPoolLine ?? '' : '';
            expect(told,).toContain(NEURALWATT_KIMI_FLEX.closed?.reason ?? '',);
            expect(told,).toContain(NEURALWATT_KIMI_FLEX.name,);
            expect(told,).toContain(POOL_LIMITS_VARIABLE,);
          },
        },),
        it({
          name: 'says nothing of a closed pool when the run names none of its own and every pool with room is full, since that run named no pool to be closed',
          fn: async () => {
            /**
             Every pool a run that names no model of its own may take, filled.
             */
            const filled = runsFilling(openDefaultPools(),);
            /**
             What the gate answered for a run that names no provider and no model while those pools are full.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptions({ name: 'copy-route', }),
              runs: filled,
              tallies: talliesOf({ runs: filled, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('no-room',);
            expect('closedPoolLine' in outcome,).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
