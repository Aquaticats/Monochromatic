/**
 Reads the command line of the batch: which list, where its runs live, and how long starting one may wait for a slot.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { optionIn, } from './option-in.ts';
import {
  requiredSeconds,
  requiredText,
} from './required-option-values.ts';
import { BATCH_DEFAULT_DEADLINE_SECONDS, } from './batch-list.ts';

/**
 The batch's command line, for the error message.
 */
export const BATCH_USAGE =
  'start-pi-runs-from-a-batch-list --batch-file=<file> [--runs-folder=<folder>] [--deadline-seconds=<n>]';

/**
 The option that names the list.
 */
const BATCH_FILE_OPTION = '--batch-file=';

/**
 The option that names the folder the runs live in.
 */
const RUNS_FOLDER_OPTION = '--runs-folder=';

/**
 The option that names how long one entry's start may wait for a slot.
 */
const DEADLINE_SECONDS_OPTION = '--deadline-seconds=';

/**
 The parts of the coordinator's runs folder below the home directory of whoever runs this.
 */
const DEFAULT_RUNS_FOLDER_IN_HOME: readonly string[] = [
  'temp',
  'agent',
  'pi-runs',
];

/**
 A command line the batch cannot use.
 */
export class BatchOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'BatchOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw batchOptionsErrorOf('--batch-file is required.',);
 ```
 */
function batchOptionsErrorOf(message: string,): Error {
  return new BatchOptionsError(message,);
}

/**
 What the batch was asked for.
 */
export type BatchOptions = {
  /**
   The list of runs to start.
   */
  readonly batchFile: string;

  /**
   Folder the runs live in.
   */
  readonly runsFolder: string;

  /**
   How long one entry's start may wait for a slot.
   */
  readonly deadlineSeconds: number;
};

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @returns what the batch was asked for

 @throws BatchOptionsError when the list is missing, or a limit cannot be read

 @example
 ```ts
 const options = parseBatchOptions(['--batch-file=/runs/batch.json',],);
 ```
 */
export function parseBatchOptions(commandLine: readonly string[],): BatchOptions {
  /**
   The list of runs to start.
   */
  const batchFile = requiredText({
    value: optionIn({
      commandLine,
      option: BATCH_FILE_OPTION,
    },),
    name: 'batch-file',
    usage: BATCH_USAGE,
    errorOf: batchOptionsErrorOf,
  },);
  /**
   Folder the runs live in.
   */
  const runsFolder = optionIn({
    commandLine,
    option: RUNS_FOLDER_OPTION,
  },);
  /**
   How long one entry's start may wait for a slot.
   */
  const deadline = optionIn({
    commandLine,
    option: DEADLINE_SECONDS_OPTION,
  },);
  return {
    batchFile,
    runsFolder: (runsFolder === '')
      ? join(
        homedir(),
        ...DEFAULT_RUNS_FOLDER_IN_HOME,
      )
      : runsFolder,
    deadlineSeconds: (deadline === '')
      ? BATCH_DEFAULT_DEADLINE_SECONDS
      : requiredSeconds({
        value: deadline,
        name: 'deadline-seconds',
        usage: BATCH_USAGE,
        errorOf: batchOptionsErrorOf,
      },),
  };
}
