/**
 The folder one run's Pi writes its temporary files in, made inside the run folder before the first start.

 Pi keeps a tool output too long for a result in a file under the folder `os.tmpdir()` names
 (`dist/utils/output-files.js:16` in `@earendil-works/pi-coding-agent` 1.5.2), which is `/tmp` unless an environment variable
 names another folder. This machine keeps scratch out of `/tmp`: a reboot wipes it, and every brief sends scratch to
 `${HOME}/temp/agent`. So a run's Pi is given a folder of its own inside the run folder, with mode `0700`, and the variables that
 name it, so no caller passes `TMPDIR` by hand and one run's files live in one folder.

 The folder is kept when its run ends: the run's session transcript names the files Pi wrote there, so a reader of that transcript
 follows those paths to what Pi kept.

 @module
 */

import {
  chmod,
  mkdir,
} from 'node:fs/promises';
import { join, } from 'node:path';

//region The folder

/**
 The name the folder carries inside the run folder, which says what it holds to a reader who only lists the folder.
 */
export const TEMPORARY_FOLDER_NAME = 'temporary-files-pi-wrote';

/**
 The mode the folder is given, which rule `ATH` states: its own account reads it, and no other does.
 */
export const TEMPORARY_FOLDER_MODE = 0o700;

/**
 Makes the folder one run's Pi writes its temporary files in.

 The driver calls this after it has checked that the run folder holds nothing of an earlier attempt, because that check refuses a
 run folder holding any entry at all: a folder made before the check would refuse every run of its own.

 @param runFolder - the folder this run writes into

 @returns the folder, made and given `TEMPORARY_FOLDER_MODE`

 @example
 ```ts
 const { folder } = await makeTemporaryFolderOfARun({ runFolder: '/runs/copy-route-4', },);
 ```
 */
export async function makeTemporaryFolderOfARun({ runFolder, }: { readonly runFolder: string },): Promise<{
  readonly folder: string
}> {
  /**
   Where this run's temporary files go.
   */
  const folder = join(
    runFolder,
    TEMPORARY_FOLDER_NAME,
  );
  await mkdir(
    folder,
    {
      recursive: true,
      mode: TEMPORARY_FOLDER_MODE,
    },
  );
  // `mkdir` applies the mode through the umask, so a shell that starts the driver can still take bits away: the folder is set to the
  // mode this package states rather than left to whatever started the run.
  await chmod(
    folder,
    TEMPORARY_FOLDER_MODE,
  );
  return { folder, };
}

//endregion The folder

//region The variables that name it

/**
 The variables a program reads its temporary folder from, in the order `os.tmpdir()` reads them here.

 Measured on 2026-11-19 with this machine's Node 26.13.1: with none of them set `os.tmpdir()` answered `/tmp`, with all three set it
 answered the `TMPDIR` value, with `TMPDIR` unset it answered the `TMP` value, and with only `TEMP` set it answered that. A command
 Pi's `bash` tool starts may read any of them, so all three name the one folder.
 */
export const TEMPORARY_FOLDER_VARIABLES = [
  'TMPDIR',
  'TMP',
  'TEMP',
] as const;

/**
 A value for every variable that names a temporary folder, so a reader sees all three named in one place.
 */
export type TemporaryFolderVariables = Readonly<Record<(typeof TEMPORARY_FOLDER_VARIABLES)[number], string>>;

/**
 Names one folder in every variable a program reads its temporary folder from.

 @param folder - the folder the variables name

 @returns the value of each variable

 @example
 ```ts
 const variables = variablesNamingTemporaryFolder({ folder: '/runs/copy-route-4/temporary-files-pi-wrote', },);
 ```
 */
export function variablesNamingTemporaryFolder({ folder, }: { readonly folder: string },): TemporaryFolderVariables {
  return {
    TMPDIR: folder,
    TMP: folder,
    TEMP: folder,
  };
}

//endregion The variables that name it
