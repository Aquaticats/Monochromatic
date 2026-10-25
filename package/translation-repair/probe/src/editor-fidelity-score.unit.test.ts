/**
 Tests for the deterministic reading of one edited text against a trial: whether the defect is fixed, exactly or up to
 the whitespace between sentences, and whether anything else was touched. No model judges.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  EDITOR_DAMAGE_KINDS,
  type EditorFidelityTrial,
  editorFidelityTrials,
  patchedWith,
  scoreEditedText,
} from '../dist/final/node/index.mjs';
import {
  REVIEW_DONOR,
  REVIEW_REFERENCE,
} from './fidelity-reference.test-fixture.ts';
import {
  inventedReference,
  regionOf,
} from './editor-fidelity.test-fixture.ts';

/**
 The invented reference's trials.
 */
const TRIALS = editorFidelityTrials({
  references: [inventedReference(),],
  damageKinds: EDITOR_DAMAGE_KINDS,
},);

/**
 The insertion trial.
 */
const INSERTION: EditorFidelityTrial = nonNullishOrThrow(TRIALS.find(function isInsertion(trial,): boolean {
  return trial.damageKind === 'insertion';
},),);

/**
 The alteration trial.
 */
const ALTERATION: EditorFidelityTrial = nonNullishOrThrow(TRIALS.find(function isAlteration(trial,): boolean {
  return trial.damageKind === 'alteration';
},),);

/**
 A trial whose inserted sentence closes the section, as the last sentence of a reviewed reference can: the clean text
 ends where the quoted region ends in the damaged text, so a perfect edit leaves the separator that preceded the
 inserted sentence at the end of the patched text.

 @returns The trial, built from the invented reference's clean text and donor sentence

 @example
 ```ts
 const trial = endOfSectionTrial();
 ```
 */
function endOfSectionTrial(): EditorFidelityTrial {
  /**
   Where the last sentence of the clean text begins.
   */
  const lastSentenceAt = REVIEW_REFERENCE.indexOf('She carefully kept',);
  /**
   The clean text with the donor sentence after one space.
   */
  const damagedText = `${REVIEW_REFERENCE} ${REVIEW_DONOR}`;
  return {
    ...INSERTION,
    cleanText: REVIEW_REFERENCE,
    damagedText,
    defectStart: REVIEW_REFERENCE.length + 1,
    damagedDefect: REVIEW_DONOR,
    cleanDefect: '',
    region: {
      startOffset: lastSentenceAt,
      endOffset: damagedText.length,
    },
  };
}

/**
 The end-of-section trial.
 */
const SECTION_END: EditorFidelityTrial = endOfSectionTrial();

await describe({
  name: scoreEditedText.name,
  children: [
    describe({
      name: 'the inserted sentence',
      children: [
        it({
          name: 'READS THE CLEAN TEXT as an exact fix with nothing else touched',
          fn: async () => {
            expect(scoreEditedText({
              trial: INSERTION,
              patchedText: REVIEW_REFERENCE,
            },),).toStrictEqual({
              fixed: 'exact',
              collateral: false,
            },);
          },
        },),
        it({
          name: 'READS THE SENTENCE REMOVED, its neighbours kept, as an exact fix: the quoted region is the sentence '
            + 'with the one either side of it, so the separator on both sides of the removed sentence is inside the '
            + 'region and the patched text is the clean text',
          fn: async () => {
            /**
             The patch an editor that removes the sentence and copies the rest produces.
             */
            const patchedText = patchedWith({
              trial: INSERTION,
              replacement: regionOf({ trial: INSERTION, },)
                .replace(
                INSERTION.damagedDefect,
                '',
                ),
            },);
            expect(patchedText,).toBe(REVIEW_REFERENCE,);
            expect(scoreEditedText({
              trial: INSERTION,
              patchedText,
            },),).toStrictEqual({
              fixed: 'exact',
              collateral: false,
            },);
          },
        },),
        it({
          name: 'READS THE SENTENCE REMOVED WITH THE SPACE BETWEEN WORDS CHANGED as an equivalent fix',
          fn: async () => {
            expect(scoreEditedText({
              trial: INSERTION,
              patchedText: patchedWith({
                trial: INSERTION,
                replacement: regionOf({ trial: INSERTION, },)
                  .replace(
                  INSERTION.damagedDefect,
                  '',
                  )
                  .replace(
                  'The well-lit',
                  'The  well-lit',
                  ),
              },),
            },),).toStrictEqual({
              fixed: 'equivalent',
              collateral: false,
            },);
          },
        },),
        it({
          name: 'READS THE QUOTED REGION DELETED OR BLANKED as defect-only with collateral, not as a fix: the '
            + 'sentence is gone but so are the two beside it, which a reply that does nothing but delete the '
            + 'quoted text always does',
          fn: async () => {
            for (const replacement of ['', ' ', '\n',]) {
              expect(scoreEditedText({
                trial: INSERTION,
                patchedText: patchedWith({
                  trial: INSERTION,
                  replacement,
                },),
              },),).toStrictEqual({
                fixed: 'defect-only',
                collateral: true,
              },);
            }
          },
        },),
        it({
          name: 'READS THE SENTENCE REMOVED AND A NEIGHBOUR REWORDED as defect-only with collateral',
          fn: async () => {
            expect(scoreEditedText({
              trial: INSERTION,
              patchedText: patchedWith({
                trial: INSERTION,
                replacement: regionOf({ trial: INSERTION, },)
                  .replace(
                  INSERTION.damagedDefect,
                  '',
                  )
                  .replace(
                  'discussed',
                  'talked about',
                  ),
              },),
            },),).toStrictEqual({
              fixed: 'defect-only',
              collateral: true,
            },);
          },
        },),
        it({
          name: 'READS A SOFTENED SENTENCE as not fixed, with nothing else touched: the rule is that an addition is '
            + 'removed, and a shorter sentence in its place is still an addition',
          fn: async () => {
            expect(scoreEditedText({
              trial: INSERTION,
              patchedText: patchedWith({
                trial: INSERTION,
                replacement: regionOf({ trial: INSERTION, },)
                  .replace(
                  INSERTION.damagedDefect,
                  'She travelled to a distant village. ',
                  ),
              },),
            },),).toStrictEqual({
              fixed: 'none',
              collateral: false,
            },);
          },
        },),
        it({
          name: 'READS THE UNCHANGED TEXT as not fixed and nothing touched',
          fn: async () => {
            expect(scoreEditedText({
              trial: INSERTION,
              patchedText: INSERTION.damagedText,
            },),).toStrictEqual({
              fixed: 'none',
              collateral: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'the inserted sentence that closes a section',
      children: [
        it({
          name: 'READS THE LAST SENTENCE KEPT as an exact fix: the separator before the removed sentence is inside '
            + 'the quoted region',
          fn: async () => {
            expect(scoreEditedText({
              trial: SECTION_END,
              patchedText: REVIEW_REFERENCE,
            },),).toStrictEqual({
              fixed: 'exact',
              collateral: false,
            },);
          },
        },),
        it({
          name: 'READS A FIX THAT LEAVES WHITESPACE AT THE END OF THE SECTION as equivalent with nothing else '
            + 'touched: the apply gate joins nothing, so a separator at the end of the text is not the editor\'s '
            + 'change to the words',
          fn: async () => {
            for (const ending of [' ', '\n', '\n\n',]) {
              expect(scoreEditedText({
                trial: SECTION_END,
                patchedText: `${REVIEW_REFERENCE}${ending}`,
              },),).toStrictEqual({
                fixed: 'equivalent',
                collateral: false,
              },);
            }
          },
        },),
        it({
          name: 'READS THE QUOTED REGION DELETED AT THE END OF A SECTION as defect-only with collateral, as it does '
            + 'in the middle of one',
          fn: async () => {
            expect(scoreEditedText({
              trial: SECTION_END,
              patchedText: SECTION_END.damagedText.slice(
                0,
                SECTION_END.region.startOffset,
              ),
            },),).toStrictEqual({
              fixed: 'defect-only',
              collateral: true,
            },);
          },
        },),
        it({
          name: 'READS A WORD CHANGED NEXT TO TRAILING WHITESPACE as not equivalent: only the whitespace is ignored',
          fn: async () => {
            expect(scoreEditedText({
              trial: SECTION_END,
              patchedText: `${REVIEW_REFERENCE.replace('liked', 'loved')} `,
            },).fixed,).not.toBe('equivalent',);
          },
        },),
      ],
    },),

    describe({
      name: 'the altered number',
      children: [
        it({
          name: 'READS THE CLEAN TEXT as an exact fix with nothing else touched',
          fn: async () => {
            expect(scoreEditedText({
              trial: ALTERATION,
              patchedText: REVIEW_REFERENCE,
            },),).toStrictEqual({
              fixed: 'exact',
              collateral: false,
            },);
          },
        },),
        it({
          name: 'READS A FIX THAT ONLY DIFFERS IN THE SPACE BETWEEN WORDS as equivalent: the number is back and the '
            + 'sentence is the clean sentence once runs of whitespace count as one',
          fn: async () => {
            expect(scoreEditedText({
              trial: ALTERATION,
              patchedText: patchedWith({
                trial: ALTERATION,
                replacement: regionOf({ trial: ALTERATION, },)
                  .replace(
                  ALTERATION.damagedDefect,
                  ALTERATION.cleanDefect,
                  )
                  .replace(
                  'arrived in',
                  'arrived  in',
                  ),
              },),
            },),).toStrictEqual({
              fixed: 'equivalent',
              collateral: false,
            },);
          },
        },),
        it({
          name: 'READS A FIX THAT ALSO REWORDS THE SENTENCE as fixed at the number and collateral: the editor mended '
            + 'the defect and changed words nobody flagged',
          fn: async () => {
            expect(scoreEditedText({
              trial: ALTERATION,
              patchedText: patchedWith({
                trial: ALTERATION,
                replacement: regionOf({ trial: ALTERATION, },)
                  .replace(
                  ALTERATION.damagedDefect,
                  ALTERATION.cleanDefect,
                  )
                  .replace(
                  'arrived',
                  'came',
                  ),
              },),
            },),).toStrictEqual({
              fixed: 'defect-only',
              collateral: true,
            },);
          },
        },),
        it({
          name: 'READS ANOTHER WRONG NUMBER as not fixed with nothing else touched, and the same wrong number with '
            + 'the sentence reworded as not fixed and collateral',
          fn: async () => {
            /**
             A number neither side states.
             */
            const wrong = '1999';
            expect(scoreEditedText({
              trial: ALTERATION,
              patchedText: patchedWith({
                trial: ALTERATION,
                replacement: regionOf({ trial: ALTERATION, },)
                  .replace(
                  ALTERATION.damagedDefect,
                  wrong,
                  ),
              },),
            },),).toStrictEqual({
              fixed: 'none',
              collateral: false,
            },);
            expect(scoreEditedText({
              trial: ALTERATION,
              patchedText: patchedWith({
                trial: ALTERATION,
                replacement: regionOf({ trial: ALTERATION, },)
                  .replace(
                  ALTERATION.damagedDefect,
                  wrong,
                  )
                  .replace(
                  'arrived',
                  'came',
                  ),
              },),
            },),).toStrictEqual({
              fixed: 'none',
              collateral: true,
            },);
          },
        },),
        it({
          name: 'READS THE UNCHANGED TEXT as not fixed and nothing touched',
          fn: async () => {
            expect(scoreEditedText({
              trial: ALTERATION,
              patchedText: ALTERATION.damagedText,
            },),).toStrictEqual({
              fixed: 'none',
              collateral: false,
            },);
          },
        },),
        it({
          name: 'READS A CHANGE OUTSIDE THE REGION as collateral, which the apply gate does not let through and so a '
            + 'nonzero reading is a finding about the gate: the defect cannot be isolated, so only an equal text '
            + 'counts as fixed',
          fn: async () => {
            /**
             The region mended, and a later sentence rewritten.
             */
            const patchedText = patchedWith({
              trial: ALTERATION,
              replacement: regionOf({ trial: ALTERATION, },)
                .replace(
                ALTERATION.damagedDefect,
                ALTERATION.cleanDefect,
                ),
            },)
              .replace(
              'Every morning',
              'Each morning',
              );
            expect(scoreEditedText({
              trial: ALTERATION,
              patchedText,
            },),).toStrictEqual({
              fixed: 'none',
              collateral: true,
            },);
          },
        },),
        it({
          name: 'READS A CHANGE OF WHITESPACE ALONE OUTSIDE THE REGION as collateral beside an equivalent fix',
          fn: async () => {
            /**
             The region mended and a space doubled elsewhere.
             */
            const patchedText = patchedWith({
              trial: ALTERATION,
              replacement: regionOf({ trial: ALTERATION, },)
                .replace(
                ALTERATION.damagedDefect,
                ALTERATION.cleanDefect,
                ),
            },)
              .replace(
              'Every morning',
              'Every  morning',
              );
            expect(scoreEditedText({
              trial: ALTERATION,
              patchedText,
            },),).toStrictEqual({
              fixed: 'equivalent',
              collateral: true,
            },);
          },
        },),
      ],
    },),
  ],
},);
