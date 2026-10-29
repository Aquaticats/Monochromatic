/**
 Tests the cited-reference channel (class thirty-five, 2026-09-16): which
 pages an original links, how one page is bought off the contents endpoint,
 how it is cached and read back, and the lines and block the critic and
 panel sheets carry.

 THE TRANSPORT IS A STUB: the real endpoint was exercised once by hand on
 2026-09-16 (the Mio blog's archived copy answered its text) and these tests
 cover what the module does around it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { setImmediate as turnOfTheLoop, } from 'node:timers/promises';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CITED_REFERENCE_CANDIDATE_RULE,
  CITED_REFERENCE_RULE,
  citedReferenceBlockText,
  citedReferenceCandidateLines,
  citedReferenceEvidence,
  CitedReferenceFetchError,
  EXA_CONTENTS_URL,
  fetchCitedReference,
  fetchedOf,
  REFERENCE_TEXT_CHARACTERS,
} from '@monochromatic-dev/translation-repair-slice-reading';
import {
  citedReferenceBlock,
  citedReferenceUrlsOf,
  isReferenceRecord,
  LOOKUP_CACHE_DIR_VAR,
  lookupCitedReference,
  MAX_CITED_REFERENCES,
  readCachedReference,
  referenceCacheDir,
  referenceCachePath,
  referenceLineOf,
  type ReferenceRecord,
  writeCachedReference,
} from '../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { untilAborted, } from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';

/**
 Abort signal that never fires.
 */
const SIGNAL = new AbortController().signal;

/**
 Fixed clock.
 */
const NOW = new Date('2026-09-16T16:00:00.000Z',);

/**
 Logger for the block function.
 */
const l = tagged({ tag: 'cited-reference-test', },);

/**
 Page the invented original cites.
 */
const POST_URL = 'https://cats.example/posts/in-memory-of-mittens';

/**
 Archived copy of that page, cited beside it.
 */
const ARCHIVE_URL = 'https://web.archive.org/web/20260101000000/https://cats.example/posts/in-memory-of-mittens';

/**
 Invented original citing a page, its archive, a bare link ending a sentence,
 the corpus's own site and a photo path.
 */
const SOURCE_TEXT = `喵喵有一个姐姐。参考链接：[博客・怀念喵喵](${POST_URL})（[存档](${ARCHIVE_URL})）

另见 https://cats.example/posts/in-memory-of-mittens 和 https://one-among.us/people/mittens。

<PhotoScroll photos={['\${path}/photos/photo1.webp',]} />
`;

/**
 One call a stub transport saw.
 */
type SeenCall = {
  readonly url: string;
  readonly body: string;
  readonly key: string;
};

/**
 Stub transport answering one fixed body and recording every call.

 @param status - HTTP status to answer

 @param body - JSON body to answer

 @returns Transport and the calls it saw

 @example
 ```ts
 const { fetchFn, seen, } = stubTransport({ status: 200, body: {}, },);
 ```
 */
function stubTransport(
  {
    status,
    body,
  }: {
    readonly status: number;
    readonly body: unknown;
  },
): {
  readonly fetchFn: typeof fetch;
  readonly seen: SeenCall[];
} {
  /**
   Calls seen so far.
   */
  const seen: SeenCall[] = [];
  /**
   Transport under test.

   @param input - what production would hand the transport

   @param init - request as production builds it

   @returns Fixed answer
   */
  async function stubbed(
    input: string | URL | Request,
    init?: RequestInit,
  ): Promise<Response> {
    /**
     Headers as sent.
     */
    const headers = new Headers(init?.headers,);
    /**
     Body as sent, text or nothing.
     */
    const sent = init?.body;
    seen.push({
      url: ((typeof input) === 'string') ? input : ((input instanceof URL) ? input.href : input.url),
      body: ((typeof sent) === 'string') ? sent : '',
      key: headers.get('x-api-key',) ?? '',
    },);
    return Response.json(
      body,
      { status, },
    );
  }
  return {
    fetchFn: stubbed,
    seen,
  };
}

/**
 Body the endpoint answers for a readable page.
 */
const READABLE_BODY = {
  results: [{
    url: POST_URL,
    title: 'In memory of Mittens',
    text: 'Mittens had an older sister.\nThe sister was also a tabby.',
  },],
  statuses: [{
    id: POST_URL,
    status: 'success',
  },],
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: citedReferenceUrlsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS every link once, in order, trailing punctuation removed, the corpus\'s own site and photo paths left out',
          fn: async () => {
            expect(citedReferenceUrlsOf({ text: SOURCE_TEXT, },),).toEqual([
              POST_URL,
              ARCHIVE_URL,
            ],);
            expect(citedReferenceUrlsOf({ text: '没有链接。', },),).toEqual([],);
          },
        },),
        it({
          name: 'LEAVES OUT profile pages (a GitHub user, a twitter handle, a bilibili space, a zhihu person) and keeps pages under them',
          fn: async () => {
            expect(citedReferenceUrlsOf({
              text: [
                'https://github.com/whiskers',
                'https://twitter.com/whiskers',
                'https://x.com/whiskers/',
                'https://space.bilibili.com/123456',
                'https://www.zhihu.com/people/whiskers',
                'https://t.me/whiskers',
                'https://github.com/whiskers/naps',
                'https://twitter.com/whiskers/status/1',
                'https://www.zhihu.com/question/1',
              ].join(' ',),
            },),).toEqual([
              'https://github.com/whiskers/naps',
              'https://twitter.com/whiskers/status/1',
              'https://www.zhihu.com/question/1',
            ],);
          },
        },),
        it({
          name: 'STOPS at the reference cap, in order of citation',
          fn: async () => {
            /**
             Original citing more pages than the cap.
             */
            const many = Array.from(
              { length: MAX_CITED_REFERENCES + 2, },
              function link(_,
                at,): string {
                return `https://cats.example/p/${String(at,)}`;
              },
            )
              .join(' ',);
            expect(citedReferenceUrlsOf({ text: many, },).length,).toBe(MAX_CITED_REFERENCES,);
            expect(citedReferenceUrlsOf({ text: many, },).at(0,),).toBe('https://cats.example/p/0',);
          },
        },),
        it({
          name: 'PASSES OVER an http that opens no scheme and a bare scheme that parses as no link, and keeps the page after them',
          fn: async () => {
            expect(citedReferenceUrlsOf({ text: 'httpd 的日志见 https:// 与 https://cats.example/p/nap', },),).toEqual([
              'https://cats.example/p/nap',
            ],);
          },
        },),
        it({
          name: 'READS the links in order of citation where the two schemes alternate',
          fn: async () => {
            expect(citedReferenceUrlsOf({
              text: '见 https://cats.example/p/a 与 http://cats.example/p/b 与 https://cats.example/p/c 与 http://cats.example/p/d。',
            },),).toEqual([
              'https://cats.example/p/a',
              'http://cats.example/p/b',
              'https://cats.example/p/c',
              'http://cats.example/p/d',
            ],);
          },
        },),
        it({
          name: 'READS one link where a scheme directly follows a scheme, since no stop character parts them',
          fn: async () => {
            expect(citedReferenceUrlsOf({
              text: 'https://http://cats.example/p/nap 与 http://https://cats.example/p/purr',
            },),).toEqual([
              'https://http://cats.example/p/nap',
              'http://https://cats.example/p/purr',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: fetchedOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a readable page, an endpoint error tag, and an empty answer, and REFUSES a body without results',
          fn: async () => {
            expect(fetchedOf({ parsed: READABLE_BODY, },),).toEqual({
              status: 'success',
              title: 'In memory of Mittens',
              text: 'Mittens had an older sister.\nThe sister was also a tabby.',
            },);
            expect(fetchedOf({
              parsed: {
                results: [],
                statuses: [{
                  id: POST_URL,
                  status: 'error',
                  error: { tag: 'CRAWL_NOT_FOUND', httpStatusCode: 404, },
                },],
              },
            },),).toEqual({
              status: 'error',
              title: '',
              text: '',
              failure: 'CRAWL_NOT_FOUND',
            },);
            expect(fetchedOf({ parsed: { results: [], }, },),).toEqual({
              status: 'error',
              title: '',
              text: '',
              failure: 'no result',
            },);
            expect(function refuses(): void {
              fetchedOf({ parsed: { data: [], }, },);
            },).toThrow(CitedReferenceFetchError,);
          },
        },),
      ],
    },),

    describe({
      name: fetchCitedReference.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS the contents endpoint for one url with the key in the header and the character cap in the body',
          fn: async () => {
            const {
              fetchFn,
              seen,
            } = stubTransport({
              status: 200,
              body: READABLE_BODY,
            },);
            /**
             What the fetch read.
             */
            const fetched = await fetchCitedReference({
              apiKey: 'whisker-key',
              url: POST_URL,
              signal: SIGNAL,
              fetchFn,
            },);
            expect(fetched.status,).toBe('success',);
            expect(seen.at(0,)?.url,).toBe(EXA_CONTENTS_URL,);
            expect(seen.at(0,)?.key,).toBe('whisker-key',);
            expect(JSON.parse(seen.at(0,)?.body ?? '{}',),).toEqual({
              urls: [POST_URL,],
              text: { maxCharacters: REFERENCE_TEXT_CHARACTERS, },
            },);
          },
        },),
        it({
          name: 'REFUSES a non-2xx answer naming the HTTP status and never the key, the url or the provider body',
          fn: async () => {
            const { fetchFn, } = stubTransport({
              status: 401,
              body: { error: 'no', },
            },);
            try {
              await fetchCitedReference({
                apiKey: 'whisker-key',
                url: POST_URL,
                signal: SIGNAL,
                fetchFn,
              },);
              throw new Error('the fetch did not refuse',);
            } catch (error) {
              expect(error,).toBeInstanceOf(CitedReferenceFetchError,);
              expect(String(error,),).toBe('CitedReferenceFetchError: contents responded 401',);
            }
          },
        },),
      ],
    },),

    describe({
      name: 'reference cache',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SITS under the lookup cache, WRITES a record its url reads back, and MISSES on absence or a foreign file',
          fn: async () => {
            /**
             Throwaway cache home.
             */
            await using scratch = await scratchDir({ prefix: 'reference-cache-', },);
            const dir = scratch.path;
            expect(referenceCacheDir({ env: { [LOOKUP_CACHE_DIR_VAR]: dir, }, },),).toBe(join(
              dir,
              'reference',
            ),);
            /**
             Record to keep.
             */
            const record: ReferenceRecord = {
              url: POST_URL,
              fetchedAt: NOW.toISOString(),
              status: 'success',
              title: 'In memory of Mittens',
              text: 'Mittens had an older sister.',
            };
            expect(isReferenceRecord(record,),).toBe(true,);
            expect(isReferenceRecord({ url: POST_URL, },),).toBe(false,);
            await writeCachedReference({
              dir,
              record,
            },);
            expect(await readCachedReference({
              dir,
              url: POST_URL,
            },),).toEqual({
              kind: 'hit',
              record,
            },);
            expect(await readCachedReference({
              dir,
              url: ARCHIVE_URL,
            },),).toEqual({ kind: 'miss', },);
            await writeFile(
              referenceCachePath({
                dir,
                url: ARCHIVE_URL,
              },),
              '{"not":"a record"}\n',
              'utf8',
            );
            expect(await readCachedReference({
              dir,
              url: ARCHIVE_URL,
            },),).toEqual({ kind: 'miss', },);
          },
        },),
        it({
          name: 'BUYS a page once and READS IT BACK on every later ask, so a resumed run never hits the page again',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'reference-lookup-', },);
            const dir = scratch.path;
            const {
              fetchFn,
              seen,
            } = stubTransport({
              status: 200,
              body: READABLE_BODY,
            },);
            /**
             Asks the same page.

             @returns Record for the page
             */
            async function ask(): Promise<ReferenceRecord> {
              return lookupCitedReference({
                url: POST_URL,
                apiKey: 'whisker-key',
                dir,
                signal: SIGNAL,
                fetchFn,
                now: () => NOW,
              },);
            }
            /**
             First answer, bought.
             */
            const first = await ask();
            /**
             Second answer, read back.
             */
            const second = await ask();
            expect(first,).toEqual(second,);
            expect(first.fetchedAt,).toBe(NOW.toISOString(),);
            expect(seen.length,).toBe(1,);
          },
        },),
        it({
          name: 'REFUSES a record that is not an object at all, and ACCEPTS an error record carrying its failure tag',
          fn: async () => {
            expect(isReferenceRecord(null,),).toBe(false,);
            expect(isReferenceRecord({
              url: POST_URL,
              fetchedAt: NOW.toISOString(),
              status: 'error',
              title: '',
              text: '',
              failure: 'CRAWL_NOT_FOUND',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES an error record without its failure tag and a success record carrying one, the two shapes no fetch writes',
          fn: async () => {
            expect(isReferenceRecord({
              url: POST_URL,
              fetchedAt: NOW.toISOString(),
              status: 'error',
              title: '',
              text: '',
            },),).toBe(false,);
            expect(isReferenceRecord({
              url: POST_URL,
              fetchedAt: NOW.toISOString(),
              status: 'success',
              title: 'In memory of Mittens',
              text: 'Mittens had an older sister.',
              failure: 'CRAWL_NOT_FOUND',
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: referenceLineOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RENDERS the page on one line with its title, says when nothing was readable, and names the failure',
          fn: async () => {
            expect(referenceLineOf({
              index: 1,
              record: {
                url: POST_URL,
                fetchedAt: NOW.toISOString(),
                status: 'success',
                title: 'In memory\nof Mittens',
                text: 'Mittens had an older sister.\n\nThe sister was also a tabby.',
              },
            },),).toBe(`- reference 1 ${POST_URL} ("In memory of Mittens"): Mittens had an older sister. The sister was also a tabby.`,);
            expect(referenceLineOf({
              index: 2,
              record: {
                url: ARCHIVE_URL,
                fetchedAt: NOW.toISOString(),
                status: 'success',
                title: '',
                text: ' \n ',
              },
            },),).toBe(`- reference 2 ${ARCHIVE_URL}: nothing readable on the page`,);
            expect(referenceLineOf({
              index: 3,
              record: {
                url: ARCHIVE_URL,
                fetchedAt: NOW.toISOString(),
                status: 'error',
                title: '',
                text: '',
                failure: 'CRAWL_NOT_FOUND',
              },
            },),).toBe(`- reference 3 ${ARCHIVE_URL}: could not be fetched (CRAWL_NOT_FOUND)`,);
          },
        },),
      ],
    },),

    describe({
      name: citedReferenceBlock.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS NOTHING without a link or without a key, and otherwise CARRIES one line per cited page in order',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'reference-block-', },);
            const dir = scratch.path;
            const {
              fetchFn,
              seen,
            } = stubTransport({
              status: 200,
              body: READABLE_BODY,
            },);
            /**
             Asks for the block over one original with one key.

             @param sourceText - original

             @param apiKey - key, empty for none

             @returns Block
             */
            async function block(
              {
                sourceText,
                apiKey,
              }: {
                readonly sourceText: string;
                readonly apiKey: string;
              },
            ): Promise<string> {
              return citedReferenceBlock({
                sourceText,
                apiKey,
                dir,
                signal: SIGNAL,
                fetchFn,
                now: () => NOW,
                logger: l,
              },);
            }
            expect(await block({
              sourceText: '没有链接。',
              apiKey: 'whisker-key',
            },),).toBe('',);
            expect(await block({
              sourceText: SOURCE_TEXT,
              apiKey: '',
            },),).toBe('',);
            expect(seen.length,).toBe(0,);
            /**
             Block over the citing original.
             */
            const lines = await block({
              sourceText: SOURCE_TEXT,
              apiKey: 'whisker-key',
            },);
            expect(lines.split('\n',),).toEqual([
              `- reference 1 ${POST_URL} ("In memory of Mittens"): Mittens had an older sister. The sister was also a tabby.`,
              `- reference 2 ${ARCHIVE_URL} ("In memory of Mittens"): Mittens had an older sister. The sister was also a tabby.`,
            ],);
            expect(seen.length,).toBe(2,);
          },
        },),
        it({
          name: 'NAMES a page the transport could not read instead of dropping the line',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'reference-block-fail-', },);
            const dir = scratch.path;
            /**
             Transport that refuses everything.

             @returns Closed door
             */
            async function failing(): Promise<Response> {
              return new Response(
                'closed',
                { status: 503, },
              );
            }
            expect(await citedReferenceBlock({
              sourceText: `见 ${POST_URL}`,
              apiKey: 'whisker-key',
              dir,
              signal: SIGNAL,
              fetchFn: failing,
              now: () => NOW,
              logger: l,
            },),).toBe(`- reference 1 ${POST_URL}: could not be fetched`,);
          },
        },),
        it({
          name: 'LOGS A PAGE IT COULD NOT FETCH BY THE FAILURE\'S CLASS ALONE where the failure does not declare its '
            + 'message free of quoted text, so a contents body that is not JSON never reaches the log, where the '
            + 'parser\'s message quoted its opening',
          fn: async () => {
            const {
              logger,
              lines: logged,
            } = capturingLoggerPair();
            await using scratch = await scratchDir({ prefix: 'reference-block-prose-', },);
            /**
             Transport answering a body that is not JSON.

             @returns Prose where the endpoint writes JSON
             */
            async function proseAnswer(): Promise<Response> {
              return new Response(
                'Mittens chased the red dot down the hall',
                { status: 200, },
              );
            }
            expect(await citedReferenceBlock({
              sourceText: `见 ${POST_URL}`,
              apiKey: 'whisker-key',
              dir: scratch.path,
              signal: SIGNAL,
              fetchFn: proseAnswer,
              now: () => NOW,
              logger,
            },),).toBe(`- reference 1 ${POST_URL}: could not be fetched`,);
            expect(logged,).toEqual([
              `[citedReferenceBlock] reference 1 ${POST_URL} could not be fetched: refused by SyntaxError`,
              '[citedReferenceBlock] REFERENCES cited=1 cached=0 bought=1',
            ],);
          },
        },),
        it({
          name: 'LOGS A PAGE THE ENDPOINT REFUSED WITH THE HTTP STATUS and never the provider body the refusal '
            + 'answered with',
          fn: async () => {
            const {
              logger,
              lines: logged,
            } = capturingLoggerPair();
            await using scratch = await scratchDir({ prefix: 'reference-block-refused-', },);
            /**
             Transport refusing with cat prose in the body.

             @returns 429 whose body is prose
             */
            async function refusingWithProse(): Promise<Response> {
              return new Response(
                'Mittens chased the red dot down the hall',
                { status: 429, },
              );
            }
            expect(await citedReferenceBlock({
              sourceText: `见 ${POST_URL}`,
              apiKey: 'whisker-key',
              dir: scratch.path,
              signal: SIGNAL,
              fetchFn: refusingWithProse,
              now: () => NOW,
              logger,
            },),).toBe(`- reference 1 ${POST_URL}: could not be fetched`,);
            expect(logged,).toEqual([
              `[citedReferenceBlock] reference 1 ${POST_URL} could not be fetched: contents responded 429`,
              '[citedReferenceBlock] REFERENCES cited=1 cached=0 bought=1',
            ],);
          },
        },),
        it({
          name: 'REFUSES WITH THE CALLER\'S REASON once the caller stops the block, writing no unfetched page for a '
            + 'fetch that ended in that reason, and still writes a fetch\'s own failure that landed as the stop did',
          fn: async () => {
            const {
              logger,
              lines: logged,
            } = capturingLoggerPair();
            await using scratch = await scratchDir({ prefix: 'reference-block-stopped-', },);
            /**
             Second page the invented original cites.
             */
            const dotUrl = 'https://cats.example/posts/mittens-and-the-red-dot';
            /**
             The caller's steering, which the first page's fetch stops.
             */
            const steering = new AbortController();
            /**
             Why the caller stopped the block.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             Transport whose fetch of the first page stops the caller and then drops, and whose fetch of the
             second waits for the stop and rejects with the request signal's reason, as `fetch` does.

             @param _input - endpoint, unread

             @param init - request, whose body names the page and whose signal is the caller's

             @returns Never; every fetch rejects
             */
            async function stoppedTransport(_input: string | URL | Request, init?: RequestInit,): Promise<Response> {
              /**
               Body as sent, which names the page asked for.
               */
              const sent = init?.body;
              if (((typeof sent) === 'string') && sent.includes(POST_URL,)) {
                steering.abort(reason,);
                throw new TypeError('fetch failed',);
              }
              /**
               The request's signal, which `fetch` honours.
               */
              const signal = init?.signal ?? SIGNAL;
              await untilAborted({ signal, },);
              signal.throwIfAborted();
              throw new Error('unreachable: a fetch that waited for its abort was handed a signal that had not aborted',);
            }
            /**
             What the block refused with.
             */
            const refusal = await rejectionOf(async function readsUnderTheStop(): Promise<unknown> {
              return await citedReferenceBlock({
                sourceText: `见 ${POST_URL} 和 ${dotUrl}`,
                apiKey: 'whisker-key',
                dir: scratch.path,
                signal: steering.signal,
                fetchFn: stoppedTransport,
                now: () => NOW,
                logger,
              },);
            },);
            // The fetch that dropped has its line written before the block
            // refuses, since the block waits for every lookup to end.
            expect(refusal,).toBe(reason,);
            expect(logged,).toEqual([
              `[citedReferenceBlock] reference 1 ${POST_URL} could not be fetched: refused by TypeError`,
            ],);
          },
        },),
        it({
          name: 'READS a page the cache already holds without asking the transport, BUYS only the one it lacks, '
            + 'and says which was which on its lines',
          fn: async () => {
            const {
              logger,
              lines: logged,
            } = capturingLoggerPair();
            await using scratch = await scratchDir({ prefix: 'reference-block-held-', },);
            const dir = scratch.path;
            await writeCachedReference({
              dir,
              record: {
                url: POST_URL,
                fetchedAt: NOW.toISOString(),
                status: 'success',
                title: 'Already read',
                text: 'Mittens napped by the stove.',
              },
            },);
            const {
              fetchFn,
              seen,
            } = stubTransport({
              status: 200,
              body: READABLE_BODY,
            },);
            expect((await citedReferenceBlock({
              sourceText: SOURCE_TEXT,
              apiKey: 'whisker-key',
              dir,
              signal: SIGNAL,
              fetchFn,
              now: () => NOW,
              logger,
            },)).split('\n',),).toEqual([
              `- reference 1 ${POST_URL} ("Already read"): Mittens napped by the stove.`,
              `- reference 2 ${ARCHIVE_URL} ("In memory of Mittens"): Mittens had an older sister. The sister was also a tabby.`,
            ],);
            expect(seen.length,).toBe(1,);
            expect(JSON.parse(seen.at(0,)?.body ?? '{}',),).toEqual({
              urls: [ARCHIVE_URL,],
              text: { maxCharacters: REFERENCE_TEXT_CHARACTERS, },
            },);
            // In order of citation, whichever page's lookup ended first.
            expect(logged,).toEqual([
              `[citedReferenceBlock] REFERENCE 1 ${POST_URL}: success, 28 chars, cached`,
              `[citedReferenceBlock] REFERENCE 2 ${ARCHIVE_URL}: success, 57 chars, bought`,
              '[citedReferenceBlock] REFERENCES cited=2 cached=1 bought=1',
            ],);
          },
        },),
        it({
          name: 'WRITES EACH PAGE\'S LINE IN ORDER OF CITATION when a later page ends first, the earlier page\'s '
            + 'REFERENCE line before the later page\'s failure, and returns the block in that order',
          fn: async () => {
            const {
              logger,
              lines: logged,
            } = capturingLoggerPair();
            await using scratch = await scratchDir({ prefix: 'reference-block-later-first-', },);
            /**
             Second page the invented original cites, whose fetch is refused at once.
             */
            const dotUrl = 'https://cats.example/posts/mittens-and-the-red-dot';
            /**
             Opened once the second page's fetch has been refused, which the first page's fetch waits for.
             */
            const laterFailed = Promise.withResolvers<undefined>();
            /**
             Transport refusing the second page at once and answering the first once that refusal has run its course.

             @param _input - endpoint, unread

             @param init - request, whose body names the page

             @returns Refusal for the second page, readable body for the first
             */
            async function laterFirst(_input: string | URL | Request, init?: RequestInit,): Promise<Response> {
              /**
               Body as sent, which names the page asked for.
               */
              const sent = init?.body;
              if (((typeof sent) === 'string') && sent.includes(dotUrl,)) {
                laterFailed.resolve(undefined,);
                return new Response(
                  'closed',
                  { status: 503, },
                );
              }
              await laterFailed.promise;
              // Nothing but microtasks lies between the second page's failure
              // and its member's end, so a turn of the event loop later it has
              // ended.
              await turnOfTheLoop();
              return Response.json(
                READABLE_BODY,
                { status: 200, },
              );
            }
            expect((await citedReferenceBlock({
              sourceText: `见 ${POST_URL} 和 ${dotUrl}`,
              apiKey: 'whisker-key',
              dir: scratch.path,
              signal: SIGNAL,
              fetchFn: laterFirst,
              now: () => NOW,
              logger,
            },)).split('\n',),).toEqual([
              `- reference 1 ${POST_URL} ("In memory of Mittens"): Mittens had an older sister. The sister was also a tabby.`,
              `- reference 2 ${dotUrl}: could not be fetched`,
            ],);
            expect(logged,).toEqual([
              `[citedReferenceBlock] REFERENCE 1 ${POST_URL}: success, 57 chars, bought`,
              `[citedReferenceBlock] reference 2 ${dotUrl} could not be fetched: contents responded 503`,
              '[citedReferenceBlock] REFERENCES cited=2 cached=0 bought=2',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: citedReferenceBlockText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES the lines between fences with the rule, and nothing at all when the original links nowhere',
          fn: async () => {
            expect(citedReferenceBlockText({ fence: '=====', },),).toBe('',);
            expect(citedReferenceBlockText({
              fence: '=====',
              referenceContext: '',
            },),).toBe('',);
            /**
             Block over one line.
             */
            const block = citedReferenceBlockText({
              fence: '=====',
              referenceContext: `- reference 1 ${POST_URL}: Mittens had an older sister.`,
            },);
            expect(block,).toContain('===== CITED REFERENCES, EVIDENCE ONLY =====\n- reference 1',);
            expect(block,).toContain(`===== ${CITED_REFERENCE_RULE} =====\n`,);
            expect(CITED_REFERENCE_RULE,).toContain('never report or support it as accuracy/addition',);
            expect(CITED_REFERENCE_RULE,).toContain('never license adding to the TRANSLATION',);
          },
        },),
      ],
    },),

    describe({
      name: citedReferenceCandidateLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES the lines between fences with the candidate rule, and no lines when the original links nowhere',
          fn: async () => {
            expect(citedReferenceCandidateLines({ fence: '=====', },),).toEqual([],);
            expect(citedReferenceCandidateLines({
              fence: '=====',
              referenceContext: '',
            },),).toEqual([],);
            /**
             Lines over one reference.
             */
            const lines = citedReferenceCandidateLines({
              fence: '=====',
              referenceContext: `- reference 1 ${POST_URL}: Mittens had an older sister.`,
            },);
            expect(lines.at(0,),).toBe('===== CITED REFERENCES, EVIDENCE ONLY =====',);
            expect(lines.at(1,),).toContain('- reference 1',);
            expect(lines.at(2,),).toBe(`===== ${CITED_REFERENCE_CANDIDATE_RULE} =====`,);
            expect(lines.at(-1,),).toBe('',);
            expect(CITED_REFERENCE_CANDIDATE_RULE,).toContain('never count it unsupported',);
            expect(CITED_REFERENCE_CANDIDATE_RULE,).toContain('never license adding to a rendering',);
          },
        },),
      ],
    },),

    describe({
      name: citedReferenceEvidence.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RENDERS one labelled entry carrying the candidate rule, and none when the original links nowhere',
          fn: async () => {
            expect(citedReferenceEvidence({},),).toEqual([],);
            expect(citedReferenceEvidence({ referenceContext: '', },),).toEqual([],);
            /**
             Entries over one reference.
             */
            const entries = citedReferenceEvidence({
              referenceContext: `- reference 1 ${POST_URL}: Mittens had an older sister.`,
            },);
            expect(entries.length,).toBe(1,);
            expect(entries.at(0,)?.label,).toContain('CITED REFERENCES, EVIDENCE ONLY.',);
            expect(entries.at(0,)?.label,).toContain(CITED_REFERENCE_CANDIDATE_RULE,);
            expect(entries.at(0,)?.text,).toContain('- reference 1',);
          },
        },),
      ],
    },),
  ],
},);
