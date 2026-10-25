/**
 Tests for the block by block comparison of what a reader of the page sees with what a reader of the source sees.

 A block of the source that holds a footnote the two readings disagree on is set against the blocks of the page between
 its neighbours; every other block must have a block of the page that shows the same, and one that has none is a
 difference whatever stands beside it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  blockDifferenceOf,
  blocksOfPage,
  blocksOfSource,
} from '../../dist/final/node/index.mjs';

/**
 A block of the source that shows an account, on a line of its own.

 @param account - what the block shows

 @param at - index of the line

 @returns Block of the source

 @example
 ```ts
 const block = blockAt({ account: 'tabby', at: 2, },);
 ```
 */
function blockAt({ account, at, }: { readonly account: string; readonly at: number; },): { account: string; start: number; end: number; } {
  return {
    account,
    start: at * 10,
    end: (at * 10) + 5,
  };
}

await describe({
  name: blockDifferenceOf.name,
  children: [
    it({
      name: 'SAYS NOTHING DIFFERS WHERE EVERY BLOCK OF THE SOURCE HAS A BLOCK OF THE PAGE THAT SHOWS THE SAME',
      fn: async () => {
        expect(blockDifferenceOf({
          sourceBlocks: [blockAt({ account: 'tabby', at: 0, },), blockAt({ account: 'calico', at: 1, },),],
          pageBlocks: ['tabby', 'calico',],
          disagreements: [],
        },),).toEqual({ differs: false, undecidedBlocks: 0, },);
      },
    },),
    it({
      name: 'SAYS A BLOCK DIFFERS THAT HOLDS NO DISAGREEMENT AND HAS NO BLOCK OF THE PAGE, beside one that does',
      fn: async () => {
        /**
         Two blocks, the second holding the disagreement.
         */
        const sourceBlocks = [blockAt({ account: 'tabby', at: 0, },), blockAt({ account: 'calico', at: 1, },),];
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['tabby!', 'calico',],
          disagreements: [10,],
        },),).toEqual({ differs: true, undecidedBlocks: 1, },);
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['tabby', 'calico!',],
          disagreements: [10,],
        },),).toEqual({ differs: false, undecidedBlocks: 1, },);
      },
    },),
    it({
      name: 'SETS A BLOCK THAT HOLDS A DISAGREEMENT AGAINST THE BLOCKS OF THE PAGE BETWEEN ITS NEIGHBOURS, whatever their number',
      fn: async () => {
        /**
         A footnote the page ends sooner, so it has two blocks where the source has one.
         */
        const sourceBlocks = [
          blockAt({ account: 'tabby', at: 0, },),
          blockAt({ account: '[^1]: x text', at: 1, },),
          blockAt({ account: 'calico', at: 2, },),
        ];
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['tabby', '[^1]: x', 'other', 'calico',],
          disagreements: [10,],
        },),).toEqual({ differs: false, undecidedBlocks: 1, },);
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['tabby', '[^1]: x', 'other', 'calico',],
          disagreements: [12,],
        },),).toEqual({ differs: false, undecidedBlocks: 1, },);
      },
    },),
    it({
      name: 'SAYS NOTHING OF A BLOCK THAT HOLDS A DISAGREEMENT AND SHOWS WHAT THE BLOCKS OF THE PAGE SHOW',
      fn: async () => {
        expect(blockDifferenceOf({
          sourceBlocks: [blockAt({ account: 'tabby', at: 0, },), blockAt({ account: 'a b', at: 1, },)],
          pageBlocks: ['tabby', 'a', 'b',],
          disagreements: [10,],
        },),).toEqual({ differs: false, undecidedBlocks: 0, },);
      },
    },),
    it({
      name: 'SAYS THE PAGE DIFFERS WHEN IT HAS A BLOCK THE SOURCE HAS NONE FOR, between two blocks that hold no disagreement',
      fn: async () => {
        expect(blockDifferenceOf({
          sourceBlocks: [blockAt({ account: 'tabby', at: 0, },), blockAt({ account: 'calico', at: 1, },),],
          pageBlocks: ['tabby', 'extra', 'calico',],
          disagreements: [],
        },),).toEqual({ differs: true, undecidedBlocks: 0, },);
        expect(blockDifferenceOf({
          sourceBlocks: [blockAt({ account: 'tabby', at: 0, },),],
          pageBlocks: ['tabby', 'extra',],
          disagreements: [],
        },),).toEqual({ differs: true, undecidedBlocks: 0, },);
      },
    },),
    it({
      name: 'LOOKS FOR EACH BLOCK THAT HOLDS NO DISAGREEMENT AFTER THE LAST ONE IT FOUND, so that two blocks alike are two',
      fn: async () => {
        expect(blockDifferenceOf({
          sourceBlocks: [blockAt({ account: 'tabby', at: 0, },), blockAt({ account: 'tabby', at: 1, },),],
          pageBlocks: ['tabby',],
          disagreements: [],
        },),).toEqual({ differs: true, undecidedBlocks: 0, },);
      },
    },),
    it({
      name: 'COUNTS EVERY BLOCK UNDECIDED WHEN A DISAGREEMENT STANDS IN NO BLOCK, since none can be set apart',
      fn: async () => {
        expect(blockDifferenceOf({
          sourceBlocks: [blockAt({ account: 'tabby', at: 0, },), blockAt({ account: 'calico', at: 1, },),],
          pageBlocks: ['tabby', 'calico!',],
          disagreements: [7,],
        },),).toEqual({ differs: false, undecidedBlocks: 2, },);
      },
    },),
  ],
},);

await describe({
  name: blocksOfSource.name,
  children: [
    it({
      name: 'GIVES WHAT EACH TOP-LEVEL BLOCK SHOWS AND WHERE IT STANDS IN THE TEXT',
      fn: async () => {
        expect(blocksOfSource({ source: '她睡在柜台上。\n\n> 猫 [^1]\n', },),).toEqual([
          {
            account: '她睡在柜台上。',
            start: 0,
            end: 7,
          },
          {
            account: '猫 [^1]',
            start: 9,
            end: 17,
          },
        ],);
      },
    },),
    it({
      name: 'GIVES THE OFFSETS OF A TEXT THAT OPENS WITH A BYTE ORDER MARK IN THE TEXT, not in the note and the text',
      fn: async () => {
        /**
         The blocks of a text led by the mark.
         */
        const blocks = blocksOfSource({ source: '\uFEFF她睡。\n\n猫\n', },);
        expect(blocks.map(function startOf({ start, },): number {
          return start;
        },),).toEqual([0, 6,],);
      },
    },),
  ],
},);

await describe({
  name: blocksOfPage.name,
  children: [
    it({
      name: 'GIVES WHAT EACH TOP-LEVEL BLOCK OF THE PAGE SHOWS AFTER THE NOTE',
      fn: async () => {
        expect(blocksOfPage({ carried: '她睡在柜台上。\n\n猫 \\{\n', },),).toEqual(['她睡在柜台上。', '猫 {',],);
      },
    },),
  ],
},);
