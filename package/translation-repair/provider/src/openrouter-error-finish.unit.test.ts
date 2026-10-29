/**
 Tests for reading an error finish off a whole OpenRouter stream.

 WHAT THESE PIN is the eighteenth class (2026-09-08): a stream that reasons,
 writes no content, closes its choice with `finish_reason: "error"`, carries
 no error object and sends `[DONE]`, which the reply ladder had been reading
 as the model's own empty answer and counting as a lost voice.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ERROR_FINISH_ABSENT,
  openRouterErrorFinishOf,
} from '../dist/final/node/index.mjs';

/**
 One chunk as the gateway frames it.

 @param chunk - object to frame

 @returns Framed event line

 @example
 ```ts
 const line = framed({ chunk: { provider: 'Sill', }, },);
 ```
 */
function framed({ chunk, }: { readonly chunk: Readonly<Record<string, unknown>>; },): string {
  return `data: ${JSON.stringify(chunk,)}\n\n`;
}

/**
 A closing chunk whose choice stopped on the given reasons.

 @param finish - gateway's normalized stop reason

 @param native - upstream's own stop reason, when forwarded

 @returns Framed closing chunk followed by the terminator

 @example
 ```ts
 const stream = closedWith({ finish: 'error', native: 'upstream_error', },);
 ```
 */
function closedWith(
  {
    finish,
    native,
  }: {
    readonly finish: string;
    readonly native?: string;
  },
): string {
  return `${
    framed({
      chunk: {
        id: 'gen-cat',
        provider: 'Sill',
        choices: [{
          index: 0,
          delta: { content: '', },
          finish_reason: finish,
          ...((native === undefined) ? {} : { native_finish_reason: native, }),
        },],
      },
    },)
  }data: [DONE]\n\n`;
}

await describe({
  name: openRouterErrorFinishOf.name,
  children: [
    it({
      name: 'FINDS THE ERROR FINISH on a whole stream and carries the upstream\'s own reason when the '
        + 'gateway forwarded one',
      fn: async () => {
        expect(openRouterErrorFinishOf({
          bodyText: closedWith({
            finish: 'error',
            native: 'upstream_error',
          },),
        },),).toEqual({
          found: true,
          nativeReason: 'upstream_error',
        },);
      },
    },),

    it({
      name: 'LEAVES THE NATIVE REASON ABSENT rather than inventing one, when the closing choice names '
        + 'none or names it empty',
      fn: async () => {
        expect(openRouterErrorFinishOf({ bodyText: closedWith({ finish: 'error', },), },),).toEqual({
          found: true,
        },);
        expect(openRouterErrorFinishOf({
          bodyText: closedWith({
            finish: 'error',
            native: '',
          },),
        },),).toEqual({ found: true, },);
      },
    },),

    it({
      name: 'FINDS NOTHING on an ordinary stop, a length stop, an empty body or a chunk whose choices '
        + 'field is not an array, so no answer ever reads as a failure',
      fn: async () => {
        expect(openRouterErrorFinishOf({ bodyText: closedWith({ finish: 'stop', },), },),).toEqual(
          ERROR_FINISH_ABSENT,
        );
        expect(openRouterErrorFinishOf({ bodyText: closedWith({ finish: 'length', },), },),).toEqual(
          ERROR_FINISH_ABSENT,
        );
        expect(openRouterErrorFinishOf({ bodyText: '', },),).toEqual(ERROR_FINISH_ABSENT,);
        expect(openRouterErrorFinishOf({
          bodyText: framed({ chunk: { choices: 'error', }, },),
        },),).toEqual(ERROR_FINISH_ABSENT,);
      },
    },),

    it({
      name: 'READS THE CHOICE PAST A REASONING CHUNK, since the class reasons for tens of seconds '
        + 'before the closing chunk names the failure',
      fn: async () => {
        /**
         A reasoning delta with no stop reason yet, ahead of the closing chunk.
         */
        const reasoned = framed({
          chunk: {
            id: 'gen-cat',
            provider: 'Sill',
            choices: [{
              index: 0,
              delta: {
                content: '',
                reasoning: 'is the sill warm',
              },
              finish_reason: null,
            },],
          },
        },) + closedWith({
          finish: 'error',
          native: 'upstream_error',
        },);
        expect(openRouterErrorFinishOf({ bodyText: reasoned, },),).toEqual({
          found: true,
          nativeReason: 'upstream_error',
        },);
      },
    },),
  ],
},);
