import { rejects } from 'node:assert/strict';
import {
  mkdir,
  mkdtempDisposable,
  readdir,
  readFile,
  symlink,
} from 'node:fs/promises';
import { join } from 'node:path';

import { describe, expect, it } from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  plantedControlsRecordName,
  writePlantedControlsRecord,
} from './native-planted-controls-record.ts';

await describe({
  name: writePlantedControlsRecord.name,
  children: [
    it({
      name: 'keeps the previous evidence and removes the temporary record when sync is missing or fails',
      fn: async () => {
        /** Evidence directory and isolated executable search path for this test process. */
        await using root = await mkdtempDisposable(join(scratchRoot(), 'planted-controls-sync-'));
        /** The evidence is separate from the command fixture so its entries can be checked exactly. */
        const evidence = join(root.path, 'evidence');
        /** An empty PATH directory initially makes sync unavailable. */
        const bin = join(root.path, 'bin');
        await Promise.all([mkdir(evidence), mkdir(bin)]);
        /** Write the prior record through the real sync before replacing PATH. */
        const record = { prior: true };
        const target = await writePlantedControlsRecord({ evidence, record });
        /** Restore PATH even if an assertion fails. */
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
        await rejects(
          writePlantedControlsRecord({ evidence, record: { replacement: true } }),
          function missingSync(error: unknown) {
            expect(String(error)).toContain(`DurableWriteSyncError: Cannot sync ${target}.`);
            expect(String(error)).toContain('GNU coreutils sync --data: Error: spawn sync ENOENT');
            return true;
          },
        );
        expect(await readdir(evidence)).toEqual([plantedControlsRecordName]);
        await symlink('/usr/bin/false', join(bin, 'sync'));
        await rejects(
          writePlantedControlsRecord({ evidence, record: { replacement: true } }),
          function failedSync(error: unknown) {
            expect(String(error)).toContain(`DurableWriteSyncError: Cannot sync ${target}.`);
            expect(String(error)).toContain('GNU coreutils sync --data: Error: Command failed: sync --data -- ');
            return true;
          },
        );
        expect(await readdir(evidence)).toEqual([plantedControlsRecordName]);
        expect(await readFile(target, 'utf8')).toBe(`${JSON.stringify(record, null, 2)}\n`);
      },
    }),
  ],
});
