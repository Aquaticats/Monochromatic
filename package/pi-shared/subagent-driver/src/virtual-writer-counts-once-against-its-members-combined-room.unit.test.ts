/**
 What a run on one of Pi's virtual selections holds, and what the driver answers for it: one run counted once against the combined
 room of the pools that selection's router may take, a start while those members have room together, and a refusal that says how
 full they are together.

 A virtual selection is the plugin's own (`package/pi-plugin/rate-limit-fallback/README.md`): its router starts every request at
 the first member and moves the request on when a member refuses with a 429 before any output. So a run on it may take any member's
 room at its next request, which is why it is counted against their room together, never as a slot in each of them
 (`doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md`, "The virtual writer is not capped at its smallest
 member": the owner, 2026-11-26, "Just because the 1st pool is 5 at a time doesn't mean you gotta limit yourself.").

 Everything here is read from the pools, the counts and the runs a test hands over, so no unit, no process, no systemd and no disk
 of this machine is involved.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  BYTES_IN_A_GIBIBYTE,
  FREE_SPACE_FLOOR_BYTES,
  type FreeSpace,
  type GateOutcome,
  type GoingRun,
  NEURALWATT_DEEPSEEK_FLEX,
  NEURALWATT_STANDARD,
  NO_SLOT_EXIT_STATUS,
  NO_VIRTUAL_POOL,
  OPENCODE_GO,
  PROVIDER_POOLS,
  QWEN_PLAN_DEEPSEEK,
  type PoolGateOptions,
  type PoolMembership,
  type ProviderPool,
  VIRTUAL_POOLS,
  VIRTUAL_REVIEWER,
  VIRTUAL_WRITER,
  type VirtualPool,
  gateOutcomeOf,
  membershipOfSelection,
  poolChoiceFor,
  poolHasRoom,
  poolOfARunNamingNoProvider,
  poolOfExecStart,
  poolsWithTheseLimits,
  roomIn,
  summaryTextOf,
  talliesOf,
  virtualPoolNamed,
  virtualRoomOf,
} from '../dist/final/node/index.mjs';

/**
 The volume every room in this file is on, which is no volume of this machine.
 */
const TEST_VOLUME = '/a-build-volume-of-the-test';

/**
 The provider both virtual selections belong to, which is the plugin's own name for itself.
 */
const VIRTUAL_PROVIDER = 'rate-limit-fallback';

/**
 The folder a gate in this file is asked about, which holds no run.
 */
const A_RUNS_FOLDER = '/runs';

/**
 The names of a selection's member pools, in the selection's own order.

 @param pools - the pools to name

 @returns one name per pool

 @example
 ```ts
 const names = namesOf(VIRTUAL_WRITER.memberPools);
 ```
 */
function namesOf(pools: readonly ProviderPool[],): readonly string[] {
  return pools.map(function nameOf(pool,) {
    return pool.name;
  },);
}

/**
 The largest wall factor of a set of pools, read from those pools rather than written down here.

 @param pools - the pools to read

 @returns the largest factor

 @example
 ```ts
 const factor = largestWallFactorOf(VIRTUAL_WRITER.memberPools);
 ```
 */
function largestWallFactorOf(pools: readonly ProviderPool[],): number {
  return pools.reduce(function largerOf(larger, pool,) {
    return Math.max(
      larger,
      pool.wallFactor,
    );
  }, 1,);
}

/**
 The text a virtual selection is named by in the counts line, read from the selection rather than written down here.

 @param virtual - the selection to name

 @returns its provider and model, as a run names them

 @example
 ```ts
 const named = virtualNameOf(VIRTUAL_WRITER);
 ```
 */
function virtualNameOf(virtual: VirtualPool,): string {
  return `${virtual.provider}/${virtual.model}`;
}

/**
 The room a set of pools has together, which a closed pool adds nothing to.

 @param pools - the pools, as the counted table holds them

 @returns how many runs they take together now

 @example
 ```ts
 const room = combinedRoomOf(VIRTUAL_WRITER.memberPools);
 ```
 */
function combinedRoomOf(pools: readonly ProviderPool[],): number {
  return pools.reduce(function sumOf(sum, pool,) {
    return sum + roomIn(pool,);
  }, 0,);
}

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
 The table's pool of a name another table names, so a test reads a caller's limit rather than the shipped one.

 @param pool - the pool to read

 @param pools - the table to read it from

 @returns that table's pool of the name

 @example
 ```ts
 const counted = countedAs({ pool: QWEN_PLAN_DEEPSEEK, pools });
 ```
 */
function countedAs({
  pool,
  pools,
}: {
  readonly pool: ProviderPool;
  readonly pools: readonly ProviderPool[];
},): ProviderPool {
  /**
   The pool of that name, when the table holds one.
   */
  const found = pools.find(function isTheSamePool(candidate,) {
    return candidate.name === pool.name;
  },);
  if (found === undefined) {
    throw new Error(`the table this test counts against holds no pool named ${pool.name}`,);
  }
  return found;
}

/**
 One run going now, which its own command line places on a virtual selection.

 @param name - the run's name

 @param virtual - the selection its command line names

 @returns the run

 @example
 ```ts
 const run = aRunOnAVirtualSelection('copy-route', VIRTUAL_WRITER);
 ```
 */
function aRunOnAVirtualSelection(name: string, virtual: VirtualPool,): GoingRun {
  return {
    name,
    // What a run naming that selection holds, read the way the driver reads a run's own command line.
    membership: membershipOfSelection({
      provider: virtual.provider,
      model: virtual.model,
    },),
  };
}

/**
 Runs on one virtual selection, named after it so a failure prints which count is wrong.

 @param virtual - the selection they name

 @param count - how many runs to make

 @returns the runs

 @example
 ```ts
 const runs = virtualRunsGoing(VIRTUAL_WRITER, 6);
 ```
 */
function virtualRunsGoing(virtual: VirtualPool, count: number,): readonly GoingRun[] {
  return Array.from(
    { length: count, },
    function named(_unused, index,) {
      return aRunOnAVirtualSelection(
        `${virtual.model}-${String(index,)}`,
        virtual,
      );
    },
  );
}

/**
 One run going now, in the pool a test names.

 @param name - the run's name

 @param pool - the pool its command line names

 @returns the run

 @example
 ```ts
 const run = aRunIn('another', OPENCODE_GO);
 ```
 */
function aRunIn(name: string, pool: ProviderPool,): GoingRun {
  /**
   The membership of a run whose own command line names one pool.
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
 Runs of one pool, named after their pool so a failure prints which count is wrong.

 @param pool - the pool to fill

 @param count - how many runs to make

 @returns the runs

 @example
 ```ts
 const runs = runsIn(OPENCODE_GO, OPENCODE_GO.limit);
 ```
 */
function runsIn(pool: ProviderPool, count: number,): readonly GoingRun[] {
  return Array.from(
    { length: count, },
    function named(_unused, index,) {
      return aRunIn(
        `${pool.name}-${String(index,)}`,
        pool,
      );
    },
  );
}

/**
 Room on the build volume well above the floor, so a gate in a test starts a run.

 @returns the room

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
 The pool gate's options for a run that names the virtual writer.

 @param extra - the words this test adds to the run's own

 @returns the options

 @example
 ```ts
 const options = gateOptionsForTheVirtualWriter({ wallSeconds: 600 });
 ```
 */
function gateOptionsForTheVirtualWriter({
  wallSeconds,
}: {
  readonly wallSeconds: number;
},): PoolGateOptions {
  return {
    runsFolder: A_RUNS_FOLDER,
    countsOnly: false,
    name: 'copy-route',
    provider: VIRTUAL_WRITER.provider,
    model: VIRTUAL_WRITER.model,
    wallSeconds,
    commandLine: [
      '--name=copy-route',
      `--provider=${VIRTUAL_WRITER.provider}`,
      `--model=${VIRTUAL_WRITER.model}`,
      `--wall=${String(wallSeconds,)}`,
    ],
  };
}

/**
 A session file reader the gate of this file never calls, because no run here resumes one.

 @returns the file

 @example
 ```ts
 const outcome = await gateOutcomeOf({ options, runs: [], tallies, sessionFileOf: aSessionFile, freeSpace: roomToSpare() });
 ```
 */
async function aSessionFile(): Promise<string> {
  return '/sessions/a.jsonl';
}

await describe({
  name: '',
  children: [
    describe({
      name: virtualPoolNamed.name,
      children: [
        it({
          name: 'reads the virtual writer and the virtual reviewer, and answers that a pair naming neither is no selection at all',
          fn: async () => {
            expect(virtualPoolNamed({
              provider: VIRTUAL_PROVIDER,
              model: VIRTUAL_WRITER.model,
            },),).toEqual(VIRTUAL_WRITER,);
            expect(virtualPoolNamed({
              provider: VIRTUAL_PROVIDER,
              model: VIRTUAL_REVIEWER.model,
            },),).toEqual(VIRTUAL_REVIEWER,);
            expect(virtualPoolNamed({
              provider: OPENCODE_GO.provider,
              model: OPENCODE_GO.model,
            },),).toBe(NO_VIRTUAL_POOL,);
          },
        },),
      ],
    },),
    describe({
      name: 'the member pools of each virtual selection',
      children: [
        it({
          name: 'names the owner\'s three writer pools in the owner\'s order, and no pool the owner closed and no provider named deepseek',
          fn: async () => {
            // `doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md`, "The writers' order": "Qwen,
            // opencode-go, flex (Recommended)".
            expect(namesOf(VIRTUAL_WRITER.memberPools,),).toEqual([
              QWEN_PLAN_DEEPSEEK.name,
              OPENCODE_GO.name,
              NEURALWATT_DEEPSEEK_FLEX.name,
            ],);
            for (const selection of VIRTUAL_POOLS) {
              for (const pool of selection.memberPools) {
                expect(pool,).not.toEqual(NEURALWATT_STANDARD,);
                expect(pool.provider,).not.toBe('deepseek',);
                expect(PROVIDER_POOLS,).toContain(pool,);
              }
            }
          },
        },),
        it({
          name: 'starts the reviewer at the qwen plan\'s DeepSeek pool as the owner ordered, and lists no pool for Synthetic, which holds no slot here',
          fn: async () => {
            expect(VIRTUAL_REVIEWER.memberPools[0],).toEqual(QWEN_PLAN_DEEPSEEK,);
            // Synthetic is a reviewer member and belongs to no pool of this table, so it holds no slot: the three pools below are
            // the ones a reviewer run is counted against.
            expect(namesOf(VIRTUAL_REVIEWER.memberPools,),).toEqual([
              QWEN_PLAN_DEEPSEEK.name,
              OPENCODE_GO.name,
              NEURALWATT_DEEPSEEK_FLEX.name,
            ],);
            expect(VIRTUAL_POOLS.length,).toBe(2,);
          },
        },),
      ],
    },),
    describe({
      name: membershipOfSelection.name,
      children: [
        it({
          name: 'holds a run that names the virtual writer once against its members together, and a run that names one pool in that pool alone',
          fn: async () => {
            expect(membershipOfSelection({
              provider: VIRTUAL_WRITER.provider,
              model: VIRTUAL_WRITER.model,
            },),).toEqual({
              holds: 'a-virtual-selection',
              virtual: VIRTUAL_WRITER,
              pools: [
                QWEN_PLAN_DEEPSEEK,
                OPENCODE_GO,
                NEURALWATT_DEEPSEEK_FLEX,
              ],
            },);
            expect(membershipOfSelection({
              provider: OPENCODE_GO.provider,
              model: OPENCODE_GO.model,
            },),).toEqual({
              holds: 'a-pool',
              pool: OPENCODE_GO,
            },);
            expect(membershipOfSelection({
              provider: 'synthetic',
              model: 'moonshotai/Kimi-K3.1-Pro',
            },),).toEqual({ holds: 'no-pool', },);
          },
        },),
        it({
          name: 'reads the same selection off a unit that names it, which is how a run is counted once its launcher is gone',
          fn: async () => {
            expect(poolOfExecStart({
              shown: `{ path=node ; argv[]=node drive-pi-subagent.ts --provider=${VIRTUAL_WRITER.provider} --model=${VIRTUAL_WRITER.model} ; }`,
            },),).toEqual(membershipOfSelection({
              provider: VIRTUAL_WRITER.provider,
              model: VIRTUAL_WRITER.model,
            },),);
          },
        },),
      ],
    },),
    describe({
      name: talliesOf.name,
      children: [
        it({
          name: 'counts a run on the virtual writer once, against its members\' combined room, and in no single member\'s own count',
          fn: async () => {
            /**
             One run on the virtual writer, and one run pinned to the first pool of its order.
             */
            const runs = [
              aRunOnAVirtualSelection('copy-route', VIRTUAL_WRITER,),
              aRunIn('another-writer', QWEN_PLAN_DEEPSEEK,),
            ];
            /**
             What those runs hold of each pool, and of the selection's members together.
             */
            const tallies = talliesOf({ runs, },);
            // Only the run pinned to the qwen pool holds a slot in that pool's own room: the virtual run holds their room together.
            expect(tallies.counts.get(QWEN_PLAN_DEEPSEEK.name,),).toBe(1,);
            expect(tallies.counts.get(OPENCODE_GO.name,),).toBe(0,);
            expect(tallies.counts.get(NEURALWATT_DEEPSEEK_FLEX.name,),).toBe(0,);
            // A pool no member shares, and one only a run naming its own model takes, hold nothing.
            expect(tallies.counts.get(NEURALWATT_STANDARD.name,),).toBe(0,);
            expect(tallies.counts.get('qwen-token-plan',),).toBe(0,);
            /**
             What the selection's open members take together.
             */
            const together = combinedRoomOf(VIRTUAL_WRITER.memberPools,);
            // The line a coordinator reads carries the combined count and room, so a virtual run's slot is visible once.
            expect(summaryTextOf(tallies,),).toContain(`${virtualNameOf(VIRTUAL_WRITER,)} 2/${String(together,)} together`,);
            // The reviewer's members are the same three pools, so its room is the writer's and both runs are counted in it.
            expect(summaryTextOf(tallies,),).toContain(`${virtualNameOf(VIRTUAL_REVIEWER,)} 2/${String(together,)} together`,);
          },
        },),
        it({
          name: 'leaves a closed member out of the combined room, and still counts the runs pinned to it there',
          fn: async () => {
            /**
             The shipped table with the flex pool closed by a caller.
             */
            const pools = poolsAt(new Map([[NEURALWATT_DEEPSEEK_FLEX.name, 0],]),);
            /**
             One run on the virtual writer, and one pinned to the closed member.
             */
            const runs = [
              aRunOnAVirtualSelection('copy-route', VIRTUAL_WRITER,),
              aRunIn('another', NEURALWATT_DEEPSEEK_FLEX,),
            ];
            /**
             What those runs hold.
             */
            const tallies = talliesOf({
              runs,
              pools,
            },);
            /**
             The members the caller left open, whose room the selection has together.
             */
            const open = VIRTUAL_WRITER.memberPools
              .map(function asTheCallersTableHoldsIt(pool,) {
                return countedAs({
                  pool,
                  pools,
                },);
              },)
              .filter(function takesRuns(pool,) {
                return roomIn(pool,) > 0;
              },);
            /**
             The member the caller closed.
             */
            const closed = countedAs({
              pool: NEURALWATT_DEEPSEEK_FLEX,
              pools,
            },);
            expect(open.length,).toBe(VIRTUAL_WRITER.memberPools.length - 1,);
            // A closed pool adds no room, and the run already pinned to it is still counted where it is.
            expect(summaryTextOf(tallies,),).toContain(`${virtualNameOf(VIRTUAL_WRITER,)} 2/${String(combinedRoomOf(open,),)} together`,);
            expect(tallies.counts.get(NEURALWATT_DEEPSEEK_FLEX.name,),).toBe(1,);
            expect(roomIn(closed,),).toBe(0,);
          },
        },),
        it({
          name: 'counts a run whose own pool could not be read once in the combined room of every selection it could be on',
          fn: async () => {
            /**
             What a run whose command line could not be read holds: every pool it could be running on.
             */
            const unreadable: PoolMembership = {
              holds: 'some-pools',
              pools: PROVIDER_POOLS,
            };
            /**
             What that run holds of each selection, on a table where nothing else is going.
             */
            const tallies = talliesOf({ runs: [{ name: 'copy-route', membership: unreadable, },], },);
            for (const selection of VIRTUAL_POOLS) {
              const together = combinedRoomOf(selection.memberPools,);
              expect(summaryTextOf(tallies,),).toContain(`${virtualNameOf(selection,)} 1/${String(together,)} together`,);
            }
          },
        },),
      ],
    },),
    describe({
      name: poolChoiceFor.name,
      children: [
        it({
          name: 'takes the virtual selection itself while its members have room together, and gives the run the largest wall factor of those pools',
          fn: async () => {
            expect(poolChoiceFor({
              requested: membershipOfSelection({
                provider: VIRTUAL_WRITER.provider,
                model: VIRTUAL_WRITER.model,
              },),
              tallies: talliesOf({ runs: [], },),
              resumes: false,
            },),).toEqual({
              destination: 'a-virtual-pool',
              pools: [
                QWEN_PLAN_DEEPSEEK,
                OPENCODE_GO,
                NEURALWATT_DEEPSEEK_FLEX,
              ],
              wallFactor: largestWallFactorOf(VIRTUAL_WRITER.memberPools,),
            },);
          },
        },),
        it({
          name: 'takes the virtual writer while the first pool of its order is full, because a run on it takes their room together rather than a slot in each',
          fn: async () => {
            /**
             Every slot of the first member's own room, taken by runs pinned to it.
             */
            const tallies = talliesOf({ runs: runsIn(QWEN_PLAN_DEEPSEEK, QWEN_PLAN_DEEPSEEK.limit,), },);
            expect(poolChoiceFor({
              requested: membershipOfSelection({
                provider: VIRTUAL_WRITER.provider,
                model: VIRTUAL_WRITER.model,
              },),
              tallies,
              resumes: false,
            },),).toEqual({
              destination: 'a-virtual-pool',
              pools: VIRTUAL_WRITER.memberPools,
              wallFactor: largestWallFactorOf(VIRTUAL_WRITER.memberPools,),
            },);
            // The reviewer selection follows the same rule.
            expect(poolChoiceFor({
              requested: membershipOfSelection({
                provider: VIRTUAL_REVIEWER.provider,
                model: VIRTUAL_REVIEWER.model,
              },),
              tallies,
              resumes: false,
            },),).toEqual({
              destination: 'a-virtual-pool',
              pools: VIRTUAL_REVIEWER.memberPools,
              wallFactor: largestWallFactorOf(VIRTUAL_REVIEWER.memberPools,),
            },);
          },
        },),
        it({
          name: 'takes no run on the virtual writer once the runs on its members reach their combined room, although every member has room of its own',
          fn: async () => {
            /**
             Every member's own room given two slots for this command, so the three have six together.
             */
            const pools = poolsAt(new Map([
              [QWEN_PLAN_DEEPSEEK.name, 2,],
              [OPENCODE_GO.name, 2,],
              [NEURALWATT_DEEPSEEK_FLEX.name, 2,],
            ],),);
            /**
             What the three members take together, read from the table rather than written down here.
             */
            const together = combinedRoomOf(VIRTUAL_WRITER.memberPools
              .map(function asTheCallersTableHoldsIt(pool,) {
                return countedAs({
                  pool,
                  pools,
                },);
              },),);
            /**
             A run on the virtual writer, which is the run the selection is asked for.
             */
            const requested = membershipOfSelection({
              provider: VIRTUAL_WRITER.provider,
              model: VIRTUAL_WRITER.model,
              pools,
            },);
            // One fewer run than the members take together leaves the room the selection needs.
            expect(poolChoiceFor({
              requested,
              tallies: talliesOf({
                runs: virtualRunsGoing(VIRTUAL_WRITER, together - 1,),
                pools,
              },),
              resumes: false,
            },),).toMatchObject({ destination: 'a-virtual-pool', },);
            // One run more reaches that room, while every member's own count still reads nothing.
            expect(poolChoiceFor({
              requested,
              tallies: talliesOf({
                runs: virtualRunsGoing(VIRTUAL_WRITER, together,),
                pools,
              },),
              resumes: false,
            },),).toEqual({ destination: 'no-room', },);
          },
        },),
        it({
          name: 'gives a run pinned to one member no room once its members together are full, and gives it its own room back while they have room',
          fn: async () => {
            /**
             Every member of the writer order given a room of its own, of which the last takes two.
             */
            const pools = poolsAt(new Map([
              [QWEN_PLAN_DEEPSEEK.name, 1,],
              [OPENCODE_GO.name, 1,],
              [NEURALWATT_DEEPSEEK_FLEX.name, 2,],
            ],),);
            /**
             What those members take together, read from the table rather than written down here.
             */
            const together = combinedRoomOf(VIRTUAL_WRITER.memberPools
              .map(function asTheCallersTableHoldsIt(pool,) {
                return countedAs({
                  pool,
                  pools,
                },);
              },),);
            /**
             The flex pool, whose own room is what a run pinned to it needs.
             */
            const flex = countedAs({
              pool: NEURALWATT_DEEPSEEK_FLEX,
              pools,
            },);
            /**
             What as many runs on the virtual writer as those members take together hold.
             */
            const allTogether = talliesOf({
              runs: virtualRunsGoing(VIRTUAL_WRITER, together,),
              pools,
            },);
            // The flex pool's own count is empty, because a run on the virtual writer holds no member's own room.
            expect(allTogether.counts.get(flex.name,),).toBe(0,);
            // Its room is free, and the run pinned to it is still refused while the members together are full.
            expect(roomIn(flex,),).toBeGreaterThan(0,);
            expect(poolHasRoom({
              tallies: allTogether,
              pool: flex,
            },),).toBe(false,);
            // One run fewer leaves the members' room together a slot, so the pool's own room decides.
            expect(poolHasRoom({
              tallies: talliesOf({
                runs: virtualRunsGoing(VIRTUAL_WRITER, together - 1,),
                pools,
              },),
              pool: flex,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'counts the members of a virtual selection as a caller\'s limits leave them, so a member a caller closed adds no room and takes no run of it',
          fn: async () => {
            /**
             The shipped table with the last pool of the writer order closed by a caller.
             */
            const pools = poolsAt(new Map([[NEURALWATT_DEEPSEEK_FLEX.name, 0],]),);
            /**
             The selection at those limits.
             */
            const requested = membershipOfSelection({
              provider: VIRTUAL_WRITER.provider,
              model: VIRTUAL_WRITER.model,
              pools,
            },);
            expect(poolChoiceFor({
              requested,
              tallies: talliesOf({
                runs: [],
                pools,
              },),
              resumes: false,
            },),).toMatchObject({ destination: 'a-virtual-pool', },);
            // A run on the selection may start while the members a caller left open have room between them.
            expect(poolChoiceFor({
              requested,
              tallies: talliesOf({
                runs: virtualRunsGoing(VIRTUAL_WRITER, combinedRoomOf(VIRTUAL_WRITER.memberPools
                  .map(function asTheCallersTableHoldsIt(pool,) {
                    return countedAs({
                      pool,
                      pools,
                    },);
                  },)),),
                pools,
              },),
              resumes: false,
            },),).toEqual({ destination: 'no-room', },);
            // Every member closed leaves no room at all.
            const allClosed = poolsAt(new Map([
              [QWEN_PLAN_DEEPSEEK.name, 0,],
              [OPENCODE_GO.name, 0,],
              [NEURALWATT_DEEPSEEK_FLEX.name, 0,],
            ],),);
            expect(poolChoiceFor({
              requested: membershipOfSelection({
                provider: VIRTUAL_WRITER.provider,
                model: VIRTUAL_WRITER.model,
                pools: allClosed,
              },),
              tallies: talliesOf({
                runs: [],
                pools: allClosed,
              },),
              resumes: false,
            },),).toEqual({ destination: 'no-room', },);
          },
        },),
        it({
          name: 'refuses to place a virtual selection as a run that names no provider, because such a run keeps the pair it named',
          fn: async () => {
            expect(function placesAVirtualSelection() {
              poolOfARunNamingNoProvider({
                requested: membershipOfSelection({
                  provider: VIRTUAL_WRITER.provider,
                  model: VIRTUAL_WRITER.model,
                },),
                resumes: false,
                tallies: talliesOf({ runs: [], },),
              },);
            },).toThrow('virtual selection',);
          },
        },),
      ],
    },),
    describe({
      name: virtualRoomOf.name,
      children: [
        it({
          name: 'reads the member pools of a selection, the runs on them and their room together, and refuses to read a selection the tallies never counted',
          fn: async () => {
            /**
             One run on the virtual writer, and one pinned to the second pool of its order.
             */
            const runs = [
              aRunOnAVirtualSelection('copy-route', VIRTUAL_WRITER,),
              aRunIn('another-writer', OPENCODE_GO,),
            ];
            /**
             What the table counted of that selection.
             */
            const room = virtualRoomOf({
              tallies: talliesOf({ runs, },),
              virtual: VIRTUAL_WRITER,
            },);
            expect(room.virtual,).toEqual(VIRTUAL_WRITER,);
            expect(namesOf(room.pools,),).toEqual(namesOf(VIRTUAL_WRITER.memberPools,),);
            // The pinned run is on a member too, so both runs are counted once in the selection's room.
            expect(room.going,).toBe(runs.length,);
            expect(room.room,).toBe(combinedRoomOf(VIRTUAL_WRITER.memberPools,),);
            // A tallies table counted for no selection at all says so rather than inventing a room.
            expect(function countsNoSelection() {
              virtualRoomOf({
                tallies: talliesOf({
                  runs: [],
                  virtuals: [],
                },),
                virtual: VIRTUAL_WRITER,
              },);
            },).toThrow('was not counted',);
          },
        },),
      ],
    },),
    describe({
      name: gateOutcomeOf.name,
      children: [
        it({
          name: 'starts a run on the virtual writer with the pair it named, while the first pool of its order is full, and names the members\' combined count',
          fn: async () => {
            /**
             The wall clock the caller asked for.
             */
            const wallSeconds = 600;
            /**
             Every slot of the first member's own room, taken by runs pinned to it.
             */
            const runs = runsIn(QWEN_PLAN_DEEPSEEK, QWEN_PLAN_DEEPSEEK.limit,);
            /**
             What the gate answered.
             */
            const outcome: GateOutcome = await gateOutcomeOf({
              options: gateOptionsForTheVirtualWriter({ wallSeconds, },),
              runs,
              tallies: talliesOf({ runs, },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('start',);
            // The line names the selection and every pool its router may take, so a reader sees what the run holds.
            expect(outcome.lines[0],).toContain(`${VIRTUAL_WRITER.provider}/${VIRTUAL_WRITER.model}`,);
            for (const pool of VIRTUAL_WRITER.memberPools) {
              expect(outcome.lines[0],).toContain(pool.name,);
            }
            // The count the line carries for the selection is the members' own, so a reader sees how full they are together.
            expect(outcome.lines[0],).toContain(`${virtualNameOf(VIRTUAL_WRITER,)} ${String(QWEN_PLAN_DEEPSEEK.limit,)}/${String(combinedRoomOf(VIRTUAL_WRITER.memberPools,),)} together`,);
            /**
             The words the launcher is given.
             */
            const words = (outcome.action === 'start') ? outcome.arguments.words : [];
            // The pair the run named is kept: rewriting it into one member's pair would pin the run to that member and lose the
            // router's fallback.
            expect(words,).toContain(`--provider=${VIRTUAL_WRITER.provider}`,);
            expect(words,).toContain(`--model=${VIRTUAL_WRITER.model}`,);
            expect(words,).toContain(`--wall=${String(wallSeconds * largestWallFactorOf(VIRTUAL_WRITER.memberPools,),)}`,);
          },
        },),
        it({
          name: 'answers no slot with the counts once the members together are full, and starts nothing',
          fn: async () => {
            /**
             Every member's own room given one slot, so the three have three together.
             */
            const pools = poolsAt(new Map([
              [QWEN_PLAN_DEEPSEEK.name, 1,],
              [OPENCODE_GO.name, 1,],
              [NEURALWATT_DEEPSEEK_FLEX.name, 1,],
            ],),);
            /**
             As many runs on the virtual writer as those members take together.
             */
            const runs = virtualRunsGoing(VIRTUAL_WRITER, combinedRoomOf(VIRTUAL_WRITER.memberPools
              .map(function asTheCallersTableHoldsIt(pool,) {
                return countedAs({
                  pool,
                  pools,
                },);
              },),),);
            /**
             What the gate answered while those members were full together.
             */
            const outcome = await gateOutcomeOf({
              options: gateOptionsForTheVirtualWriter({ wallSeconds: 600, },),
              runs,
              tallies: talliesOf({
                runs,
                pools,
              },),
              sessionFileOf: aSessionFile,
              freeSpace: roomToSpare(),
            },);
            expect(outcome.action,).toBe('no-room',);
            expect((outcome.action === 'no-room') ? outcome.status : 0,).toBe(NO_SLOT_EXIT_STATUS,);
            // The refusal carries the counts, so a reader sees how full the members are together.
            expect(outcome.lines[0],).toContain(`${virtualNameOf(VIRTUAL_WRITER,)} ${String(runs.length,)}/${String(runs.length,)} together`,);
          },
        },),
      ],
    },),
  ],
},);
