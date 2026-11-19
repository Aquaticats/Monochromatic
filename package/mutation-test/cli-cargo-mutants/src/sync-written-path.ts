import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

/**
 Failure to sync a claim file or its containing directory.
 */
class DurableWriteSyncError extends Error {
  /**
   Names the operation that failed rather than the write that already ran.
   */
  override name = 'DurableWriteSyncError';
}

/**
 Node's own `execFile` promise adapter signature, avoiding the generic void callback overload.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs GNU coreutils `sync` without a shell and waits for its result.
 */
const execFileAsync = promisifyExecFile(execFile);

/**
 Syncs a closed claim file before publication, or its directory after publication.
 Node disables descriptor sync under its permission model.

 @param path - Claim file or directory to sync.

 @throws {@link DurableWriteSyncError} when `sync` cannot start or finish successfully.

 @example
 ```ts
 await syncWrittenPath('/evidence');
 ```
 */
export async function syncWrittenPath(path: string): Promise<void> {
  try {
    await execFileAsync(
      'sync',
      [
        '--data',
        '--',
        path,
      ],
    );
  }
  catch (error) {
    throw new DurableWriteSyncError(
      `Cannot sync ${path} with GNU coreutils sync --data: ${String(error)}`,
      { cause: error },
    );
  }
}
