/**
 The counts a command reads before it places a run: the pools at the limits its caller set, and the runs going now counted the
 way the pool gate counts them.

 Both are read here rather than in each command, so a launch that places a run on one pool and a launch that places it on a virtual
 selection count the same runs against the same table.

 @module
 */

import {
  type GoingRun,
  LAUNCHER_ENTRY_NAMES,
  REPLACED_LAUNCHER_ENTRY_NAMES,
  goingRunsNow,
} from './going-runs.ts';
import {
  POOL_LIMITS_VARIABLE,
  PoolLimitsError,
  type ProviderPool,
  limitsIn,
  poolsWithTheseLimits,
} from './pool-limits.ts';
import type { RunCommand, } from './run-command.ts';
import { PiUnitsUnreadableError, } from './running-pi-units.ts';

/**
 Reads the pools at the limits the caller set, which say which pools take runs, how many, and why a closed one is closed.

 @param environment - the environment the command runs with, whose `PI_RUNS_POOL_LIMITS` sets the pools' limits

 @returns the pools at those limits

 @throws PoolLimitsError when the setting names no pool of the table or holds anything but a whole number, which the caller
 answers by naming the setting and its fault and starting nothing

 @example
 ```ts
 const pools = poolsWithTheCallersLimits({ environment: process.env, },);
 ```
 */
export function poolsWithTheCallersLimits({
  environment,
}: {
  readonly environment: Readonly<NodeJS.ProcessEnv>;
},): readonly ProviderPool[] {
  return poolsWithTheseLimits({
    limits: limitsIn({ listed: environment[POOL_LIMITS_VARIABLE] ?? '', },),
  },);
}

/**
 Says what a unit whose own command line could not be read is counted as, which is the line the pool gate prints for the same
 problem.

 @param problem - what the manager answered
 */
function reportAUnitWhosePoolWasNotRead(problem: string,): void {
  process.stderr
    .write(`${problem}\n`,);
}

/**
 Reads the runs going now, counted the way the pool gate counts them: the launchers that have not made their unit yet, and the
 units systemd lists, each with the pool its own command line names.

 @param pools - the pools to count against

 @param processFolder - the folder holding one entry per process

 @param run - how a command is run

 @returns one entry per run going now

 @throws PiUnitsUnreadableError when systemd could not list the units, which the caller answers by refusing the start: a run
 started on a guessed count could take a pool that is full

 @example
 ```ts
 const runs = await runsGoingForTheCount({ pools, processFolder: '/proc', run: runCommandWithChildProcess, },);
 ```
 */
export async function runsGoingForTheCount({
  pools,
  processFolder,
  run,
}: {
  readonly pools: readonly ProviderPool[];
  readonly processFolder: string;
  readonly run: RunCommand;
},): Promise<readonly GoingRun[]> {
  return await goingRunsNow({
    run,
    processFolder,
    launcherEntryNames: [
      ...LAUNCHER_ENTRY_NAMES,
      ...REPLACED_LAUNCHER_ENTRY_NAMES,
    ],
    pools,
    onProblem: reportAUnitWhosePoolWasNotRead,
  },);
}
