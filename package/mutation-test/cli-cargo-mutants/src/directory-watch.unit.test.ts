import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { scratchDirectory } from './claim-test-support.ts';
import { watchDirectoryChanges } from './directory-watch.ts';

/**
 Milliseconds that count as "did not happen" when a test waits for something that must not occur.
 */
const quietMs = 300;

/**
 Longest a test waits for the kernel to report a change, in milliseconds.
 */
const eventWaitMs = 10_000;

/**
 Race a watch against a timer.

 @param changed - the watch's promise

 @param timeoutMs - how long to wait

 @returns `changed` when the watch fired first, `quiet` when the timer did

 @example
 ```ts
 await firstOf({ changed: watch.changed, timeoutMs: 100 });
 ```
 */
async function firstOf({
  changed,
  timeoutMs,
}: {
  readonly changed: Promise<void>;
  readonly timeoutMs: number;
}): Promise<'changed' | 'quiet'> {
  return await Promise.race([
    changed.then(function fired() {
      return 'changed' as const;
    }),
    sleep(timeoutMs).then(function timedOut() {
      return 'quiet' as const;
    }),
  ]);
}

await describe({
  name: watchDirectoryChanges.name,
  children: [
    it({
      name: 'reports a file created in the directory after the watch started',
      fn: async () => {
        await using directory = await scratchDirectory('watch-created-');
        const watch = watchDirectoryChanges(directory.path);
        await writeFile(
          join(
            directory.path,
            'slot-1.owner.1',
          ),
          'x',
        );
        expect(await firstOf({
          changed: watch.changed,
          timeoutMs: eventWaitMs,
        })).toBe('changed');
        watch.close();
      },
    }),
    it({
      name: 'stays silent while nothing changes in the directory',
      fn: async () => {
        await using directory = await scratchDirectory('watch-quiet-');
        const watch = watchDirectoryChanges(directory.path);
        expect(await firstOf({
          changed: watch.changed,
          timeoutMs: quietMs,
        })).toBe('quiet');
        watch.close();
      },
    }),
    it({
      name: 'reports nothing after it was closed',
      fn: async () => {
        await using directory = await scratchDirectory('watch-closed-');
        const watch = watchDirectoryChanges(directory.path);
        watch.close();
        await writeFile(
          join(
            directory.path,
            'slot-1.owner.1',
          ),
          'x',
        );
        expect(await firstOf({
          changed: watch.changed,
          timeoutMs: quietMs,
        })).toBe('quiet');
      },
    }),
    it({
      name: 'does not throw for a directory that is missing, and stays silent so the caller\'s bounded re-check decides',
      fn: async () => {
        await using directory = await scratchDirectory('watch-missing-');
        const watch = watchDirectoryChanges(join(
          directory.path,
          'missing',
        ));
        expect(await firstOf({
          changed: watch.changed,
          timeoutMs: quietMs,
        })).toBe('quiet');
        watch.close();
      },
    }),
  ],
});
