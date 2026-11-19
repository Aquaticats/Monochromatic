import { rejects } from 'node:assert/strict';
import {
  mkdir,
  mkdtempDisposable,
  readdir,
  readFile,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { describe, expect, it } from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  plantedControlsRecordName,
  writePlantedControlsRecord,
} from './native-planted-controls-record.ts';
import { NativeVerificationError } from './native-verification-process.ts';


/** Existing wg-quicker command stall bound, also required for a durable sync. */
const syncDeadlineMs = 60_000;

/** The stand-in eventually succeeds if the caller does not enforce its deadline. */
const slowSyncMs = syncDeadlineMs + 2_000;

/**
 Install a private executable search path for this serial suite.

 @param bin - Directory containing the stand-in sync, or empty for a missing command

 @returns Restoration for the test's previous PATH
 */
function usePath(bin: string): Disposable {
  /** Previous search path, including whether it was set. */
  const previousPath = process.env.PATH;
  process.env.PATH = bin;
  return {
    [Symbol.dispose](): void {
      if (previousPath === undefined)
        delete process.env.PATH;
      else
        process.env.PATH = previousPath;
    },
  };
}

await describe({
  name: writePlantedControlsRecord.name,
  concurrency: 1,
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
        using path = usePath(bin);
        await rejects(
          writePlantedControlsRecord({ evidence, record: { replacement: true } }),
          function missingSync(error: unknown) {
            expect(error).toBeInstanceOf(NativeVerificationError);
            expect(String(error)).toContain(target);
            expect(String(error)).toContain('sync --data');
            expect(String(error)).toContain('spawn sync ENOENT');
            return true;
          },
        );
        expect(await readdir(evidence)).toEqual([plantedControlsRecordName]);
        await symlink('/usr/bin/false', join(bin, 'sync'));
        await rejects(
          writePlantedControlsRecord({ evidence, record: { replacement: true } }),
          function failedSync(error: unknown) {
            expect(error).toBeInstanceOf(NativeVerificationError);
            expect(String(error)).toContain(target);
            expect(String(error)).toContain('status 1');
            return true;
          },
        );
        expect(await readdir(evidence)).toEqual([plantedControlsRecordName]);
        expect(await readFile(target, 'utf8')).toBe(`${JSON.stringify(record, null, 2)}\n`);
      },
    }),
    it({
      name: 'fails a stalled sync at the command deadline and keeps the prior record',
      timeout: slowSyncMs + 15_000,
      fn: async () => {
        await using root = await mkdtempDisposable(join(scratchRoot(), 'planted-controls-stall-'));
        const evidence = join(root.path, 'evidence');
        const bin = join(root.path, 'bin');
        await Promise.all([mkdir(evidence), mkdir(bin)]);
        const record = { prior: true };
        const target = await writePlantedControlsRecord({ evidence, record });
        await writeFile(join(bin, 'sync'), `#!${process.execPath}\nsetTimeout(() => {}, ${String(slowSyncMs)});\n`, { mode: 0o700 });
        using path = usePath(bin);
        await rejects(
          writePlantedControlsRecord({ evidence, record: {} }),
          function deadline(error: unknown) {
            expect(error).toBeInstanceOf(NativeVerificationError);
            expect(String(error)).toContain(target);
            expect(String(error)).toContain(`still running after ${String(syncDeadlineMs)} ms`);
            return true;
          },
        );
        expect(await readdir(evidence)).toEqual([plantedControlsRecordName]);
        expect(await readFile(target, 'utf8')).toBe(`${JSON.stringify(record, null, 2)}\n`);
      },
    }),
    it({
      name: 'passes one option-like temporary path with spaces, quotes and a newline after --',
      fn: async () => {
        await using root = await mkdtempDisposable(join(scratchRoot(), 'planted-controls-argv-'));
        const bin = join(root.path, 'bin');
        const cwd = process.cwd();
        const evidence = `-evidence 'one' "two"\nnext`;
        const log = join(root.path, 'argv.json');
        using restoreCwd = {
          [Symbol.dispose](): void {
            process.chdir(cwd);
          },
        };
        process.chdir(root.path);
        await Promise.all([mkdir(evidence), mkdir(bin)]);
        await writeFile(join(bin, 'sync'), `#!${process.execPath}
import { writeFileSync } from 'node:fs';
writeFileSync(${JSON.stringify(log)}, JSON.stringify(process.argv.slice(2)));
`, { mode: 0o700 });
        using path = usePath(bin);
        const target = await writePlantedControlsRecord({ evidence, record: {} });
        const [mode, end, operand, ...extra] = JSON.parse(await readFile(log, 'utf8')) as readonly string[];
        expect([mode, end, ...extra]).toEqual(['--data', '--']);
        expect(target).toBe(join(evidence, plantedControlsRecordName));
        expect(operand?.startsWith(`${target}.`)).toBe(true);
        expect(await readdir(evidence)).toEqual([plantedControlsRecordName]);
      },
    }),
  ],
});
