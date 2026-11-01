import { mkdir, mkdtemp, readFile, readdir, rm, } from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { createFileSink, } from '@monochromatic-dev/logger-core/node';

//region Complete records survive overlapping asynchronous file appends

await describe({
  name: 'file sink concurrent record integrity',
  children: [
    it({
      name: 'preserves complete large JSONL records in invocation order',
      fn: async () => {
        /** Private disposable project keeps verification away from shared log files. */
        const scratch = join(homedir(), 'temp/agent',);
        await mkdir(scratch, { recursive: true, mode: 0o700, },);
        /** Fresh project contains only this test's file sink output. */
        const directory = await mkdtemp(join(scratch, 'logger-record-integrity-',),);
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(directory, { recursive: true, force: true, },);
          },
        };
        await mkdir(join(directory, 'node_modules',),);
        /** Consumer-facing built sink must persist both small and multi-write records. */
        const sink = createFileSink({ directory: join(directory, 'node_modules/.monochromatic',), });
        expect(await sink.verify(),).toBe(true,);
        /** Drain the retention sweep so the reader sees one settled artifact. */
        await sink.flush?.();
        await sink.write({ level: 'info', timestamp: -1, message: 'ordinary record', },);
        /** Each payload crosses the filesystem helper's chunk boundary and includes JSON escapes. */
        const messages = ['a', 'b',].map(function largeMessage(marker,) {
          return `${marker}${'"\\\n世界'.repeat(150_000,)}`;
        },);
        await Promise.all(messages.map(async function appendLargeRecord(message, index,) {
          await sink.write({ level: 'info', timestamp: index, message, },);
        },),);
        /** Read the persisted artifact rather than inspecting append calls. */
        const logDirectory = join(directory, 'node_modules/.monochromatic',);
        /** The root holds one day directory plus the retention stamp; the log sits one level down. */
        const entries = await readdir(logDirectory, { withFileTypes: true, },);
        const nested = await Promise.all(entries.map(async function collect(entry,) {
          if (!entry.isDirectory())
            return entry.name.endsWith('.log.jsonl',) ? [entry.name,] : [];
          const inner = await readdir(join(logDirectory, entry.name,),);
          return inner.filter((name: string,) => name.endsWith('.log.jsonl',),)
            .map((name: string,) => join(entry.name, name,),);
        },),);
        const filenames = nested.flat();
        expect(filenames,).toHaveLength(1,);
        const [filename,] = filenames;
        if (filename === undefined)
          throw new Error('The verified sink did not create its log file',);
        const persisted = await readFile(join(logDirectory, filename,), 'utf8',);
        const rows = persisted.trimEnd().split('\n',);
        expect(rows,).toHaveLength(3,);
        /** Parsing errors report their location without including the full record. */
        const records = rows.map(function parseRecord(row, index,) {
          try {
            return JSON.parse(row,) as { readonly timestamp: number; readonly message?: string; };
          }
          catch (error) {
            throw new Error(`Malformed JSONL record at index ${index}`, { cause: error, },);
          }
        },);
        expect(records.map(function recordTime(entry,) {
          return entry.timestamp;
        },),)
          .toEqual([-1, 0, 1,],);
        expect(records[1]?.message === messages[0],).toBe(true,);
        expect(records[2]?.message === messages[1],).toBe(true,);
      },
    },),
  ],
},);

//endregion Complete records survive overlapping asynchronous file appends
