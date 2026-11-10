import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BLOCK_TAG_NAMES,
  parseMutationContractBlocks,
} from '../dist/final/node/index.mjs';

/**
 Builds a comment body whose open `@mutates` block is followed by one block
 tag, so the parse result shows whether that tag ended the block.
 */
function commentBodyOpeningOn(tag: string,): string {
  return `@mutates state - Updates\n${tag} trailing`;
}

const blockTagNames: readonly string[] = [...BLOCK_TAG_NAMES,];

await describe({
  name: '',
  children: [
    describe({
      name: 'BLOCK_TAG_NAMES',
      children: [
        it({
          name: 'names every entry as an at-sign tag',
          fn: async () => {
            const malformed = blockTagNames.filter(function notATag(tag,): boolean {
              return (!tag.startsWith('@',)) || (tag.length === 1);
            },);
            expect(malformed,).toEqual([],);
          },
        },),
        it({
          name: 'carries the project tags as well as the standard ones',
          fn: async () => {
            const absent = [
              '@mutates',
              '@yields',
            ].filter(function missing(tag,): boolean {
              return !BLOCK_TAG_NAMES.has(tag,);
            },);
            expect(absent,).toEqual([],);
          },
        },),
        it({
          name: 'is the list that ends an open @mutates block',
          fn: async () => {
            // Derived from the list under test: a tag added to BLOCK_TAG_NAMES
            // is scanned here without editing this case.
            const unended = blockTagNames.filter(function endsNothing(tag,): boolean {
              const [first] = parseMutationContractBlocks({
                commentValue: commentBodyOpeningOn(tag,),
              },);
              return (first === undefined) || (first.description !== 'Updates');
            },);
            expect(unended,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
