/**
 Who holds a run's name: the unit the run is going as, or the launcher that has not made that unit yet, with the moment each took
 it. A refusal names its holder, because a name that reads as taken without one leaves a reader nothing to act on.

 @module
 */

import { stat, } from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  NO_FINISHED_RUN_OF_THAT_NAME,
  SUMMARY_FILE_NAME,
  newestFinishedRunOf,
} from './newest-finished-run-of-a-name.ts';
import type { RunCommand, } from './run-command.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The failures that mean a process's own entry cannot be read: it ended between the listing and the read.
 */
const SKIPPED_ENTRY_CODES: ReadonlySet<string> = new Set<string>([
  MISSING_PATH_CODE,
  'ESRCH',
]);

/**
 What holds a run's name.
 */
export type HolderOfARunName =
  | {
    /**
     The run's own unit.
     */
    readonly kind: 'unit';

    /**
     The unit's name as systemd prints it.
     */
    readonly unit: string;

    /**
     The state the unit is in.
     */
    readonly state: string;

    /**
     When the unit entered that state, as systemd prints it, or nothing when it printed none.
     */
    readonly since?: string;
  }
    | {
    /**
     The launcher process that has not made the run's unit yet.
     */
      readonly kind: 'launcher';

    /**
     The launcher's process number.
     */
      readonly pid: number;

    /**
     When the launcher began, as ISO 8601, or nothing when its process entry answered nothing about it.
     */
      readonly since?: string;
    };

/**
 Reads the properties one `systemctl show` printed for a unit, by their names.

 @param shown - what the manager printed

 @returns one entry per `Name=Value` line, and nothing for a line that is not one

 @example
 ```ts
 const facts = unitFactsIn({ shown: 'ActiveState=active\n', },);
 ```
 */
export function unitFactsIn({ shown, }: { readonly shown: string },): Readonly<Record<string, string>> {
  /**
   The pairs the lines hold.
   */
  const pairs: [
    string,
    string
  ][] = [];
  for (const line of shown.split('\n',)) {
    /**
     Where the name ends and the value begins.
     */
    const at = line.indexOf('=',);
    if (at > 0) {
      pairs.push([
        line.slice(
          0,
          at,
        ),
        line.slice(at + 1,),
      ],);
    }
  }
  return Object.fromEntries(pairs,);
}

/**
 Reads what the manager says about one unit.

 @param run - how a command is run

 @param unitName - the unit's name as systemd prints it

 @returns the properties the manager printed, and nothing when it could not answer

 @example
 ```ts
 const facts = await unitFactsOf({ run, unitName: 'pi-a.service', },);
 ```
 */
export async function unitFactsOf({
  run,
  unitName,
}: {
  readonly run: RunCommand;
  readonly unitName: string;
},): Promise<Readonly<Record<string, string>>> {
  /**
   What the manager answered.
   */
  const shown = await run({
    command: 'systemctl',
    words: [
      '--user',
      'show',
      unitName,
      '--property=LoadState',
      '--property=ActiveState',
      '--property=ActiveEnterTimestamp',
    ],
  },);
  return (shown.status === 0)
    ? unitFactsIn({ shown: shown.stdout, },)
    : {};
}

/**
 Reads when a process began, from its own entry in the process folder, which the kernel makes when it makes the process.
 
 @param processFolder - the folder holding one entry per process

 @param entry - the process's entry in that folder

 @returns the moment it began as ISO 8601, and empty text when that entry could not be read

 @example
 ```ts
 const since = await startedAtOf({ processFolder: '/proc', entry: '12', },);
 ```
 */
export async function startedAtOf({
  processFolder,
  entry,
}: {
  readonly processFolder: string;
  readonly entry: string;
},): Promise<string> {
  try {
    return (await stat(join(
      processFolder,
      entry,
    ),)).mtime
      .toISOString();
  }
  catch (error) {
    if (SKIPPED_ENTRY_CODES.has(errorCodeOf(error,),)) {
      return '';
    }
    throw error;
  }
}

/**
 Says whether the newest summary of a run's name was written by the run the launcher it is asked about started.

 A summary speaks for the attempt that wrote it and for no other: a run folder an earlier attempt left says nothing about a launcher
 that began after that attempt ended. So the summary's own time is held against the launcher's start, and only a summary written
 strictly after it says this launcher's run ended: two writes in the same tick of the file system's clock read as the same moment, and
 a moment that cannot be told apart counts as no evidence against a live launcher. A launcher whose start could not be read is going
 for the same reason.

 @param runsFolder - folder the driver writes one run folder per run into

 @param runName - the run's name, which every attempt's folder starts with

 @param since - when the launcher began, as ISO 8601, or empty text when that could not be read

 @returns whether a summary written strictly after that launcher began is there, which says its run ended

 @example
 ```ts
 const ended = await runEndedForLauncher({ runsFolder: '/runs', runName: 'a', since: '2026-11-15T09:00:00.000Z', },);
 ```
 */
export async function runEndedForLauncher({
  runsFolder,
  runName,
  since,
}: {
  readonly runsFolder: string;
  readonly runName: string;
  readonly since: string;
},): Promise<boolean> {
  // A launcher whose own start could not be read is going: nothing ties a summary to it, and dropping a live launcher from the
  // counts hands its name to a second start.
  if (since === '') {
    return false;
  }
  /**
   The newest attempt of that name that finished, which speaks for itself and for no other attempt.
   */
  const finished = await newestFinishedRunOf({
    runsFolder,
    runName,
  },);
  if (finished === NO_FINISHED_RUN_OF_THAT_NAME) {
    return false;
  }
  /**
   When that run's summary was written, which is held against the launcher's own start.
   */
  const written = await stat(join(
    runsFolder,
    finished,
    SUMMARY_FILE_NAME,
  ),);
  return written.mtimeMs > Date.parse(since,);
}
