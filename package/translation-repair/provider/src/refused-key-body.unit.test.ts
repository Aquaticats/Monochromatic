/**
 Tests for the transport guard that drops the words of a refused key's 401 (`refused-key-body.ts`), shared by every
 route that sends a key: a provider's refusal of a key may echo a fragment of it, and the status alone says what the
 router and an operator act on. Fixtures are cat-themed invention, and the keys are invented.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type {
  ModelTransport,
  TransportExchange,
  TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';
import { withoutRefusedKeyBody, } from '../dist/final/node/index.mjs';

/**
 Exchange the cases send over the guard.
 */
const EXCHANGE: TransportExchange = {
  url: 'https://example.invalid/v1/messages',
  label: 'cat',
  method: 'POST',
  headers: {},
  bodyJson: '{}',
  signal: new AbortController().signal,
};

/**
 Transport answering one fixed reply.

 @param reply - what the wire answers

 @returns Transport that gives it back for every exchange

 @example
 ```ts
 const transport = answeringWith({ reply: { status: 200, bodyText: 'purr', }, },);
 ```
 */
function answeringWith({ reply, }: { readonly reply: TransportReply; },): ModelTransport {
  return function answer(): Promise<TransportReply> {
    return Promise.resolve(reply,);
  };
}

await describe({
  name: withoutRefusedKeyBody.name,
  children: [
    it({
      name: 'EMPTIES THE BODY OF A 401, which may echo the tail of the key, and keeps its status and headers',
      fn: async () => {
        const guarded = withoutRefusedKeyBody({
          transport: answeringWith({
            reply: { status: 401, bodyText: 'invalid x-api-key ****Qx4d', headers: { 'x-cat': 'grey', }, },
          },),
        },);
        expect(await guarded(EXCHANGE,),).toEqual({ status: 401, bodyText: '', headers: { 'x-cat': 'grey', }, },);
      },
    },),

    it({
      name: 'PASSES EVERY OTHER REPLY THROUGH UNCHANGED, a refusal for another reason included',
      fn: async () => {
        await Promise.all([200, 400, 402, 403, 429, 529,].map(async function passes(status,) {
          /**
           Reply the wire answered.
           */
          const reply: TransportReply = { status, bodyText: 'a body worth reading', };
          expect(await withoutRefusedKeyBody({ transport: answeringWith({ reply, },), },)(EXCHANGE,),).toBe(reply,);
        },),);
      },
    },),
  ],
},);
