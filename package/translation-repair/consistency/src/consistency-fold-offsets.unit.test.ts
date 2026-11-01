/**
 Tests for the fold that keeps its offsets: curly quotation marks straightened, a run of whitespace folded to one space and
 a blank line to two line breaks, and for each folded unit the stretch of the original it stands for.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { foldWithOffsets, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: foldWithOffsets.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STRAIGHTENS A CURLY MARK AND FOLDS A RUN OF WHITESPACE TO ONE SPACE, each unit standing for its stretch of the '
            + 'original, the character before the fold among them',
          fn: async () => {
            const { folded, starts, ends, originals, } = foldWithOffsets({ text: 'girl,\nwasn’t  she', },);
            expect({
              folded,
              newline: [starts[5], ends[5], originals[5],],
              mark: [folded[10], starts[10], ends[10], originals[10],],
              run: [starts[12], ends[12], originals[12],],
              lengths: [starts.length, ends.length, originals.length,],
            },).toEqual({
              folded: 'girl, wasn\'t she',
              newline: [5, 6, '\n',],
              mark: ['\'', 10, 11, '’',],
              run: [12, 14, '  ',],
              lengths: [16, 16, 16,],
            },);
          },
        },),
        it({
          name: 'FOLDS A RUN OF WHITESPACE WITH TWO LINE BREAKS TO A BLANK LINE, two units standing for the whole run, and a run '
            + 'with one to a space',
          fn: async () => {
            const blank = foldWithOffsets({ text: 'a\n \nb', },);
            const soft = foldWithOffsets({ text: 'a \nb', },);
            expect({
              blank: [blank.folded, blank.starts, blank.ends,],
              soft: [soft.folded, soft.starts, soft.ends,],
            },).toEqual({
              blank: ['a\n\nb', [0, 1, 1, 4,], [1, 4, 4, 5,],],
              soft: ['a b', [0, 1, 3,], [1, 3, 4,],],
            },);
          },
        },),
        it({
          name: 'GIVES EACH UTF-16 UNIT OF A CHARACTER THE WHOLE CHARACTER AS ITS STRETCH, and closes a run of whitespace at the '
            + 'end of the text',
          fn: async () => {
            const cat = foldWithOffsets({ text: '😺 cat  ', },);
            expect({
              folded: cat.folded,
              starts: cat.starts,
              ends: cat.ends,
              originals: cat.originals,
            },).toEqual({
              folded: '😺 cat ',
              starts: [0, 0, 2, 3, 4, 5, 6,],
              ends: [2, 2, 3, 4, 5, 6, 8,],
              originals: ['😺', '😺', ' ', 'c', 'a', 't', '  ',],
            },);
            expect(foldWithOffsets({ text: '', },),).toEqual({ folded: '', starts: [], ends: [], originals: [], },);
          },
        },),
      ],
    },),
  ],
},);
