import { OpenSnitchConfigError, } from './errors.ts';
import { run, } from './runner.ts';

/**
 Syncs a closed OpenSnitch file or its directory without writing to the watched inode.
 Node disables descriptor sync under its permission model.

 @param path - OpenSnitch file or directory to sync.

 @throws {@link OpenSnitchConfigError} when `sync` cannot start or finish successfully within its deadline.

 @example
 ```ts
 await syncWrittenPath('/etc/opensnitchd/system-fw.json');
 ```
 */
export async function syncWrittenPath(path: string,): Promise<void> {
  try {
    await run({
      command: 'sync',
      args: [
        '--data',
        '--',
        path,
      ],
    },);
  }
  catch (error) {
    throw new OpenSnitchConfigError(
      `Cannot sync OpenSnitch path ${path}: ${String(error,)}`,
      { cause: error, },
    );
  }
}
