/**
 Tests the cached web lookup of official English titles.

 THE TRANSPORT IS A STUB: the real endpoint was exercised once by hand on
 2026-09-02 (《活着》 answered "To Live", 《魔法少女小圆》 answered "Puella Magi
 Madoka Magica", about 1.5 s and $0.007 a query) and these tests cover what
 the module does around it: which titles are asked, how a record is cached
 and read back, how hits become lines, and what a failure or a missing key
 leaves behind.

 @module
 */

import { setImmediate as turnOfTheLoop, } from 'node:timers/promises';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CREDENTIAL_MARKER, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  EXA_SEARCH_URL,
  isLookupRecord,
  lookupCacheDir,
  lookupLinesOf,
  lookupQueryFor,
  type LookupRecord,
  lookupWorkTitle,
  packageCacheDir,
  readCachedLookup,
  searchWorkTitle,
  WorkTitleLookupError,
  workTitleLookupLines,
  workTitlesOf,
  writeCachedLookup,
} from '../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  bodyCutBy,
  invalidHeaderRejection,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/body-cut-response.test-fixture.ts';
import { WHISKER_KEY, } from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { untilAborted, } from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';

/**
 What a request that failed before any answer came back says it can be.
 */
const TAIL = 'failed before any answer came back: either the network could not be reached, or the transport refused to send the request, as it does for a key or header holding a line break; check the connection and the key';

/**
 Abort signal that never fires.
 */
const SIGNAL = new AbortController().signal;

/**
 Fixed clock.
 */
const NOW = new Date('2026-09-02T10:00:00.000Z',);

/**
 Logger for the lines function.
 */
const l = tagged({ tag: 'work-title-lookup-test', },);

/**
 One call a stub transport saw.
 */
type SeenCall = {
  readonly url: string;
  readonly init: RequestInit;
};

/**
 Where a fetch input points, whatever form it takes.

 @param input - first argument of `fetch`

 @returns Its url as text

 @example
 ```ts
 urlOf({ input: 'https://x', },);
 ```
 */
function urlOf({ input, }: { readonly input: string | URL | Request; },): string {
  if ((typeof input) === 'string')
    return input;
  if (input instanceof URL)
    return input.href;
  return input.url;
}

/**
 Builds a transport answering a fixed body and counting calls.

 @param status - HTTP status to answer

 @param body - JSON body to answer

 @returns Transport plus the calls it saw

 @example
 ```ts
 const { fetchFn, calls, } = stubFetch({ status: 200, body: { results: [], }, },);
 ```
 */
function stubFetch(
  {
    status,
    body,
  }: {
    readonly status: number;
    readonly body: unknown;
  },
): {
  readonly fetchFn: typeof fetch;
  readonly calls: SeenCall[];
} {
  /**
   Calls seen.
   */
  const calls: SeenCall[] = [];
  return {
    calls,
    fetchFn: async function fetchFn(input, init,): Promise<Response> {
      calls.push({
        url: urlOf({ input, },),
        init: init ?? {},
      },);
      return Response.json(
        body,
        { status, },
      );
    },
  };
}

/**
 One endpoint result as the reference describes it.
 */
const TO_LIVE_RESULT = {
  title: 'To Live (novel) - Wikipedia',
  url: 'https://en.wikipedia.org/wiki/To_Live_(novel)',
  highlights: ['To Live (活着) is a novel\nby Yu Hua.',],
  highlightScores: [0.9,],
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: workTitlesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS every 《…》 span once, in order, marks included, and stops at an unclosed mark',
          fn: async () => {
            expect(workTitlesOf({ text: '她读《活着》，又读《活着》，还读《不安》。《未完', },),)
              .toEqual(['《活着》', '《不安》',],);
            expect(workTitlesOf({ text: '没有书名号。', },),).toEqual([],);
            expect(lookupQueryFor({ title: '《活着》', },),).toBe('《活着》 official English title',);
          },
        },),
        it({
          name: 'READS AN OPENING MARK THAT NEVER CLOSED AS NO TITLE (ledger B38), and looks up the title after it',
          fn: async () => {
            expect(workTitlesOf({ text: '她读《猫，又读《猫经》。', },),).toEqual(['《猫经》',],);
          },
        },),
      ],
    },),

    describe({
      name: lookupCacheDir.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PREFERS the override, then XDG_CACHE_HOME, then the home cache directory',
          fn: async () => {
            expect(lookupCacheDir({
              env: {
                TRANSLATION_REPAIR_LOOKUP_CACHE_DIR: '/tmp/x',
                XDG_CACHE_HOME: '/tmp/y',
              },
            },),).toBe('/tmp/x',);
            expect(lookupCacheDir({ env: { XDG_CACHE_HOME: '/tmp/y', }, },),).toBe('/tmp/y/translation-repair/lookup',);
            expect(lookupCacheDir({ env: {}, },),).toContain('/.cache/translation-repair/lookup',);
          },
        },),
      ],
    },),

    describe({
      name: packageCacheDir.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES translation-repair UNDER XDG_CACHE_HOME, and under the home cache directory when it is unset or empty',
          fn: async () => {
            expect(packageCacheDir({ env: { XDG_CACHE_HOME: '/tmp/y', }, },),).toBe('/tmp/y/translation-repair',);
            expect(packageCacheDir({ env: {}, },).endsWith('/.cache/translation-repair',),).toBe(true,);
            expect(packageCacheDir({ env: { XDG_CACHE_HOME: '', }, },),).toBe(packageCacheDir({ env: {}, },),);
          },
        },),
      ],
    },),

    describe({
      name: lookupWorkTitle.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'BUYS a title once and READS IT BACK from the cache on every later ask, so a resumed run '
            + 'sees the same lines',
          fn: async () => {
            /**
             Empty cache.
             */
            await using scratch = await scratchDir({ prefix: 'work-title-lookup-', },);
            const dir = scratch.path;
            /**
             Transport answering one hit.
             */
            const { fetchFn, calls, } = stubFetch({
              status: 200,
              body: { results: [TO_LIVE_RESULT,], },
            },);
            /**
             Shared arguments.
             */
            const ask = {
              title: '《活着》',
              apiKey: 'test-key',
              dir,
              signal: SIGNAL,
              fetchFn,
              now: () => NOW,
            };
            /**
             First ask, bought.
             */
            const first = await lookupWorkTitle(ask,);
            /**
             Second ask, read back.
             */
            const second = await lookupWorkTitle(ask,);
            expect(calls.length,).toBe(1,);
            expect(first,).toEqual({
              query: '《活着》 official English title',
              fetchedAt: NOW.toISOString(),
              hits: [{
                title: TO_LIVE_RESULT.title,
                url: TO_LIVE_RESULT.url,
                highlight: 'To Live (活着) is a novel\nby Yu Hua.',
              },],
            },);
            expect(second,).toEqual(first,);
            expect(await readCachedLookup({
              dir,
              query: first.query,
            },),).toEqual({
              kind: 'hit',
              record: first,
            },);
            expect(isLookupRecord(first,),).toBe(true,);
            expect(isLookupRecord({ query: 1, },),).toBe(false,);

            /**
             What the transport was sent.
             */
            const [call,] = calls;
            if (call === undefined)
              throw new Error('the transport saw no call',);
            /**
             Headers as sent.
             */
            const headers = call.init.headers as Record<string, string>;
            /**
             Body as sent.
             */
            const { body, } = call.init;
            if ((typeof body) !== 'string')
              throw new Error('the request body was not text',);
            expect(call.url,).toBe(EXA_SEARCH_URL,);
            expect(headers['x-api-key'],).toBe('test-key',);
            expect(JSON.parse(body,),).toEqual({
              query: '《活着》 official English title',
              type: 'auto',
              numResults: 5,
              contents: {
                highlights: {
                  query: '《活着》 official English title',
                  maxCharacters: 300,
                },
              },
            },);
          },
        },),

        it({
          name: 'REFUSES a non-2xx answer, a body that is not an object and a body without results, naming the HTTP '
            + 'status or the check and never the key, the query or the provider body',
          fn: async () => {
            /**
             Refusing transport.
             */
            const refused = stubFetch({
              status: 401,
              body: { error: 'bad key', },
            },);
            /**
             What the refusal threw.
             */
            let thrown: unknown;
            try {
              await searchWorkTitle({
                apiKey: 'secret-key',
                query: '《活着》 official English title',
                signal: SIGNAL,
                fetchFn: refused.fetchFn,
              },);
            } catch (error) {
              thrown = error;
            }
            expect(thrown instanceof WorkTitleLookupError,).toBe(true,);
            expect(String(thrown,),).toBe('WorkTitleLookupError: search responded 401',);

            /**
             Shapeless transport.
             */
            const shapeless = stubFetch({
              status: 200,
              body: { nothing: true, },
            },);
            /**
             What the shapeless answer threw.
             */
            let thrownShapeless: unknown;
            try {
              await searchWorkTitle({
                apiKey: 'secret-key',
                query: '《活着》 official English title',
                signal: SIGNAL,
                fetchFn: shapeless.fetchFn,
              },);
            } catch (error) {
              thrownShapeless = error;
            }
            expect(thrownShapeless instanceof WorkTitleLookupError,).toBe(true,);
            expect(String(thrownShapeless,),).toBe('WorkTitleLookupError: search answered without a results array',);

            /**
             Transport answering a JSON array, which read as no results before
             `isJsonRecord` refused arrays (ledger B92).
             */
            const listed = stubFetch({
              status: 200,
              body: [{ title: 'Whiskers at Dusk', },],
            },);
            /**
             What the array answer threw.
             */
            let thrownListed: unknown;
            try {
              await searchWorkTitle({
                apiKey: 'secret-key',
                query: '《活着》 official English title',
                signal: SIGNAL,
                fetchFn: listed.fetchFn,
              },);
            } catch (error) {
              thrownListed = error;
            }
            expect(thrownListed instanceof WorkTitleLookupError,).toBe(true,);
            expect(String(thrownListed,),).toBe('WorkTitleLookupError: search answered with a body that is not an object',);
          },
        },),

        it({
          name: 'REFUSES a transport that rejects, naming the endpoint and the two things a rejection before any answer '
            + 'can be, a network failure or a request the transport refuses, and keeps the failure as the refusal\'s cause without repeating its message',
          fn: async () => {
            /**
             What a dropped connection rejects with.
             */
            const failure = new TypeError('fetch failed',);
            /**
             Transport whose connection drops.

             @returns Never; it rejects
             */
            async function dropping(): Promise<Response> {
              throw failure;
            }
            /**
             What the rejection became.
             */
            const refusal = await rejectionOf(async function searchesOverADroppedConnection(): Promise<unknown> {
              return await searchWorkTitle({
                apiKey: 'secret-key',
                query: '《活着》 official English title',
                signal: SIGNAL,
                fetchFn: dropping,
              },);
            },);
            expect(refusal instanceof WorkTitleLookupError,).toBe(true,);
            expect(String(refusal,),).toBe(
              `WorkTitleLookupError: search request to ${EXA_SEARCH_URL} ${TAIL}`,
            );
            expect(Error.isError(refusal,) ? refusal.cause : undefined,).toBe(failure,);
          },
        },),

        it({
          name: 'REFUSES a request the transport will not send, such as a header value holding a line break, in '
            + 'the same words as a network failure and without quoting the value',
          fn: async () => {
            /**
             Transport refusing a header as the runtime's `fetch` does.

             @returns Never; it rejects
             */
            async function refusingTheHeader(): Promise<Response> {
              throw invalidHeaderRejection();
            }
            /**
             What the rejection became.
             */
            const refusal = await rejectionOf(async function sendsABadHeader(): Promise<unknown> {
              return await searchWorkTitle({
                apiKey: 'secret-key',
                query: '《活着》 official English title',
                signal: SIGNAL,
                fetchFn: refusingTheHeader,
              },);
            },);
            expect(String(refusal,),).toBe(`WorkTitleLookupError: search request to ${EXA_SEARCH_URL} ${TAIL}`,);
          },
        },),

        it({
          name: 'LEAVES the caller\'s abort an abort: a transport that rejects with the signal\'s reason is not '
            + 'reported as a network failure',
          fn: async () => {
            /**
             Why the caller stopped.
             */
            const reason = new DOMException('the caller stopped waiting', 'AbortError',);
            /**
             The caller's abort, already fired.
             */
            const stopped = new AbortController();
            stopped.abort(reason,);
            /**
             Transport rejecting as `fetch` does for an aborted signal.

             @returns Never; it rejects with the signal's reason
             */
            async function abortedTransport(): Promise<Response> {
              throw stopped.signal.reason;
            }
            expect(await rejectionOf(async function searchesAfterTheAbort(): Promise<unknown> {
              return await searchWorkTitle({
                apiKey: 'secret-key',
                query: '《活着》 official English title',
                signal: stopped.signal,
                fetchFn: abortedTransport,
              },);
            },),).toBe(reason,);
          },
        },),

        it({
          name: 'REFUSES an answer whose connection fails while the body is read, saying the network failed while the '
            + 'endpoint was answering, with the failure as its cause',
          fn: async () => {
            /**
             What the connection fails with while the body streams.
             */
            const cut = new TypeError('terminated',);
            /**
             Transport answering 200 and then failing mid-body.

             @returns 200 whose body stream errors
             */
            async function cutBody(): Promise<Response> {
              return bodyCutBy({ failure: cut, },);
            }
            /**
             Refusal for the cut body.
             */
            const interrupted = await rejectionOf(async function readsACutBody(): Promise<unknown> {
              return await searchWorkTitle({
                apiKey: 'secret-key',
                query: '《活着》 official English title',
                signal: SIGNAL,
                fetchFn: cutBody,
              },);
            },);
            expect({
              interrupted: String(interrupted,),
              cause: Error.isError(interrupted,) ? interrupted.cause : undefined,
            },).toEqual({
              interrupted: `WorkTitleLookupError: search lost the network while ${EXA_SEARCH_URL} was answering`,
              cause: cut,
            },);
          },
        },),
        it({
          name: 'MASKS A SENT KEY THE ENDPOINT ECHOES before the body is parsed, in a title, an address and a '
            + 'highlight, since a hit goes into a sheet a model reads and into the lookup cache on disk',
          fn: async () => {
            /**
             Transport answering with one result that repeats the key the request carried.

             @returns 200 whose result echoes the key three times
             */
            async function echoing(): Promise<Response> {
              return Response.json({
                results: [
                  {
                    title: `Whiskers of ${WHISKER_KEY}`,
                    url: `https://cats.example/${WHISKER_KEY}`,
                    highlights: [`the key ${WHISKER_KEY} again`,],
                  },
                ],
              },);
            }
            expect(
              await searchWorkTitle({
                apiKey: WHISKER_KEY,
                query: '《活着》 official English title',
                signal: SIGNAL,
                fetchFn: echoing,
              },),
            ).toEqual([
              {
                title: `Whiskers of ${CREDENTIAL_MARKER}`,
                url: `https://cats.example/${CREDENTIAL_MARKER}`,
                highlight: `the key ${CREDENTIAL_MARKER} again`,
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: workTitleLookupLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RENDERS one line per hit with the highlight folded, WARNS on a result that never names the '
            + 'work and lists it after the ones that do (the Toka_ls rerun of 2026-09-02 renamed 《奇妙漂流》 '
            + '"Flow" off five neighbour results), says so when nothing was found, and contributes NO LINE '
            + 'for a failed lookup while still rendering the others',
          fn: async () => {
            /**
             Empty cache.
             */
            await using scratch = await scratchDir({ prefix: 'work-title-lookup-', },);
            const dir = scratch.path;
            /**
             Record already cached for one title, with no hits.
             */
            const empty: LookupRecord = {
              query: lookupQueryFor({ title: '《不安》', },),
              fetchedAt: NOW.toISOString(),
              hits: [],
            };
            await writeCachedLookup({
              dir,
              record: empty,
            },);
            /**
             Transport answering the other title.
             */
            const { fetchFn, calls, } = stubFetch({
              status: 200,
              body: {
                results: [
                  {
                    title: '喵的奇幻漂流',
                    url: 'https://example.invalid/neighbour',
                    highlights: ['A 2024 Latvian animated film.',],
                  },
                  TO_LIVE_RESULT,
                  { url: 'https://example.invalid/none', },
                ],
              },
            },);
            expect(await workTitleLookupLines({
              sourceText: '读《活着》和《不安》。',
              apiKey: 'test-key',
              dir,
              signal: SIGNAL,
              fetchFn,
              now: () => NOW,
              logger: l,
            },),).toEqual([
              '- web lookup for 《活着》: "To Live (novel) - Wikipedia" https://en.wikipedia.org/wiki/To_Live_(novel): To Live (活着) is a novel by Yu Hua.',
              '- web lookup for 《活着》 (this result does NOT name the work asked about; it is a neighbour, not its title): "喵的奇幻漂流" https://example.invalid/neighbour: A 2024 Latvian animated film.',
              '- web lookup for 《活着》 (this result does NOT name the work asked about; it is a neighbour, not its title): "" https://example.invalid/none',
              '- web lookup for 《不安》: nothing found',
            ],);
            expect(calls.length,).toBe(1,);
            expect(lookupLinesOf({
              title: '《不安》',
              record: empty,
            },),).toEqual(['- web lookup for 《不安》: nothing found',],);

            /**
             Refusing transport over a fresh cache.
             */
            const refused = stubFetch({
              status: 500,
              body: {},
            },);
            await using freshScratch = await scratchDir({ prefix: 'work-title-lookup-', },);
            expect(await workTitleLookupLines({
              sourceText: '读《活着》。',
              apiKey: 'test-key',
              dir: freshScratch.path,
              signal: SIGNAL,
              fetchFn: refused.fetchFn,
              now: () => NOW,
              logger: l,
            },),).toEqual([],);
          },
        },),

        it({
          name: 'READS A LATIN TITLE AS WORDS (ledger B23): a result whose title only runs on from the work\'s is a '
            + 'neighbour, and is warned about and listed after the one naming it',
          fn: async () => {
            /**
             Record for a Latin title, a neighbour first.
             */
            const record: LookupRecord = {
              query: lookupQueryFor({ title: '《Catcraft》', },),
              fetchedAt: NOW.toISOString(),
              hits: [
                {
                  title: 'Catcraftopia fan wiki',
                  url: 'https://example.invalid/fan',
                  highlight: '',
                },
                {
                  title: 'Catcraft (game)',
                  url: 'https://example.invalid/game',
                  highlight: '',
                },
              ],
            };
            expect(lookupLinesOf({
              title: '《Catcraft》',
              record,
            },),).toEqual([
              '- web lookup for 《Catcraft》: "Catcraft (game)" https://example.invalid/game',
              '- web lookup for 《Catcraft》 (this result does NOT name the work asked about; it is a neighbour, not its title): "Catcraftopia fan wiki" https://example.invalid/fan',
            ],);
          },
        },),
        it({
          name: 'ASKS NOTHING without a key or without a title, so a run without the secret changes nothing '
            + 'but a log line',
          fn: async () => {
            /**
             Transport that must not be called.
             */
            const { fetchFn, calls, } = stubFetch({
              status: 200,
              body: { results: [], },
            },);
            await using firstScratch = await scratchDir({ prefix: 'work-title-lookup-', },);
            expect(await workTitleLookupLines({
              sourceText: '读《活着》。',
              apiKey: '',
              dir: firstScratch.path,
              signal: SIGNAL,
              fetchFn,
              now: () => NOW,
              logger: l,
            },),).toEqual([],);
            await using secondScratch = await scratchDir({ prefix: 'work-title-lookup-', },);
            expect(await workTitleLookupLines({
              sourceText: '没有书名号。',
              apiKey: 'test-key',
              dir: secondScratch.path,
              signal: SIGNAL,
              fetchFn,
              now: () => NOW,
              logger: l,
            },),).toEqual([],);
            expect(calls.length,).toBe(0,);
          },
        },),
        it({
          name: 'LOGS A FAILED LOOKUP BY ITS CLASS ALONE where the failure does not declare its message free of quoted '
            + 'text, so a search body that is not JSON never reaches the log, where the parser\'s message quoted '
            + 'its opening',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'work-title-lookup-', },);
            const {
              logger,
              lines: logged,
            } = capturingLoggerPair();
            /**
             Transport answering a body that is not JSON.

             @returns Prose where the endpoint writes JSON
             */
            async function proseBody(): Promise<Response> {
              return new Response(
                'Pepper purred on the warm windowsill all afternoon',
                { status: 200, },
              );
            }
            expect(await workTitleLookupLines({
              sourceText: '她读《猫的午睡》。',
              apiKey: 'test-key',
              dir: scratch.path,
              signal: SIGNAL,
              fetchFn: proseBody,
              now: () => NOW,
              logger,
            },),).toEqual([],);
            expect(logged,).toEqual([
              '[workTitleLookupLines] lookup for 《猫的午睡》 failed and contributes no line: refused by SyntaxError',
              '[workTitleLookupLines] 1 work title looked up, 0 lines',
            ],);
          },
        },),
        it({
          name: 'LOGS A REFUSED SEARCH WITH ITS HTTP STATUS and never the provider body the refusal answered with',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'work-title-lookup-', },);
            const {
              logger,
              lines: logged,
            } = capturingLoggerPair();
            /**
             Transport refusing with cat prose in the body.

             @returns 401 whose body is prose
             */
            async function refusingWithProse(): Promise<Response> {
              return new Response(
                'Pepper purred on the warm windowsill all afternoon',
                { status: 401, },
              );
            }
            expect(await workTitleLookupLines({
              sourceText: '她读《猫的午睡》。',
              apiKey: 'test-key',
              dir: scratch.path,
              signal: SIGNAL,
              fetchFn: refusingWithProse,
              now: () => NOW,
              logger,
            },),).toEqual([],);
            expect(logged,).toEqual([
              '[workTitleLookupLines] lookup for 《猫的午睡》 failed and contributes no line: search responded 401',
              '[workTitleLookupLines] 1 work title looked up, 0 lines',
            ],);
          },
        },),
        it({
          name: 'LOGS A TRANSPORT THAT REJECTS as the request to the endpoint failing before any answer, in the '
            + 'refusal\'s own words, where it logged the class of the failure alone, and contributes no line',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'work-title-lookup-', },);
            const {
              logger,
              lines: logged,
            } = capturingLoggerPair();
            /**
             Transport whose connection drops.

             @returns Never; it rejects
             */
            async function dropping(): Promise<Response> {
              throw new TypeError('fetch failed',);
            }
            expect(await workTitleLookupLines({
              sourceText: '她读《猫的午睡》。',
              apiKey: 'test-key',
              dir: scratch.path,
              signal: SIGNAL,
              fetchFn: dropping,
              now: () => NOW,
              logger,
            },),).toEqual([],);
            expect(logged,).toEqual([
              `[workTitleLookupLines] lookup for 《猫的午睡》 failed and contributes no line: search request to ${EXA_SEARCH_URL} ${TAIL}`,
              '[workTitleLookupLines] 1 work title looked up, 0 lines',
            ],);
          },
        },),
        it({
          name: 'WRITES EACH FAILED LOOKUP IN TITLE ORDER when a later title\'s search fails first, the earlier '
            + 'title\'s line before the later title\'s',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'work-title-lookup-', },);
            const {
              logger,
              lines: logged,
            } = capturingLoggerPair();
            /**
             Opened once the later title's search has failed, which the earlier title's search waits for.
             */
            const laterFailed = Promise.withResolvers<undefined>();
            /**
             Transport dropping the search for the supper at once and refusing the search for the nap once that
             failure has run its course.

             @param _input - endpoint, unread

             @param init - request, whose body names the title

             @returns Refusal, for the nap
             */
            async function laterFirst(_input: string | URL | Request, init?: RequestInit,): Promise<Response> {
              /**
               Body as sent, which names the title asked about.
               */
              const sent = init?.body;
              if (((typeof sent) === 'string') && sent.includes('猫的晚餐',)) {
                laterFailed.resolve(undefined,);
                throw new TypeError('fetch failed',);
              }
              await laterFailed.promise;
              // Nothing but microtasks lies between the later search's failure
              // and its member's end, so a turn of the event loop later it has
              // ended.
              await turnOfTheLoop();
              return new Response(
                'the cat is not here',
                { status: 401, },
              );
            }
            expect(await workTitleLookupLines({
              sourceText: '她读《猫的午睡》，又读《猫的晚餐》。',
              apiKey: 'test-key',
              dir: scratch.path,
              signal: SIGNAL,
              fetchFn: laterFirst,
              now: () => NOW,
              logger,
            },),).toEqual([],);
            expect(logged,).toEqual([
              '[workTitleLookupLines] lookup for 《猫的午睡》 failed and contributes no line: search responded 401',
              `[workTitleLookupLines] lookup for 《猫的晚餐》 failed and contributes no line: search request to ${EXA_SEARCH_URL} ${TAIL}`,
              '[workTitleLookupLines] 2 work titles looked up, 0 lines',
            ],);
          },
        },),
        it({
          name: 'REFUSES WITH THE CALLER\'S REASON once the caller stops the lookups, writing no failed lookup for a '
            + 'search that ended in that reason, and still writes a search\'s own failure that landed as the stop did',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'work-title-lookup-', },);
            const {
              logger,
              lines: logged,
            } = capturingLoggerPair();
            /**
             The caller's steering, which the first title's search stops.
             */
            const steering = new AbortController();
            /**
             Why the caller stopped the lookups.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             Transport whose search for the nap stops the caller and then drops, and whose search for the
             supper waits for the stop and rejects with the request signal's reason, as `fetch` does.

             @param _input - endpoint, unread

             @param init - request, whose body names the title and whose signal is the caller's

             @returns Never; every search rejects
             */
            async function stoppedTransport(_input: string | URL | Request, init?: RequestInit,): Promise<Response> {
              /**
               Body as sent, which names the title asked about.
               */
              const sent = init?.body;
              if (((typeof sent) === 'string') && sent.includes('猫的午睡',)) {
                steering.abort(reason,);
                throw new TypeError('fetch failed',);
              }
              /**
               The request's signal, which `fetch` honours.
               */
              const signal = init?.signal ?? SIGNAL;
              await untilAborted({ signal, },);
              signal.throwIfAborted();
              throw new Error('unreachable: a search that waited for its abort was handed a signal that had not aborted',);
            }
            /**
             What the lookups refused with.
             */
            const refusal = await rejectionOf(async function looksUpUnderTheStop(): Promise<unknown> {
              return await workTitleLookupLines({
                sourceText: '她读《猫的午睡》，又读《猫的晚餐》。',
                apiKey: 'test-key',
                dir: scratch.path,
                signal: steering.signal,
                fetchFn: stoppedTransport,
                now: () => NOW,
                logger,
              },);
            },);
            // The search that dropped has its line written before the lookups
            // refuse, since they wait for every lookup to end.
            expect(refusal,).toBe(reason,);
            expect(logged,).toEqual([
              '[workTitleLookupLines] lookup for 《猫的午睡》 failed and contributes no line: refused by TypeError',
            ],);
          },
        },),
      ],
    },),
  ],
},);
