import { realpath, } from 'node:fs/promises';
import {
  isAbsolute,
  relative,
  resolve,
  sep,
} from 'node:path';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { rethrowUnlessMissingPath, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Pass log directory guard
// A DEFAULT LOGGER POINTED INTO A RUN FOLDER WOULD SWEEP IT (ledger B1115). The run log builds its file sink with no
// retention sweep, on purpose, but the default logger every other module of the process writes through takes its
// directory from `MONOCHROMATIC_LOG_DIRECTORY` and sweeps it: it removes every day directory older than its age bound
// and trims the oldest files by size and count. An operator who sets that variable to a run folder's `log` directory
// (the one place the pass's records are) hands the sweep the run's evidence. The pass refuses to start when the
// variable names the run folder or a place inside it. The default directory, when the variable is unset, is the user's
// state or the project's `node_modules/.monochromatic`, which the default runs directory sits inside and which the sweep
// reads by names that begin with a date only (`doc/audit-ledger-pass-records-in-the-run-folder.md`).

/**
 Name of the variable that sets the default logger's directory.
 */
const LOG_DIRECTORY_VARIABLE = 'MONOCHROMATIC_LOG_DIRECTORY';

/**
 A path as far as it exists: the real path of its longest existing prefix, then the rest as it was spelled.

 @param path - absolute path

 @returns Path with every link on the existing part resolved

 @example
 ```ts
 const real = await realOrSpelled({ path: '/runs/tabby/log/not-yet', },);
 ```
 */
async function realOrSpelled({ path, }: { readonly path: string; },): Promise<string> {
  try {
    return await realpath(path,);
  } catch (error) {
    // A PATH THAT IS NOT THERE YET is judged by its parent, which may be.
    rethrowUnlessMissingPath({ error, },);
    /**
     The directory above the missing one.
     */
    const parent = resolve(
      path,
      '..',
    );
    if (parent === path)
      return path;
    return resolve(
      await realOrSpelled({ path: parent, },),
      relative(
        parent,
        path,
      ),
    );
  }
}

/**
 Refuses a launch whose default logger would write into the run folder.

 @param env - environment the pass runs in

 @param runsDir - run folder the pass keeps its records in

 @throws {@link StatedRefusalError} when `MONOCHROMATIC_LOG_DIRECTORY` names the run folder or a place inside it

 @example
 ```ts
 await refuseLogDirectoryInRunFolder({ env: process.env, runsDir, },);
 ```
 */
export async function refuseLogDirectoryInRunFolder(
  {
    env,
    runsDir,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly runsDir: string;
  },
): Promise<void> {
  /**
   Directory the default logger was told to write to, absent when the launch did not say.
   */
  const named = env[LOG_DIRECTORY_VARIABLE];
  if ((named === undefined) || (named === ''))
    return;

  /**
   Where the run folder really is.
   */
  const folder = await realOrSpelled({ path: resolve(runsDir,), },);

  /**
   Where the logger's directory really is.
   */
  const directory = await realOrSpelled({ path: resolve(named,), },);
  /**
   The logger's directory seen from the run folder.
   */
  const from = relative(
    folder,
    directory,
  );
  /**
   Whether the logger's directory is above the run folder or beside it, which `relative` spells by leading `..`.
   */
  const outside = (from === '..')
    || from.startsWith(`..${sep}`,)
    || isAbsolute(from,);
  if (!outside)
    throw new StatedRefusalError({
      says: `${LOG_DIRECTORY_VARIABLE} names ${named}, which is the run folder ${runsDir} or inside it, so the default `
        + 'logger would sweep the records of this run: its retention sweep removes day directories past its age bound '
        + 'and trims the oldest files by size and count. Unset it, or name a directory outside the run folder, then '
        + 'start the pass again. Nothing was started.',
    },);
}

//endregion Pass log directory guard
