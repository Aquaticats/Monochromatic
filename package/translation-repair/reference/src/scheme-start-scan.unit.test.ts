/**
 Tests for the search both address scans share: where the next web scheme
 starts in a text.

 WHAT THESE PIN: the answer is the first place at or after the cursor where a
 whole scheme starts, whichever scheme it is; a stem that opens no scheme is
 passed over; and the text's length stands for "none". The two readers that
 call it (`corpus-run/dropped-destinations.ts`, `cited-reference-scan.ts`)
 pin what they make of a run in their own files.

 NO CASE TIMES THE SEARCH. That it costs time in proportion to the text was
 measured apart from the suite, since a clock in a case fails on a busy
 machine and proves nothing on a quiet one.

 Fixtures are invented addresses and words about a cat.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { nextSchemeStart, } from '../dist/final/node/index.mjs';

/**
 One sentence carrying one address, 32 characters long, its scheme 4 in.
 */
const SENTENCE = 'see https://cats.example/nap and ';

/**
 How many sentences the repeated text holds.
 */
const REPEATS = 1_000;

await describe({
  name: nextSchemeStart.name,
  children: [
    it({
      name: 'ANSWERS where the first scheme at or after the position starts, whichever scheme it is',
      fn: async () => {
        /**
         Text carrying the secure scheme at 4 and the plain one at 26.
         */
        const text = 'see https://a.example and http://b.example';

        expect([
          0,
          4,
          5,
          26,
          27,
        ].map(function answerFrom(from,): number {
          return nextSchemeStart({
            text,
            from,
          },);
        },),).toEqual([
          4,
          4,
          26,
          26,
          text.length,
        ],);
      },
    },),

    it({
      name: 'ANSWERS the text\'s length where no scheme starts at or after the position, for an empty text, '
        + 'a text without a scheme and a position past the text',
      fn: async () => {
        expect(nextSchemeStart({
          text: '',
          from: 0,
        },),).toBe(0,);
        expect(nextSchemeStart({
          text: 'the tabby naps',
          from: 0,
        },),).toBe(14,);
        expect(nextSchemeStart({
          text: 'http://a.example',
          from: 40,
        },),).toBe(16,);
      },
    },),

    it({
      name: 'PASSES OVER a stem that opens no scheme: a longer word, the stem alone, a scheme short of a '
        + 'slash and a scheme in capitals',
      fn: async () => {
        expect(nextSchemeStart({
          text: 'the httpd log, then http://a.example',
          from: 0,
        },),).toBe(20,);
        expect(nextSchemeStart({
          text: 'http and https, then https:/a and http:a, then HTTPS://a.example',
          from: 0,
        },),).toBe(64,);
      },
    },),

    it({
      name: 'FINDS a scheme that directly follows a stem, and the scheme that directly follows a scheme '
        + 'when asked from inside the first',
      fn: async () => {
        expect(nextSchemeStart({
          text: 'httphttp://a.example',
          from: 0,
        },),).toBe(4,);
        expect(nextSchemeStart({
          text: 'httpshttps://a.example',
          from: 0,
        },),).toBe(5,);
        expect(nextSchemeStart({
          text: 'https://http://a.example',
          from: 1,
        },),).toBe(8,);
      },
    },),

    it({
      name: 'FINDS a scheme that opens the text and a scheme that ends it',
      fn: async () => {
        expect(nextSchemeStart({
          text: 'https://a.example',
          from: 0,
        },),).toBe(0,);
        expect(nextSchemeStart({
          text: 'ends on http://',
          from: 0,
        },),).toBe(8,);
      },
    },),

    it({
      name: 'ANSWERS every address of a text repeating one sentence, each asked from one past the last '
        + 'answer, then the text\'s length',
      fn: async () => {
        /**
         The sentence, many times over.
         */
        const text = SENTENCE.repeat(REPEATS,);

        /**
         Answers in the order given, the closing "none" included.
         */
        const answers: number[] = [];
        for (
          let at = nextSchemeStart({
            text,
            from: 0,
          },);
          answers.length <= REPEATS;
          at = nextSchemeStart({
            text,
            from: at + 1,
          },)
        )
          answers.push(at,);

        expect(answers,).toEqual([
          ...Array.from(
            { length: REPEATS, },
            function schemeOf(
              _unused,
              sentence,
            ): number {
              return (sentence * SENTENCE.length) + 4;
            },
          ),
          text.length,
        ],);
      },
    },),
  ],
},);
