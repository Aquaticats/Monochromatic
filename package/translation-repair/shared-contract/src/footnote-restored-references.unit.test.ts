/**
 Tests for the restoration reading of the footnote integrity count:
 which unresolved references of a patched chunk are the patch putting back a reference the slice's own source carries
 and the translation had dropped (run XingZ6022 slice 44, `[^3]`, definition in another chunk).
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parseDocument,
  restoredFootnoteReferences,
} from '../dist/final/node/index.mjs';

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
