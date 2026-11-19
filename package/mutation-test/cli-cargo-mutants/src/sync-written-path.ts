import { CargoMutantsToolingError } from './error.ts';
import {
  type ProgramResult,
  runProgram,
} from './run-program.ts';

/**
 Sync stall bound sized against recorded write-and-sync readings, not command-start latency.
 `doc/handover/agents-at-once-on-this-machine.md` ("Readings on the new drive") records 35 readings of 4 KiB with
 up to 13 Pi units: maxima of 72 ms on home and 2,564 ms on cargo-build.
 `doc/handover/cli-git-rust-implementation.md` records a maximum of 15,512 ms on the earlier drive with 18 agents
 held for eight hours. The bound leaves 44,488 ms beyond that largest observed time.
 */
export const SYNC_DEADLINE_MILLISECONDS: number = 60_000;

/**
 Syncs a closed claim file before publication, or its directory after publication.
 Node disables descriptor sync under its permission model.

 @param path - Claim file or directory to sync.

 @param runSync - Command runner that enforces the sync deadline.

 @throws {@link CargoMutantsToolingError} when `sync` cannot start or finish successfully within its deadline.

 @example
 ```ts
 await syncWrittenPath({ path: '/evidence' });
 ```
 */
export async function syncWrittenPath({
  path,
  runSync = runProgram,
}: {
  readonly path: string;
  readonly runSync?: typeof runProgram;
}): Promise<void> {
  /**
   Operation named in spawn, timeout and nonzero status errors alike.
   */
  const operation = `Cannot sync ${path} with GNU coreutils sync --data`;
  /**
   Status and stderr available after the runner observes the child close.
   */
  const outcome = await (async function waitForSync(): Promise<ProgramResult> {
    try {
      return await runSync({
        program: 'sync',
        args: [
          '--data',
          '--',
          path,
        ],
        inheritStdio: false,
        deadlineMilliseconds: SYNC_DEADLINE_MILLISECONDS,
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
