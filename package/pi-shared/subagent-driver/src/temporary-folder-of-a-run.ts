/**
 The folder one run's Pi writes its temporary files in, made inside the run folder before the first start.

 Pi keeps a tool output too long for a result in a file under the folder `os.tmpdir()` names
 (`dist/utils/output-files.js:16` in `@earendil-works/pi-coding-agent` 1.5.2), which is `/tmp` unless an environment variable
 names another folder. This machine keeps scratch out of `/tmp`: a reboot wipes it, and every brief sends scratch to
 `${HOME}/temp/agent`. So a run's Pi is given a folder of its own inside the run folder, with mode `0700`, and the variables that
 name it, so no caller passes `TMPDIR` by hand and one run's files live in one folder.

 The folder is looked at after it is made, without following anything: a link planted under the name is refused rather than followed,
 so a folder outside the run folder never becomes Pi's.

 The folder is kept when its run ends: the run's session transcript names the files Pi wrote there, so a reader of that transcript
 follows those paths to what Pi kept.

 @module
 */

import {
  chmod,
  lstat,
  mkdir,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

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
 A run that could not give its Pi a folder of its own for the files Pi's tools keep.

 The driver makes the folder after it has checked the run folder, and starts no Pi when it cannot: a Pi with no folder of its own
 would keep a tool's whole output under `/tmp`, which rule `ATH` keeps scratch out of.
 */
export class TemporaryFolderNotMadeError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'TemporaryFolderNotMadeError';

  /**
   The folder the run was to give its Pi, named for a reader.
   */
  readonly folder: string;

  /**
   Builds the failure.

   @param folder - the folder the run was to give its Pi

   @param reason - what the operating system answered, in words that name no value

   @example
   ```ts
   throw new TemporaryFolderNotMadeError({ folder: '/runs/a-1/temporary-files-pi-wrote', reason: 'EEXIST: file already exists', },);
   ```
   */
  constructor({
    folder,
    reason,
  }: {
    readonly folder: string;
    readonly reason: string
  },) {
    super(
      `the folder ${folder} for the files this run's Pi writes its temporary files in could not be made, so no Pi was started: ${
        reason
      }`,
    );
    this.folder = folder;
  }
}

/**
 Makes the folder one run's Pi writes its temporary files in.

 The driver calls this after it has checked that the run folder holds nothing of an earlier attempt, because that check refuses a
 run folder holding any entry at all: a folder made before the check would refuse every run of its own.

 The folder is looked at after it is made and before anything is set through it: a link planted under the name is refused rather
 than followed, so a folder outside the run folder never becomes Pi's.

 @param runFolder - the folder this run writes into

 @returns the folder, made and given `TEMPORARY_FOLDER_MODE`

 @throws TemporaryFolderNotMadeError when the folder cannot be made, is a link or something else that is not a folder, or its
 mode cannot be set

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
  try {
    await mkdir(
      folder,
      {
        recursive: true,
        mode: TEMPORARY_FOLDER_MODE,
      },
    );
    /**
     What the path is now, read without following it: a link under the name fails this, before anything is set through it.
     */
    const made = await lstat(folder,);
    if (made.isSymbolicLink() || (!made.isDirectory())) {
      throw new TemporaryFolderNotMadeError({
        folder,
        reason: 'the path is not a folder of its own: it is a link, or something else that is not a folder, so this run refuses to '
          + 'follow it',
      },);
    }
    // `mkdir` applies the mode through the umask, so a shell that starts the driver can still take bits away: the mode this package
    // states is set here rather than left to whatever started the run. The mode is set by path rather than through an open descriptor
    // because this machine's Node runs with the permission model, which forbids `fchmod` (`process.permission` is present).
    await chmod(
      folder,
      TEMPORARY_FOLDER_MODE,
    );
  }
  catch (error) {
    // A refusal made here says what it is; anything else is the operating system's answer, wrapped with the folder it is about.
    if (error instanceof TemporaryFolderNotMadeError) {
      throw error;
    }
    throw new TemporaryFolderNotMadeError({
      folder,
      // The caught value's own text, which for a failure of the operating system names the call and the path it was about.
      reason: caughtValueText(error,),
    },);
  }
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
