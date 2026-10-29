/**
 Tests for naming why a voice was lost.

 The whole value is SEPARATION. Before this, every abandonment logged the same
 phrase, and three situations wanting opposite remedies were indistinguishable
 in a run log: a call that never got a byte, a call cut off part way through,
 and a call we ended ourselves for repeating itself.

 Fixtures are cat-themed invention. No corpus content appears here.

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
  StreamDegenerateError,
} from '../dist/final/node/index.mjs';

/**
 Builds a cut carrying a given amount of delivered text.

 @param partialText - what the stream delivered

 @param firstByteMs - when the first byte arrived, negative when none did

 @returns Error as the drain raises it

 @example
 ```ts
 const error = cutWith({ partialText: 'It is a cat', firstByteMs: 40, },);
 ```
 */
function cutWith(
  {
    partialText,
    firstByteMs,
  }: {
    readonly partialText: string;
    readonly firstByteMs: number;
  },
): StreamCutShortError {
  return new StreamCutShortError({
    label: 'hf:whiskers',
    partialText,
    progress: {
      firstByteMs,
      firstContentMs: firstByteMs,
      maxGapMs: 0,
      chars: partialText.length,
      elapsedMs: 120,
    },
    cause: new Error('exchange torn down by abort',),
  },);
}

await describe({
  name: describeAbandon.name,
  children: [
    it({
      name: 'SEPARATES a call that never got a byte from one cut part way through, which is the '
        + 'distinction the whole thing exists for: the first is queueing and wants the straggler '
        + 'dropped, the second was working and wants a longer window',
      fn: async () => {
        expect(
          describeAbandon({
            error: cutWith({
              partialText: '',
              firstByteMs: -1,
            },),
          },),
        ).toBe('no-first-byte, nothing was ever delivered',);

        /**
         A call that had begun answering.
         */
        const midReply = describeAbandon({
          error: cutWith({
            partialText: 'It is a cat. It did a backflip. It cras',
            firstByteMs: 40,
          },),
        },);
        expect(midReply.includes('cut-mid-reply',),).toBe(true,);
        expect(midReply.includes('39',),).toBe(true,);
        expect(midReply.includes('40ms',),).toBe(true,);
      },
    },),

    it({
      name: 'NAMES A RUNAWAY AS ITS OWN CAUSE, since waiting longer buys more of the same and the '
        + 'remedy is neither of the other two',
      fn: async () => {
        /**
         What the drain raises when it ends a cycling call.
         */
        const said = describeAbandon({
          error: new StreamDegenerateError({
            label: 'hf:whiskers',
            channel: 'reasoning',
            distinctRatio: 0.0021,
            charsSeen: 412_000,
            rawChars: 412_000,
          },),
        },);

        expect(said.includes('degenerate',),).toBe(true,);
        expect(said.includes('reasoning',),).toBe(true,);
        expect(said.includes('412000',),).toBe(true,);
      },
    },),

    it({
      name: 'NAMES THE CLASS of anything it does not recognise rather than rendering its text, since a '
        + 'provider error\'s own message may carry a body excerpt and this lands in a warn line; the '
        + 'class name still separates an unfamiliar failure from a catch-all "other"',
      fn: async () => {
        expect(describeAbandon({ error: new Error('the cat unplugged it',), },),)
          .toBe('refused by Error',);
      },
    },),
  ],
},);
