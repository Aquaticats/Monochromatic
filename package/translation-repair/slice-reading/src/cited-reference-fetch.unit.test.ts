/**
 Tests for reading the contents endpoint's answer for one cited reference.

 `fetchedOf` had no test before ledger B92. Its refusal of a body that is not
 an object names that check ("contents answered with a body that is not an
 object"); an array body reached a later, less precise refusal ("contents
 answered without a results array") while `isJsonRecord` admitted arrays, so
 the array case asserts the message, the only place the two differ.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CREDENTIAL_MARKER, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  CitedReferenceFetchError,
  EXA_CONTENTS_URL,
  fetchCitedReference,
  fetchedOf,
} from '../dist/final/node/index.mjs';
import {
  bodyCutBy,
  invalidHeaderRejection,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/body-cut-response.test-fixture.ts';
import { WHISKER_KEY, } from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';

/**
 What a request that failed before any answer came back says it can be.
 */
const TAIL = 'failed before any answer came back: either the network could not be reached, or the transport refused to send the request, as it does for a key or header holding a line break; check the connection and the key';

/**
 Abort signal that never fires.
 */
const SIGNAL = new AbortController().signal;

/**
 Reads the message a body raises, or empty when it raises none.

 @param parsed - candidate body under test

 @returns Refusal message, empty when none was raised

 @example
 ```ts
 const message = refusalOf({ parsed: 7, },);
 ```
 */
function refusalOf({ parsed, }: { readonly parsed: unknown; },): string {
  try {
    fetchedOf({ parsed, },);
    return '';
  } catch (refusal) {
    expect(refusal instanceof CitedReferenceFetchError,).toBe(true,);
    return String(refusal,);
  }
}

await describe({
  name: fetchedOf.name,
  children: [
    it({
      name: 'READS title and text off the first result, failure empty on success',
      fn: async () => {
        /**
         Body as the endpoint answers one url.
         */
        const parsed = {
          results: [{ title: 'The Lost Cat', text: 'Mittens wandered off at dusk.', },],
          statuses: [{ status: 'success', },],
        };
        expect(fetchedOf({ parsed, },),).toEqual({
          status: 'success',
          title: 'The Lost Cat',
          text: 'Mittens wandered off at dusk.',
        },);
      },
    },),

    it({
      name: 'REFUSES a body that is not an object, naming the check it failed exactly',
      fn: async () => {
        expect(refusalOf({ parsed: 7, },).includes(
          'contents answered with a body that is not an object',
        ),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES a body that is a JSON ARRAY with its own documented message, rather than '
        + 'reading past it into the results-array check and raising the less precise refusal that '
        + 'check names instead (ledger B92)',
      fn: async () => {
        expect(refusalOf({ parsed: ['stray',], },).includes(
          'contents answered with a body that is not an object',
        ),).toBe(true,);
      },
    },),

    it({
      name: 'READS a result as successful when `statuses` carries no entry at all, rather than reading '
        + 'past the empty array into a status record that is not there',
      fn: async () => {
        expect(fetchedOf({
          parsed: {
            results: [{ title: 'Clean Catch', text: 'The cat read quietly.', },],
            statuses: [],
          },
        },),).toEqual({
          status: 'success',
          title: 'Clean Catch',
          text: 'The cat read quietly.',
        },);
      },
    },),

    it({
      name: 'NAMES the failure the bare word "error" when the endpoint reports an error status with no '
        + 'error detail of its own',
      fn: async () => {
        expect(fetchedOf({
          parsed: {
            results: [],
            statuses: [{ status: 'error', },],
          },
        },),).toEqual({
          status: 'error',
          title: '',
          text: '',
          failure: 'error',
        },);
      },
    },),
    describe({
      name: fetchCitedReference.name,
      children: [
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
            const refusal = await rejectionOf(async function fetchesOverADroppedConnection(): Promise<unknown> {
              return await fetchCitedReference({
                apiKey: 'secret-key',
                url: 'https://cats.example/post',
                signal: SIGNAL,
                fetchFn: dropping,
              },);
            },);
            expect(refusal instanceof CitedReferenceFetchError,).toBe(true,);
            expect(String(refusal,),).toBe(
              `CitedReferenceFetchError: contents request to ${EXA_CONTENTS_URL} ${TAIL}`,
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
              return await fetchCitedReference({
                apiKey: 'secret-key',
                url: 'https://cats.example/post',
                signal: SIGNAL,
                fetchFn: refusingTheHeader,
              },);
            },);
            expect(String(refusal,),).toBe(`CitedReferenceFetchError: contents request to ${EXA_CONTENTS_URL} ${TAIL}`,);
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
            expect(await rejectionOf(async function fetchesAfterTheAbort(): Promise<unknown> {
              return await fetchCitedReference({
                apiKey: 'secret-key',
                url: 'https://cats.example/post',
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
              return await fetchCitedReference({
                apiKey: 'secret-key',
                url: 'https://cats.example/post',
                signal: SIGNAL,
                fetchFn: cutBody,
              },);
            },);
            expect({
              interrupted: String(interrupted,),
              cause: Error.isError(interrupted,) ? interrupted.cause : undefined,
            },).toEqual({
              interrupted: `CitedReferenceFetchError: contents lost the network while ${EXA_CONTENTS_URL} was answering`,
              cause: cut,
            },);
          },
        },),
        it({
          name: 'MASKS A SENT KEY THE ENDPOINT ECHOES before the body is parsed, in a title and in the page text, '
            + 'since a fetched reference goes into a sheet a model reads and into the reference cache on disk',
          fn: async () => {
            /**
             Transport answering with one page that repeats the key the request carried.

             @returns 200 whose page echoes the key twice
             */
            async function echoing(): Promise<Response> {
              return Response.json({
                results: [
                  {
                    title: `The Lost Cat of ${WHISKER_KEY}`,
                    text: `Mittens wandered off at dusk with ${WHISKER_KEY}.`,
                  },
                ],
                statuses: [{ status: 'success', },],
              },);
            }
            expect(
              await fetchCitedReference({
                apiKey: WHISKER_KEY,
                url: 'https://cats.example/post',
                signal: SIGNAL,
                fetchFn: echoing,
              },),
            ).toEqual({
              status: 'success',
              title: `The Lost Cat of ${CREDENTIAL_MARKER}`,
              text: `Mittens wandered off at dusk with ${CREDENTIAL_MARKER}.`,
            },);
          },
        },),
      ],
    },),
  ],
},);
