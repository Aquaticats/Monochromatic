/**
 Tests for reading a JSON object past an abandoned opening fragment.

 THE SEVENTEENTH CLASS: on 2026-09-08 reasoning streams from Bedrock's
 gpt-oss-120b and OpenRouter's Makora route for deepseek-v4-flash-0731 wrote
 an opening, abandoned it, and wrote the whole object after it, and every
 such reply was refused as a schema mismatch.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  FALSE_START_WINDOW,
  parseAnswerJson,
  readJsonPastFalseStart,
} from '../dist/final/node/index.mjs';

//region False-start tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readJsonPastFalseStart.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE OBJECT PAST each opening shape the two passes of 2026-09-08 delivered, naming how long the '
            + 'abandoned opening was',
          fn: async () => {
            expect(readJsonPastFalseStart({ text: '{"best": 1{"best": 1, "reason": "x"}', },),).toEqual({
              parsed: true,
              value: {
                best: 1,
                reason: 'x',
              },
              abandoned: 10,
            },);
            expect(readJsonPastFalseStart({ text: '{"{"best": 2, "reason": "y"}', },),).toEqual({
              parsed: true,
              value: {
                best: 2,
                reason: 'y',
              },
              abandoned: 2,
            },);
            expect(readJsonPastFalseStart({ text: '{ {   "choice": "a"}', },),).toEqual({
              parsed: true,
              value: { choice: 'a', },
              abandoned: 2,
            },);
            expect(readJsonPastFalseStart({ text: '{  {"choice": "b"}', },),).toEqual({
              parsed: true,
              value: { choice: 'b', },
              abandoned: 3,
            },);
            expect(readJsonPastFalseStart({ text: '{   "issues":{     "issues": []}', },),).toEqual({
              parsed: true,
              value: { issues: [], },
              abandoned: 13,
            },);
          },
        },),
        it({
          name: 'READS NOTHING past a reply that is simply cut, one that does not open an object, or one whose '
            + 'opening runs past the window',
          fn: async () => {
            expect(readJsonPastFalseStart({ text: '{"verdict":"na', },),).toEqual({ parsed: false, },);
            expect(readJsonPastFalseStart({ text: 'nope {"a":1}', },),).toEqual({ parsed: false, },);
            expect(readJsonPastFalseStart({
              text: `{${' '.repeat(FALSE_START_WINDOW,)}{"a":1}`,
            },),).toEqual({ parsed: false, },);
          },
        },),
      ],
    },),

    describe({
      name: parseAnswerJson.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PARSES A WHOLE ANSWER with nothing abandoned, READS PAST a false start, and KEEPS the failure '
            + 'detail of a reply that is neither',
          fn: async () => {
            expect(parseAnswerJson({ text: '{"cat":"喵"}', },),).toEqual({
              parsed: true,
              value: { cat: '喵', },
              abandoned: 0,
              trailing: 0,
            },);
            expect(parseAnswerJson({ text: '{"best": 1{"best": 1}', },),).toEqual({
              parsed: true,
              value: { best: 1, },
              abandoned: 10,
              trailing: 0,
            },);
            /**
             A cut reply, which no start inside it completes.
             */
            const cut = parseAnswerJson({ text: '{"cat":', },);
            expect(cut.parsed,).toBe(false,);
            expect(cut.parsed ? '' : cut.detail.includes('SyntaxError',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);

//endregion False-start tests
