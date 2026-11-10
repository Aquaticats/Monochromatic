/**
 Publishes one note into the inbox of every Pi run that is going, under a dot name that is then renamed, so the driver never
 reads half of it.

 This is how a coordinator steers a run it did not start: a rule it learned, a correction to a brief, or the owner's words. The
 run reads it as a steer while it works and as a new prompt while it rests.

 @module
 */

import {
  mkdir,
  rename,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { inboxOfARun, } from './pi-runs-folders.ts';
import { latestRunFolderOf, } from './run-folder-of-a-unit.ts';

/**
 The character a note's name may not hold, which would put it outside the inbox.
 */
const FOLDER_SEPARATOR = '/';

/**
 The character a name starts with while the note is being written, which the driver never reads.
 */
const UNREAD_NAME_START = '.';

/**
 A note's name, which the driver reads as the message's name, or nothing when it would leave the inbox.

 @param noteName - the name the caller asked for

 @returns why the name cannot be used, or nothing when it can

 @example
 ```ts
 const problem = noteNameProblemOf({ noteName: '../outside', });
 ```
 */
export function noteNameProblemOf({
  noteName,
}: {
  readonly noteName: string;
},): string | typeof A_USABLE_NAME {
  if (noteName === '') {
    return 'a note needs a name, which is the name the run reads it under';
  }
  if (noteName.includes(FOLDER_SEPARATOR,)) {
    return `a note's name is one file name, and "${noteName}" names a path`;
  }
  if (noteName.startsWith(UNREAD_NAME_START,)) {
    return `a note's name may not start with "${UNREAD_NAME_START}", which is how a note is written before it is read`;
  }
  return A_USABLE_NAME;
}

/**
 A name the inbox takes, as a symbol so no complaint can be mistaken for one.
 */
export const A_USABLE_NAME: unique symbol = Symbol('the inbox takes this note name',);

/**
 Writes one note into one run's inbox.

 @param runFolder - the run folder of the attempt going now

 @param note - the note's text

 @param noteName - the name the run reads the note under

 @returns the path the note now has

 @throws the error a write gave, such as an inbox that cannot be made

 @example
 ```ts
 const path = await publishNote({ runFolder: '/runs/a-1', note: 'text', noteName: 'rule', });
 ```
 */
export async function publishNote({
  runFolder,
  note,
  noteName,
}: {
  readonly runFolder: string;
  readonly note: string;
  readonly noteName: string;
},): Promise<string> {
  /**
   What the run's inbox folder is.
   */
  const inbox = inboxOfARun({ runFolder, },);
  await mkdir(
    inbox,
    { recursive: true, },
  );
  /**
   Where the note is written before anything may read it.
   */
  const unread = join(
    inbox,
    `${UNREAD_NAME_START}${noteName}`,
  );
  /**
   Where the run reads it.
   */
  const readable = join(
    inbox,
    noteName,
  );
  await writeFile(
    unread,
    note,
    { encoding: 'utf8', },
  );
  await rename(
    unread,
    readable,
  );
  return readable;
}

/**
 Publishes one note to every run named, and says what became of each.

 The newest attempt of a run is the one going now: a run folder is `<name>-<number>`, and a run that was resumed wrote a higher
 number than the attempt before it.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param note - the note's text

 @param noteName - the name every run reads the note under

 @param runs - the names of the runs going now

 @param skip - the runs to leave alone

 @returns one line per run

 @throws the error a read or a write gave

 @example
 ```ts
 const lines = await notesToRuns({ runsFolder: '/runs', note: 'text', noteName: 'rule', runs: ['a',], skip: [], });
 ```
 */
export async function notesToRuns({
  runsFolder,
  note,
  noteName,
  runs,
  skip,
}: {
  readonly runsFolder: string;
  readonly note: string;
  readonly noteName: string;
  readonly runs: readonly string[];
  readonly skip: readonly string[];
},): Promise<readonly string[]> {
  /**
   One line per run, each run's inbox written on its own, kept in the order the runs were named.
   */
  const lines = await Promise.all(runs.map(async function lineOfRun(name,) {
    if (skip.includes(name,)) {
      return '';
    }
    /**
     The run folder of the attempt going now.
     */
    const runFolder = await latestRunFolderOf({
      runsFolder,
      runName: name,
    },);
    if ((typeof runFolder) === 'symbol') {
      return `no run folder for ${name}`;
    }
    await publishNote({
      runFolder,
      note,
      noteName,
    },);
    return `sent to ${runFolder}`;
  },),);
  return lines.filter(function wasSent(line,) {
    return line !== '';
  },);
}
