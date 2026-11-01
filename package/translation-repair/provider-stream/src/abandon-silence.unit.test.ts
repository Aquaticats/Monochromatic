/**
 Tests for telling an abandoned call that delivered nothing from one that delivered something.

 `inception/mercury-2.5` left 33 streams in CuspariaKLSY22 and 11 in zheermao22 that read `0 content chars, 0 reasoning
 chars` while the raw count grew with the wait (7,450 raw characters at 125 s): framing and no model text. The same
 logs hold 12 cuts of slow reasoning models that had delivered 25,400 to 69,336 reasoning characters and no answer
 yet. The first is a dead stream and the second is a working one, and only the first may count against a seat.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  deliveredNoContent,
  StreamDegenerateError,
  StreamOverrunError,
} from '../dist/final/node/index.mjs';
import { cutStreamError, } from './cut-stream.test-fixture.ts';

await describe({
  name: deliveredNoContent.name,
  children: [
    it({
      name: 'IS TRUE FOR A CUT WITH ZERO ANSWER AND ZERO REASONING CHARACTERS though thousands of framing characters '
        + 'reached the wire, as every mercury stream of CuspariaKLSY22 read',
      fn: async () => {
        expect(deliveredNoContent({
          error: cutStreamError({ firstByte: 'arrived', generatedChars: { content: 0, reasoning: 0, }, },),
        },),).toBe(true,);
      },
    },),

    it({
      name: 'IS FALSE FOR A CUT THAT HAD REASONING BUT NO ANSWER, the slow reasoner still working that the grace window '
        + 'cuts (Qwen at 43,372 reasoning characters)',
      fn: async () => {
        expect(deliveredNoContent({
          error: cutStreamError({ firstByte: 'arrived', generatedChars: { content: 0, reasoning: 43_372, }, },),
        },),).toBe(false,);
      },
    },),

    it({
      name: 'IS FALSE FOR A CUT THAT HAD ANSWER CHARACTERS',
      fn: async () => {
        expect(deliveredNoContent({
          error: cutStreamError({ firstByte: 'arrived', generatedChars: { content: 12, reasoning: 0, }, },),
        },),).toBe(false,);
      },
    },),

    it({
      name: 'IS TRUE FOR A CUT WITH NO COUNT WHEN NO FIRST BYTE EVER ARRIVED, and false when one did, since the wire '
        + 'length alone says nothing of what the model wrote',
      fn: async () => {
        expect(deliveredNoContent({ error: cutStreamError({ firstByte: 'never', },), },),).toBe(true,);
        expect(deliveredNoContent({ error: cutStreamError({ firstByte: 'arrived', },), },),).toBe(false,);
      },
    },),

    it({
      name: 'IS FALSE FOR EVERY OTHER ERROR, since an error that is not a cut stream says nothing of what was delivered',
      fn: async () => {
        expect(deliveredNoContent({ error: new Error('the cat unplugged it',), },),).toBe(false,);
        expect(deliveredNoContent({ error: new DOMException('aborted', 'AbortError',), },),).toBe(false,);
        expect(deliveredNoContent({ error: undefined, },),).toBe(false,);
        expect(deliveredNoContent({
          error: new StreamDegenerateError({
            label: 'inception/mercury-2.5',
            channel: 'content',
            charsSeen: 4_000,
            rawChars: 4_100,
            distinctRatio: 0.01,
          },),
        },),).toBe(false,);
        expect(deliveredNoContent({
          error: new StreamOverrunError({
            label: 'inception/mercury-2.5',
            channel: 'content',
            charsSeen: 32_008,
            rawChars: 33_000,
            cap: 32_000,
          },),
        },),).toBe(false,);
      },
    },),
  ],
},);
