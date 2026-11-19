import { CargoMutantsToolingError } from './error.ts';
import {
  type ProgramResult,
  runProgram,
} from './run-program.ts';

/**
 Reuse the 60-second stall bound justified by local command measurements in
 `package/wg/cli-quicker/src/runner.ts` (see `COMMAND_DEADLINE_MILLISECONDS`).
 The sync workload on the home dataset is recorded in `doc/research/node-permission-default-with-npm.md`.
 */
const syncDeadlineMilliseconds = 60_000;

/**
 Syncs a closed claim file before publication, or its directory after publication.
 Node disables descriptor sync under its permission model.

 @param path - Claim file or directory to sync.

 @throws {@link CargoMutantsToolingError} when `sync` cannot start or finish successfully within its deadline.

 @example
 ```ts
 await syncWrittenPath('/evidence');
 ```
 */
export async function syncWrittenPath(path: string): Promise<void> {
  /**
   Operation named in spawn, timeout and nonzero status errors alike.
   */
  const operation = `Cannot sync ${path} with GNU coreutils sync --data`;
  /**
   Status and stderr available after the runner observes the child close.
   */
  const outcome = await (async function waitForSync(): Promise<ProgramResult> {
    try {
      return await runProgram({
        program: 'sync',
        args: [
          '--data',
          '--',
          path,
        ],
        inheritStdio: false,
        deadlineMilliseconds: syncDeadlineMilliseconds,
      });
    }
    catch (error) {
      throw new CargoMutantsToolingError(
        `${operation}: ${String(error)}`,
        { cause: error },
      );
    }
  })();
  if (outcome.status !== 0)
    throw new CargoMutantsToolingError(`${operation}: command failed with status ${String(outcome.status)}. ${outcome.stderr}`);
}
