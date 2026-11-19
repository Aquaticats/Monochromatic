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
import { CargoMutantsToolingError } from './error.ts';


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
  name: publishNew.name,
  concurrency: 1,
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
        using path = usePath(bin);
        await writeFile(sync, standIn('file'), { mode: 0o700 });
        await rejects(
          publishNew({ directory, target, record }),
          function fileSyncFailed(error: unknown) {
            expect(error).toBeInstanceOf(CargoMutantsToolingError);
            expect(String(error)).toContain(`${directory}/.owner-claim.tmp.`);
            expect(String(error)).toContain('fixture rejected file sync');
            return true;
          },
        );
        expect(await readdir(directory)).toEqual([]);
        await writeFile(sync, standIn('directory'));
        await rejects(
          publishNew({ directory, target, record }),
          function directorySyncFailed(error: unknown) {
            expect(error).toBeInstanceOf(CargoMutantsToolingError);
            expect(String(error)).toContain(directory);
            expect(String(error)).toContain('fixture rejected directory sync');
            return true;
          },
        );
        expect(await readdir(directory)).toEqual([name]);
        expect(await readClaimRecord(target)).toEqual(record);
        expect((await readFile(log, 'utf8')).trimEnd().split('\n')).toEqual(['file', 'file', 'directory']);
      },
    }),
    it({
      name: 'fails a stalled sync at the command deadline and removes the private claim',
      timeout: slowSyncMs + 15_000,
      fn: async () => {
        await using root = await scratchDirectory('claim-stall-');
        const directory = join(root.path, 'claims');
        const bin = join(root.path, 'bin');
        await Promise.all([mkdir(directory), mkdir(bin)]);
        const target = join(directory, 'item.owner.1');
        await writeFile(
          join(bin, 'sync'),
          `#!${process.execPath}
import { statSync } from 'node:fs';
if (statSync(process.argv.at(-1)).isFile()) setTimeout(() => {}, ${String(slowSyncMs)});
`,
          { mode: 0o700 },
        );
        using path = usePath(bin);
        await rejects(
          publishNew({ directory, target, record: { kind: 'released' } }),
          function deadline(error: unknown) {
            expect(error).toBeInstanceOf(CargoMutantsToolingError);
            expect(String(error)).toContain(directory);
            expect(String(error)).toContain(`still running after ${String(syncDeadlineMs)} ms`);
            return true;
          },
        );
        expect(await readdir(directory)).toEqual([]);
      },
    }),
    it({
      name: 'reports missing sync as a tooling error and removes the private claim',
      fn: async () => {
        await using root = await scratchDirectory('claim-no-sync-');
        const directory = join(root.path, 'claims');
        const bin = join(root.path, 'bin');
        await Promise.all([mkdir(directory), mkdir(bin)]);
        using path = usePath(bin);
        await rejects(
          publishNew({ directory, target: join(directory, 'item.owner.1'), record: { kind: 'released' } }),
          function missingSync(error: unknown) {
            expect(error).toBeInstanceOf(CargoMutantsToolingError);
            expect(String(error)).toContain(directory);
            expect(String(error)).toContain('spawn sync ENOENT');
            return true;
          },
        );
        expect(await readdir(directory)).toEqual([]);
      },
    }),
    it({
      name: 'passes the option-like directory with spaces, quotes and a newline as one operand after --',
      fn: async () => {
        await using root = await scratchDirectory('claim-argv-');
        const cwd = process.cwd();
        const directory = `-claims 'one' "two"\nnext`;
        const bin = join(root.path, 'bin');
        const log = join(root.path, 'argv.jsonl');
        using restoreCwd = {
          [Symbol.dispose](): void {
            process.chdir(cwd);
          },
        };
        process.chdir(root.path);
        await Promise.all([mkdir(directory), mkdir(bin)]);
        await writeFile(join(bin, 'sync'), `#!${process.execPath}
import { appendFileSync } from 'node:fs';
appendFileSync(${JSON.stringify(log)}, JSON.stringify(process.argv.slice(2)) + '\\n');
`, { mode: 0o700 });
        using path = usePath(bin);
        const target = join(directory, 'item.owner.1');
        expect(await publishNew({ directory, target, record: { kind: 'released' } })).toBe(true);
        const calls = (await readFile(log, 'utf8')).trimEnd().split('\n');
        expect(calls).toHaveLength(2);
        expect(calls[0]).toContain(JSON.stringify(`${directory}/.owner-claim.tmp.`).slice(0, -1));
        expect(calls.at(-1)).toBe(JSON.stringify(['--data', '--', directory]));
      },
    }),
  ],
});
