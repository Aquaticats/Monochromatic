/**
 What the machine says about the worktrees of a repository: which units are running, where every process works, and the time the
 judgement is made at. Nothing is removed on an answer this file could not get.

 @module
 */

import {
  readdir,
  readlink,
} from 'node:fs/promises';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { SYSTEMCTL_PROGRAM, } from './constants.ts';
import type { Repository, } from './git-commands.ts';
import {
  found,
  type Looked,
  missing,
} from './lookups.ts';

/**
 Tells whether a text is a whole number, so a process table's numbered entries are told from the rest.

 @param text - one entry of the process table

 @returns true when every character is a digit and there is at least one

 @example
 ```ts
 const numeric = isWholeNumber('2735425',);
 ```
 */
function isWholeNumber(text: string,): boolean {
  return (text !== '')
    && text.split('',)
      .every(function isDigit(character: string,) {
        return (character >= '0') && (character <= '9');
      },);
}

/**
 Longest text of a reason kept on a report line, so one long failure cannot fill a log.
 */
const REASON_CHARACTERS = 80;

/**
 Reads the names of the `pi-` units running now, so a worktree whose agent still works is never removed.

 @param repository - the checkout whose units are asked about

 @returns the unit names, or why the machine could not say

 @example
 ```ts
 const units = await activeAgentUnits({ repository, },);
 ```
 */
export async function activeAgentUnits({ repository, }: {
  readonly repository: Repository;
},): Promise<Looked<readonly string[]>> {
  /**
   What `systemctl` listed.
   */
  const asked = await repository.run({
    program: SYSTEMCTL_PROGRAM,
    words: [
      '--user',
      'list-units',
      '--state=active',
      '--no-legend',
      '--plain',
      'pi-*.service',
    ],
    folder: repository.folder,
    timeoutMilliseconds: repository.timeoutMilliseconds,
    outputBytes: repository.outputBytes,
  },);
  return asked.status === 0
    ? found(asked.stdout
      .split('\n',)
      .flatMap(function unitName(line: string,) {
        /**
         The first field of the listing, which is the unit's name.
         */
        const name = line.trim()
          .split(' ',)[0]
          ?? '';
        return name === '' ? [] : [name,];
      },),)
    : missing(`systemctl ended with status ${String(asked.status,)}: ${asked.stderr
      .trim()
      .slice(
        0,
        REASON_CHARACTERS,
      )}`,);
}

/**
 Reads one process's working folder.

 @param pid - the process's number

 @returns its folder, or why it has none this user may read

 @example
 ```ts
 const folder = await workingFolderOf('2735425',);
 ```
 */
async function workingFolderOf(pid: string,): Promise<Looked<string>> {
  try {
    return found(await readlink(`/proc/${pid}/cwd`,),);
  }
  catch (caught) {
    // A process that ended, or one this user may not look at, has no readable working folder.
    return missing(caughtValueText(caught,),);
  }
}

/**
 Collects the working folder of every process this machine shows, which is how a worktree in use is noticed.

 @returns the folders each once, or why the process table could not be read

 @example
 ```ts
 const folders = await workingFoldersOfProcesses();
 ```
 */
export async function workingFoldersOfProcesses(): Promise<Looked<readonly string[]>> {
  /**
   What the process table shows.
   */
  const table = await processTable();
  if (!table.found) return missing(table.why,);
  /**
   Each process's working folder.
   */
  const folders: readonly Looked<string>[] = await Promise.all(table.value
    .filter(isWholeNumber,)
    .map(workingFolderOf,),);
  return found([...new Set(folders.flatMap(function folderOfEach(each: Looked<string>,) {
    return each.found ? [each.value,] : [];
  },),),],);
}

/**
 Reads the process table's entries.

 @returns its entries, or why it cannot be read

 @example
 ```ts
 const table = await processTable();
 ```
 */
async function processTable(): Promise<Looked<readonly string[]>> {
  try {
    return found(await readdir('/proc',),);
  }
  catch (caught) {
    return missing(`the process table cannot be read: ${caughtValueText(caught,)}`,);
  }
}

/**
 Everything a worktree is judged with, so a test can hand over its own answer instead of asking the machine.

 @internal
 */
export type MachineState = {
  /**
   The names of the `pi-` units running now, or why the machine could not say.
   */
  readonly activeUnits: Looked<readonly string[]>;

  /**
   The working folder of every process this machine will show, or why it could not say.
   */
  readonly workingFolders: Looked<readonly string[]>;

  /**
   The time the judgement is made at, which decides whether a worktree was touched recently.
   */
  readonly nowMilliseconds: number;
};
