/**
 One batch list started in a capped systemd user unit of its own, so the batch, its retries and every launcher it starts outlive
 the process that asked for them, and the line a reader gets either way.

 The unit is named `launch-<the list's name>`, never `pi-*`: the slot count reads every `pi-*` unit as a run. Its output goes to a
 file beside the list, and nowhere else: a unit has no terminal.

 @module
 */

import {
  batchOutputFileOf,
  batchUnitCommandOf,
  batchUnitNameOf,
  type BatchUnitRequest,
  startedLineOf,
} from './batch-run-as-a-unit.ts';
import { checkoutFolderOf, } from './checkout-of-a-command.ts';
import { requiredOutputOf, } from './required-output-of-a-command.ts';
import {
  type RunCommand,
  runCommandWithChildProcess,
} from './run-command.ts';
import { siblingCommandEntry, } from './sibling-command-entry.ts';

/**
 What starting one batch's unit needs, beyond the list and the deadline: where its output goes, the checkout its runs work in, and
 the environment the unit is given.
 */
export type BatchUnitStartRequest = {
  /**
   The batch list the unit starts runs from.
   */
  readonly batchFile: string;

  /**
   How long the batch keeps asking for a slot, in seconds.
   */
  readonly deadlineSeconds: number;

  /**
   Where the unit writes both of its streams, which is beside the list when the caller names none.
   */
  readonly outputFile?: string;

  /**
   The checkout the batch starts its runs in.
   */
  readonly checkoutFolder: string;

  /**
   The home folder the unit runs with.
   */
  readonly home: string;

};

/**
 What starting a batch's unit ended with.
 */
export type BatchUnitStart = {
  /**
   The unit's name, without its `.service` suffix.
   */
  readonly unit: string;

  /**
   Where the unit writes both of its streams.
   */
  readonly outputFile: string;

  /**
   The status systemd-run ended with.
   */
  readonly status: number;

  /**
   What systemd-run said when it started nothing, and empty text otherwise.
   */
  readonly problem: string;

  /**
   The line a reader acts on.
   */
  readonly line: string;
};

/**
 Builds the request that starts one batch's unit, asking for the `PATH` a mise environment gives and for the batch command itself.

 @param request - the list, its deadline, its output, the checkout and the home folder

 @param run - how a command is run, which a test replaces

 @param entryOf - how another command's entry is found, which a test replaces

 @returns what starting the unit needs

 @throws CommandAnswerMissingError when the environment's `PATH` could not be read

 @example
 ```ts
 const request = await batchUnitRequestOf({ request: { batchFile: '/runs/b.json', deadlineSeconds: 7_200, checkoutFolder: '/repo', home: '/home/a', }, run: runCommandWithChildProcess });
 ```
 */
export async function batchUnitRequestOf({
  request,
  run = runCommandWithChildProcess,
  entryOf = siblingCommandEntry,
}: {
  readonly request: BatchUnitStartRequest;
  readonly run?: RunCommand;
  readonly entryOf?: (request: {
    readonly folder: string;
    readonly command: string
  }) => Promise<string>;
},): Promise<BatchUnitRequest> {
  return {
    batchFile: request.batchFile,
    deadlineSeconds: request.deadlineSeconds,
    outputFile: request.outputFile ?? batchOutputFileOf({ batchFile: request.batchFile, },),
    pathOfMiseEnvironment: await requiredOutputOf({
      run,
      command: 'mise',
      words: [
        'exec',
        '--',
        'printenv',
        'PATH',
      ],
    },),
    home: request.home,
    node: process.execPath,
    batchEntry: await entryOf({
      folder: import.meta.dirname,
      command: 'start-pi-runs-from-a-batch-list',
    },),
    checkoutFolder: await checkoutFolderOf({ from: request.checkoutFolder, },),
  };
}

/**
 Starts one batch's unit and answers the line a reader acts on.

 @param request - what starting the unit needs

 @param run - how the command is run, which a test replaces

 @returns how systemd-run ended and the line to print

 @example
 ```ts
 const started = await startABatchInItsOwnUnit({ request, run: runCommandWithChildProcess });
 ```
 */
export async function startABatchInItsOwnUnit({
  request,
  run = runCommandWithChildProcess,
}: {
  readonly request: BatchUnitRequest;
  readonly run?: RunCommand;
},): Promise<BatchUnitStart> {
  /**
   The command that starts the batch's unit.
   */
  const command = batchUnitCommandOf(request,);
  /**
   What systemd-run answered.
   */
  const started = await run({
    command: command.command,
    words: command.words,
  },);
  /**
   What systemd-run said when it started nothing, which is empty text when it started the unit.
   */
  const problem = started.problem
    ?? started.stderr
      .trim();
  return {
    unit: batchUnitNameOf({ batchFile: request.batchFile, },),
    outputFile: request.outputFile,
    status: started.status,
    problem,
    line: startedLineOf({
      unit: batchUnitNameOf({ batchFile: request.batchFile, },),
      outputFile: request.outputFile,
      status: started.status,
      problem,
    },),
  };
}

/**
 Starts one batch list in a unit of its own, resolving what the unit needs first.

 @param request - the list, its deadline, its output, the checkout and the home folder

 @param run - how a command is run, which a test replaces

 @param entryOf - how another command's entry is found, which a test replaces

 @returns how systemd-run ended and the line to print

 @throws CommandAnswerMissingError when the environment's `PATH` could not be read

 @example
 ```ts
 const started = await startABatchFile({ request: { batchFile: '/runs/b.json', deadlineSeconds: 7_200, checkoutFolder: '/repo', home: '/home/a', }, run: runCommandWithChildProcess });
 ```
 */
export async function startABatchFile({
  request,
  run = runCommandWithChildProcess,
  entryOf = siblingCommandEntry,
}: {
  readonly request: BatchUnitStartRequest;
  readonly run?: RunCommand;
  readonly entryOf?: (request: {
    readonly folder: string;
    readonly command: string
  }) => Promise<string>;
},): Promise<BatchUnitStart> {
  return await startABatchInItsOwnUnit({
    request: await batchUnitRequestOf({
      request,
      run,
      entryOf,
    },),
    run,
  },);
}
