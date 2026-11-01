/**
 The three sheets a recorded death adds a block to, rendered with the death
 recorded, for `rendered-sheets.test-fixture.ts` (the owner's tense decision
 of 2026-10-11).

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  TRANSLATE_SELECTION_TASK,
  translateSelectionCriteria,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { buildCandidateSelectMessages, } from '@monochromatic-dev/translation-repair-adjudication';
import { translateSlateEvidence, } from '@monochromatic-dev/translation-repair-translate-stage';
import { buildLaneContestMessages, } from '@monochromatic-dev/translation-repair-lane-contest';
import { buildConsolidateGateMessages, } from '@monochromatic-dev/translation-repair-consolidate';

import {
  type RenderedSheet,
  SOURCE,
  ARCHIVE,
  REPAIR,
  IDENTITY,
  REFERENCES,
  NEIGHBOUR_ARCHIVE,
  NEIGHBOUR_SOURCE,
  joined,
} from '@monochromatic-dev/translation-repair-repair/ts/rendered-sheets-texts.test-fixture.ts';

//region Death-recorded sheets

/**
 The lane contest, the consolidate gate and the translate slate, each rendered
 with the death of a subject recorded.

 @returns One entry per sheet, named by its stage

 @example
 ```ts
 const sheets = deathRecordedSheets();
 ```
 */
export function deathRecordedSheets(): readonly RenderedSheet[] {
  return [
    {
      // THE THREE SHEETS A RECORDED DEATH ADDS A BLOCK TO (the owner's tense
      // decision of 2026-10-11), rendered once more with the death recorded
      // so the guards over every sheet read the block's wording too.
      name: 'lane contest, death recorded',
      text: joined({
        messages: buildLaneContestMessages({
          subject: {
            lineStructured: false,
            sourceText: SOURCE,
            incumbentText: ARCHIVE,
            repairText: REPAIR,
            translateText: REPAIR,
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
            lifeEnded: true,
            subjectNames: ['Mittens',],
          },
        },),
      },),
    },
    {
      name: 'consolidate gate, death recorded',
      text: joined({
        messages: buildConsolidateGateMessages({
          subject: {
            lineStructured: false,
            sourceText: SOURCE,
            incumbentText: ARCHIVE,
            consolidatedText: REPAIR,
            standingText: ARCHIVE,
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
            lifeEnded: true,
            subjectNames: ['Mittens',],
          },
        },),
      },),
    },
    {
      name: 'translate slate, death recorded',
      text: joined({
        messages: buildCandidateSelectMessages({
          task: TRANSLATE_SELECTION_TASK,
          criteria: translateSelectionCriteria({ lineStructured: false, },),
          evidence: translateSlateEvidence({
            sourceText: SOURCE,
            neighbouringSourceText: NEIGHBOUR_SOURCE,
            neighbouringIncumbentText: NEIGHBOUR_ARCHIVE,
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
            lifeEnded: true,
            subjectNames: ['Mittens',],
            ballotTexts: [
              ARCHIVE,
              REPAIR,
            ],
          },),
          rendered: [
            ARCHIVE,
            REPAIR,
          ],
          sourceText: SOURCE,
        },),
      },),
    },
  ];
}

//endregion Death-recorded sheets
