import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isRecord,
  parseMutationContractBlocks,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: parseMutationContractBlocks.name,
      children: [
        it({
          name: 'parses multiline blocks and ignores fenced examples',
          fn: async () => {
            /** Comment body covering continuation, block termination, and fence exclusion. */
            const commentValue = '@mutates state - Updates\ncontinued\n@param value - Input\n@example\n```ts\n@mutates hidden - ignored\n```\n@mutates cache - Clears cache';
            expect(parseMutationContractBlocks({ commentValue, },),).toEqual([
              {
                parameterName: 'state',
                description: 'Updates\ncontinued',
                hasDescription: true,
                lineOffset: 0,
                blockStartOffset: 0,
                blockEndOffset: 35,
              },
              {
                parameterName: 'cache',
                description: 'Clears cache',
                hasDescription: true,
                lineOffset: 7,
                blockStartOffset: 101,
                blockEndOffset: 130,
              },
            ],);
          },
        },),
        it({
          name: 'preserves absent target and description facts',
          fn: async () => {
            expect(parseMutationContractBlocks({
              commentValue: '@mutates -',
            },),).toEqual([
              {
                parameterName: '',
                description: '',
                hasDescription: false,
                lineOffset: 0,
                blockStartOffset: 0,
                blockEndOffset: 10,
              },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: isRecord.name,
      children: [
        it({
          name: 'accepts object values',
          fn: async () => {
            expect(isRecord({ value: true, },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects null and primitives',
          fn: async () => {
            expect(isRecord(null,),).toBe(false,);
            expect(isRecord('value',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
