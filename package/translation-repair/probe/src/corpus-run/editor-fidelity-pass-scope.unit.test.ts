/**
 Tests for the wrapper that names the pass on every exchange of a row, so the prompt reuse layer of the run client
 treats the same prompt asked in another pass as a new exchange.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ChatTextReply,
  ChatTextRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  passScopeOf,
  scopedToPass,
} from '../../dist/final/node/index.mjs';
import { LOW_CARD, } from '../editor-fidelity.test-fixture.ts';

/**
 Client recording every request it is handed and answering each with an empty edit list.

 @param seen - array each request is appended to

 @returns Inner client the wrapper wraps

 @example
 ```ts
 const inner = recordingClient({ seen: [], },);
 ```
 */
function recordingClient({ seen, }: { readonly seen: ChatTextRequest[]; },): ModelClient {
  return {
    chatText: (request,): Promise<ChatTextReply> => {
      seen.push(request,);
      return Promise.resolve({ text: 'meow', },);
    },
    chatJson: <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      seen.push(request,);
      return Promise.resolve({
        kind: 'ok',
        value: { edits: [], } as ValueT,
        rawText: '{"edits":[]}',
      },);
    },
    quotas: () => Promise.reject(new Error('quotas unused',),),
  };
}

/**
 One exchange of a thousand characters, as a trial asks it.
 */
const REQUEST: ChatTextRequest = {
  modelId: LOW_CARD,
  messages: [{ role: 'user', content: 'Where does the cat nap?', },],
  signal: new AbortController().signal,
};

await describe({
  name: '',
  children: [
    describe({
      name: passScopeOf.name,
      children: [
        it({
          name: 'COUNTS PASSES FROM ONE IN THE WORDS THE ROW LINE USES, the first pass being pass zero of the walk: '
            + 'zero is "pass 1", one is "pass 2", two is "pass 3"',
          fn: async () => {
            expect(passScopeOf({ pass: 0, },),).toBe('pass 1',);
            expect(passScopeOf({ pass: 1, },),).toBe('pass 2',);
            expect(passScopeOf({ pass: 2, },),).toBe('pass 3',);
          },
        },),
      ],
    },),

    describe({
      name: scopedToPass.name,
      children: [
        it({
          name: 'NAMES THE PASS ON A TEXT AND ON A JSON EXCHANGE and changes nothing else in the request',
          fn: async () => {
            /**
             Requests the inner client was handed.
             */
            const seen: ChatTextRequest[] = [];
            const client = scopedToPass({
              client: recordingClient({ seen, },),
              pass: 1,
            },);
            await client.chatText(REQUEST,);
            await client.chatJson({ ...REQUEST, validate: (value: unknown,): value is unknown => value !== undefined, },);
            expect(seen.map(function scopeOf({ reuseScope, },): string {
              return reuseScope ?? 'no scope';
            },),).toStrictEqual(['pass 2', 'pass 2',],);
            expect(seen.map(function modelOf({ modelId, },): string {
              return modelId;
            },),).toStrictEqual([LOW_CARD, LOW_CARD,],);
            expect(seen[0]?.messages,).toStrictEqual(REQUEST.messages,);
            expect(seen[0]?.signal,).toBe(REQUEST.signal,);
          },
        },),

        it({
          name: 'KEEPS THE REST OF THE CLIENT it wraps, so the quota read is the wrapped client\'s own',
          fn: async () => {
            /**
             Client whose quota read the wrapper must keep.
             */
            const inner = recordingClient({ seen: [], },);
            const client = scopedToPass({
              client: inner,
              pass: 0,
            },);
            expect(client.quotas,).toBe(inner.quotas,);
          },
        },),
      ],
    },),
  ],
},);
