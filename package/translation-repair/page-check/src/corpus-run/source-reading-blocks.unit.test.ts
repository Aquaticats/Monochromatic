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
  name: 'the block by block comparison of a page with its source',
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
          disagreements: [{ start: 10, end: 11, },],
        },),).toEqual({ differs: true, undecidedBlocks: 1, },);
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['tabby', 'calico!',],
          disagreements: [{ start: 10, end: 11, },],
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
          disagreements: [{ start: 10, end: 11, },],
        },),).toEqual({ differs: false, undecidedBlocks: 1, },);
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['tabby', '[^1]: x', 'other', 'calico',],
          disagreements: [{ start: 12, end: 13, },],
        },),).toEqual({ differs: false, undecidedBlocks: 1, },);
      },
    },),
    it({
      name: 'SAYS NOTHING OF A BLOCK THAT HOLDS A DISAGREEMENT AND SHOWS WHAT THE BLOCKS OF THE PAGE SHOW',
      fn: async () => {
        expect(blockDifferenceOf({
          sourceBlocks: [blockAt({ account: 'tabby', at: 0, },), blockAt({ account: 'a b', at: 1, },)],
          pageBlocks: ['tabby', 'a', 'b',],
          disagreements: [{ start: 10, end: 11, },],
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
      name: 'SETS EVERY BLOCK A DISAGREEING RANGE RUNS OVER AGAINST THE PAGE, not only the block that holds its start',
      fn: async () => {
        /**
         A range that starts in the second block and runs over the third, the page having one block for the two.
         */
        const sourceBlocks = [
          blockAt({ account: 'tabby', at: 0, },),
          blockAt({ account: '[^1]: x', at: 1, },),
          blockAt({ account: '<script>', at: 2, },),
        ];
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['tabby', '[^1]: x [r]: u <script>',],
          disagreements: [{ start: 10, end: 22, },],
        },),).toEqual({ differs: false, undecidedBlocks: 2, },);
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['tabby', '[^1]: x [r]: u <script>',],
          disagreements: [{ start: 10, end: 11, },],
        },),).toEqual({ differs: false, undecidedBlocks: 1, },);
      },
    },),
    it({
      name: 'READS THE WORDS OF A BLOCK NO RANGE REACHES WHEREVER THE PAGE SHOWS THEM, in one block or in several',
      fn: async () => {
        /**
         A block a range reaches and a block of two words, which the page shows in two blocks of one word each.
         */
        const sourceBlocks = [blockAt({ account: 'tabby', at: 0, },), blockAt({ account: 'x y', at: 1, },),];
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['tabby!', 'x', 'y',],
          disagreements: [{ start: 0, end: 1, },],
        },),).toEqual({ differs: false, undecidedBlocks: 1, },);
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['tabby!', 'x', 'z',],
          disagreements: [{ start: 0, end: 1, },],
        },),).toEqual({ differs: true, undecidedBlocks: 1, },);
      },
    },),
    it({
      name: 'PLACES THE WORDS OF A BLOCK WHERE THE BLOCKS AFTER IT CAN BE PLACED, not at the first place the same words stand',
      fn: async () => {
        /**
         A block a range reaches whose words end with a tag, the block after it that is the same tag, and one more block.
         */
        const sourceBlocks = [
          blockAt({ account: 'quote <a>', at: 0, },),
          blockAt({ account: '<a>', at: 1, },),
          blockAt({ account: 'end', at: 2, },),
        ];
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['quote <a> [r]:', '<a>', 'end',],
          disagreements: [{ start: 0, end: 1, },],
        },),).toEqual({ differs: false, undecidedBlocks: 1, },);
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['quote <a> [r]:', '<a>', 'end', 'more',],
          disagreements: [{ start: 0, end: 1, },],
        },),).toEqual({ differs: true, undecidedBlocks: 1, },);
      },
    },),
    it({
      name: 'PLACES THE WORDS OF TWO BLOCKS ALIKE AT TWO PLACES OF THE PAGE, with a block that shows nothing and a range reaches between them',
      fn: async () => {
        /**
         Two blocks of the same word with a block between them that shows nothing and that a range reaches, the page
         showing the word once.
         */
        const sourceBlocks = [
          blockAt({ account: 'tabby', at: 0, },),
          blockAt({ account: '', at: 1, },),
          blockAt({ account: 'tabby', at: 2, },),
        ];
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['tabby',],
          disagreements: [{ start: 10, end: 11, },],
        },),).toEqual({ differs: true, undecidedBlocks: 1, },);
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['tabby', 'tabby',],
          disagreements: [{ start: 10, end: 11, },],
        },),).toEqual({ differs: false, undecidedBlocks: 0, },);
      },
    },),
    it({
      name: 'DOES NOT SET THE BLOCKS A RANGE REACHES APART WHERE A BLOCK THAT SHOWS NOTHING AND NO RANGE REACHES STANDS BETWEEN THEM',
      fn: async () => {
        /**
         A block a range reaches, a block that shows nothing and none reaches, and a block that shows nothing and one does,
         the page showing nothing.
         */
        const sourceBlocks = [
          blockAt({ account: 'tabby', at: 0, },),
          blockAt({ account: '', at: 1, },),
          blockAt({ account: '', at: 2, },),
        ];
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: [],
          disagreements: [{ start: 0, end: 1, }, { start: 20, end: 21, },],
        },),).toEqual({ differs: false, undecidedBlocks: 2, },);
      },
    },),
    it({
      name: 'COUNTS THE FEWEST BLOCKS THE READINGS CANNOT BE ASKED ABOUT OVER EVERY WAY THE WORDS CAN BE PLACED',
      fn: async () => {
        /**
         A block a range reaches, a block no range reaches that shows the same word, and a block a range reaches that shows
         nothing, the page showing the word twice: the word of the second block stands at either place, and only the second
         place shows the first block what it shows.
         */
        const sourceBlocks = [
          blockAt({ account: 'tabby', at: 0, },),
          blockAt({ account: 'tabby', at: 1, },),
          blockAt({ account: '', at: 2, },),
        ];
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: ['tabby', 'tabby',],
          disagreements: [{ start: 0, end: 1, }, { start: 20, end: 21, },],
        },),).toEqual({ differs: false, undecidedBlocks: 0, },);
      },
    },),
    it({
      name: 'PLACES A BLOCK THAT NO RANGE REACHES AT THE PLACE WHERE THE BLOCK A RANGE REACHES ENDS, the same word twice',
      fn: async () => {
        /**
         A block a range reaches and a block after it that shows the same word, the page showing the word once: the second
         block stands where the first ends and the first has no words to match, so one block stays undecided.
         */
        expect(blockDifferenceOf({
          sourceBlocks: [blockAt({ account: 'tabby', at: 0, },), blockAt({ account: 'tabby', at: 1, },),],
          pageBlocks: ['tabby',],
          disagreements: [{ start: 0, end: 1, },],
        },),).toEqual({ differs: false, undecidedBlocks: 1, },);
      },
    },),
    it({
      name: 'TAKES THE CHEAPEST OF THE PLACES THE BLOCKS BEFORE CAN END AT FOR A BLOCK THAT FOLLOWS ONE A RANGE REACHES',
      fn: async () => {
        /**
         A block that shows nothing and a range reaches, a word, a block a range reaches, and the word of the last block
         twice, the page showing words that fit one way of placing them only: the cheapest way leaves one block undecided.
         */
        expect(blockDifferenceOf({
          sourceBlocks: [
            blockAt({ account: '', at: 0, },),
            blockAt({ account: 'calico', at: 1, },),
            blockAt({ account: 'tabby', at: 2, },),
            blockAt({ account: 'tabby', at: 3, },),
          ],
          pageBlocks: ['calico calico tabby',],
          disagreements: [{ start: 0, end: 1, }, { start: 20, end: 21, },],
        },),).toEqual({ differs: false, undecidedBlocks: 1, },);
      },
    },),
    it({
      name: 'ANSWERS FOR A PAGE OF MANY BLOCKS OF THE SAME WORDS, which the placing of each block must not set against every other',
      fn: async () => {
        /**
         Eight hundred blocks of the same two words, every fourth reached by a range, and the page showing the same blocks:
         the placing took seconds before it read the places in order and minutes at sixteen hundred blocks.
         */
        const sourceBlocks = Array.from({ length: 800, }, function block(
          _,
          at,
        ): { account: string; start: number; end: number; } {
          return blockAt({ account: 'tabby calico', at, },);
        },);
        expect(blockDifferenceOf({
          sourceBlocks,
          pageBlocks: sourceBlocks.map(function account(): string {
            return 'tabby calico';
          },),
          disagreements: Array.from({ length: 200, }, function range(
            _,
            at,
          ): { start: number; end: number; } {
            return { start: (at * 40) + 1, end: (at * 40) + 2, };
          },),
        },),).toEqual({ differs: false, undecidedBlocks: 0, },);
      },
    },),
    it({
      name: 'COUNTS EVERY BLOCK UNDECIDED WHEN A DISAGREEMENT STANDS IN NO BLOCK, since none can be set apart',
      fn: async () => {
        expect(blockDifferenceOf({
          sourceBlocks: [blockAt({ account: 'tabby', at: 0, },), blockAt({ account: 'calico', at: 1, },),],
          pageBlocks: ['tabby', 'calico!',],
          disagreements: [{ start: 6, end: 9, },],
        },),).toEqual({ differs: false, undecidedBlocks: 2, },);
      },
    },),
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
    it({
      name: 'GIVES WHAT EACH TOP-LEVEL BLOCK OF THE PAGE SHOWS AFTER THE NOTE',
      fn: async () => {
        expect(blocksOfPage({ carried: '她睡在柜台上。\n\n猫 \\{\n', },),).toEqual(['她睡在柜台上。', '猫 {',],);
      },
    },),
  ],
},);
