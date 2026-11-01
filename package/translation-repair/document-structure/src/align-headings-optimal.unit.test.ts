/**
 Tests for reading every optimal alignment out of the heading table.

 WHY THIS ONE IS WORTH PINNING DIRECTLY. It is known what a wrong answer here
 costs: facing 14 source sections and 12 target ones, the old aligner slid
 every pairing by two, so every critic call afterwards compared the wrong
 original against the wrong translation and every issue it filed was noise. A
 gap belongs where it is, and the sections around it keep their partners.

 The other half is WIDTH. This is exported so a probe can ask how wide an
 ambiguity is rather than only that there was one, and hesitating between two
 adjacent boundaries wants a different remedy from hesitating across a page.
 Width shows up as a partner set larger than one.

 THIS TABLE ANSWERS "what does SOME optimal alignment do", not "what may we
 claim". Two headings sharing nothing still pair here, and the caller decides
 whether that pairing is admissible. A case at the end pins that boundary so a
 reader does not mistake this for the policy.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { scanOptimalPaths, } from '../dist/final/node/index.mjs';
import { listed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/sorted-number-sets.test-fixture.ts';

/**
 Three sections a document might carry.
 */
const THREE = [
  'Sill',
  'Bowl',
  'Door',
];

await describe({
  name: scanOptimalPaths.name,
  children: [
    it({
      name:
        'POSITIVE CONTROL: pairing is not always by index, so a scan that answered with the identity '
          + 'mapping would pass the "pairs by index when both sides carry the same sections" case and fail '
          + 'here. With the MIDDLE target gone, '
          + 'source 2 pairs with target 1',
      fn: async () => {
        expect(listed({
          sets: scanOptimalPaths({
            sourceHeadings: THREE,
            targetHeadings: [
              'Sill',
              'Door',
            ],
          },).partnersOfSource,
        },),).toEqual([
          [0,],
          [],
          [1,],
        ],);
      },
    },),

    it({
      name:
        'pairs by index when both sides carry the same sections, with no gaps on either side, which is '
          + 'the shape most of the corpus has and the one every other case is a departure from',
      fn: async () => {
        /**
         What the table says about two identical sides.
         */
        const paths = scanOptimalPaths({
          sourceHeadings: THREE,
          targetHeadings: THREE,
        },);

        expect(listed({ sets: paths.partnersOfSource, },),).toEqual([
          [0,],
          [1,],
          [2,],
        ],);
        expect(listed({ sets: paths.sourceGapColumns, },),).toEqual([
          [],
          [],
          [],
        ],);
        expect(paths.targetCanGap,).toEqual([
          false,
          false,
          false,
        ],);
      },
    },),

    it({
      name:
        'leaves the earlier pairs alone when the LAST section is missing, and records the gap at the '
          + 'END column. This is the section-alignment defect exactly: a gap at the end used to slide every pairing before it, '
          + 'so a document lost its alignment from the first section rather than the last',
      fn: async () => {
        /**
         What the table says when the translation stops early.
         */
        const paths = scanOptimalPaths({
          sourceHeadings: THREE,
          targetHeadings: [
            'Sill',
            'Bowl',
          ],
        },);

        expect(listed({ sets: paths.partnersOfSource, },),).toEqual([
          [0,],
          [1,],
          [],
        ],);
        expect(listed({ sets: paths.sourceGapColumns, },),).toEqual([
          [],
          [],
          [2,],
        ],);
      },
    },),

    it({
      name:
        'records the gap at the MIDDLE column when the middle section is missing, so an insertion for '
          + 'it lands between its neighbours rather than at the end: knowing a section is unpaired and '
          + 'not knowing where it belongs is half an answer, and insertion cannot proceed on half',
      fn: async () => {
        expect(listed({
          sets: scanOptimalPaths({
            sourceHeadings: THREE,
            targetHeadings: [
              'Sill',
              'Door',
            ],
          },).sourceGapColumns,
        },),).toEqual([
          [],
          [1,],
          [],
        ],);
      },
    },),

    it({
      name:
        'reports a section the TRANSLATION has and the original does not as a target that may gap, '
          + 'rather than pairing it with a source section that means something else',
      fn: async () => {
        /**
         What the table says when the translation carries an extra section.
         */
        const paths = scanOptimalPaths({
          sourceHeadings: [
            'Sill',
            'Door',
          ],
          targetHeadings: THREE,
        },);

        expect(paths.targetCanGap,).toEqual([
          false,
          true,
          false,
        ],);
        expect(listed({ sets: paths.partnersOfSource, },),).toEqual([
          [0,],
          [2,],
        ],);
      },
    },),

    it({
      name:
        'reports AMBIGUITY AS WIDTH rather than picking one of the tied paths: two identical source '
          + 'sections against one target give that target both of them as partners, which is what lets a '
          + 'caller tell a hesitation between neighbours from a hesitation across a page',
      fn: async () => {
        /**
         What the table says when two sections are indistinguishable.
         */
        const paths = scanOptimalPaths({
          sourceHeadings: [
            'Nap',
            'Nap',
          ],
          targetHeadings: ['Nap',],
        },);

        expect(listed({ sets: paths.partnersOfTarget, },),).toEqual([[
          0,
          1,
        ],],);

        // AND EACH SOURCE CAN ALSO GAP, at its own column, because on the path
        // where the other one pairs this one is skipped. A caller reading only
        // the partners would see two sections claiming one target and no way to
        // put the loser anywhere.
        expect(listed({ sets: paths.sourceGapColumns, },),).toEqual([
          [0,],
          [1,],
        ],);
      },
    },),

    it({
      name:
        'answers for an empty side without raising, reporting the other side`s units as unpaired: a '
          + 'document with no headings at all is a shape the corpus has, and it is not a malformed input',
      fn: async () => {
        /**
         Original with nothing to pair.
         */
        const noSource = scanOptimalPaths({
          sourceHeadings: [],
          targetHeadings: ['Sill',],
        },);

        expect(noSource.partnersOfSource,).toEqual([],);
        expect(noSource.targetCanGap,).toEqual([true,],);

        /**
         Translation with nothing to pair.
         */
        const noTarget = scanOptimalPaths({
          sourceHeadings: ['Sill',],
          targetHeadings: [],
        },);

        expect(listed({ sets: noTarget.partnersOfSource, },),).toEqual([[],],);
        expect(listed({ sets: noTarget.sourceGapColumns, },),).toEqual([[0,],],);
      },
    },),

    it({
      name:
        'PAIRS TWO HEADINGS THAT SHARE NOTHING, which is the boundary this file sits on: the table '
          + 'answers what some optimal alignment does, and whether such a pairing may be claimed is the '
          + 'caller`s policy. A reader taking this for the policy would expect a refusal that never comes',
      fn: async () => {
        expect(listed({
          sets: scanOptimalPaths({
            sourceHeadings: ['Sill',],
            targetHeadings: ['Bowl',],
          },).partnersOfSource,
        },),).toEqual([[0,],],);
      },
    },),
  ],
},);
