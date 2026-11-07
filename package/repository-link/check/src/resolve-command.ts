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
import { errorCodeOf, } from './error-code-of.ts';

/**
 What looking at one `PATH` candidate found: it is executable, it is absent, or it failed for a reason that is not an absence.
 */
type CandidateOutcome = {
  readonly candidate: string;
  readonly found: boolean;
  readonly failure?: unknown;
};

/**
 Resolves a command to a file path.

 @param command - a path (when it has a slash) or a bare program name
 
 @param pathVariable - the `PATH` value to search
 
 @returns the path of the first executable match, or the command unchanged when none matches

 @throws Error when a candidate cannot be inspected for a reason other than absence or missing permission, with the failure as its cause

 @example
 ```ts
 import { resolveCommand, } from '\@monochromatic-dev/repository-link-check';
 
 const result = await resolveCommand({ command, pathVariable, },);
 ```
 */
export async function resolveCommand(
  {
    command,
    pathVariable,
  }: {
    readonly command: string;
    readonly pathVariable: string;
  },
): Promise<string> {
  if (command.includes('/',)) {
    return command;
  }
  /**
   What looking at each candidate found, in `PATH` order.
   */
  const outcomes = await Promise.all(pathVariable.split(':',)
    .map(async function probeCandidate(folder,): Promise<CandidateOutcome> {
      /**
       The candidate of this step.
       */
      const candidate = join(
        folder === '' ? '.' : folder,
        command,
      );
      try {
        await access(
          candidate,
          constants.X_OK,
        );
        return {
          candidate,
          found: true,
        };
      }
      catch (error) {
        /**
         The code of the failure.
         */
        const code = errorCodeOf(error,);
        return {
          candidate,
          found: false,
          failure: ((code === 'ENOENT') || (code === 'EACCES')) ? undefined : error,
        };
      }
    },),);
  /**
   The first outcome that is not an absence: the first executable match, or a failure that stops the search.
   */
  const decisive = outcomes.find(function isNotAbsent(outcome,) {
    return outcome.found || (outcome.failure !== undefined);
  },);
  if (decisive?.found === true) {
    return decisive.candidate;
  }
  if (decisive !== undefined) {
    throw new Error(
      `could not look at ${decisive.candidate} while searching PATH for ${command}`,
      { cause: decisive.failure, },
    );
  }
  return command;
}
