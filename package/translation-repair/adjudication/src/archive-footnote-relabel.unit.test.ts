/**
 Tests for the archive's footnote labels following the original's.

 THE NINETEENTH CLASS, found on one page of 2026-09-08: the original writes
 two names with their notes numbered against first appearance, the archive
 renumbered them by first appearance with definitions to match, and the page
 shipped the original's markers above the archive's definitions, each
 pointing at the other's note.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  footnoteRelabelOf,
  footnoteRelabelOfDefinitions,
} from '@monochromatic-dev/translation-repair-corpus-source';
import { applyFootnoteRelabel, } from '@monochromatic-dev/translation-repair-page-pass';
import { prepareDocumentPair, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Original: the old cat who took her in is the second note, the kitten next door the first.
 */
const SOURCE_TEXT = '## 生平\n\n团团[^2]收留了她，豆豆[^1]陪伴她。\n\n[^1]: 隔壁的小猫，总爱跟着她。\n\n[^2]: 收留她的老猫，像妈妈一样。\n';

/**
 Archive: renumbered by first appearance, definitions to match.
 */
const TARGET_TEXT = '## Life\n\nTuantuan[^1] took her in, and Doudou[^2] kept her company.\n\n'
  + '[^1]: The old cat who took her in, like a mother.\n\n[^2]: The kitten next door, always following her.\n';

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: footnoteRelabelOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'maps the archive labels to the original ones position by position within each paired slice, '
            + 'and rewrites references and definitions together',
          fn: async () => {
            /**
             Preparation over the fixture pair.
             */
            const prepared = prepareDocumentPair({
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
            },);
            /**
             The reading.
             */
            const reading = footnoteRelabelOf(prepared,);
            if (reading.kind !== 'relabel')
              throw new Error(`expected a relabel, read ${reading.kind}`,);
            expect(reading.map,).toStrictEqual([
              {
                from: '1',
                to: '2',
              },
              {
                from: '2',
                to: '1',
              },
            ],);
            expect(reading.skipped,).toStrictEqual([],);
            /**
             The archive under the original's labels.
             */
            const relabelled = applyFootnoteRelabel({
              text: TARGET_TEXT,
              map: reading.map,
            },);
            expect(relabelled,).toBe(
              '## Life\n\nTuantuan[^2] took her in, and Doudou[^1] kept her company.\n\n'
                + '[^2]: The old cat who took her in, like a mother.\n\n[^1]: The kitten next door, always following her.\n',
            );
            // A SECOND READING OF THE RELABELLED ARCHIVE CHANGES NOTHING.
            expect(
              footnoteRelabelOf(prepareDocumentPair({
                sourceText: SOURCE_TEXT,
                targetText: relabelled,
              },),),
            ).toStrictEqual({
              kind: 'unchanged',
              correspondences: [{ from: '2', to: '2', }, { from: '1', to: '1', },],
              skipped: [],
            },);
          },
        },),

        it({
          name: 'reads nothing to change where the labels already agree or no slice carries a marker on both sides',
          fn: async () => {
            expect(
              footnoteRelabelOf(prepareDocumentPair({
                sourceText: '她[^1]。\n\n[^1]: 注。\n',
                targetText: 'She[^1].\n\n[^1]: Note.\n',
              },),),
            ).toStrictEqual({
              kind: 'unchanged',
              correspondences: [{ from: '1', to: '1', },],
              skipped: [],
            },);
            expect(
              footnoteRelabelOf(prepareDocumentPair({
                sourceText: '她。\n',
                targetText: 'She.\n',
              },),),
            ).toStrictEqual({
              kind: 'unchanged',
              correspondences: [],
              skipped: [],
            },);
          },
        },),

        it({
          name: 'leaves a slice whose two sides reference different counts of notes out of the reading, naming it, '
            + 'and reads the map off the rest (one archive carries no [^2] at all)',
          fn: async () => {
            /**
             The reading over a slice with one marker against two and a slice
             with a swap.
             */
            const reading = footnoteRelabelOf(prepareDocumentPair({
              sourceText: '## 甲\n\n她[^1]和他[^2]。\n\n## 乙\n\n团团[^4]，豆豆[^3]。\n\n'
                + '[^1]: 一。\n\n[^2]: 二。\n\n[^3]: 三。\n\n[^4]: 四。\n',
              targetText: '## A\n\nShe[^1] and he.\n\n## B\n\nTuantuan[^3], Doudou[^4].\n\n'
                + '[^1]: One.\n\n[^3]: Three.\n\n[^4]: Four.\n',
            },),);
            if (reading.kind !== 'relabel')
              throw new Error(`expected a relabel, read ${reading.kind}`,);
            expect(reading.map,).toStrictEqual([
              {
                from: '3',
                to: '4',
              },
              {
                from: '4',
                to: '3',
              },
            ],);
            expect(reading.skipped
              .length,).toBe(1,);
            expect(reading.skipped[0],).toContain('2 distinct notes in the original and 1 in the archive',);
          },
        },),

        it({
          name: 'leaves the archive as it is, naming the slice, where two slices map one archive label to different '
            + 'original labels',
          fn: async () => {
            /**
             The reading where [^1] is [^2] in one slice and [^3] in the next.
             */
            const reading = footnoteRelabelOf(prepareDocumentPair({
              sourceText: '## 甲\n\n她[^2]。\n\n## 乙\n\n他[^3]。\n\n[^2]: 二。\n\n[^3]: 三。\n',
              targetText: '## A\n\nShe[^1].\n\n## B\n\nHe[^1].\n\n[^1]: One.\n',
            },),);
            expect(reading,).toStrictEqual({
              kind: 'ambiguous',
              detail: 'slice 1 maps archive [^1] to original [^3], where slice 0 mapped that archive label to original [^2]',
            },);
          },
        },),

        it({
          name: 'NAMES WHICH SIDE AND WHICH SLICE an earlier claim held, in the spelling the document carries: two '
            + 'archive labels claiming one original label once read "where an earlier slice mapped [^1]", an archive '
            + 'label in the place where an original label stands for the other conflict, and a label the parser '
            + 'case-folds read in its folded form, which neither document spells',
          fn: async () => {
            /**
             The reading where archive [^1] and [^3] both meet original [^2].
             */
            const twoOntoOne = footnoteRelabelOf(prepareDocumentPair({
              sourceText: '## 甲\n\n她[^2]。\n\n## 乙\n\n他[^2]。\n\n[^2]: 二。\n',
              targetText: '## A\n\nShe[^1].\n\n## B\n\nHe[^3].\n\n[^1]: One.\n\n[^3]: Three.\n',
            },),);
            expect(twoOntoOne,).toStrictEqual({
              kind: 'ambiguous',
              detail: 'slice 1 maps archive [^3] to original [^2], where slice 0 mapped archive [^1] to that original label',
            },);
            /**
             The reading where archive [^1] meets original [^Tabby] and then
             [^Ginger].
             */
            const oneOntoTwo = footnoteRelabelOf(prepareDocumentPair({
              sourceText: '## 甲\n\n她[^Tabby]。\n\n## 乙\n\n他[^Ginger]。\n\n[^Tabby]: 虎斑。\n\n[^Ginger]: 橘猫。\n',
              targetText: '## A\n\nShe[^1].\n\n## B\n\nHe[^1].\n\n[^1]: One.\n',
            },),);
            expect(oneOntoTwo,).toStrictEqual({
              kind: 'ambiguous',
              detail: 'slice 1 maps archive [^1] to original [^Ginger], where slice 0 mapped that archive label to '
                + 'original [^Tabby]',
            },);
          },
        },),
      ],
    },),

    describe({
      name: footnoteRelabelOfDefinitions.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads the map off the definitions the roster paired by content, and nothing off none',
          fn: async () => {
            expect(footnoteRelabelOfDefinitions({
              pairs: [
                {
                  sourceLabel: '1',
                  targetLabel: '2',
                },
                {
                  sourceLabel: '2',
                  targetLabel: '1',
                },
              ],
            },),).toStrictEqual({
              kind: 'relabel',
              correspondences: [{ from: '2', to: '1', }, { from: '1', to: '2', },],
              map: [
                {
                  from: '2',
                  to: '1',
                },
                {
                  from: '1',
                  to: '2',
                },
              ],
              skipped: [],
            },);
            expect(footnoteRelabelOfDefinitions({ pairs: [], },),).toStrictEqual({
              kind: 'unchanged',
              correspondences: [],
              skipped: [],
            },);
          },
        },),

        it({
          name: 'leaves the archive as it is where two pairs map one archive label to different original labels',
          fn: async () => {
            /**
             The reading where archive [^1] pairs with original [^2] and then [^3].
             */
            const reading = footnoteRelabelOfDefinitions({
              pairs: [
                {
                  sourceLabel: '2',
                  targetLabel: '1',
                },
                {
                  sourceLabel: '3',
                  targetLabel: '1',
                },
              ],
            },);
            expect(reading,).toStrictEqual({
              kind: 'ambiguous',
              detail: 'definition pair 1 maps archive [^1] to original [^3], where definition pair 0 mapped that '
                + 'archive label to original [^2]',
            },);
          },
        },),
      ],
    },),

    describe({
      name: applyFootnoteRelabel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'rewrites only the mapped labels and leaves every other marker and the text between them alone',
          fn: async () => {
            expect(applyFootnoteRelabel({
              text: 'A[^1] B[^3] C[^1].\n\n[^1]: one\n\n[^3]: three\n',
              map: [ {
                from: '1',
                to: '2',
              }, ],
            },),).toBe('A[^2] B[^3] C[^2].\n\n[^2]: one\n\n[^3]: three\n',);
            expect(applyFootnoteRelabel({
              text: 'No markers.\n',
              map: [],
            },),).toBe('No markers.\n',);
          },
        },),
      ],
    },),
  ],
},);
