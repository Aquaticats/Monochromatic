/**
 Tests for the lookup cache: its guards, what its read makes of a damaged, an
 absent and an unreadable file, and what its write leaves on disk.

 A CACHE FILE THAT HOLDS NO RECORD IS IGNORED, AND SAID, not refused: a
 stale, hand-made or cut-short file costs a lookup, not a run, and the
 warning names the file and none of its text. A FILE THAT IS THERE AND
 CANNOT BE READ IS REFUSED: a miss would buy the same lookup on every run
 over a file no write can replace.

 FIXTURES ARE CAT-THEMED, over a throwaway cache directory.

 @module
 */

import {
  mkdir,
  open,
  readdir,
  writeFile,
} from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type CachedLookup,
  CacheFileUnreadableError,
  EXA_SEARCH_URL,
  isLookupHit,
  isLookupRecord,
  lookupCachePath,
  type LookupRecord,
  lookupWorkTitle,
  readCachedLookup,
  workTitleLookupLines,
  writeCachedLookup,
} from '../dist/final/node/index.mjs';
import {
  directoryRefusalMessage,
  directoryRefusalText,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/cache-file-refusal.test-fixture.ts';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Lookup cache tests

/**
 Query the cases ask the cache about, as `lookupQueryFor` builds it for the
 title 《猫的午睡》.
 */
const NAP_QUERY = '《猫的午睡》 official English title';

/**
 Record a lookup of that title keeps.
 */
const NAP_RECORD: LookupRecord = {
  query: NAP_QUERY,
  fetchedAt: '2026-10-05T10:00:00.000Z',
  hits: [{
    title: 'The Nap of Mittens',
    url: 'https://cats.example/nap',
    highlight: 'Mittens naps by the stove.',
  },],
};

/**
 Opening of that record's file as a write cut short leaves it: the bytes
 stop inside the `fetchedAt` key.
 */
const CUT_SHORT = '{\n  "query": "《猫的午睡》 official English title",\n  "fetched';

/**
 Reads the nap query's record out of a throwaway cache whose file for it
 holds exactly the given bytes, keeping what the read warned.

 @param contents - exact file bytes, so a damaged file stays damaged

 @returns What the read answered, what it warned, and the file it read

 @example
 ```ts
 const { result, warned, path, } = await lookupReadOver({ contents: '5', },);
 ```
 */
async function lookupReadOver(
  { contents, }: { readonly contents: string; },
): Promise<{
  readonly result: CachedLookup;
  readonly warned: readonly string[];
  readonly path: string;
}> {
  await using scratch = await scratchDir({ prefix: 'lookup-cache-', },);
  /**
   File the nap query's record lives in.
   */
  const path = lookupCachePath({
    dir: scratch.path,
    query: NAP_QUERY,
  },);
  await writeFile(
    path,
    contents,
    'utf8',
  );
  /**
   What the read answered and warned.
   */
  const read = await warnLinesDuring({
    run: async () => readCachedLookup({
      dir: scratch.path,
      query: NAP_QUERY,
    },),
  },);
  return {
    ...read,
    path,
  };
}

/**
 Warning the read writes for a file whose text is not JSON.

 @param path - file the warning names

 @returns Line from its first tag on

 @example
 ```ts
 expect(warned,).toEqual([unparsedLookupWarning({ path, },),],);
 ```
 */
function unparsedLookupWarning({ path, }: { readonly path: string; },): string {
  return `[translation-repair] [readCachedLookup] cached lookup at ${path} does not parse as JSON; ignoring it`;
}

/**
 Transport answering the nap title's one result and keeping what it was
 asked.

 @returns Transport, and the urls it was asked in order

 @example
 ```ts
 const { fetchFn, asked, } = napTransport();
 ```
 */
function napTransport(): {
  readonly fetchFn: typeof fetch;
  readonly asked: string[];
} {
  /**
   Urls asked so far.
   */
  const asked: string[] = [];
  return {
    asked,
    fetchFn: async function answering(input,): Promise<Response> {
      asked.push(((typeof input) === 'string') ? input : ((input instanceof URL) ? input.href : input.url),);
      return Response.json({
        results: [{
          title: 'The Nap of Mittens',
          url: 'https://cats.example/nap',
          highlights: ['Mittens naps by the stove.',],
        },],
      },);
    },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'lookup cache guards',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS NO HIT and NO RECORD where the parsed value is no object',
          fn: async () => {
            expect(isLookupHit(5,),).toBe(false,);
            expect(isLookupRecord(5,),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: readCachedLookup.name,
      // SEQUENTIAL: each case diverts the one global `console.warn` across an await.
      concurrency: 1,
      children: [
        it({
          name: 'MISSES on a file cut short inside its record, and WARNS that it does not parse, naming the file and '
            + 'none of its text',
          fn: async () => {
            const {
              result,
              warned,
              path,
            } = await lookupReadOver({ contents: CUT_SHORT, },);
            expect(result,).toEqual({ kind: 'miss', },);
            expect(warned,).toEqual([unparsedLookupWarning({ path, },),],);
          },
        },),
        it({
          name: 'MISSES on an empty file, which a write cut short before its first byte leaves, and WARNS that it does '
            + 'not parse',
          fn: async () => {
            const {
              result,
              warned,
              path,
            } = await lookupReadOver({ contents: '', },);
            expect(result,).toEqual({ kind: 'miss', },);
            expect(warned,).toEqual([unparsedLookupWarning({ path, },),],);
          },
        },),
        it({
          name: 'MISSES on a file holding JSON that is no record, and WARNS that it is not one, naming the file',
          fn: async () => {
            const {
              result,
              warned,
              path,
            } = await lookupReadOver({ contents: '5', },);
            expect(result,).toEqual({ kind: 'miss', },);
            expect(warned,).toEqual([
              `[translation-repair] [readCachedLookup] cached lookup at ${path} is not a record; ignoring it`,
            ],);
          },
        },),
        it({
          name: 'MISSES and WARNS NOTHING where no file stands at the query\'s path, the ordinary state before a first lookup',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'lookup-cache-', },);
            expect(await warnLinesDuring({
              run: async () => readCachedLookup({
                dir: scratch.path,
                query: NAP_QUERY,
              },),
            },),).toEqual({
              result: { kind: 'miss', },
              warned: [],
            },);
          },
        },),
        it({
          name: 'REFUSES a path that is there and cannot be read, a directory standing where the file belongs, naming '
            + 'the path and the filesystem code, where a miss bought the lookup again on every run',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'lookup-cache-', },);
            /**
             Path the nap query's record lives at, holding a directory.
             */
            const path = lookupCachePath({
              dir: scratch.path,
              query: NAP_QUERY,
            },);
            await mkdir(path,);
            const {
              result: refusal,
              warned,
            } = await warnLinesDuring({
              run: async () => rejectionOf(async function overADirectory(): Promise<CachedLookup> {
                return readCachedLookup({
                  dir: scratch.path,
                  query: NAP_QUERY,
                },);
              },),
            },);
            expect(refusal,).toBeInstanceOf(CacheFileUnreadableError,);
            expect(String(refusal,),).toBe(directoryRefusalText({ path, },),);
            expect(warned,).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: workTitleLookupLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS NOTHING for a title whose cache path is there and cannot be read, and WARNS naming the path and '
            + 'the filesystem code, where the title was bought on every run and its record never kept',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'lookup-cache-', },);
            const dir = scratch.path;
            /**
             Path the nap query's record lives at, holding a directory.
             */
            const path = lookupCachePath({
              dir,
              query: NAP_QUERY,
            },);
            await mkdir(path,);
            const {
              fetchFn,
              asked,
            } = napTransport();
            const {
              logger,
              lines: logged,
            } = capturingLoggerPair();
            expect(await workTitleLookupLines({
              sourceText: '她读《猫的午睡》。',
              apiKey: 'whisker-key',
              dir,
              signal: new AbortController().signal,
              fetchFn,
              now: () => new Date(NAP_RECORD.fetchedAt,),
              logger,
            },),).toEqual([],);
            expect(asked,).toEqual([],);
            expect(logged,).toEqual([
              `[workTitleLookupLines] lookup for 《猫的午睡》 failed and contributes no line: ${directoryRefusalMessage({ path, },)}`,
              '[workTitleLookupLines] 1 work title looked up, 0 lines',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: lookupWorkTitle.name,
      // SEQUENTIAL: its case diverts the one global `console.warn` across an await.
      concurrency: 1,
      children: [
        it({
          name: 'BUYS AGAIN over a record cut short, WARNS once naming the file, and REWRITES the record whole, where '
            + 'the read threw and the title went without its evidence on every later run',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'lookup-cache-', },);
            const dir = scratch.path;
            /**
             File the nap query's record lives in, cut short.
             */
            const path = lookupCachePath({
              dir,
              query: NAP_QUERY,
            },);
            await writeFile(
              path,
              CUT_SHORT,
              'utf8',
            );
            const {
              fetchFn,
              asked,
            } = napTransport();
            const {
              result: bought,
              warned,
            } = await warnLinesDuring({
              run: async () => lookupWorkTitle({
                title: '《猫的午睡》',
                apiKey: 'whisker-key',
                dir,
                signal: new AbortController().signal,
                fetchFn,
                now: () => new Date(NAP_RECORD.fetchedAt,),
              },),
            },);
            expect(bought,).toEqual(NAP_RECORD,);
            expect(warned,).toEqual([unparsedLookupWarning({ path, },),],);
            expect(asked,).toEqual([EXA_SEARCH_URL,],);
            expect(await readCachedLookup({
              dir,
              query: NAP_QUERY,
            },),).toEqual({
              kind: 'hit',
              record: NAP_RECORD,
            },);
          },
        },),
      ],
    },),

    describe({
      name: writeCachedLookup.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES a record its query reads back whole, making the directory and leaving no other file in it',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'lookup-cache-', },);
            /**
             Cache directory that does not exist yet.
             */
            const dir = join(
              scratch.path,
              'lookup',
            );
            await writeCachedLookup({
              dir,
              record: NAP_RECORD,
            },);
            expect(await readCachedLookup({
              dir,
              query: NAP_QUERY,
            },),).toEqual({
              kind: 'hit',
              record: NAP_RECORD,
            },);
            expect(await readdir(dir,),).toEqual([
              basename(lookupCachePath({
                dir,
                query: NAP_QUERY,
              },),),
            ],);
          },
        },),
        it({
          name: 'LEAVES the earlier record whole under a reader that opened it before a rewrite, the later record '
            + 'standing at the path, where a rewrite in place showed that reader the later bytes',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'lookup-cache-', },);
            const dir = scratch.path;
            await writeCachedLookup({
              dir,
              record: NAP_RECORD,
            },);
            /**
             Reader holding the record's file open since before the rewrite.
             */
            await using earlier = await open(
              lookupCachePath({
                dir,
                query: NAP_QUERY,
              },),
              'r',
            );
            /**
             Record a later lookup of the same query keeps.
             */
            const later: LookupRecord = {
              query: NAP_QUERY,
              fetchedAt: '2026-10-06T10:00:00.000Z',
              hits: [],
            };
            await writeCachedLookup({
              dir,
              record: later,
            },);
            /**
             What that reader reads once the rewrite has returned.
             */
            const stillRead: unknown = JSON.parse(await earlier.readFile('utf8',),);
            expect(stillRead,).toEqual(NAP_RECORD,);
            expect(await readCachedLookup({
              dir,
              query: NAP_QUERY,
            },),).toEqual({
              kind: 'hit',
              record: later,
            },);
          },
        },),
      ],
    },),
  ],
},);

//endregion Lookup cache tests
