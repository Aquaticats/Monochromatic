/**
 Tests for the floors a changed slice passes before the consistency pass lets
 it stand: the ground the publication rule can read, the rule itself, the size
 the original can account for, the quoted passages and the declared names the
 slice carries now.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { DisputedWording, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  type ConsistencyFloorVerdict,
  consistencyFloorVerdict,
} from '../../dist/final/node/index.mjs';

/**
 The original of the slice every case reads.
 */
const SOURCE = '## 一\n\n窗台上的阳光让小猫睡着了。';

/**
 The English the slice carries now.
 */
const CURRENT = '## One\n\nThe sunny window put the kitten to sleep.';

/**
 Runs the floors over the slice with the fields a case replaces.

 @param over - fields this case replaces

 @returns What the floors found

 @example
 ```ts
 const verdict = floorsWith({ candidateText: CURRENT, },);
 ```
 */
function floorsWith(
  {
    sourceText = SOURCE,
    currentText = CURRENT,
    candidateText,
    declaredNames = [],
    declaredSubjectForms,
    disputedWordings = [],
  }: {
    readonly sourceText?: string;
    readonly currentText?: string;
    readonly candidateText: string;
    readonly declaredNames?: readonly string[];
    readonly declaredSubjectForms?: readonly string[];
    readonly disputedWordings?: readonly DisputedWording[];
  },
): ConsistencyFloorVerdict {
  return consistencyFloorVerdict({
    sliceIndex: 4,
    sourceText,
    currentText,
    candidateText,
    lineStructured: false,
    declared: [],
    declaredNames,
    ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
    disputedWordings,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: consistencyFloorVerdict.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PASSES A REPLACEMENT THAT LEAVES THE BLOCKS, THE QUOTED PASSAGES AND THE DECLARED NAMES ALONE',
          fn: async () => {
            expect(floorsWith({ candidateText: '## One\n\nThe sunlit window put the kitten to sleep.', },),)
              .toEqual({ kind: 'passes', },);
          },
        },),
        it({
          name: 'REFUSES WHAT THE PUBLICATION RULE REFUSES, with the rule\'s own sentences: a block added',
          fn: async () => {
            expect(floorsWith({
              candidateText: '## One\n\nThe sunlit window put the kitten to sleep.\n\nAnd another paragraph.',
            },),).toEqual({
              kind: 'refused',
              findings: [
                'Your translation is 3 blocks (heading (level 2), paragraph, paragraph) and the PAGE AS IT STANDS is 2. '
                  + 'Add a block only to carry something the ORIGINAL has and the text you are replacing left out.',
              ],
            },);
          },
        },),
        it({
          name: 'REFUSES A WORDING A DISPUTED SLICE REFUSES, since the rule reads the refused wordings beside the blocks',
          fn: async () => {
            const candidateText = '## One\n\nThe sunlit window put the kitten to sleep.';
            const verdict = floorsWith({
              candidateText,
              disputedWordings: [{ text: candidateText, reason: 'the archive rendering the adjudicators disputed', },],
            },);
            expect(verdict.kind,).toBe('refused',);
          },
        },),
        it({
          name: 'REFUSES A SLICE WHOSE CURRENT WORDING IS LONGER THAN ITS ORIGINAL CAN ACCOUNT FOR',
          fn: async () => {
            expect(floorsWith({
              sourceText: '猫。',
              currentText: 'x'.repeat(400,),
              candidateText: 'y'.repeat(400,),
            },),).toEqual({
              kind: 'refused',
              findings: [
                'the wording the slice carries is longer than its original can account for (400 against 2 code points), '
                  + 'so no replacement is made',
              ],
            },);
          },
        },),
        it({
          name: 'REFUSES A REPLACEMENT THAT DROPS A DECLARED NAME the slice carries now, with the guard\'s own sentence',
          fn: async () => {
            expect(floorsWith({
              currentText: '## One\n\nMittens put the kitten to sleep.',
              candidateText: '## One\n\nThe sunlit window put the kitten to sleep.',
              declaredNames: ['Mittens',],
            },),).toEqual({
              kind: 'refused',
              findings: [
                'translate-refused-declared-name (slice 4: archive text carries "Mittens" and the replacement does not; '
                  + 'keeping the archive text)',
              ],
            },);
          },
        },),
        it({
          name: 'PASSES A REPLACEMENT THAT NAMES THE PERSON BY ANOTHER DECLARED FORM of hers, and refuses the same '
            + 'wording when the floor is given no subject identity to read the two forms by',
          fn: async () => {
            expect(floorsWith({
              currentText: '## One\n\nMittens put the kitten to sleep.',
              candidateText: '## One\n\nWhiskers put the kitten to sleep.',
              declaredNames: ['Mittens', 'Whiskers',],
              declaredSubjectForms: ['Mittens', 'Whiskers',],
            },),).toEqual({ kind: 'passes', },);
            expect(floorsWith({
              currentText: '## One\n\nMittens put the kitten to sleep.',
              candidateText: '## One\n\nWhiskers put the kitten to sleep.',
              declaredNames: ['Mittens', 'Whiskers',],
            },),).toEqual({
              kind: 'refused',
              findings: [
                'translate-refused-declared-name (slice 4: archive text carries "Mittens" and the replacement does not; '
                  + 'keeping the archive text)',
              ],
            },);
          },
        },),
        it({
          name: 'REFUSES A REPLACEMENT THAT TURNS A QUOTED PASSAGE INTO PLAIN TEXT, at any depth: the block kinds stand, '
            + 'the passage does not',
          fn: async () => {
            expect(floorsWith({
              sourceText: '- 猫\n  > 窗台上的阳光让小猫睡着了。',
              currentText: '- cat\n  > The sunny window put the kitten to sleep.',
              candidateText: '- cat\n  The sunny window put the kitten to sleep.',
            },),).toEqual({
              kind: 'refused',
              findings: [
                'The PAGE AS IT STANDS carries 1 quoted passage (blockquotes at any depth, including inside a container '
                  + 'tag, a list, a footnote, or another blockquote) and your translation carries 0. Every quoted passage of '
                  + 'the PAGE AS IT STANDS has to stay a quoted passage in your translation, where it stands.',
              ],
            },);
            expect(floorsWith({
              sourceText: '- 猫\n  > 窗台上的阳光让小猫睡着了。',
              currentText: '- cat\n  > The sunny window put the kitten to sleep.',
              candidateText: '- cat\n  > The sunlit window put the kitten to sleep.',
            },),).toEqual({ kind: 'passes', },);
          },
        },),
        it({
          name: 'REFUSES A SLICE WHOSE GROUND THE RULE CANNOT READ, saying what could not be read and on which side',
          fn: async () => {
            const verdict = floorsWith({
              sourceText: '窗台 <Cat 上的阳光。',
              currentText: 'The <Cat sill sun.',
              candidateText: 'The <Cat window sun.',
            },);
            expect(verdict.kind === 'refused' ? verdict.findings[0]?.startsWith(
              'the floor can compare nothing here (original could not be read: MdxParseError: ',
            ) : null,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
