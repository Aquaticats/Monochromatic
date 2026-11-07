/**
 Finds the program a command names, the way a shell would, because `process.execve` runs exactly the file it is given
 and does not search `PATH`.

 @module
 */

import {
  access,
  constants,
} from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Resolves a command to a file path.

 @param command - a path (when it has a slash) or a bare program name
 @param pathVariable - the `PATH` value to search
 @returns the path of the first executable match, or the command unchanged when none matches
 */
export async function resolveCommand(
  { command, pathVariable, }: {
    readonly command: string;
    readonly pathVariable: string;
  },
): Promise<string> {
  if (command.includes('/',)) {
    return command;
  }
  for (const folder of pathVariable.split(':',)) {
    const candidate = join(folder === '' ? '.' : folder, command,);
    try {
      await access(candidate, constants.X_OK,);
      return candidate;
    }
    catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT' && (error as NodeJS.ErrnoException).code !== 'EACCES') {
        throw error;
      }
    }
  }
  return command;
}
