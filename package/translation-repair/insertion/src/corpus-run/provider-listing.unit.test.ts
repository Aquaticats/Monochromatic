/**
 Tests for the read of a provider's model listing both listing commands share.

 WHAT A LISTING READ OWES ITS CALLER: a body parsed only when the provider
 answered with a success, a refusal in the command's own words otherwise, and
 never a credential the request carried, since a provider's error body may
 echo the key it was sent. The transport is scripted, so nothing here reaches
 a provider, and the key is an invented stand-in.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { CREDENTIAL_MARKER, } from '@monochromatic-dev/translation-repair-provider-stream';
import { readProviderListing, } from '../../dist/final/node/index.mjs';
import { recordedTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts/recorded-transport.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Listing URL every case asks.
 */
const LISTING = 'https://cats.invalid/v1/models';

/**
 Invented stand-in key, never a real one.
 */
const KEY = 'whisker-key-7421';

/**
 Sentence the command refuses a bad status with.

 @param status - status the provider answered

 @returns Sentence naming the status

 @example
 ```ts
 const says = statusRefusal({ status: 404, },);
 ```
 */
function statusRefusal({ status, }: { readonly status: number; },): string {
  return `${LISTING} answered ${String(status,)}`;
}

/**
 Reads the listing over a transport replying once.

 @param status - status the scripted provider answers

 @param bodyText - body the scripted provider answers

 @returns What the read resolved to and the exchanges the transport saw

 @example
 ```ts
 const { body, } = await readReplying({ status: 200, bodyText: '{}', },);
 ```
 */
async function readReplying(
  {
    status,
    bodyText,
  }: {
    readonly status: number;
    readonly bodyText: string;
  },
): Promise<{
  readonly body: unknown;
  readonly exchanges: ReturnType<typeof recordedTransport>['exchanges'];
}> {
  const {
    transport,
    exchanges,
  } = recordedTransport({
    replies: [
      {
        status,
        bodyText,
      },
    ],
  },);
  const body = await readProviderListing({
    url: LISTING,
    apiKey: KEY,
    transport,
    timeoutMs: HANG_STOP_MS,
    statusRefusal,
  },);
  return {
    body,
    exchanges,
  };
}

await describe({
  name: readProviderListing.name,
  children: [
    it({
      name: 'RETURNS the parsed listing of a success and sends one GET carrying the key as a bearer token',
      fn: async () => {
        const {
          body,
          exchanges,
        } = await readReplying({
          status: 200,
          bodyText: '{"data":[{"id":"hf:cat/Mittens-1"}]}',
        },);

        expect(body,).toEqual({ data: [{ id: 'hf:cat/Mittens-1', },], },);
        expect(exchanges.length,).toBe(1,);
        expect(exchanges[0]?.url,).toBe(LISTING,);
        expect(exchanges[0]?.label,).toBe(LISTING,);
        expect(exchanges[0]?.method,).toBe('GET',);
        expect(exchanges[0]?.headers,).toEqual({ authorization: `Bearer ${KEY}`, },);
        expect(exchanges[0]?.bodyJson,).toBeUndefined();
        expect(exchanges[0]?.signal.aborted,).toBe(false,);
      },
    },),
    it({
      name: 'MASKS the key a success body repeats in another spelling before it is parsed, a JSON unicode escape '
        + 'and base64',
      fn: async () => {
        const { body, } = await readReplying({
          status: 200,
          bodyText: `{"data":[{"id":"whisker\\u002dkey\\u002d7421"},{"id":"${
            Buffer.from(`kit:${KEY}`,)
              .toString('base64url',)
          }"}]}`,
        },);

        // The base64 run's first characters hold bits of the four bytes before the key.
        expect(body,).toEqual({ data: [{ id: CREDENTIAL_MARKER, }, { id: `a2l0O${CREDENTIAL_MARKER}`, },], },);
      },
    },),
    it({
      name: 'READS the lowest and the highest success status as a listing',
      fn: async () => {
        const low = await readReplying({
          status: 200,
          bodyText: '[]',
        },);
        const high = await readReplying({
          status: 299,
          bodyText: '[]',
        },);

        expect([
          low.body,
          high.body,
        ],).toEqual([
          [],
          [],
        ],);
      },
    },),
    it({
      name: 'REFUSES in the command\'s words, with the status alone, a status below 200, at 300, at 404 or at 500',
      fn: async () => {
        for (const status of [
          199,
          300,
          404,
          500,
        ]) {
          // The body is the provider's wording, so it must not reach the refusal.
          /* oxlint-disable no-await-in-loop -- one status at a time, each case's refusal read before the next */
          const refusal = await rejectionOf(async function readFailing(): Promise<void> {
            await readReplying({
              status,
              bodyText: 'the cat is not here',
            },);
          },);
          /* oxlint-enable no-await-in-loop */

          expect(refusal,).toBeInstanceOf(StatedRefusalError,);
          expect(String(refusal,),).toBe(`StatedRefusalError: ${LISTING} answered ${String(status,)}`,);
        }
      },
    },),
    it({
      name: 'MASKS a key the provider echoes in a listing before the body is parsed, so the parsed values never hold it',
      fn: async () => {
        const { body, } = await readReplying({
          status: 200,
          bodyText: `{"data":[{"id":"${KEY}"}]}`,
        },);

        expect(body,).toEqual({ data: [{ id: CREDENTIAL_MARKER, },], },);
      },
    },),
    it({
      name: 'MASKS the token inside an echoed authorization value as well as the whole value',
      fn: async () => {
        const { body, } = await readReplying({
          status: 200,
          bodyText: `{"echo":"Bearer ${KEY}"}`,
        },);

        expect(body,).toEqual({ echo: CREDENTIAL_MARKER, },);
      },
    },),
    it({
      name: 'REFUSES a body that is not JSON as stated, without quoting the body or the key it echoes',
      fn: async () => {
        const failure = await rejectionOf(async function readEcho(): Promise<void> {
          await readReplying({
            status: 200,
            bodyText: `Unauthorized: ${KEY} is not a key`,
          },);
        },);

        expect(failure,).toBeInstanceOf(StatedRefusalError,);
        expect(String(failure,),).toBe(
          `StatedRefusalError: ${LISTING} answered with a body that is not JSON; the body is the provider's wording `
            + 'and is not repeated',
        );
      },
    },),
  ],
},);
