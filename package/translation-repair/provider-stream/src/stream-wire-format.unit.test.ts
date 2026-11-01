/**
 Tests for the wire-format choice.

 THE SILENT FAILURE IS THE WHOLE POINT. Draining a stream with a reader that
 does not understand its grammar produces no error and no warning. The
 scanner simply matches nothing, the answer channel stays empty, and every
 runaway guard downstream reads a call that behaved perfectly and produced
 nothing. That is indistinguishable from a model that declined, so a
 misrouted stream would surface as a lost voice and be blamed on a provider.

 The cases in this file therefore assert the WRONG reader sees nothing, not only
 that the right one sees something. Only the pair proves the choice matters.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DEFAULT_WIRE_FORMAT,
  scannerFor,
  watchRunaway,
} from '../dist/final/node/index.mjs';

/**
 One Anthropic answer, as this provider streamed it on 2026-08-24.
 */
const ANTHROPIC_STREAM = [
  'data: {"type":"message_start","message":{"usage":{"input_tokens":0,"output_tokens":0}}}',
  'data: {"type":"ping"}',
  'data: {"type":"content_block_start","index":0,"content_block":{"type":"text"}}',
  `data: ${JSON.stringify({
    type: 'content_block_delta',
    index: 0,
    delta: {
      type: 'text_delta',
      text: 'The cat naps on the sill.',
    },
  },)}`,
  'data: {"type":"content_block_stop","index":0}',
  'data: {"type":"message_stop"}',
  '',
].join('\n\n',);

/**
 One OpenAI-compatible answer, in the grammar the first provider streams.
 */
const OPENAI_STREAM = [
  `data: ${JSON.stringify({
    choices: [
      {
        delta: { content: 'The cat naps on the sill.', },
      },
    ],
  },)}`,
  'data: [DONE]',
  '',
].join('\n\n',);

/**
 Characters one reader takes off one stream.

 @param wireFormat - grammar to read it as

 @param stream - body text to read

 @returns Answer-channel characters the guards would have counted

 @example
 ```ts
 const seen = charsSeen({ wireFormat: 'anthropic', stream: ANTHROPIC_STREAM, },);
 ```
 */
function charsSeen(
  {
    wireFormat,
    stream,
  }: {
    readonly wireFormat: 'openai' | 'anthropic';
    readonly stream: string;
  },
): number {
  /**
   Watch opened on the named grammar, as the drain opens one.
   */
  const watch = watchRunaway({ wireFormat, },);

  watch.notifyChunk({ chunk: stream, },);

  /**
   What the guards counted, per channel; only the answer channel is at issue.
   */
  const { content, } = watch.generatedChars();

  return content;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: scannerFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'OPENS a reader per grammar, and a fresh one each time, since a scanner carries the '
            + 'state of one stream and a shared one would carry the last call into the next',
          fn: async () => {
            expect(scannerFor({ wireFormat: 'anthropic', },),)
              .not.toBe(scannerFor({ wireFormat: 'anthropic', },),);
          },
        },),

        it({
          name: 'DEFAULTS to the older grammar, so every call site that predates the choice drains '
            + 'exactly the stream it drained before',
          fn: async () => {
            expect(DEFAULT_WIRE_FORMAT,).toBe('openai',);
          },
        },),
      ],
    },),

    describe({
      name: watchRunaway.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS AN ANTHROPIC STREAM ONLY WHEN TOLD IT IS ONE. Read as the older grammar the '
            + 'same bytes yield zero characters, which every guard downstream would take for a model '
            + 'that answered nothing rather than for a stream nobody could parse',
          fn: async () => {
            expect(charsSeen({
              wireFormat: 'anthropic',
              stream: ANTHROPIC_STREAM,
            },),).toBe('The cat naps on the sill.'.length,);

            expect(charsSeen({
              wireFormat: 'openai',
              stream: ANTHROPIC_STREAM,
            },),).toBe(0,);
          },
        },),

        it({
          name: 'READS AN OPENAI STREAM ONLY WHEN TOLD IT IS ONE, which is the same trap facing the '
            + 'other way and the reason the default had to stay the older grammar',
          fn: async () => {
            expect(charsSeen({
              wireFormat: 'openai',
              stream: OPENAI_STREAM,
            },),).toBe('The cat naps on the sill.'.length,);

            expect(charsSeen({
              wireFormat: 'anthropic',
              stream: OPENAI_STREAM,
            },),).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
