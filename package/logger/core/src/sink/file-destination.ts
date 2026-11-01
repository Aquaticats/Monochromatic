/**
 Project-first default destination selection, with strict explicit destinations. @module
 */
import { stat, } from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { reportLoggerFallback, } from '../error-format.ts';
import { probeLogDirectory, } from './file-open.ts';
import {
  findNodeModulesUp,
  NO_NODE_MODULES_FOUND,
} from './file-project.ts';
import { userLogDirectory, } from './file-state.ts';

//region Fallback is visible and cannot redirect a caller-selected destination.
/**
 Probe user state before announcing that records will be written there.

 The directory is verified,
 not a log file,
 because the sink creates its file only on first record.

 @param reason - evidence explaining why project-local logging is unavailable

 @returns Verified fallback directory.
 */
async function probeFallback(reason: string,): Promise<string> {
  /**
   Fallback destination is resolved only after default project selection fails.
   */
  const directory = userLogDirectory();
  await probeLogDirectory(directory,);
  reportLoggerFallback({
    directory,
    reason,
  },);
  return directory;
}

/**
 Select a default project directory or visibly fall back to verified user state.

 The returned directory receives the sink's day directories;
 the log file itself appears only with the first record.

 @returns Verified directory owned by this sink's process tree.

 @example
 ```ts
 const directory = await resolveDefaultLogDirectory();
 ```
 */
export async function resolveDefaultLogDirectory(): Promise<string> {
  /**
   Nearest project dependency directory or the explicit missing-project sentinel.
   */
  const nodeModules = await findNodeModulesUp({
    cwd: process.cwd(),
    stat,
    dirname,
    join,
  },);
  if (nodeModules === NO_NODE_MODULES_FOUND)
    return await probeFallback('No ancestor node_modules directory was found',);
  /**
   Existing default project location remains first choice.
   */
  const projectDirectory = join(
    nodeModules,
    '.monochromatic',
  );
  try {
    await probeLogDirectory(projectDirectory,);
    return projectDirectory;
  }
  catch (error: unknown) {
    return await probeFallback(`Project directory ${JSON.stringify(projectDirectory,)}: ${caughtValueText(error,)}`,);
  }
}
//endregion
