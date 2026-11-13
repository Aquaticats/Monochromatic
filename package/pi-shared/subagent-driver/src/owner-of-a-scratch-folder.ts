/**
 Proving, at the moment of the deletion, that the run which ended still owns the scratch folder that is about to be emptied.

 A scratch folder is named by the run's name, and a run's name is free as soon as its unit is collected: a new attempt of that name
 starts, reuses the folder, and writes its own work there while the attempt before it is still ending. The readings an end step takes
 on its way (the process table, the unit's state) are old by the time it acts, so the proof is taken where the deletion begins, in
 the command that deletes.

 Three things are asked, and a folder is emptied only when all three say it is the ended run's own: no unit of that name is going, no
 process of the account works in the folder, and no launcher of that name is still running, because a launcher is alive from before it
 makes the folder until after the end step that empties it. A reading that could not be taken is no proof, so the folder stays and the
 line says which reading failed rather than emptying on a guess.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  LAUNCHER_ENTRY_NAMES,
  REPLACED_LAUNCHER_ENTRY_NAMES,
} from './going-runs.ts';
import type { RunCommand, } from './run-command.ts';
import { isDigitsOnly, } from './text-of-an-option.ts';
import {
  type ProcessAtAFolder,
  processesAtTheirFolders,
} from './wake-a-run-whose-work-outlives-its-turn.ts';
import {
  unitStateIsRunning,
  unitStateOf,
} from './wait-for-a-unit-to-end.ts';
import { isInside, } from './finished-worktrees.ts';

/**
 The unit a run of one name is started as, which is the name the manager knows it by.
 */
const UNIT_PREFIX = 'pi-';

/**
 The unit's own suffix.
 */
const UNIT_SUFFIX = '.service';

/**
 The option a launcher names the run it started with.
 */
const NAME_OPTION = '--name=';

/**
 The basenames a launcher of a run is run as: this package's own, and the scratch launcher it replaced, which is still counted so a
 run started before the switch is seen.
 */
const LAUNCHERS: ReadonlySet<string> = new Set<string>([
  ...LAUNCHER_ENTRY_NAMES,
  ...REPLACED_LAUNCHER_ENTRY_NAMES,
]);

/**
 The code the operating system gives a path that is not there, which a process entry that ended between two reads answers.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Reads the entries of one folder, or nothing when the folder is not there.

 @param folder - the folder to read

 @returns the entries, or none

 @example
 ```ts
 const entries = await entriesOrNothingIn({ folder: '/proc', },);
 ```
 */
async function entriesOrNothingIn({ folder, }: { readonly folder: string },): Promise<readonly string[]> {
  try {
    return await readdir(folder,);
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return [];
    }
    throw error;
  }
}

/**
 Reads the words of one process's command line, or nothing when its process ended before it could be read.

 @param file - the process's command line, as the kernel keeps it

 @returns the words, which a process that ended leaves empty

 @throws the error a read gave, other than a path that is not there

 @example
 ```ts
 const words = await wordsOf({ file: '/proc/4242/cmdline', },);
 ```
 */
async function wordsOf({ file, }: { readonly file: string },): Promise<readonly string[]> {
  try {
    return (await readFile(
      file,
      'utf8',
    )).split('\0',)
      .filter(function isAWord(word: string,) {
        return word !== '';
      },);
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return [];
    }
    throw error;
  }
}

/**
 Reads the launchers of one run's name that are still running, which is what says whether a later attempt of that name has started.

 A launcher is alive from before it makes the run's scratch folder until after the end step that empties it, so a launcher of the same
 name alive means the folder has an owner and must not be emptied by an earlier attempt's end step.

 @param processFolder - the folder holding one entry per process, `/proc` here

 @param runName - the run's name

 @param excludingPids - the processes that asked for the emptying and are doing it, which are launchers of that name themselves and
 no later attempt

 @returns the process numbers of the launchers of that name still running, in the order the process folder lists them

 @example
 ```ts
 const launchers = await launchersStillRunningFor({ processFolder: '/proc', runName: 'copy-route', excludingPids: [1], },);
 ```
 */
export async function launchersStillRunningFor({
  processFolder,
  runName,
  excludingPids,
}: {
  readonly processFolder: string;
  readonly runName: string;
  readonly excludingPids: readonly number[];
},): Promise<readonly number[]> {
  /**
   The process entries, of which the ones that are no process number are skipped.
   */
  const entries = await entriesOrNothingIn({ folder: processFolder, },);
  /**
   The launchers found so far.
   */
  const found: number[] = [];
  for (const entry of entries) {
    if ((!isDigitsOnly(entry,)) || (excludingPids.includes(Number(entry,),))) {
      continue;
    }
    /**
     The words of that process's command line.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one process after another, each read once, so a launcher's line is only read while its process lives; see doc/lint/sequential-awaits.md
    const words = await wordsOf({ file: join(
      processFolder,
      entry,
      'cmdline',
    ), },);
    /**
     The entry point the process runs, which Node and a shebang script both leave as the second word.
     */
    const [, entryPoint,] = words;
    if ((entryPoint === undefined) || (!LAUNCHERS.has(basename(entryPoint,),))) {
      continue;
    }
    if (!words.includes(`${NAME_OPTION}${runName}`,)) {
      continue;
    }
    found.push(Number(entry,),);
  }
  return found;
}

/**
 Says why one folder is not provably the ended run's own, from the readings a caller took at the moment of the deletion.

 @param runName - the run that ended

 @param unitState - the state the manager answered for that run's unit, or empty text when it could not be asked

 @param processes - what the process table shows every process working in

 @param launchers - the process numbers of the launchers of that name still running, other than the one asking

 @param folder - the folder about to be emptied

 @param excludingPids - the processes that asked for the emptying and are doing it, which are no later attempt: the end step's own
 launcher is alive while its emptying runs, and so is the emptying's own process

 @returns why the folder is not provably that run's own, or nothing when it is

 @example
 ```ts
 const problem = ownershipProblemOf({ runName: 'copy-route', unitState: 'inactive', processes: [], launchers: [], folder: '/home/a/temp/agent/copy-route-scratch', excludingPids: [1], },);
 ```
 */
export function ownershipProblemOf({
  runName,
  unitState,
  processes,
  launchers,
  folder,
  excludingPids,
}: {
  readonly runName: string;
  readonly unitState: string;
  readonly processes: readonly ProcessAtAFolder[];
  readonly launchers: readonly number[];
  readonly folder: string;
  readonly excludingPids: readonly number[];
},): { readonly problem?: string } {
  /**
   The unit this run is started as, which the manager was asked about.
   */
  const unitName = `${UNIT_PREFIX}${runName}${UNIT_SUFFIX}`;
  // A state nobody could read is not a state that says the unit has gone: the older attempt's launcher may be waiting on a manager
  // that stopped answering while a newer attempt of the same name works in the folder.
  if (unitState === '') {
    return { problem: `${unitName} is in no state anybody could read, so there is no proof that a run of that name has ended and ${folder} may be a later run's`, };
  }
  if (unitStateIsRunning({ state: unitState, },)) {
    return { problem: `a run of that name is going now (${unitName} is ${unitState}), so ${folder} may be a later run's and was left as it is`, };
  }
  /**
   The process working in the folder, if one does.
   */
  const working = processes.find(function worksThere(process,) {
    return (!excludingPids.includes(process.pid,)) && isInside({
      entry: process.folder,
      folder,
    },);
  },);
  if (working !== undefined) {
    return { problem: `a process still works in ${working.folder} (${String(working.pid,)} ${working.command}), so ${folder} may be its own and was left as it is`, };
  }
  // A launcher of that name alive is a later attempt of the same name already started: its folder is the one this would empty.
  /**
   The first launcher of that name found still running, which is none when no later attempt has started.
   */
  const [later,] = launchers;
  if (later !== undefined) {
    return { problem: `a launcher of a run named ${runName} is still running (${String(later,)}), so a later attempt of that name may own ${folder} and it was left as it is`, };
  }
  return {};
}

/**
 Reads, now, whether the run that ended still owns its scratch folder.

 @param run - how a command is run, which a test replaces

 @param runName - the run that ended

 @param folder - the folder about to be emptied

 @param processFolder - the folder holding one entry per process, `/proc` here

 @param excludingPids - the processes that asked for the emptying and are doing it

 @returns why the folder is not provably that run's own, or nothing when it is

 @example
 ```ts
 const problem = await ownershipProblemNow({ run: runCommandWithChildProcess, runName: 'copy-route', folder, processFolder: '/proc', excludingPids: [process.pid], },);
 ```
 */
export async function ownershipProblemNow({
  run,
  runName,
  folder,
  processFolder,
  excludingPids,
}: {
  readonly run: RunCommand;
  readonly runName: string;
  readonly folder: string;
  readonly processFolder: string;
  readonly excludingPids: readonly number[];
},): Promise<{ readonly problem?: string }> {
  /**
   The state the manager answers for the run's unit, which is asked first: a run that is going is the whole reason this proof exists.
   */
  const unitState = await unitStateOf({
    run,
    unitName: `${UNIT_PREFIX}${runName}${UNIT_SUFFIX}`,
  },);
  return ownershipProblemOf({
    runName,
    unitState,
    processes: await processesAtTheirFolders({ processFolder, },),
    launchers: await launchersStillRunningFor({
      processFolder,
      runName,
      excludingPids,
    },),
    folder,
    excludingPids,
  },);
}
