/**
 Tests for the footnote integrity signal the chunk gate compares.
 Every finding kind counts alike, a clean document counts zero, and the count
 rises with damage, which is the only property the gate reads.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  footnoteBreakCount,
  parseDocument,
} from '../dist/final/node/index.mjs';

/**
 Count for one invented document, parsed the way the pipeline parses.
 */
function countFor({ text, }: { readonly text: string; },): number {
  return footnoteBreakCount({ document: parseDocument({ text, },), },);
}

await describe({
  name: footnoteBreakCount.name,
  children: [
    //region Clean documents

    it({
      name: 'counts zero for a document carrying no footnotes at all',
      fn: async () => {
        expect(countFor({ text: '小猫在窗台上打盹。\n', },),).toBe(0,);
      },
    },),
    it({
      name: 'counts zero for a reference its definition answers',
      fn: async () => {
        expect(
          countFor({ text: '小猫喜欢鱼干[^1]。\n\n[^1]: 尤其是鲣鱼味的。\n', },),
        ).toBe(0,);
      },
    },),

    //endregion Clean documents

    //region Every kind counts alike

    it({
      name: 'counts a reference no definition answers',
      fn: async () => {
        expect(countFor({ text: '小猫喜欢鱼干[^1]。\n', },),).toBe(1,);
      },
    },),
    it({
      name: 'counts a definition no reference cites',
      fn: async () => {
        expect(
          countFor({ text: '小猫在打盹。\n\n[^1]: 尤其是鲣鱼味的。\n', },),
        ).toBe(1,);
      },
    },),
    it({
      name: 'counts BOTH definitions when one label is defined twice, not just the later one',
      fn: async () => {
        expect(
          countFor({ text: '小猫喜欢鱼干[^1]。\n\n[^1]: 鲣鱼味。\n\n[^1]: 又一次。\n', },),
        ).toBe(2,);
      },
    },),
    it({
      name: 'sums across kinds, since the gate treats them as one signal',
      fn: async () => {
        expect(
          countFor({
            text: '小猫喜欢鱼干[^1]，也喜欢晒太阳[^2]。\n\n[^3]: 没有人引用。\n',
          },),
        ).toBe(3,);
      },
    },),

    //endregion Every kind counts alike

    //region The property the gate reads

    it({
      name: 'RISES when damage is added, which is what makes the comparison a gate',
      fn: async () => {
        /**
         Translation arriving with one dangling reference already.
         */
        const arrived = '小猫喜欢鱼干[^1]。\n\n[^1]: 鲣鱼味。\n\n老猫也在打盹[^2]。\n';

        /**
         Same text after a patch dropped the definition it had.
         */
        const damaged = '小猫喜欢鱼干[^1]。\n\n老猫也在打盹[^2]。\n';
        expect(countFor({ text: arrived, },),).toBe(1,);
        expect(countFor({ text: damaged, },),).toBe(2,);
        expect(countFor({ text: damaged, },) > countFor({ text: arrived, },),).toBe(true,);
      },
    },),
    it({
      name: 'stays EQUAL when a patch leaves an inherited break alone, so repair is not blocked',
      fn: async () => {
        /**
         Dangling reference the translation arrived with.
         */
        const arrived = '小猫喜欢鱼干[^1]。\n\n老猫在打盹。\n';

        /**
         Same break, with unrelated prose repaired around it.
         */
        const repaired = '小猫喜欢鱼干[^1]。\n\n老猫在窗台上打盹。\n';
        expect(countFor({ text: repaired, },),).toBe(countFor({ text: arrived, },),);
      },
    },),

    //endregion The property the gate reads
  ],
},);
