/**
 Which pool a run whose command line names no provider takes, and what it is refused with: the owner's order, the pools that take
 only their own model, a resumed run's own pool, and the refusal that names each pool and why.

 Everything here is read from the pools and the counts a test hands over, so no unit, no process, no systemd and no disk of this
 machine is involved.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type GoingRun,
  NEURALWATT_DEEPSEEK_FLEX,
  NEURALWATT_STANDARD,
  OPENCODE_GO,
  POOL_LIMITS_VARIABLE,
  PROVIDER_POOLS,
  type PoolMembership,
  type ProviderPool,
  QWEN_PLAN_DEEPSEEK,
  QWEN_PLAN_QWEN4_MAX_PREVIEW,
  poolNamed,
  poolOfARunNamingNoProvider,
  poolsWithTheseLimits,
  talliesOf,
} from '../dist/final/node/index.mjs';

/**
 The pools a test counts against, at the limits it set.

 @param limits - the limit each named pool is given, by the pool's name

 @returns the pools, in the owner's order

 @example
 ```ts
 const pools = poolsAt(new Map([[OPENCODE_GO.name, 0]]));
 ```
 */
function poolsAt(limits: ReadonlyMap<string, number>,): readonly ProviderPool[] {
  return poolsWithTheseLimits({ limits, },);
}

/**
 One run going now, in the pool it names.

 @param name - the run's name

 @param pool - the pool its own command line names

 @returns the run

 @example
 ```ts
 const run = aRun('another', OPENCODE_GO);
 ```
 */
function aRun(name: string, pool: ProviderPool,): GoingRun {
  /**
   The membership of a run whose own command line names a pool.
   */
  const membership: PoolMembership = {
    holds: 'a-pool',
    pool,
  };
  return {
    name,
    membership,
  };
}

/**
 The pool a run that names no provider and no model is read as naming, which is Neuralwatt's standard pool.

 @returns the membership

 @example
 ```ts
 const requested = aRunNamingNone();
 ```
 */
function aRunNamingNone(): PoolMembership {
  return poolNamed({
    provider: NEURALWATT_STANDARD.provider,
    model: NEURALWATT_STANDARD.model,
  },);
}

await describe({
  name: 'the pool a run that names no provider takes',
  children: [
    it({
      name: 'takes the first pool of the owner\'s order with room, skipping the pools that take only a run naming their own model',
      fn: async () => {
        /**
         The table at its own limits, where the qwen plan's DeepSeek pool is open and stands before every other pool.
         */
        const pools = PROVIDER_POOLS;
        /**
         What the run takes.
         */
        const taken = poolOfARunNamingNoProvider({
          requested: aRunNamingNone(),
          resumes: false,
          tallies: talliesOf({
            runs: [],
            pools,
          },),
        },);
        // The qwen plan's DeepSeek pool stands first in the table and is open, and this run may not take it: only a run naming that
        // model does.
        expect(taken.took,).toBe('a-pool',);
        expect(taken.took === 'a-pool' ? taken.pool.name : '',).toBe(OPENCODE_GO.name,);
        expect(taken.took === 'a-pool' ? taken.movesTheRun : false,).toBe(true,);
        expect(taken.took === 'a-pool' ? taken.pool.name : '',).not.toBe(QWEN_PLAN_DEEPSEEK.name,);
        expect(taken.took === 'a-pool' ? taken.pool.name : '',).not.toBe(QWEN_PLAN_QWEN4_MAX_PREVIEW.name,);
        expect(QWEN_PLAN_DEEPSEEK.limit,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'takes the next pool of the order when the first is full, and the pool after a closed one',
      fn: async () => {
        /**
         The table with `opencode-go` taking one run, which one run already holds.
         */
        const fullFirst = poolsAt(new Map([[OPENCODE_GO.name, 1],]),);
        /**
         What the run takes when the first pool is full.
         */
        const whenFull = poolOfARunNamingNoProvider({
          requested: aRunNamingNone(),
          resumes: false,
          tallies: talliesOf({
            runs: [aRun('another', OPENCODE_GO,),],
            pools: fullFirst,
          },),
        },);
        expect(whenFull.took === 'a-pool' ? whenFull.pool.name : '',).toBe(NEURALWATT_DEEPSEEK_FLEX.name,);
        /**
         The table with `opencode-go` closed for one command.
         */
        const closedFirst = poolsAt(new Map([[OPENCODE_GO.name, 0],]),);
        /**
         What the run takes when the first pool is closed.
         */
        const whenClosed = poolOfARunNamingNoProvider({
          requested: aRunNamingNone(),
          resumes: false,
          tallies: talliesOf({
            runs: [],
            pools: closedFirst,
          },),
        },);
        expect(whenClosed.took === 'a-pool' ? whenClosed.pool.name : '',).toBe(NEURALWATT_DEEPSEEK_FLEX.name,);
      },
    },),
    it({
      name: 'keeps the pool a resumed run names while that pool takes new runs, and applies no wall factor to it',
      fn: async () => {
        /**
         What the resumed run takes.
         */
        const taken = poolOfARunNamingNoProvider({
          requested: poolNamed({
            provider: NEURALWATT_DEEPSEEK_FLEX.provider,
            model: NEURALWATT_DEEPSEEK_FLEX.model,
          },),
          resumes: true,
          tallies: talliesOf({ runs: [], },),
        },);
        expect(taken.took === 'a-pool' ? taken.pool.name : '',).toBe(NEURALWATT_DEEPSEEK_FLEX.name,);
        // A resume that kept a pool the gate would have moved it off would stop being resumable for ever, so the pool it names is
        // the pool it keeps, and the caller's own wall clock stays as it wrote it.
        expect(taken.took === 'a-pool' ? taken.movesTheRun : true,).toBe(false,);
      },
    },),
    it({
      name: 'refuses a run no pool has room for, naming every pool it may take with why that pool takes no run now',
      fn: async () => {
        /**
         The table with both open pools a run naming no provider may take closed by the caller.
         */
        const pools = poolsAt(new Map([
          [OPENCODE_GO.name, 0,],
          [NEURALWATT_DEEPSEEK_FLEX.name, 0,],
        ],),);
        /**
         What the run is refused with.
         */
        const taken = poolOfARunNamingNoProvider({
          requested: aRunNamingNone(),
          resumes: false,
          tallies: talliesOf({
            runs: [],
            pools,
          },),
        },);
        expect(taken.took,).toBe('no-pool',);
        if (taken.took !== 'no-pool') {
          return;
        }
        /**
         The owner's reason for the pool he replaced, which the refusal carries beside the pool's name.
         */
        const replaced = NEURALWATT_STANDARD.closed?.reason ?? '';
        expect(replaced.length,).toBeGreaterThan(0,);
        // The owner's reason reaches the reader, and so does the caller's own zero for the two pools it closed.
        expect(taken.refusal,).toContain(`  ${NEURALWATT_STANDARD.name}: takes no new run: ${replaced}`,);
        expect(taken.refusal,).toContain(`  ${OPENCODE_GO.name}: takes no new run: ${POOL_LIMITS_VARIABLE} set its limit to zero`,);
        expect(taken.refusal,).toContain(`  ${NEURALWATT_DEEPSEEK_FLEX.name}: takes no new run: ${POOL_LIMITS_VARIABLE} set its limit to zero`,);
        // No pool a run naming no provider may not take is named: the qwen plan's two pools are open here and are not the run's.
        expect(taken.refusal,).not.toContain(QWEN_PLAN_DEEPSEEK.name,);
        expect(taken.refusal,).not.toContain(QWEN_PLAN_QWEN4_MAX_PREVIEW.name,);
        expect(taken.refusal,).toContain(POOL_LIMITS_VARIABLE,);
      },
    },),
    it({
      name: 'refuses a run every pool of the order is full for, naming each pool with the count it was read at',
      fn: async () => {
        /**
         The table with both open pools a run naming no provider may take taking one run each.
         */
        const pools = poolsAt(new Map([
          [OPENCODE_GO.name, 1,],
          [NEURALWATT_DEEPSEEK_FLEX.name, 1,],
        ],),);
        /**
         What the run is refused with.
         */
        const taken = poolOfARunNamingNoProvider({
          requested: aRunNamingNone(),
          resumes: false,
          tallies: talliesOf({
            runs: [
              aRun('another-song', OPENCODE_GO,),
              aRun('another-review', NEURALWATT_DEEPSEEK_FLEX,),
            ],
            pools,
          },),
        },);
        if (taken.took !== 'no-pool') {
          throw new Error(`expected no pool to take the run, and ${taken.pool.name} did`,);
        }
        expect(taken.refusal,).toContain(`  ${OPENCODE_GO.name}: full: 1 going of the 1 it takes`,);
        expect(taken.refusal,).toContain(`  ${NEURALWATT_DEEPSEEK_FLEX.name}: full: 1 going of the 1 it takes`,);
      },
    },),
    it({
      name: 'throws when its caller hands it a pair no pool holds, which is the caller\'s own check to make',
      fn: async () => {
        expect(function placesAPairNoPoolHolds() {
          poolOfARunNamingNoProvider({
            requested: { holds: 'no-pool', },
            resumes: false,
            tallies: talliesOf({ runs: [], },),
          },);
        },).toThrow('no pool holds',);
      },
    },),
  ],
},);
