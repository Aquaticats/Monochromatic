/**
 Tests for the readers that take a line for a fence: any run of three
 backticks or tildes, and the stricter reading the nesting scan skips a
 fenced block by, which accepts only a fence the parser certainly reads.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  certainFenceOf,
  fenceOf,
  NO_FENCE,
} from '../dist/final/node/index.mjs';

//region Nesting fence tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: fenceOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a run of three or more backticks or tildes as a fence, with its character and length',
          fn: async () => {
            expect(fenceOf({ line: '```ts', },),).toEqual({ character: '`', length: 3, },);
            expect(fenceOf({ line: '~~~~', },),).toEqual({ character: '~', length: 4, },);
          },
        },),
        it({
          name: 'READS two backticks, prose and an empty line as no fence',
          fn: async () => {
            expect(fenceOf({ line: '``', },),).toEqual(NO_FENCE,);
            expect(fenceOf({ line: 'cat', },),).toEqual(NO_FENCE,);
            expect(fenceOf({ line: '', },),).toEqual(NO_FENCE,);
            expect(NO_FENCE,).toEqual({ character: '', length: 0, },);
          },
        },),
      ],
    },),

    describe({
      name: certainFenceOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a line that begins with three backticks or tildes, info string and all, as a fence',
          fn: async () => {
            expect(certainFenceOf({ line: '```ts', },),).toEqual({ character: '`', length: 3, },);
            expect(certainFenceOf({ line: '~~~~ cat', },),).toEqual({ character: '~', length: 4, },);
            expect(certainFenceOf({ line: '~~~a`b', },),).toEqual({ character: '~', length: 3, },);
          },
        },),
        it({
          name: 'READS a backtick fence with a backtick in its info string or its meta as no fence',
          fn: async () => {
            expect(certainFenceOf({ line: '```a`b', },),).toEqual(NO_FENCE,);
            expect(certainFenceOf({ line: '```ts meta`', },),).toEqual(NO_FENCE,);
            expect(certainFenceOf({ line: '```` `` ', },),).toEqual(NO_FENCE,);
          },
        },),
        it({
          name: 'READS a fence after any space, tab or container marker as no fence, since only a line that begins '
            + 'with the fence is certain',
          fn: async () => {
            expect(certainFenceOf({ line: ' ```', },),).toEqual(NO_FENCE,);
            expect(certainFenceOf({ line: '    ```', },),).toEqual(NO_FENCE,);
            expect(certainFenceOf({ line: '\t```', },),).toEqual(NO_FENCE,);
            expect(certainFenceOf({ line: '> ```', },),).toEqual(NO_FENCE,);
            expect(certainFenceOf({ line: '- ```', },),).toEqual(NO_FENCE,);
          },
        },),
        it({
          name: 'READS two backticks and prose as no fence',
          fn: async () => {
            expect(certainFenceOf({ line: '``', },),).toEqual(NO_FENCE,);
            expect(certainFenceOf({ line: 'cat', },),).toEqual(NO_FENCE,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Nesting fence tests
