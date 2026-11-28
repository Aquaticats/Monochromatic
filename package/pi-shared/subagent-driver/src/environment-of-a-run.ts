/**
 What one run's Pi is started with: the driver's own environment with every credential-named variable removed, the variables that
 name the run's temporary folder, and the folders its guard may not change, which is also where the guard's extension is read from.

 Kept in a file of its own so the whole of a run's start reads one line of it, and so neither list outgrows the line budget a file
 is allowed.

 @module
 */

import { armingOfTheMainCheckoutGuard, } from './arming-the-main-checkout-guard-of-a-run.ts';
import type { PiRecord, } from './driver-types.ts';
import {
  type EnvironmentWithoutCredentials,
  stripCredentialsFromEnvironment,
} from './environment-without-credentials.ts';
import { variablesNamingTemporaryFolder, } from './temporary-folder-of-a-run.ts';

//region Types

/**
 What a run's Pi starts with, and the words that load the extension refusing a change of the main checkout.
 */
export type EnvironmentOfARun = {
  /**
   The environment Pi inherits.
   */
  readonly environment: NodeJS.ProcessEnv;

  /**
   The words added to Pi's command line for the guard's extension.
   */
  readonly piArguments: readonly string[];
};

//endregion Types

//region Building it

/**
 Builds the environment of one run, noting what was taken from the driver's own and which folders the guard holds.

 A credential-named variable is removed for the reason `environment-without-credentials.ts` gives: Pi hands each command the whole
 of its own environment, and everything a command prints reaches a model provider. The temporary folder is named here so no caller
 passes `TMPDIR` by hand and one run's files live in one folder. The guard's folders are named here because a run works in a
 worktree, and its repository's main checkout is read-only for it.

 @param environment - the driver's own environment

 @param temporaryFolder - the folder this run's temporary files go in

 @param workingFolder - the folder this run's Pi works in, which is its worktree

 @param folder - the folder the driver's own entry is in, which the guard's entry sits beside

 @param note - where the run's feed says what was taken away and which folders the guard holds

 @returns what Pi is started with, and the guard's words for its command line

 @example
 ```ts
 const started = await environmentOfARun({ environment: process.env, temporaryFolder: '/tmp/run', workingFolder: '/repo/pi-a', folder: import.meta.dirname, note });
 ```
 */
export async function environmentOfARun({
  environment,
  temporaryFolder,
  workingFolder,
  folder,
  note,
}: {
  readonly environment: NodeJS.ProcessEnv;
  readonly temporaryFolder: string;
  readonly workingFolder: string;
  readonly folder: string;
  readonly note: (entry: PiRecord,) => void;
},): Promise<EnvironmentOfARun> {
  /**
   The driver's own environment without the variables whose names mark a credential.
   */
  const givenToPi: EnvironmentWithoutCredentials = stripCredentialsFromEnvironment({ environment, },);
  /**
   What this run's guard holds, read from the worktree it works in, and the words that load its extension.
   */
  const guardArming = await armingOfTheMainCheckoutGuard({
    workingFolder,
    folder,
    note,
  },);
  note({
    kind: 'environment',
    removedNames: givenToPi.removedNames,
  },);
  return {
    environment: {
      ...givenToPi.environment,
      ...variablesNamingTemporaryFolder({ folder: temporaryFolder, },),
      ...guardArming.environment,
    },
    piArguments: guardArming.piArguments,
  };
}

//endregion Building it
