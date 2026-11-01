import {
  spawnSync,
  type SpawnSyncReturns,
} from 'node:child_process';

import { childEnvironment, } from './child-process-environment.ts';
import { isMissingPathError, } from './missing-path-error.ts';
import { MAX_REPLY_BYTES, } from './semantic-wrap-binary-constants.ts';
import { LintBinaryMissingError, } from './semantic-wrap-binary-error.ts';
import { LintBinaryRunError, } from './semantic-wrap-run-error.ts';

//region Lint program
// STARTING THE LINTER AND READING HOW IT ENDED, the two steps every question
// put to it shares: the version question and the wrap itself. Nothing here
// knows what a passage is or which rule runs.

/* oxlint-disable no-restricted-syntax/no-sync -- every caller of the wrap is synchronous and the rule has no option to allow one call; package/translation-repair/typography/doc/semantic-wrap-sync-spawn.md */
/**
 Starts the program once, with no key and no setting of the package in its
 environment, and reads its answer as text.

 STARTED SYNCHRONOUSLY ON PURPOSE, which `no-restricted-syntax/no-sync` bans:
 `wrapReplacementText` is a plain function that its callers use inside
 comparisons and map callbacks, and an asynchronous start would turn each of
 them, and each of their callers, into an asynchronous function. The reasons,
 the measured cost and the alternatives are in `package/translation-repair/typography/doc/semantic-wrap-sync-spawn.md`.

 @param binary - program to start, by name for the PATH lookup or by path

 @param args - arguments, never holding a passage

 @param input - passage for standard input, absent for a question that takes none

 @param cwd - directory to run in, the one holding the configuration

 @returns The finished run; its `error` is set where the program could not be started or read

 @example
 ```ts
 const run = startProgram({ binary: 'monochromatic-lint', args: ['--version',], cwd, },);
 ```
 */
export function startProgram(
  {
    binary,
    args,
    input,
    cwd,
  }: {
    readonly binary: string;
    readonly args: readonly string[];
    readonly input?: string;
    readonly cwd: string;
  },
): SpawnSyncReturns<string> {
  return spawnSync(
    binary,
    [...args,],
    {
      input,
      cwd,
      encoding: 'utf8',
      maxBuffer: MAX_REPLY_BYTES,
      env: childEnvironment({ parent: process.env, },),
    },
  );
}
/* oxlint-enable no-restricted-syntax/no-sync */

/**
 Raises the failure a finished run names, where it names one.

 @param binary - program that was run

 @param characters - length of the passage it was handed, zero for the version question

 @param run - the finished run

 @throws {@link LintBinaryMissingError} when nothing stands at the program's path

 @throws {@link LintBinaryRunError} when it could not be run to completion, was stopped by a signal, or
 exited with a status other than zero

 @example
 ```ts
 refuseFailedRun({ binary, characters: text.length, run, },);
 ```
 */
export function refuseFailedRun(
  {
    binary,
    characters,
    run,
  }: {
    readonly binary: string;
    readonly characters: number;
    readonly run: SpawnSyncReturns<string>;
  },
): void {
  if (run.error !== undefined) {
    if (isMissingPathError({ error: run.error, },))
      throw new LintBinaryMissingError({ binary, },);
    throw new LintBinaryRunError({
      binary,
      characters,
      failure: {
        kind: 'code',
        code: ('code' in run.error) && ((typeof run.error
          .code) === 'string') ? run.error
            .code : 'UNNAMED',
      },
    },);
  }
  if (run.signal !== null) {
    throw new LintBinaryRunError({
      binary,
      characters,
      failure: {
        kind: 'signal',
        signal: run.signal,
      },
    },);
  }
  if (run.status === null)
    throw new Error('unreachable: a finished run names neither an error, a signal nor a status',);
  if (run.status !== 0) {
    throw new LintBinaryRunError({
      binary,
      characters,
      failure: {
        kind: 'status',
        status: run.status,
        stderrBytes: Buffer.byteLength(run.stderr,),
      },
    },);
  }
}

//endregion Lint program
