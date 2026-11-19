import { rejects } from 'node:assert/strict';
import {
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { describe, expect, it } from '@monochromatic-dev/test-harness-core/ts';

import { publishNew, readClaimRecord } from './claim-files.ts';
import { scratchDirectory } from './claim-test-support.ts';

await describe({
  name: publishNew.name,
  children: [
    it({
      name: 'removes a private claim on file sync failure and waits for directory sync after linking',
      fn: async () => {
        /** Separate claim and command fixture directories. */
        await using root = await scratchDirectory('claim-sync-');
        const directory = join(root.path, 'claims');
        const bin = join(root.path, 'bin');
        await Promise.all([mkdir(directory), mkdir(bin)]);
        /** Target generation and its record. */
        const name = 'item.owner.1';
        const target = join(directory, name);
        const record = { kind: 'released' } as const;
        /** Stands in for GNU coreutils, recording operands and failing only on the chosen path kind. */
        const sync = join(bin, 'sync');
        const log = join(root.path, 'sync-paths.jsonl');
        /**
         Generates a stand-in that fails one kind of sync.

         @param failKind - Path kind that should produce a non-zero exit.

         @returns Executable fixture text.
         */
        function standIn(failKind: 'file' | 'directory'): string {
          return `#!${process.execPath}
import { appendFileSync, statSync } from 'node:fs';
const [mode, end, path] = process.argv.slice(2);
if (mode !== '--data' || end !== '--' || !path) throw new Error('unexpected sync arguments');
const kind = statSync(path).isDirectory() ? 'directory' : 'file';
appendFileSync(${JSON.stringify(log)}, kind + '\\n');
if (kind === ${JSON.stringify(failKind)}) throw new Error('fixture rejected ' + kind + ' sync');
`;
        }
        const previousPath = process.env.PATH;
        using restorePath = {
          [Symbol.dispose](): void {
            if (previousPath === undefined)
              delete process.env.PATH;
            else
              process.env.PATH = previousPath;
          },
        };
        process.env.PATH = bin;
        await writeFile(sync, standIn('file'), { mode: 0o700 });
        await rejects(
          publishNew({ directory, target, record }),
          function fileSyncFailed(error: unknown) {
            expect(String(error)).toContain(`DurableWriteSyncError: Cannot sync ${directory}/.owner-claim.tmp.`);
            expect(String(error)).toContain('fixture rejected file sync');
            return true;
          },
        );
        expect(await readdir(directory)).toEqual([]);
        await writeFile(sync, standIn('directory'));
        await rejects(
          publishNew({ directory, target, record }),
          function directorySyncFailed(error: unknown) {
            expect(String(error)).toContain(`DurableWriteSyncError: Cannot sync ${directory} with GNU coreutils sync --data`);
            expect(String(error)).toContain('fixture rejected directory sync');
            return true;
          },
        );
        expect(await readdir(directory)).toEqual([name]);
        expect(await readClaimRecord(target)).toEqual(record);
        expect((await readFile(log, 'utf8')).trimEnd().split('\n')).toEqual(['file', 'file', 'directory']);
      },
    }),
  ],
});
