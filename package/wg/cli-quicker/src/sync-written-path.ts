import { run, } from './runner.ts';

/**
 Syncs a closed OpenSnitch file without rewriting it or changing the watched inode.
 Node disables descriptor sync under its permission model.

 @param path - OpenSnitch file to sync.

 @throws {@link import('./errors.ts').CommandSpawnError} when `sync` cannot start.

 @throws {@link import('./errors.ts').CommandError} when `sync` exits unsuccessfully.

 @throws {@link import('./errors.ts').CommandTimeoutError} when `sync` times out.

 @example
 ```ts
 await syncWrittenPath('/etc/opensnitchd/system-fw.json');
 ```
 */
export async function syncWrittenPath(path: string,): Promise<void> {
  await run({
    command: 'sync',
    args: [
      '--data',
      '--',
      path,
    ],
  },);
}
