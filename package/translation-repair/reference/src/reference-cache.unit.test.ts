/**
 Tests for the reference cache's read and write over a damaged, an absent and
 an unreadable file. Its guard and its place under the lookup cache are
 tested with the channel they serve, in `cited-reference.unit.test.ts`.

 A CACHE FILE THAT HOLDS NO RECORD IS IGNORED, AND SAID, not refused: a
 cut-short or hand-made file costs one fetch of the page, not the entry
 citing it, and the warning names the file and none of its text. A FILE
 THAT IS THERE AND CANNOT BE READ IS REFUSED: a miss would buy the same page
 on every run over a file no write can replace.

 THE TRANSPORT IS A STUB. Fixtures are cat-themed, over a throwaway cache
 directory. No corpus content appears here.

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

import { EXA_CONTENTS_URL, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  type CachedReference,
  CacheFileUnreadableError,
  citedReferenceBlock,
  readCachedReference,
  referenceCachePath,
  type ReferenceRecord,
  writeCachedReference,
} from '../dist/final/node/index.mjs';
import { directoryRefusalText, } from '@monochromatic-dev/translation-repair-test-fixture/ts/cache-file-refusal.test-fixture.ts';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Reference cache tests

/**
 Page the cases ask the cache about.
 */
const POST_URL = 'https://cats.example/posts/in-memory-of-mittens';

/**
 Record a fetch of that page keeps.
 */
const POST_RECORD: ReferenceRecord = {
  url: POST_URL,
  fetchedAt: '2026-10-05T10:00:00.000Z',
  status: 'success',
  title: 'In memory of Mittens',
  text: 'Mittens had an older sister.',
};

/**
 Opening of that record's file as a write cut short leaves it: the bytes
 stop inside the `fetchedAt` key.
 */
const CUT_SHORT = '{\n  "url": "https://cats.example/posts/in-memory-of-mittens",\n  "fetched';

/**
 Reads the page's record out of a throwaway cache whose file for it holds
 exactly the given bytes, keeping what the read warned.

 @param contents - exact file bytes, so a damaged file stays damaged

 @returns What the read answered, what it warned, and the file it read

 @example
 ```ts
 const { result, warned, path, } = await referenceReadOver({ contents: '5', },);
 ```
 */
async function referenceReadOver(
  { contents, }: { readonly contents: string; },
): Promise<{
  readonly result: CachedReference;
  readonly warned: readonly string[];
  readonly path: string;
}> {
  await using scratch = await scratchDir({ prefix: 'reference-cache-', },);
  /**
   File the page's record lives in.
   */
  const path = referenceCachePath({
    dir: scratch.path,
    url: POST_URL,
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
    run: async () => readCachedReference({
      dir: scratch.path,
      url: POST_URL,
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
 expect(warned,).toEqual([unparsedReferenceWarning({ path, },),],);
 ```
 */
function unparsedReferenceWarning({ path, }: { readonly path: string; },): string {
  return `[translation-repair] [readCachedReference] cached reference at ${path} does not parse as JSON; ignoring it`;
}

/**
 Transport answering the page's text and keeping what it was asked.

 @returns Transport, and the urls it was asked in order

 @example
 ```ts
 const { fetchFn, asked, } = postTransport();
 ```
 */
function postTransport(): {
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
          url: POST_URL,
          title: 'In memory of Mittens',
          text: 'Mittens had an older sister.',
        },],
        statuses: [{
          id: POST_URL,
          status: 'success',
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
      name: readCachedReference.name,
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
            } = await referenceReadOver({ contents: CUT_SHORT, },);
            expect(result,).toEqual({ kind: 'miss', },);
            expect(warned,).toEqual([unparsedReferenceWarning({ path, },),],);
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
            } = await referenceReadOver({ contents: '', },);
            expect(result,).toEqual({ kind: 'miss', },);
            expect(warned,).toEqual([unparsedReferenceWarning({ path, },),],);
          },
        },),
        it({
          name: 'MISSES on a file holding JSON that is no record, and WARNS that it is not one, naming the file',
          fn: async () => {
            const {
              result,
              warned,
              path,
            } = await referenceReadOver({ contents: '{"not":"a record"}\n', },);
            expect(result,).toEqual({ kind: 'miss', },);
            expect(warned,).toEqual([
              `[translation-repair] [readCachedReference] cached reference at ${path} is not a record; ignoring it`,
            ],);
          },
        },),
        it({
          name: 'MISSES and WARNS NOTHING where no file stands at the page\'s path, the ordinary state before a first fetch',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'reference-cache-', },);
            expect(await warnLinesDuring({
              run: async () => readCachedReference({
                dir: scratch.path,
                url: POST_URL,
              },),
            },),).toEqual({
              result: { kind: 'miss', },
              warned: [],
            },);
          },
        },),
        it({
          name: 'REFUSES a path that is there and cannot be read, a directory standing where the file belongs, naming '
            + 'the path and the filesystem code, where a miss bought the page again on every run',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'reference-cache-', },);
            /**
             Path the page's record lives at, holding a directory.
             */
            const path = referenceCachePath({
              dir: scratch.path,
              url: POST_URL,
            },);
            await mkdir(path,);
            const {
              result: refusal,
              warned,
            } = await warnLinesDuring({
              run: async () => rejectionOf(async function overADirectory(): Promise<CachedReference> {
                return readCachedReference({
                  dir: scratch.path,
                  url: POST_URL,
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
      name: citedReferenceBlock.name,
      // SEQUENTIAL: its case diverts the one global `console.warn` across an await.
      concurrency: 1,
      children: [
        it({
          name: 'CARRIES the page\'s line over a record cut short, BUYS the page once, WARNS at each of its two reads '
            + 'of the file and REWRITES the record whole, where the block rejected and stopped the entry on every '
            + 'later run',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'reference-cache-', },);
            const dir = scratch.path;
            /**
             File the page's record lives in, cut short.
             */
            const path = referenceCachePath({
              dir,
              url: POST_URL,
            },);
            await writeFile(
              path,
              CUT_SHORT,
              'utf8',
            );
            const {
              fetchFn,
              asked,
            } = postTransport();
            const {
              logger,
              lines: logged,
            } = capturingLoggerPair();
            const {
              result: block,
              warned,
            } = await warnLinesDuring({
              run: async () => citedReferenceBlock({
                sourceText: `见 ${POST_URL}`,
                apiKey: 'whisker-key',
                dir,
                signal: new AbortController().signal,
                fetchFn,
                now: () => new Date(POST_RECORD.fetchedAt,),
                logger,
              },),
            },);
            expect(block,).toBe(`- reference 1 ${POST_URL} ("In memory of Mittens"): Mittens had an older sister.`,);
            expect(asked,).toEqual([EXA_CONTENTS_URL,],);
            expect(logged,).toEqual([
              `[citedReferenceBlock] REFERENCE 1 ${POST_URL}: success, 28 chars, bought`,
              '[citedReferenceBlock] REFERENCES cited=1 cached=0 bought=1',
            ],);
            expect(warned,).toEqual([
              unparsedReferenceWarning({ path, },),
              unparsedReferenceWarning({ path, },),
            ],);
            expect(await readCachedReference({
              dir,
              url: POST_URL,
            },),).toEqual({
              kind: 'hit',
              record: POST_RECORD,
            },);
          },
        },),
        it({
          name: 'REJECTS over a page whose cache path is there and cannot be read, before anything is bought, where '
            + 'the page was bought on every run, its record never kept and its line said it could not be fetched',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'reference-cache-', },);
            const dir = scratch.path;
            /**
             Path the page's record lives at, holding a directory.
             */
            const path = referenceCachePath({
              dir,
              url: POST_URL,
            },);
            await mkdir(path,);
            const {
              fetchFn,
              asked,
            } = postTransport();
            const {
              logger,
              lines: logged,
            } = capturingLoggerPair();
            /**
             What the block rejected with.
             */
            const refusal = await rejectionOf(async function overAnUnreadablePath(): Promise<string> {
              return citedReferenceBlock({
                sourceText: `见 ${POST_URL}`,
                apiKey: 'whisker-key',
                dir,
                signal: new AbortController().signal,
                fetchFn,
                now: () => new Date(POST_RECORD.fetchedAt,),
                logger,
              },);
            },);
            expect(refusal,).toBeInstanceOf(CacheFileUnreadableError,);
            expect(String(refusal,),).toBe(directoryRefusalText({ path, },),);
            expect(asked,).toEqual([],);
            expect(logged,).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: writeCachedReference.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES a record its url reads back whole, making the directory and leaving no other file in it',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'reference-cache-', },);
            /**
             Cache directory that does not exist yet.
             */
            const dir = join(
              scratch.path,
              'reference',
            );
            await writeCachedReference({
              dir,
              record: POST_RECORD,
            },);
            expect(await readCachedReference({
              dir,
              url: POST_URL,
            },),).toEqual({
              kind: 'hit',
              record: POST_RECORD,
            },);
            expect(await readdir(dir,),).toEqual([
              basename(referenceCachePath({
                dir,
                url: POST_URL,
              },),),
            ],);
          },
        },),
        it({
          name: 'LEAVES the earlier record whole under a reader that opened it before a rewrite, the later record '
            + 'standing at the path, where a rewrite in place showed that reader the later bytes',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'reference-cache-', },);
            const dir = scratch.path;
            await writeCachedReference({
              dir,
              record: POST_RECORD,
            },);
            /**
             Reader holding the record's file open since before the rewrite.
             */
            await using earlier = await open(
              referenceCachePath({
                dir,
                url: POST_URL,
              },),
              'r',
            );
            /**
             Record a later fetch of the same page keeps.
             */
            const later: ReferenceRecord = {
              url: POST_URL,
              fetchedAt: '2026-10-06T10:00:00.000Z',
              status: 'error',
              title: '',
              text: '',
              failure: 'CRAWL_NOT_FOUND',
            };
            await writeCachedReference({
              dir,
              record: later,
            },);
            /**
             What that reader reads once the rewrite has returned.
             */
            const stillRead: unknown = JSON.parse(await earlier.readFile('utf8',),);
            expect(stillRead,).toEqual(POST_RECORD,);
            expect(await readCachedReference({
              dir,
              url: POST_URL,
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

//endregion Reference cache tests
