/**
 Tests for the decisions endpoint's documented reply shapes: what counts as
 an answer, and what a body must carry to be read at all.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DecisionReplyShapeError,
  isDecisionAnswer,
  readDecisionReplyBody,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Reply carrying one answer of each documented kind and nothing else.
 */
const MINIMAL_BODY =
  '{"answers":{"a":{"type":"choice","choice":"cat"},"b":{"type":"noul","noul":0.5},'
    + '"c":{"type":"score","score":3}},"model":"kitty"}';

/**
 What a refused read of one body threw, as the three facts a refusal is
 checked by.

 @param bodyText - reply body the read must refuse

 @returns Whether the refusal is the shape error, its whole text, and
 whether it carries a cause

 @example
 ```ts
 const reading = refusalReading({ bodyText: '"cat"', },);
 ```
 */
function refusalReading(
  { bodyText, }: { readonly bodyText: string; },
): {
  readonly isShapeError: boolean;
  readonly text: string;
  readonly carriesCause: boolean;
} {
  /**
   What the read threw.
   */
  const refusal = caught(function reads(): unknown {
    return readDecisionReplyBody({ bodyText, },);
  },);
  return {
    isShapeError: refusal instanceof DecisionReplyShapeError,
    text: String(refusal,),
    carriesCause: Error.isError(refusal,) && ('cause' in refusal),
  };
}

//endregion Fixtures

await describe({
  name: 'decision contract',
  children: [
    describe({
      name: isDecisionAnswer.name,
      children: [
        it({
          name: 'READS the three documented answers and REFUSES one that is none of them, a non-record, or a '
            + 'record whose value carries the wrong shape',
          fn: async () => {
            expect(isDecisionAnswer({
              type: 'choice',
              choice: 'cat',
            },),).toBe(true,);
            expect(isDecisionAnswer({
              type: 'noul',
              noul: 0.5,
            },),).toBe(true,);
            expect(isDecisionAnswer({
              type: 'score',
              score: 3,
            },),).toBe(true,);
            expect(isDecisionAnswer({
              type: 'meow',
            },),).toBe(false,);
            expect(isDecisionAnswer('cat',),).toBe(false,);
            expect(isDecisionAnswer({
              type: 'noul',
              noul: 'x',
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: readDecisionReplyBody.name,
      children: [
        it({
          name: 'REFUSES a body that is not JSON as a shape error naming only that, and CARRIES the parse '
            + 'failure as its cause',
          fn: async () => {
            /**
             What the read threw.
             */
            const refusal = caught(function reads(): unknown {
              return readDecisionReplyBody({ bodyText: 'cat', },);
            },);
            if (!(refusal instanceof DecisionReplyShapeError))
              throw new Error(`unreachable: the read threw ${String(refusal,)} rather than a shape error`,);
            expect(String(refusal,),).toBe(
              'DecisionReplyShapeError: decisions reply is not the documented shape: body is not JSON',
            );
            /**
             What parsing the same body throws on its own.
             */
            const parseRefusal = caught(function parses(): unknown {
              return JSON.parse('cat',);
            },);
            expect(refusal.cause,).toBeInstanceOf(SyntaxError,);
            expect(String(refusal.cause,),).toBe(String(parseRefusal,),);
          },
        },),

        it({
          name: 'REFUSES a body that is JSON but no object, a string or an array, with no cause beside the '
            + 'refusal',
          fn: async () => {
            /**
             The refusal every such body earns.
             */
            const notAnObject = {
              isShapeError: true,
              text: 'DecisionReplyShapeError: decisions reply is not the documented shape: body is not an object',
              carriesCause: false,
            };
            expect(refusalReading({ bodyText: '"cat"', },),).toEqual(notAnObject,);
            expect(refusalReading({ bodyText: '[]', },),).toEqual(notAnObject,);
          },
        },),

        it({
          name: 'REFUSES a reply with no answers block as missing, and one whose answers block is no object as '
            + 'such, with no cause beside either',
          fn: async () => {
            expect(refusalReading({ bodyText: '{"model":"kitty"}', },),).toEqual({
              isShapeError: true,
              text: 'DecisionReplyShapeError: decisions reply is not the documented shape: answers missing',
              carriesCause: false,
            },);
            expect(refusalReading({ bodyText: '{"answers":[],"model":"kitty"}', },),).toEqual({
              isShapeError: true,
              text: 'DecisionReplyShapeError: decisions reply is not the documented shape: answers is not an object',
              carriesCause: false,
            },);
          },
        },),

        it({
          name: 'REFUSES a reply carrying an answer that is none of the three, naming the answer',
          fn: async () => {
            expect(refusalReading({ bodyText: '{"answers":{"a":{"type":"meow"}},"model":"kitty"}', },),).toEqual({
              isShapeError: true,
              text: 'DecisionReplyShapeError: decisions reply is not the documented shape: answer a is not a '
                + 'choice, noul or score',
              carriesCause: false,
            },);
          },
        },),

        it({
          name: 'REFUSES a reply naming no model as missing, and one whose model is no string as such',
          fn: async () => {
            expect(refusalReading({ bodyText: '{"answers":{"a":{"type":"choice","choice":"cat"}}}', },),).toEqual({
              isShapeError: true,
              text: 'DecisionReplyShapeError: decisions reply is not the documented shape: model missing',
              carriesCause: false,
            },);
            expect(refusalReading({ bodyText: '{"answers":{},"model":5}', },),).toEqual({
              isShapeError: true,
              text: 'DecisionReplyShapeError: decisions reply is not the documented shape: model is not a string',
              carriesCause: false,
            },);
          },
        },),

        it({
          name: 'READS a reply carrying no usage as its answers and model alone',
          fn: async () => {
            expect(readDecisionReplyBody({
              bodyText: MINIMAL_BODY,
            },),).toEqual({
              answers: {
                a: {
                  type: 'choice',
                  choice: 'cat',
                },
                b: {
                  type: 'noul',
                  noul: 0.5,
                },
                c: {
                  type: 'score',
                  score: 3,
                },
              },
              model: 'kitty',
            },);
          },
        },),

        it({
          name: 'CARRIES usage only when both token counts are numbers, and costUsd only when cost is one',
          fn: async () => {
            expect(readDecisionReplyBody({
              bodyText: '{"answers":{},"model":"kitty","usage":{"input_tokens":3}}',
            },),).toEqual({
              answers: {},
              model: 'kitty',
            },);
            expect(readDecisionReplyBody({
              bodyText: '{"answers":{},"model":"kitty","usage":{"input_tokens":3,"output_tokens":4}}',
            },),).toEqual({
              answers: {},
              model: 'kitty',
              usage: {
                prompt_tokens: 3,
                completion_tokens: 4,
                total_tokens: 7,
              },
            },);
            expect(readDecisionReplyBody({
              bodyText: '{"answers":{},"model":"kitty","usage":{"input_tokens":3,"output_tokens":4,"cost":0.5}}',
            },),).toEqual({
              answers: {},
              model: 'kitty',
              usage: {
                prompt_tokens: 3,
                completion_tokens: 4,
                total_tokens: 7,
              },
              costUsd: 0.5,
            },);
          },
        },),
      ],
    },),
  ],
},);
