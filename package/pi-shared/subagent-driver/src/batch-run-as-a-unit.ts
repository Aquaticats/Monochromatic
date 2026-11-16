/**
 A batch started as its own systemd user unit, so the batch, its retries and every launcher it starts outlive the process that
 asked for them.

 Before 2026-11-11T17:31-05:00 batches ran as background shells of the coordinating process: when that process was restarted, about
 fifty launchers ended with it while every agent kept running in its own `pi-*` unit, and the launchers' after-run steps were
 lost. The unit is named `launch-<the batch's name>`, never `pi-*`: the slot count reads every `pi-*` unit as a run.

 @module
 */

import {
  basename,
  dirname,
  extname,
  join,
} from 'node:path';
import {
  limitsIn,
  POOL_LIMITS_VARIABLE,
} from './pool-limits.ts';

/**
 How long the batch's own unit may last, in seconds. The longest wall a launcher gives a run is 43,500 seconds, a pool that
 doubles the wall clock (`neuralwatt-flex` and `opencode-go`) makes it 87,000, and a wake may resume it again; two days bounds
 that whole tree with room to spare.
 */
export const BATCH_UNIT_SECONDS = 172_800;

/**
 The processor share the batch's unit gets, in percent. The launchers only link dependencies and wait; the agents run in units of
 their own, so the batch needs room to read its list and start the next launcher, not room for the work itself.
 */
export const BATCH_UNIT_CPU_PERCENT = 400;

/**
 The memory the batch's unit may hold. Launchers link dependencies with `pnpm install --offline`, which fits in this several times
 over, and the runs they start are bounded by their own units.
 */
export const BATCH_UNIT_MEMORY = '16G';

/**
 The number of tasks the batch's unit may hold.
 */
export const BATCH_UNIT_TASK_COUNT = 4_096;

/**
 The weight the batch's input and output get against other units', which is what a launcher's own unit carries.
 */
export const BATCH_UNIT_IO_WEIGHT = 10;

/**
 The start of the unit name of a batch. It is not `pi-`, which every slot count reads as a run.
 */
const BATCH_UNIT_START = 'launch-';

/**
 The extension a batch list carries.
 */
const LIST_EXTENSION = '.json';

/**
 What starting one batch's unit needs.
 */
export type BatchUnitRequest = {
  /**
   The batch list the unit starts runs from.
   */
  readonly batchFile: string;

  /**
   How long the batch keeps asking for a slot, in seconds.
   */
  readonly deadlineSeconds: number;

  /**
   Where the unit writes both of its streams.
   */
  readonly outputFile: string;

  /**
   The `PATH` the unit gets, which it does not inherit from the shell that asked for it.
   */
  readonly pathOfMiseEnvironment: string;

  /**
   The home folder the unit runs with.
   */
  readonly home: string;

  /**
   The node that runs the batch command.
   */
  readonly node: string;

  /**
   The batch command's entry.
   */
  readonly batchEntry: string;

  /**
   The checkout the batch starts its runs in, which is the folder its unit works in: a launcher started from a folder outside the
   checkout cannot read which repository a run's worktree belongs to, and the folder a batch list lives in is outside it.
   */
  readonly checkoutFolder: string;

  /**
   The pool limits the caller set, as the value of {@link POOL_LIMITS_VARIABLE}, or nothing when it set none. A unit inherits
   nothing from the shell that asked for it, so these are the only limits the gate inside the batch can read.
   */
  readonly poolLimits?: string;
};

/**
 Reads the limits a batch's unit gets, in a form no part of which can be read as another option or expanded.

 Every pair is read here, before any unit is made, so a setting the gate could not read stops the batch in the command that
 asked for it, naming the variable, rather than inside a unit whose output a reader would have to go and find. What the unit is
 given is those same pairs written again with no space and no other character: a pool name and a whole number are the only
 things a setting holds, so what systemd is handed is letters, digits, dashes, equals signs and the commas between settings.

 @param poolLimits - the value the caller's environment holds, as a caller writes it

 @returns the value the unit gets

 @throws PoolLimitsError when a pair is not `name=<whole number>` or names no pool

 @example
 ```ts
 const value = limitsTheBatchUnitGets('opencode-go=0, neuralwatt-kimi-flex=20');
 ```
 */
export function limitsTheBatchUnitGets(poolLimits: string,): string {
  /**
   The settings, as the gate inside the batch reads them.
   */
  const limits = limitsIn({ listed: poolLimits, },);
  /**
   Each setting written again, in the order the caller wrote them.
   */
  const settings: string[] = [];
  for (const [name, limit,] of limits) {
    settings.push(`${name}=${String(limit,)}`,);
  }
  return settings.join(',',);
}

/**
 Reads the name of one batch's unit.

 @param batchFile - the batch list

 @returns the unit's name, without its `.service` suffix

 @example
 ```ts
 const unit = batchUnitNameOf({ batchFile: '/runs/agents-batch-round-231.json', },);
 ```
 */
export function batchUnitNameOf({ batchFile, }: { readonly batchFile: string },): string {
  /**
   The list's file name without its extension, which is the batch's name.
   */
  const listName = basename(
    batchFile,
    extname(batchFile,),
  );
  return `${BATCH_UNIT_START}${listName}`;
}

/**
 Reads the file a batch's unit writes both of its streams to, which is the list's own name beside it.

 @param batchFile - the batch list

 @returns the output file's path

 @example
 ```ts
 const output = batchOutputFileOf({ batchFile: '/runs/agents-batch-round-231.json', },);
 ```
 */
export function batchOutputFileOf({ batchFile, }: { readonly batchFile: string },): string {
  return join(
    dirname(batchFile,),
    `${basename(
      batchFile,
      LIST_EXTENSION,
    )}.out`,
  );
}

/**
 Builds the command that starts one batch's unit.

 @param request - the batch, its deadline, where its output goes, and the environment its unit gets

 @returns the program and its arguments, each kept whole

 @example
 ```ts
 const command = batchUnitCommandOf({ batchFile: '/runs/b.json', deadlineSeconds: 7_200, outputFile: '/runs/b.out', pathOfMiseEnvironment: '/bin', home: '/home/a', node: '/bin/node', batchEntry: '/pkg/src/start-pi-runs-from-a-batch-list.ts', checkoutFolder: '/repo', },);
 ```
 */
export function batchUnitCommandOf(request: BatchUnitRequest,): {
  readonly command: string;
  readonly words: readonly string[];
} {
  /**
   The limits the unit gets, read and written again here so that no character of them can be read as another option or
   expanded, and empty when the caller set none or set nothing a pool is named in.
   */
  const limitsForTheUnit = (request.poolLimits === undefined) ? '' : limitsTheBatchUnitGets(request.poolLimits,);
  return {
    command: 'systemd-run',
    words: [
      '--user',
      `--unit=${batchUnitNameOf({ batchFile: request.batchFile, },)}`,
      '--collect',
      `--property=RuntimeMaxSec=${String(BATCH_UNIT_SECONDS,)}`,
      `--property=MemoryMax=${BATCH_UNIT_MEMORY}`,
      `--property=CPUQuota=${String(BATCH_UNIT_CPU_PERCENT,)}%`,
      `--property=TasksMax=${String(BATCH_UNIT_TASK_COUNT,)}`,
      `--property=IOWeight=${String(BATCH_UNIT_IO_WEIGHT,)}`,
      `--property=StandardOutput=append:${request.outputFile}`,
      `--property=StandardError=append:${request.outputFile}`,
      `--setenv=PATH=${request.pathOfMiseEnvironment}`,
      `--setenv=HOME=${request.home}`,
      ...((limitsForTheUnit === '')
        ? []
        : [`--setenv=${POOL_LIMITS_VARIABLE}=${limitsForTheUnit}`]),
      `--working-directory=${request.checkoutFolder}`,
      request.node,
      request.batchEntry,
      `--batch-file=${request.batchFile}`,
      `--deadline-seconds=${String(request.deadlineSeconds,)}`,
    ],
  };
}

/**
 Says what starting the batch answered, as the line a reader acts on.

 @param unit - the unit's name, without its `.service` suffix

 @param outputFile - where the unit writes both of its streams

 @param status - the status systemd-run ended with

 @param problem - what systemd-run said when it started nothing

 @returns the line to print

 @example
 ```ts
 const line = startedLineOf({ unit: 'launch-b', outputFile: '/runs/b.out', status: 0, problem: '' },);
 ```
 */
export function startedLineOf({
  unit,
  outputFile,
  status,
  problem,
}: {
  readonly unit: string;
  readonly outputFile: string;
  readonly status: number;
  readonly problem: string;
},): string {
  return (status === 0)
    ? `started ${unit}.service; output in ${outputFile}`
    : `could not start ${unit}: ${problem}`;
}
