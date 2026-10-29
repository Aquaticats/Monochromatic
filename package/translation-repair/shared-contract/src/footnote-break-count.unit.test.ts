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
  restoredFootnoteReferences,
} from '../dist/final/node/index.mjs';

/**
 Count for one invented document, parsed the way the pipeline parses.
 */
function countFor({ text, }: { readonly text: string; },): number {
  return footnoteBreakCount({ document: parseDocument({ text, },), },);
}

/**
 Restored references of one invented patch, given the text of its source, its translation and itself.

 @param sourceText - original-language text of the slice

 @param targetText - translation before the patch

 @param patchedText - translation after the patch

 @returns Restored findings as `convention:identifier` strings, in graph order

 @example
 ```ts
 const restored = restoredFor({ sourceText: 'a[^3]', targetText: 'a', patchedText: 'a[^3]', },); // ['gfm:3']
 ```
 */
function restoredFor(
  {
    sourceText,
    targetText,
    patchedText,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly patchedText: string;
  },
): readonly string[] {
  return restoredFootnoteReferences({
    patchedDocument: parseDocument({ text: patchedText, },),
    targetDocument: parseDocument({ text: targetText, },),
    sourceReferences: parseDocument({ text: sourceText, },).footnoteGraph
      .references,
  },).map(function toLabel(finding,): string {
    return `${finding.convention}:${finding.identifier}`;
  },);
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

await describe({
  name: restoredFootnoteReferences.name,
  children: [
    it({
      name: 'names an unresolved reference the source carries and the translation lacked',
      fn: async () => {
        expect(
          restoredFor({
            sourceText: '小猫在窗台上打盹[^3]。\n',
            targetText: 'Whiskers naps on the sill.\n',
            patchedText: 'Whiskers naps on the sill[^3].\n',
          },),
        ).toStrictEqual(['gfm:3',],);
      },
    },),
    it({
      name: 'names nothing for a reference the source does not carry',
      fn: async () => {
        expect(
          restoredFor({
            sourceText: '小猫在窗台上打盹[^3]。\n',
            targetText: 'Whiskers naps on the sill.\n',
            patchedText: 'Whiskers naps on the sill[^4].\n',
          },),
        ).toStrictEqual([],);
      },
    },),
    it({
      name: 'compares labels by their folded key, so `[^Note]` in the source matches `[^NOTE]` in the patch',
      fn: async () => {
        expect(
          restoredFor({
            sourceText: '小猫在窗台上打盹[^Note]。\n',
            targetText: 'Whiskers naps on the sill.\n',
            patchedText: 'Whiskers naps on the sill[^NOTE].\n',
          },),
        ).toStrictEqual(['gfm:note',],);
      },
    },),
    it({
      name: 'names every mention of a restored key, since the allowance is per key and not per mention',
      fn: async () => {
        expect(
          restoredFor({
            sourceText: '小猫[^3]在窗台上打盹[^3]。\n',
            targetText: 'Whiskers naps on the sill.\n',
            patchedText: 'Whiskers[^3] naps on the sill[^3] and the rug[^3].\n',
          },),
        ).toStrictEqual(['gfm:3', 'gfm:3', 'gfm:3',],);
      },
    },),
    it({
      name: 'names nothing for a key the translation already carried, resolved or not',
      fn: async () => {
        expect(
          restoredFor({
            sourceText: '小猫在窗台上打盹[^3]。\n',
            targetText: 'Whiskers naps on the sill[^3].\n',
            patchedText: 'Whiskers naps on the sill[^3] and the rug[^3].\n',
          },),
        ).toStrictEqual([],);
      },
    },),
    it({
      name: 'never names an orphan or a duplicated definition, whose key the source also carries',
      fn: async () => {
        expect(
          restoredFor({
            sourceText: '小猫在窗台上打盹[^3]。\n\n[^3]: 窗台。\n',
            targetText: 'Whiskers naps on the sill.\n',
            patchedText: 'Whiskers naps on the sill.\n\n[^3]: Sill.\n\n[^3]: Sill again.\n',
          },),
        ).toStrictEqual([],);
      },
    },),
    it({
      name: 'keys the two syntax families apart, so a full-width marker in the source restores no `[^3]`',
      fn: async () => {
        expect(
          restoredFor({
            sourceText: '小猫在窗台上打盹〔3〕。\n',
            targetText: 'Whiskers naps on the sill.\n',
            patchedText: 'Whiskers naps on the sill[^3].\n',
          },),
        ).toStrictEqual([],);
        expect(
          restoredFor({
            sourceText: '小猫在窗台上打盹〔3〕。\n',
            targetText: 'Whiskers naps on the sill.\n',
            patchedText: 'Whiskers naps on the sill〔3〕.\n',
          },),
        ).toStrictEqual(['fullwidth-bracket:3',],);
      },
    },),
  ],
},);
