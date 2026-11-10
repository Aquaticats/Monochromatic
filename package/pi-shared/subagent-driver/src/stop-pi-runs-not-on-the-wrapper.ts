#!/usr/bin/env node
/**
 `stop-pi-runs-not-on-the-wrapper`: sorts every Pi run going now into the Git wrapper's work and other work by the branch its
 worktree has checked out, and, with `--apply`, asks each other-work run to stop by writing its run folder's stop file.

 It writes nothing without `--apply`, so a coordinator can read which runs it would end first.

 Usage: `stop-pi-runs-not-on-the-wrapper [--apply] [--runs-folder=<folder>] [--repository=<folder>]`

 @module
 */

import { basename, } from 'node:path';
import {
  argv,
  stdout,
} from 'node:process';
import {
  defaultRepository,
  defaultRunsFolder,
  worktreeOfARun,
} from './pi-runs-folders.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import { latestRunFolderOf, } from './run-folder-of-a-unit.ts';
import { startRunningPiUnits, } from './running-pi-units.ts';
import {
  type RunToSort,
  stopRunsNotOnTheWrapper,
} from './wrapper-runs.ts';

/**
 The command's command line, for the error message.
 */
const USAGE = 'stop-pi-runs-not-on-the-wrapper [--apply] [--runs-folder=<folder>] [--repository=<folder>]';

/**
 How many characters one option's prefix takes, which is the dashes, the name and the equals sign.
 */
const OPTION_PREFIX_CHARACTERS = 3;

/**
 An option that was not given, as a symbol so no value can be mistaken for it.
 */
const NOT_GIVEN: unique symbol = Symbol('the option was not given',);

/**
 Reads the value of one option of this command.

 @param name - the option's name, without the leading dashes

 @returns the value, or {@link NOT_GIVEN}

 @example
 ```ts
 const folder = optionOf('runs-folder',);
 ```
 */
function optionOf(name: string,): string | typeof NOT_GIVEN {
  /**
   The option as it was written.
   */
  const written = argv.find(function startsWithTheOption(argument,) {
    return argument.startsWith(`--${name}=`,);
  },);
  return (written === undefined) ? NOT_GIVEN : written.slice(name.length + OPTION_PREFIX_CHARACTERS,);
}

/**
 Reads the branch a run's worktree has checked out.

 @param worktree - where the run's branch would be

 @returns the branch, or empty text when the run has no worktree

 @example
 ```ts
 const branch = await branchOf({ worktree: '/repo/.claude/worktrees/pi-a', });
 ```
 */
async function branchOf({
  worktree,
}: {
  readonly worktree: string;
},): Promise<string> {
  /**
   What git said of the branch, or why it said nothing.
   */
  const shown = await runCommandWithChildProcess({
    command: 'git',
    words: [
      '-C',
      worktree,
      'branch',
      '--show-current',
    ],
  },);
  return (shown.status === 0) ? shown.stdout
    .trim() : '';
}

/**
 Reads one option of this command that must be given.

 @param value - the option's value as this command read it

 @param name - the option's name, without the leading dashes

 @param fallback - what to use when the option was not given

 @returns the value

 @example
 ```ts
 const folder = givenOptionOf({ value: NOT_GIVEN, name: 'runs-folder', fallback: '/runs', });
 ```
 */
function givenOptionOf({
  value,
  name,
  fallback,
}: {
  readonly value: string | typeof NOT_GIVEN;
  readonly name: string;
  readonly fallback: string;
},): string {
  if ((typeof value) === 'symbol') {
    return fallback;
  }
  if (value === '') {
    throw new Error(`--${name} needs a folder\nUsage: ${USAGE}`,);
  }
  return value;
}

/**
 Folder the driver writes one run folder per run into.
 */
const runsFolder = givenOptionOf({
  value: optionOf('runs-folder',),
  name: 'runs-folder',
  fallback: defaultRunsFolder({},),
},);

/**
 The repository whose worktrees are read.
 */
const repository = givenOptionOf({
  value: optionOf('repository',),
  name: 'repository',
  fallback: defaultRepository({},),
},);

/**
 Whether every other-work run is asked to stop.
 */
const apply = argv.includes('--apply',);

/**
 Every Pi run going now.
 */
const units = await startRunningPiUnits({ run: runCommandWithChildProcess, },)();

/**
 Every run going now, with the branch its worktree has checked out and the folder it writes into.
 */
const runs: readonly RunToSort[] = await Promise.all(units.map(async function describe(unit,) {
  /**
   Where this run's branch is checked out.
   */
  const worktree = worktreeOfARun({
    repository,
    name: unit.name,
  },);
  /**
   The branch that worktree has checked out, or empty text when there is no worktree.
   */
  const branch = await branchOf({ worktree, },);
  /**
   The run folder of the attempt going now.
   */
  const folder = await latestRunFolderOf({
    runsFolder,
    runName: unit.name,
  },);
  return {
    name: unit.name,
    branch,
    runFolder: ((typeof folder) === 'symbol') ? folder : basename(folder,),
  };
},),);

/**
 The lines to print.
 */
const lines = await stopRunsNotOnTheWrapper({
  runs,
  runsFolder,
  apply,
},);

stdout.write(`${lines.join('\n',)}\n`,);
