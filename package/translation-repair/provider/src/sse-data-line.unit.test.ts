/**
 Tests the one reading of a streamed body's `data:` lines that the live delta
 scanners and the answer readers share (audit area six): which lines carry an
 event payload, and what the payload is, by the event-stream format.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  scanStreamDeltas,
  ssePayloadOf,
} from '@monochromatic-dev/translation-repair-provider-stream';
import { extractStreamedCompletion, } from '../dist/final/node/index.mjs';

/**
 Reads one line's payload.

 @param line - one line of a streamed body

 @returns Payload the shared reader finds

 @example
 ```ts
 const payload = payloadOf('data: {}',);
 ```
 */
function payloadOf(line: string,): string {
  return ssePayloadOf({ line, },);
}

await describe({
  name: 'sse data lines',
  children: [
    it({
      name: 'READS THE TIGHT AND THE SPACED FORM AS ONE MESSAGE, removing one space and only one',
      fn: async () => {
        expect([
          'data: {"cat":"nap"}',
          'data:{"cat":"nap"}',
          'data:  {"cat":"nap"}',
        ].map(payloadOf,),).toEqual([
          '{"cat":"nap"}',
          '{"cat":"nap"}',
          ' {"cat":"nap"}',
        ],);
      },
    },),
    it({
      name: 'DROPS ONE CARRIAGE RETURN a server pairs with its newline, and keeps the rest of the value as sent',
      fn: async () => {
        expect([
          'data: [DONE]\r',
          'data: purr \r',
        ].map(payloadOf,),).toEqual([
          '[DONE]',
          'purr ',
        ],);
      },
    },),
    it({
      name: 'READS NO PAYLOAD from a comment, another field, a blank line, a keep-alive, an indented or '
        + 'capitalized field name, or a field name alone',
      fn: async () => {
        expect([
          ': the cat is thinking',
          'event: message_stop',
          '',
          'data:',
          ' data: {"cat":"nap"}',
          'DATA: {"cat":"nap"}',
          'data',
        ].map(payloadOf,),).toEqual([
          '',
          '',
          '',
          '',
          '',
          '',
          '',
        ],);
      },
    },),
    it({
      name: 'THE WATCH AND THE ANSWER READ ONE STREAM THE SAME WAY: a line the format does not count as an '
        + 'event reaches neither the delta scanner nor the folded answer',
      fn: async () => {
        /**
         Body whose second content line is indented, so the format reads its
         field name as " data".
         */
        const body = [
          'data: {"choices":[{"delta":{"content":"The cat "}}]}',
          ' data: {"choices":[{"delta":{"content":"hid "}}]}',
          'data: {"choices":[{"delta":{"content":"napped."}}]}',
          'data: [DONE]',
          '',
        ].join('\n',);

        /**
         What the answer reader folds.
         */
        const folded = extractStreamedCompletion({ bodyText: body, },);

        /**
         What the live scanner sees on the answer channel.
         */
        const watched = scanStreamDeltas()
          .feed({ chunk: body, },)
          .filter(function isContent(delta,): boolean {
            return delta.channel === 'content';
          },)
          .map(function textOf(delta,): string {
            return delta.text;
          },)
          .join('',);
        expect(folded.text,).toBe('The cat napped.',);
        expect(watched,).toBe(folded.text,);
      },
    },),
  ],
},);
