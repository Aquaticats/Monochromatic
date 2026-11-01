/**
 Unit tests for the CSV parser, the tar reader, and the dump cache fill.

 The tar fixtures are written by the system `tar` (GNU tar or bsdtar), an
 implementation independent of the reader under test.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import spawn from 'nano-spawn';

import {
  columnIndexes,
  createCsvParser,
  DumpFormatError,
} from './csv.ts';
import {
  dumpTimestamp,
  fillHistoriesFromDump,
  needsApi,
} from './dump.ts';
import type { VersionRecord, } from './policy.ts';
import {
  createTarReader,
  SKIP_ENTRY,
  STOP_ARCHIVE,
} from './tar.ts';

//region Fixtures

/**
 Parses CSV text fed in chunks of the given size.

 @param text - CSV text

 @param chunkSize - bytes per push, to exercise state carried across chunks

 @returns parsed rows

 @example
 ```ts
 parseAll({ text: 'a,b\n', chunkSize: 1, }); // [['a', 'b']]
 ```
 */
function parseAll({ text, chunkSize, }: { readonly text: string; readonly chunkSize: number; },): readonly (readonly string[])[] {
  /**
   Rows seen.
   */
  const rows: (readonly string[])[] = [];
  /**
   Parser under test.
   */
  const parser = createCsvParser({
    onRow: function collect(row: readonly string[],): void {
      rows.push(row,);
    },
  },);
  /**
   Encoded text.
   */
  const bytes = new TextEncoder().encode(text,);
  for (let offset = 0; offset < bytes.length; offset += chunkSize)
    parser.push(bytes.subarray(offset, offset + chunkSize,),);
  parser.end();
  return rows;
}

/**
 Writes files into a temp directory and archives them with the system `tar`.

 @param files - archive path to content, in archive order

 @returns archive bytes and a cleanup handle

 @example
 ```ts
 await using fixture = await systemTar({ 'd/data/crates.csv': 'id,name\n', });
 ```
 */
async function systemTar(files: Readonly<Record<string, string>>,): Promise<AsyncDisposable & { readonly bytes: Uint8Array; readonly root: string; }> {
  /**
   Fixture directory.
   */
  const root = await mkdtemp(join(scratchRoot(), 'rust-catalog-update-tar-',),);
  await Promise.all(Object.entries(files,).map(async function writeEntry([path, content,],): Promise<void> {
    await mkdir(join(root, 'src', path, '..',), { recursive: true, },);
    await writeFile(join(root, 'src', path,), content,);
  },),);
  await spawn('tar', ['--create', '--format=ustar', '--file', join(root, 'fixture.tar',), '--directory', join(root, 'src',), ...Object.keys(files,),],);
  return {
    bytes: new Uint8Array(
      await readFile(join(root, 'fixture.tar',),),
    ),
    root,
    [Symbol.asyncDispose]: async function removeFixture(): Promise<void> {
      await rm(root, { recursive: true, force: true, },);
    },
  };
}

/**
 Yields bytes in fixed-size chunks.

 @param bytes - all bytes

 @param chunkSize - bytes per chunk

 @returns chunks in order

 @example
 ```ts
 chunked({ bytes, chunkSize: 100, });
 ```
 */
async function* chunked({ bytes, chunkSize, }: { readonly bytes: Uint8Array; readonly chunkSize: number; },): AsyncGenerator<Uint8Array> {
  for (let offset = 0; offset < bytes.length; offset += chunkSize)
    yield bytes.subarray(offset, offset + chunkSize,);
}

/**
 crates.csv fixture with a multiline quoted readme, as in the real dump.
 */
const CRATES_CSV = [
  'created_at,id,name,readme',
  '2020-01-01 00:00:00+00,1,anyhow,"line one',
  'line two, with a comma and ""quotes"""',
  '2020-01-01 00:00:00+00,2,oxc_parser,',
  '2020-01-01 00:00:00+00,3,unlocked,',
  '2020-01-01 00:00:00+00,4,kept,',
  '',
].join('\n',);

/**
 versions.csv fixture: anyhow has token publishes only; oxc_parser has a publisher-less release after Trusted Publishing began.
 */
const VERSIONS_CSV = [
  'crate_id,id,num,created_at,published_by,repository',
  '1,10,1.0.0,2020-01-01 00:00:00.5+00,7,https://github.com/dtolnay/anyhow',
  '1,11,1.0.1,2021-01-01 00:00:00+00,7,https://github.com/dtolnay/anyhow',
  '2,20,0.1.0,2024-01-01 00:00:00+00,9,https://github.com/oxc-project/oxc',
  '2,21,0.2.0,2025-09-01 00:00:00+00,,https://github.com/oxc-project/oxc',
  '3,30,1.0.0,2020-01-01 00:00:00+00,5,',
  '4,40,1.0.0,2020-01-01 00:00:00+00,5,',
  '',
].join('\n',);

/**
 One release record for the ambiguity tests.

 @param createdAt - publish time

 @param publisherId - publishing account, empty for none

 @returns release record

 @example
 ```ts
 release({ createdAt: '2025-09-01T00:00:00Z', publisherId: '', });
 ```
 */
function release({ createdAt, publisherId, }: { readonly createdAt: string; readonly publisherId: string; },): VersionRecord {
  return { num: '1.0.0', createdAt, publisherId, trustpubRepository: '', repository: '', };
}

//endregion Fixtures

await describe({
  name: createCsvParser.name,
  children: [
    it({
      name: 'parses quoted multiline fields, doubled quotes, empty fields, CRLF rows, and a final row without a newline, in any chunking',
      fn: async () => {
        /**
         Input covering each case.
         */
        const text = 'a,"b\nc","say ""hi""",\r\n"",x,,\nlast,row';
        /**
         Expected rows.
         */
        const expected = [['a', 'b\nc', 'say "hi"', '',], ['', 'x', '', '',], ['last', 'row',],];
        for (const chunkSize of [1, 2, 3, 7, 1_024,])
          expect(parseAll({ text, chunkSize, },),).toEqual(expected,);
      },
    },),
    it({
      name: 'decodes UTF-8 split across chunks',
      fn: async () => {
        expect(parseAll({ text: 'é,日本\n', chunkSize: 1, },),).toEqual([['é', '日本',],],);
      },
    },),
  ],
},);

await describe({
  name: columnIndexes.name,
  children: [
    it({
      name: 'finds columns by name and names a missing one',
      fn: async () => {
        expect(columnIndexes({ header: ['id', 'name',], columns: ['name', 'id',], },).get('name',),).toBe(1,);
        expect(function missing(): void {
          columnIndexes({ header: ['id',], columns: ['num',], },);
        },).toThrow(DumpFormatError,);
      },
    },),
  ],
},);

await describe({
  name: createTarReader.name,
  children: [
    it({
      name: 'reads wanted entries from a system tar archive, skips others, and stops on request',
      fn: async () => {
        await using fixture = await systemTar({ 'd/a.txt': 'alpha', 'd/skip.txt': 'x'.repeat(700,), 'd/b.txt': 'bravo', 'd/never.txt': 'n', },);
        /**
         Text read per entry.
         */
        const seen = new Map<string, string>();
        /**
         Reader under test.
         */
        const reader = createTarReader({
          decide: function decide(entry,) {
            if (entry.name.endsWith('never.txt',))
              return STOP_ARCHIVE;
            if (entry.name.endsWith('skip.txt',))
              return SKIP_ENTRY;
            /**
             Bytes of this entry.
             */
            const parts: Uint8Array[] = [];
            return {
              push: function push(bytes: Uint8Array,): void {
                parts.push(bytes.slice(),);
              },
              end: function end(): void {
                seen.set(entry.name, Buffer.concat(parts,).toString('utf8',),);
              },
            };
          },
        },);
        /**
         Whether the reader asked to stop before the end.
         */
        const stopped = { value: false, };
        for await (const chunk of chunked({ bytes: fixture.bytes, chunkSize: 100, },)) {
          if (!reader.push(chunk,)) {
            stopped.value = true;
            break;
          }
        }
        expect([...seen,],).toEqual([['d/a.txt', 'alpha',], ['d/b.txt', 'bravo',],],);
        expect(stopped.value,).toBe(true,);
      },
    },),
  ],
},);

await describe({
  name: fillHistoriesFromDump.name,
  children: [
    it({
      name: 'writes histories for requested crates, leaves possible Trusted Publishing crates to the API, and keeps existing files',
      fn: async () => {
        await using fixture = await systemTar({
          '2026-10-29-020020/data/crates.csv': CRATES_CSV,
          '2026-10-29-020020/data/versions.csv': VERSIONS_CSV,
          '2026-10-29-020020/data/dependencies.csv': 'never read\n',
        },);
        /**
         Cache directory with one pre-existing (richer) API history.
         */
        const cacheDir = join(fixture.root, 'cache',);
        await mkdir(cacheDir, { recursive: true, },);
        await writeFile(join(cacheDir, 'kept.json',), '[]\n',);
        /**
         Fill outcome.
         */
        const result = await fillHistoriesFromDump({
          tarChunks: chunked({ bytes: fixture.bytes, chunkSize: 333, },),
          names: new Set(['anyhow', 'oxc_parser', 'kept', 'not-in-dump',],),
          cacheDir,
        },);
        expect(result,).toEqual({ found: 3, written: 1, leftToApi: ['oxc_parser',], kept: 1, },);
        expect(
          JSON.parse(await readFile(join(cacheDir, 'anyhow.json',), 'utf8',),),
        ).toEqual([
          { num: '1.0.0', createdAt: '2020-01-01T00:00:00.5Z', publisherId: '7', trustpubRepository: '', repository: 'https://github.com/dtolnay/anyhow', },
          { num: '1.0.1', createdAt: '2021-01-01T00:00:00Z', publisherId: '7', trustpubRepository: '', repository: 'https://github.com/dtolnay/anyhow', },
        ],);
        expect(await readFile(join(cacheDir, 'kept.json',), 'utf8',),).toBe('[]\n',);
      },
    },),
    it({
      name: 'rejects an archive that lists versions.csv before crates.csv or lacks versions.csv',
      fn: async () => {
        await using reversed = await systemTar({ 'd/data/versions.csv': VERSIONS_CSV, 'd/data/crates.csv': CRATES_CSV, },);
        await expect(fillHistoriesFromDump({ tarChunks: chunked({ bytes: reversed.bytes, chunkSize: 512, },), names: new Set(['anyhow',],), cacheDir: join(reversed.root, 'cache',), },),).rejects.toThrow(DumpFormatError,);
        await using partial = await systemTar({ 'd/data/crates.csv': CRATES_CSV, },);
        await expect(fillHistoriesFromDump({ tarChunks: chunked({ bytes: partial.bytes, chunkSize: 512, },), names: new Set(['anyhow',],), cacheDir: join(partial.root, 'cache',), },),).rejects.toThrow(DumpFormatError,);
      },
    },),
  ],
},);

await describe({
  name: dumpTimestamp.name,
  children: [
    it({
      name: 'converts PostgreSQL UTC timestamps to ISO and rejects other text',
      fn: async () => {
        expect(dumpTimestamp('2019-02-22 19:31:11.109728+00',),).toBe('2019-02-22T19:31:11.109728Z',);
        expect(function bad(): void {
          dumpTimestamp('yesterday',);
        },).toThrow(DumpFormatError,);
      },
    },),
  ],
},);

await describe({
  name: needsApi.name,
  children: [
    it({
      name: 'sends a crate to the API only for a publisher-less release since Trusted Publishing began',
      fn: async () => {
        expect(needsApi([release({ createdAt: '2018-01-01T00:00:00Z', publisherId: '', },),],),).toBe(false,);
        expect(needsApi([release({ createdAt: '2025-09-01T00:00:00Z', publisherId: '7', },),],),).toBe(false,);
        expect(needsApi([release({ createdAt: '2025-09-01T00:00:00Z', publisherId: '', },),],),).toBe(true,);
      },
    },),
  ],
},);
