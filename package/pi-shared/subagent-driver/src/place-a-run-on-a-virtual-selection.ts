/**
 What a launch on Pi's virtual writer takes, and what it is refused with: the pair it named, and the room its router's members have
 together.

 A virtual selection has no single pool to place a run on. Its router starts every request at the first member and moves the request
 on when a member refuses with a `429` before any output, so the run takes whatever room those members have between them. The run
 therefore keeps the provider and model it named, which is what lets the router move the request on, and it is counted once against
 that room rather than as a slot in each member, which is what capped these runs at the smallest member's room
 (`doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md`, "The virtual writer is not capped at its smallest
 member").

 @module
 */

import { wallSecondsOn, } from './launcher-arguments.ts';
import {
  POOL_LIMITS_VARIABLE,
  type PoolMembership,
  poolTakesNoNewRun,
  type VirtualPool,
  membershipOfAVirtualSelection,
  virtualTextOf,
} from './pool-limits.ts';
import {
  poolsWithTheCallersLimits,
  runsGoingForTheCount,
} from './pool-counts-before-a-launch.ts';
import {
  poolChoiceFor,
  NO_SLOT_EXIT_STATUS,
} from './pool-gate.ts';
import { whyItTakesNoRun, } from './pool-of-a-run-naming-no-provider.ts';
import {
  summaryTextOf,
  talliesOf,
  virtualRoomOf,
} from './pool-tallies.ts';
import type { RunCommand, } from './run-command.ts';
import type { WorktreeLaunchPlacement, } from './worktree-launch-placement.ts';

/**
 Says what a run on a virtual selection is refused with: the pools its router may take, how full they are together, and the
 setting that gives a member room again.

 @param virtual - the selection the run named

 @param tallies - how many runs each pool holds, and how full the selections are together, with a caller's limits

 @returns the refusal, one line per thing a reader needs

 @example
 ```ts
 const text = virtualRunRefusalTextOf({ virtual: VIRTUAL_WRITER, tallies });
 ```
 */
function virtualRunRefusalTextOf({
  virtual,
  tallies,
}: {
  readonly virtual: VirtualPool;
  readonly tallies: ReturnType<typeof talliesOf>;
},): string {
  /**
   The names of the pools that selection's router may take, in its own order.
   */
  const members = virtual.memberPools
    .map(function nameOf(pool,) {
      return pool.name;
    },)
    .join(', ',);
  /**
   How full those members are together, which is the count that refuses this run.
   */
  const room = virtualRoomOf({
    tallies,
    virtual,
  },);
  /**
   The lines the refusal is made of, which start with what this command could not find.
   */
  const lines: string[] = [
    `--provider=${virtual.provider} --model=${virtual.model} is a virtual selection whose router may take ${members}, and a run on it is counted once against the room those members have together.`,
    'No run on it starts while the runs on those members fill that room:',
  ];
  for (const pool of room.pools) {
    lines.push(`  ${pool.name}: ${whyItTakesNoRun({
      tallies,
      pool,
    },)}`,);
  }
  lines.push(`  together: ${String(room.going,)} going of the ${String(room.room,)} those open members take`,);
  /**
   The members a caller closed, which add no room to that total.
   */
  const closed = room.pools
    .filter(function takesNoNewRun(pool,) {
      return poolTakesNoNewRun({ pool, },);
    },);
  if (closed.length > 0) {
    /**
     Their names, so a reader knows which members the total leaves out.
     */
    const named = closed.map(function nameOf(pool,) {
      return pool.name;
    },)
      .join(', ',);
    lines.push(`  ${named} ${(closed.length === 1) ? 'adds' : 'add'} no room while closed, and the runs already there are still counted in the total.`,);
  }
  /**
   The pool the refusal names in its closing line, which is the first open member a reader could reopen.
   */
  const named = room.pools
    .find(function isOpen(pool,) {
      return !poolTakesNoNewRun({ pool, },);
    },)
    ?.name
    ?? room.pools[0]
      ?.name
    ?? '';
  lines.push(
    `A pool takes new runs again when ${POOL_LIMITS_VARIABLE} gives it a limit above zero, as ${POOL_LIMITS_VARIABLE}=${named}=<n> does for one command.`,
  );
  return lines.join('\n',);
}

/**
 Places a run on a virtual selection: it keeps the provider and model it named, so its router can still move a request on, and it
 starts while the runs on those members are fewer than the room they have together.

 This run's own name is left out of the count, as it is for a run placed on one pool: the launcher is already a launcher of that
 name, and the slot it is choosing is the one it would hold.

 @param virtual - the selection the run named

 @param resumes - whether the run resumes a session

 @param runName - the run's name, which is left out of the count

 @param wallSeconds - how long the caller asked the run to last

 @param provider - the provider the command line named

 @param model - the model the command line named

 @param environment - the environment the command runs with, whose limits decide which pools have room

 @param processFolder - the folder holding one entry per process

 @param run - how a command is run

 @returns the run's placement, which is a refusal naming how full the members are together

 @throws PoolLimitsError when the limits setting cannot be read

 @throws PiUnitsUnreadableError when the runs going now cannot be read

 @example
 ```ts
 const placed = await placeARunOnAVirtualSelection({ virtual: VIRTUAL_WRITER, resumes: false, runName: 'copy-route', wallSeconds: 600, provider: 'rate-limit-fallback', model: 'writer', environment: process.env, processFolder: '/proc', run: runCommandWithChildProcess });
 ```
 */
export async function placeARunOnAVirtualSelection({
  virtual,
  resumes,
  runName,
  wallSeconds,
  provider,
  model,
  environment,
  processFolder,
  run,
}: {
  readonly virtual: VirtualPool;
  readonly resumes: boolean;
  readonly runName: string;
  readonly wallSeconds: number;
  readonly provider: string;
  readonly model: string;
  readonly environment: Readonly<NodeJS.ProcessEnv>;
  readonly processFolder: string;
  readonly run: RunCommand;
},): Promise<WorktreeLaunchPlacement> {
  /**
   The pools at the limits the caller set.
   */
  const pools = poolsWithTheCallersLimits({ environment, },);
  /**
   How many runs each pool holds now, without this run's own name.
   */
  const tallies = talliesOf({
    runs: (await runsGoingForTheCount({
      pools,
      processFolder,
      run,
    },)).filter(function isAnotherRun(going,) {
      return going.name !== runName;
    },),
    pools,
  },);
  /**
   What the run holds: one slot against the room the selection's members have together, as the caller's table holds them.
   */
  const requested: PoolMembership = membershipOfAVirtualSelection({
    virtual,
    pools,
  },);
  /**
   Which of those members the run may take now, which is the pool gate's own reading of the owner's order.
   */
  const choice = poolChoiceFor({
    requested,
    tallies,
    resumes,
  },);
  if (choice.destination === 'no-room') {
    return {
      took: 'no-run',
      status: NO_SLOT_EXIT_STATUS,
      message: virtualRunRefusalTextOf({
        virtual,
        tallies,
      },),
    };
  }
  if (choice.destination !== 'a-virtual-pool') {
    throw new Error('a run on a virtual selection was placed on something else than that selection',);
  }
  /**
   The names of the pools that selection's router may take, in its own order.
   */
  const members = choice.pools
    .map(function nameOf(pool,) {
      return pool.name;
    },)
    .join(', ',);
  return {
    took: 'a-run',
    provider,
    model,
    wallSeconds: wallSecondsOn({
      wallSeconds,
      wallFactor: choice.wallFactor,
    },),
    lines: [`pool: ${virtualTextOf({ virtual, },)}, its members have room together (${members}): ${summaryTextOf(tallies,)}`,],
  };
}
