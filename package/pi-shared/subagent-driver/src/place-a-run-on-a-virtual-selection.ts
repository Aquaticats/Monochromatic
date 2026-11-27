/**
 What a launch on Pi's virtual writer takes: the pair it named, the members its router may take, and no slot in any pool.

 The owner, 2026-11-26, asked how the driver's pool gate counts a run on the virtual writer, answered "No count (Recommended)":
 the gate counts only the runs pinned to one pool, and a run on `--provider=rate-limit-fallback --model=writer`, or on the
 reviewer selection the same way, holds no slot in any member pool, because the router moves a refused request on by itself. So no
 member's room decides whether this run may start. It keeps the provider and model it named, which is what lets the router move the
 request on, gets the largest wall factor of those members, and its line names them.

 @module
 */

import { wallSecondsOn, } from './launcher-arguments.ts';
import {
  type PoolMembership,
  type VirtualPool,
  membershipOfAVirtualSelection,
  virtualTextOf,
} from './pool-limits.ts';
import {
  poolsWithTheCallersLimits,
  runsGoingForTheCount,
} from './pool-counts-before-a-launch.ts';
import { poolChoiceFor, } from './pool-gate.ts';
import {
  summaryTextOf,
  talliesOf,
} from './pool-tallies.ts';
import type { RunCommand, } from './run-command.ts';
import type { WorktreeLaunchPlacement, } from './worktree-launch-placement.ts';

/**
 Places a run on a virtual selection: it keeps the provider and model it named, so its router can still move a request on, and it
 holds no slot in any pool.

 This run's own name is left out of the count the line prints, as it is for a run placed on one pool: the launcher is already a
 launcher of that name.

 @param virtual - the selection the run named

 @param resumes - whether the run resumes a session

 @param runName - the run's name, which is left out of the count

 @param wallSeconds - how long the caller asked the run to last

 @param provider - the provider the command line named

 @param model - the model the command line named

 @param environment - the environment the command runs with, whose limits decide which pools are open

 @param processFolder - the folder holding one entry per process

 @param run - how a command is run

 @returns the run's placement, which names the members its router may take and the counts of the runs pinned to each pool

 @throws PoolLimitsError when the limits setting cannot be read

 @throws PiUnitsUnreadableError when the runs going now cannot be read

 @throws Error when the run is placed on something other than the selection it named

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
   How many runs each pool holds now, without this run's own name, which is the count the line prints.
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
   What the run holds: the selection it named and the members its router may take, with no slot in any of them.
   */
  const requested: PoolMembership = membershipOfAVirtualSelection({
    virtual,
    pools,
  },);
  /**
   What the run takes, which is the selection itself: no pool's room decides it.
   */
  const choice = poolChoiceFor({
    requested,
    tallies,
    resumes,
  },);
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
    lines: [`pool: ${virtualTextOf({ virtual, },)}, its router may take ${members}, and the run holds no slot in any pool: ${summaryTextOf(tallies,)}`,],
  };
}
