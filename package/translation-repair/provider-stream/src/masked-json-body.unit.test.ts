/**
 Tests for the read of an answer's body as JSON from its masked text.

 WHAT A LOOKUP READS IS KEPT, in a sheet a model reads and in a cache on disk,
 so a key the endpoint echoes must be gone before anything is parsed. The key
 here is an invented one, never one from the environment.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CREDENTIAL_MARKER,
  readMaskedJsonBody,
} from '../dist/final/node/index.mjs';
import { bodyCutBy, } from '@monochromatic-dev/translation-repair-test-fixture/ts/body-cut-response.test-fixture.ts';
import { WHISKER_KEY, } from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';

await describe({
  name: readMaskedJsonBody.name,
  children: [
    it({
      name: 'MASKS the key of an `x-api-key` header and the token of a bearer value wherever the body repeats them, '
        + 'in a string, in a key of an object and inside a longer word',
      fn: async () => {
        expect(
          await readMaskedJsonBody({
            response: Response.json({
              title: `Mittens and ${WHISKER_KEY}`,
              [`named-${WHISKER_KEY}`]: [`x${WHISKER_KEY}y`,],
            },),
            headers: { 'x-api-key': WHISKER_KEY, },
          },),
        ).toEqual({
          title: `Mittens and ${CREDENTIAL_MARKER}`,
          [`named-${CREDENTIAL_MARKER}`]: [`x${CREDENTIAL_MARKER}y`,],
        },);
        expect(
          await readMaskedJsonBody({
            response: Response.json({ said: `Bearer ${WHISKER_KEY}`, again: WHISKER_KEY, },),
            headers: { Authorization: `Bearer ${WHISKER_KEY}`, },
          },),
        ).toEqual({
          said: CREDENTIAL_MARKER,
          again: CREDENTIAL_MARKER,
        },);
      },
    },),
    it({
      name: 'MASKS a key the endpoint echoes in another spelling: a JSON unicode escape, a percent escape and base64',
      fn: async () => {
        expect(
          await readMaskedJsonBody({
            response: new Response(
              `{"a":"whisker\\u002dkey\\u002d7421","b":"x?k=whisker%2Dkey%2D7421","c":"${
                Buffer.from(WHISKER_KEY,)
                  .toString('base64',)
              }"}`,
            ),
            headers: { 'x-api-key': WHISKER_KEY, },
          },),
        ).toEqual({
          a: CREDENTIAL_MARKER,
          b: `x?k=${CREDENTIAL_MARKER}`,
          c: CREDENTIAL_MARKER,
        },);
      },
    },),
    it({
      name: 'READS a body that repeats no credential as it was sent, and one whose request carried no credential '
        + 'header at all',
      fn: async () => {
        expect(
          await readMaskedJsonBody({
            response: Response.json({ results: [{ title: 'The Lost Cat', },], },),
            headers: { 'x-api-key': WHISKER_KEY, },
          },),
        ).toEqual({ results: [{ title: 'The Lost Cat', },], },);
        expect(
          await readMaskedJsonBody({
            response: Response.json({ title: `Mittens and ${WHISKER_KEY}`, },),
            headers: { accept: 'application/json', },
          },),
        ).toEqual({ title: `Mittens and ${WHISKER_KEY}`, },);
      },
    },),
    it({
      name: 'PASSES ON the parser\'s own failure for a body that is not JSON, whose message quotes the masked text '
        + 'and never the key',
      fn: async () => {
        /**
         What the parser raised for a body that is prose.
         */
        const refused = await rejectionOf(async function readsProse(): Promise<unknown> {
          return await readMaskedJsonBody({
            response: new Response(`${WHISKER_KEY} is no JSON`,),
            headers: { 'x-api-key': WHISKER_KEY, },
          },);
        },);
        expect({
          isSyntaxError: refused instanceof SyntaxError,
          namesTheKey: String(refused,).includes(WHISKER_KEY,),
        },).toEqual({
          isSyntaxError: true,
          namesTheKey: false,
        },);
      },
    },),
    it({
      name: 'PASSES ON what the body stream raises while it arrives, unchanged, for the caller to name the endpoint',
      fn: async () => {
        /**
         What the connection fails with while the body streams.
         */
        const cut = new TypeError('terminated',);
        expect(
          await rejectionOf(async function readsACutBody(): Promise<unknown> {
            return await readMaskedJsonBody({
              response: bodyCutBy({ failure: cut, },),
              headers: { 'x-api-key': WHISKER_KEY, },
            },);
          },),
        ).toBe(cut,);
      },
    },),
  ],
},);
