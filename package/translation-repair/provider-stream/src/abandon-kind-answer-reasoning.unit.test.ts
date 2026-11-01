/**
 Tests for the abandonment line of a stream cut mid-reply (the lead's ruling of 2026-10-15 on the run of TianqiChen66625,
 where three streams of Qwen were cut with no answer at all and 49,293, 36,301 and 37,784 characters of reasoning, and the
 line reported a wire length of 1,308,758 characters, about 27 times the reasoning): the line reports the characters of the
 answer and of the reasoning that the stream counted, and keeps the wire length as a separate, named field.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  describeAbandon,
  StreamCutShortError,
} from '../dist/final/node/index.mjs';

/**
 Builds a cut that carries the characters the stream counted on each channel.

 @param wire - length of the stream as it arrived, framing included
 @param generatedChars - characters of the answer and of the reasoning the stream counted
 @param firstByteMs - when the first byte arrived

 @returns Error as the drain raises it

 @example
 ```ts
 const error = cutAfter({ wire: 1_308_758, generatedChars: { content: 0, reasoning: 49_293, }, firstByteMs: 812, },);
 ```
 */
function cutAfter(
  {
    wire,
    generatedChars,
    firstByteMs,
  }: {
    readonly wire: number;
    readonly generatedChars: { readonly content: number; readonly reasoning: number; };
    readonly firstByteMs: number;
  },
): StreamCutShortError {
  return new StreamCutShortError({
    label: 'hf:whiskers',
    partialText: 'x'.repeat(wire,),
    progress: {
      firstByteMs,
      firstContentMs: firstByteMs,
      maxGapMs: 0,
      chars: wire,
      elapsedMs: 120,
    },
    generatedChars,
    cause: new Error('exchange torn down by abort',),
  },);
}

await describe({
  name: describeAbandon.name,
  children: [
    it({
      name: 'REPORTS THE ANSWER\'S AND THE REASONING\'S CHARACTERS THE STREAM COUNTED, and the wire length as a separate named '
        + 'field, for a stream cut with reasoning and no answer',
      fn: async () => {
        expect(
          describeAbandon({
            error: cutAfter({ wire: 1_308_758, generatedChars: { content: 0, reasoning: 49_293, }, firstByteMs: 812, },),
          },),
        ).toBe(
          'cut-mid-reply after 0 answer chars and 49293 reasoning chars (1308758 chars on the wire), first byte at 812ms',
        );
      },
    },),
    it({
      name: 'USES THE SINGULAR FOR ONE CHARACTER ON A CHANNEL',
      fn: async () => {
        expect(
          describeAbandon({
            error: cutAfter({ wire: 40, generatedChars: { content: 1, reasoning: 1, }, firstByteMs: 5, },),
          },),
        ).toBe('cut-mid-reply after 1 answer char and 1 reasoning char (40 chars on the wire), first byte at 5ms',);
      },
    },),
    it({
      name: 'KEEPS THE WIRE LENGTH ALONE FOR A CUT THAT CARRIES NO COUNT OF THE CHANNELS, and still tells a cut from no first byte',
      fn: async () => {
        expect(
          describeAbandon({
            error: new StreamCutShortError({
              label: 'hf:whiskers',
              partialText: 'It is a cat. It did a backflip. It cras',
              progress: { firstByteMs: 40, firstContentMs: 40, maxGapMs: 0, chars: 39, elapsedMs: 120, },
              cause: new Error('exchange torn down by abort',),
            },),
          },),
        ).toBe('cut-mid-reply after 39 chars on the wire, first byte at 40ms',);
      },
    },),
  ],
},);
