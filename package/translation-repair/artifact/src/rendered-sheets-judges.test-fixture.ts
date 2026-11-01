/**
 The repair lane's judges and the translate retry, rendered for
 `rendered-sheets.test-fixture.ts` (ledger X17).

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  buildTranslateMessages,
  buildTranslateRepairMessages,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  buildDerivabilityMessages,
  buildRestorationJudgeMessages,
} from '@monochromatic-dev/translation-repair-repair';
import { buildConsistencyMessages, } from '@monochromatic-dev/translation-repair-consistency';
import {
  buildAdjudicationMessages,
  buildIntroducedDefectMessages,
} from '@monochromatic-dev/translation-repair-adjudication';

import {
  type RenderedSheet,
  SOURCE,
  ARCHIVE,
  REPAIR,
  IDENTITY,
  REFERENCES,
  TENSE_ISSUE,
  JUDGE_REFERENCE,
  NEIGHBOUR_ARCHIVE,
  NEIGHBOUR_SOURCE,
  TENSE_REGION,
  TRANSLATORS_NOTE,
  joined,
} from '@monochromatic-dev/translation-repair-repair/ts/rendered-sheets-texts.test-fixture.ts';

//region Judge sheets

/**
 The repair lane's judges and the translate lane's retry: adjudication, the
 derivability and restoration probes, the introduced-defect check, the
 translate writer asked again, and the page-level consistency judge asked about
 a repeated sentence and about a refrain (ledger X17).

 @returns One entry per sheet

 @example
 ```ts
 const sheets = judgeSheets();
 ```
 */
export function judgeSheets(): readonly RenderedSheet[] {
  return [
    {
      name: 'adjudication',
      text: joined({
        messages: buildAdjudicationMessages({
          sourceText: SOURCE,
          targetText: ARCHIVE,
          clusters: [{
            clusterId: 'cluster/tense',
            position: 0,
            members: TENSE_ISSUE.claims,
          },],
          neighbouringSourceText: NEIGHBOUR_SOURCE,
          neighbouringIncumbentText: NEIGHBOUR_ARCHIVE,
          identityContext: IDENTITY,
          referenceContext: REFERENCES,
        },)
          .messages,
      },),
    },
    {
      name: 'derivability',
      text: joined({
        messages: buildDerivabilityMessages({
          sourceText: SOURCE,
          references: [JUDGE_REFERENCE,],
        },)
          .messages,
      },),
    },
    {
      name: 'restoration judge',
      text: joined({
        messages: buildRestorationJudgeMessages({
          sourceText: SOURCE,
          repairedText: REPAIR,
          references: [JUDGE_REFERENCE,],
        },)
          .messages,
      },),
    },
    {
      name: 'introduced defect',
      text: joined({
        messages: buildIntroducedDefectMessages({
          sourceText: SOURCE,
          baselineText: ARCHIVE,
          regions: [TENSE_REGION,],
          issues: [TENSE_ISSUE,],
          neighbouringSourceText: NEIGHBOUR_SOURCE,
          neighbouringIncumbentText: NEIGHBOUR_ARCHIVE,
          identityContext: IDENTITY,
        },)
          .messages,
      },),
    },
    {
      name: 'page consistency, repeated sentence',
      text: joined({
        messages: buildConsistencyMessages({
          subject: {
            repeat: {
              kind: 'maximal',
              wording: '窗台上的阳光让小猫睡着了',
            },
            occurrences: [
              {
                sliceIndex: 0,
                paragraph: '窗台上的阳光让小猫睡着了。',
                markStart: 0,
                markEnd: 12,
                english: {
                  kind: 'present',
                  text: `The sunny window put the kitten to sleep. ${TRANSLATORS_NOTE}`,
                },
              },
              {
                sliceIndex: 2,
                paragraph: '傍晚的时候，窗台上的阳光让小猫睡着了，小鱼干也笑了。',
                markStart: 6,
                markEnd: 18,
                english: {
                  kind: 'present',
                  text: `In the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too. ${TRANSLATORS_NOTE}`,
                },
              },
            ],
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
          },
        },),
      },),
    },
    {
      name: 'page consistency, refrain',
      text: joined({
        messages: buildConsistencyMessages({
          subject: {
            repeat: {
              kind: 'refrain',
              prefix: '所以这只猫一定是',
              suffix: '的小宝贝',
            },
            occurrences: [
              {
                sliceIndex: 0,
                paragraph: '所以这只猫一定是胖胖的小宝贝。',
                markStart: 0,
                markEnd: 14,
                slotSource: '胖胖',
                english: {
                  kind: 'present',
                  text: `So this cat must be a chubby little treasure. ${TRANSLATORS_NOTE}`,
                },
              },
              {
                sliceIndex: 1,
                paragraph: '所以这只猫一定是懒懒的小宝贝。',
                markStart: 0,
                markEnd: 14,
                slotSource: '懒懒',
                english: {
                  kind: 'present',
                  text: `So this cat is surely a lazy little treasure. ${TRANSLATORS_NOTE}`,
                },
              },
            ],
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
          },
        },),
      },),
    },
    {
      // THE TENSE BLOCK A RECORDED DEATH ADDS (the lead's decision of 2026-10-12,
      // after the owner's tense rulings of 2026-10-11), rendered once more with the
      // death recorded and English that narrates its subject in the present, so
      // the guards over every sheet read the block's wording here too.
      name: 'page consistency, repeated sentence, death recorded',
      text: joined({
        messages: buildConsistencyMessages({
          subject: {
            repeat: {
              kind: 'maximal',
              wording: '窗台上的阳光让咪咪睡着了',
            },
            occurrences: [
              {
                sliceIndex: 0,
                paragraph: '窗台上的阳光让咪咪睡着了。',
                markStart: 0,
                markEnd: 12,
                english: {
                  kind: 'present',
                  text: `Mittens is asleep in the sunny window. ${TRANSLATORS_NOTE}`,
                },
              },
              {
                sliceIndex: 2,
                paragraph: '傍晚的时候，窗台上的阳光让咪咪睡着了，小鱼干也笑了。',
                markStart: 6,
                markEnd: 18,
                english: {
                  kind: 'present',
                  text: `In the evening, the sunlight on the sill lulled Mittens to sleep, and the fish snacks smiled too. ${TRANSLATORS_NOTE}`,
                },
              },
            ],
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
            lifeEnded: true,
            subjectNames: ['Mittens',],
          },
        },),
      },),
    },
    {
      name: 'page consistency, refrain, death recorded',
      text: joined({
        messages: buildConsistencyMessages({
          subject: {
            repeat: {
              kind: 'refrain',
              prefix: '所以咪咪一定是',
              suffix: '的小宝贝',
            },
            occurrences: [
              {
                sliceIndex: 0,
                paragraph: '所以咪咪一定是胖胖的小宝贝。',
                markStart: 0,
                markEnd: 13,
                slotSource: '胖胖',
                english: {
                  kind: 'present',
                  text: `So Mittens is a chubby little treasure. ${TRANSLATORS_NOTE}`,
                },
              },
              {
                sliceIndex: 1,
                paragraph: '所以咪咪一定是懒懒的小宝贝。',
                markStart: 0,
                markEnd: 13,
                slotSource: '懒懒',
                english: {
                  kind: 'present',
                  text: `So Mittens surely was a lazy little treasure. ${TRANSLATORS_NOTE}`,
                },
              },
            ],
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
            lifeEnded: true,
            subjectNames: ['Mittens',],
          },
        },),
      },),
    },
    {
      name: 'translate repair',
      text: joined({
        messages: buildTranslateRepairMessages({
          priorMessages: buildTranslateMessages({
            sourceText: SOURCE,
            existingText: ARCHIVE,
            identityContext: IDENTITY,
          },)
            .messages,
          priorTranslation: REPAIR,
          findings: ['The translation drops what the kitten promised.',],
        },),
      },),
    },
  ];
}

//endregion Judge sheets
