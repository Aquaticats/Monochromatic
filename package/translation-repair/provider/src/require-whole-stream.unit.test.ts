/**
 Tests for the whole-stream check the retry ladder runs on a success reply.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { MalformedCompletionError, } from '@monochromatic-dev/translation-repair-provider-stream';
import { requireWholeStream, } from '../dist/final/node/index.mjs';

/**
 Content chunk with no terminator after it.
 */
const CHUNK = `data: ${JSON.stringify({ choices: [{ delta: { content: 'purr', }, },], },)}\n\n`;

await describe({
  name: requireWholeStream.name,
  children: [
    it({
      name: 'PASSES a success reply that reached [DONE]',
      fn: async () => {
        expect(requireWholeStream({ status: 200, bodyText: `${CHUNK}data: [DONE]\n\n`, },),).toBeUndefined();
      },
    },),
    it({
      name: 'REFUSES a success reply that stopped before [DONE], as the transport failure it is',
      fn: async () => {
        expect(function cutShort(): void {
          requireWholeStream({ status: 200, bodyText: CHUNK, },);
        },).toThrow(MalformedCompletionError,);
      },
    },),
    it({
      name: 'LEAVES a failing status to the status check, whatever its body',
      fn: async () => {
        expect(requireWholeStream({ status: 500, bodyText: CHUNK, },),).toBeUndefined();
      },
    },),
  ],
},);
