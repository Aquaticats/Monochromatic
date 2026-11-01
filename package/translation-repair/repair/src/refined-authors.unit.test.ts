/**
 Tests for layering the naturalness lane's rewriters onto the authorship the
 editor stage already established.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { collectRefinedAuthors, } from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Issue the cases in this file credit.
 */
const WHISKER = 'adjudicated/whisker';

/**
 Model that repaired the text the refiners rewrote.
 */
const AUTHOR: RosterModelId = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 Second repairing model, for the case where nothing was rewritten at all.
 */
const HELPER: RosterModelId = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 Model that only ever rewrites, never repairs.
 */
const REFINER: RosterModelId = SEAT_SYNTHETIC_TEXT_EVERYWHERE;

await describe({
  name: collectRefinedAuthors.name,
  children: [
    it({
      name: 'ADDS THE REFINERS TO THE EDITOR\'S ANSWER rather than replacing it, because refined '
        + 'text is the editor\'s repair rewritten and a checker that had a hand in either half is '
        + 'judging its own work',
      fn: async function bothStagesAreNamed() {
        expect(collectRefinedAuthors({
          editorAuthorship: {
            perIssue: { [WHISKER]: [AUTHOR,], },
            everyIssue: [],
          },
          refineContributors: [REFINER,],
        },),).toEqual({
          perIssue: { [WHISKER]: [AUTHOR,], },
          everyIssue: [REFINER,],
        },);
      },
    },),

    it({
      name: 'LEAVES THE EDITOR\'S ANSWER EXACTLY AS IT WAS when no refinement won, since the text '
        + 'that ships is then precisely what the editor produced',
      fn: async function aLostRefinementAddsNobody() {
        expect(collectRefinedAuthors({
          editorAuthorship: {
            perIssue: { [WHISKER]: [AUTHOR,], },
            everyIssue: [HELPER,],
          },
          refineContributors: [],
        },),).toEqual({
          perIssue: { [WHISKER]: [AUTHOR,], },
          everyIssue: [HELPER,],
        },);
      },
    },),

    it({
      name: 'NAMES A MODEL ONCE when it both wrote the repair and refined it, so the discount is '
        + 'a half rather than a quarter',
      fn: async function oneModelInBothStagesIsNamedOnce() {
        expect(collectRefinedAuthors({
          editorAuthorship: {
            perIssue: {},
            everyIssue: [AUTHOR,],
          },
          refineContributors: [
            AUTHOR,
            REFINER,
          ],
        },),).toEqual({
          perIssue: {},
          everyIssue: [
            AUTHOR,
            REFINER,
          ],
        },);
      },
    },),
  ],
},);
