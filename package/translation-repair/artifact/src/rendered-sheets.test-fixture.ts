import { translateSlateEvidence, } from '@monochromatic-dev/translation-repair-translate-stage';
import { HOUSE_POLICY_BLOCK, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  buildTranslateMessages,
  TRANSLATE_SELECTION_TASK,
  translateSelectionCriteria,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { buildRenderingAuditMessages, } from '@monochromatic-dev/translation-repair-rendering-audit';
import { buildLaneContestMessages, } from '@monochromatic-dev/translation-repair-lane-contest';
import {
  buildEditorMessages,
  buildRefineMessages,
} from '@monochromatic-dev/translation-repair-repair';
import {
  buildConsolidateGateMessages,
  buildConsolidateMessages,
  buildConsolidationPolishGateMessages,
} from '@monochromatic-dev/translation-repair-consolidate';
import {
  buildCandidateSelectMessages,
  buildCriticMessages,
  buildResolutionMessages,
} from '@monochromatic-dev/translation-repair-adjudication';
import {
  buildAbsoluteNaturalnessReviewMessages,
  buildCoverageMessages,
} from '@monochromatic-dev/translation-repair-archive';
import { buildPageTitleLexiconMessages, } from '../dist/final/node/index.mjs';
import {
  type RenderedSheet,
  SOURCE,
  ARCHIVE,
  REPAIR,
  IDENTITY,
  REFERENCES,
  TENSE_ISSUE,
  ARCHIVE_PARAGRAPH_ENVELOPE,
  NEIGHBOUR_ARCHIVE,
  NEIGHBOUR_SOURCE,
  TENSE_ENVELOPE,
  joined,
} from '@monochromatic-dev/translation-repair-repair/ts/rendered-sheets-texts.test-fixture.ts';
import { evidenceSheets, } from '@monochromatic-dev/translation-repair-repair/ts/rendered-sheets-preparation.test-fixture.ts';
import { carrySheets, } from './rendered-sheets-carry.test-fixture.ts';
import { deathRecordedSheets, } from './rendered-sheets-death.test-fixture.ts';
import { judgeSheets, } from './rendered-sheets-judges.test-fixture.ts';
import {
  ARCHIVE_DISPUTE,
  polishSlateSheets,
  REPAIR_CLAIMS,
} from './rendered-sheets-polish.test-fixture.ts';
import { slateSheets, } from '@monochromatic-dev/translation-repair-repair/ts/rendered-sheets-slates.test-fixture.ts';

export type { RenderedSheet, } from '@monochromatic-dev/translation-repair-repair/ts/rendered-sheets-texts.test-fixture.ts';
export {
  ARCHIVE_DISPUTE,
  REPAIR_CLAIMS,
} from './rendered-sheets-polish.test-fixture.ts';

//region Rendered sheets
// EVERY MODEL-FACING SHEET, rendered once with invented cat fixtures, so a
// guard about what the models read (a spelling, a precedence line, a rule
// every sheet must carry) checks every sheet at once rather than the one
// sheet its class happened to find. Built for the whole-package audit of
// 2026-09-27, whose sheet audits rendered the same sheets by hand. These
// strings are authored test data, never copied corpus passages.
//
// "EVERY" WAS FIFTEEN UNTIL LEDGER X17 (2026-09-28): the attestation, the
// pairing rounds, the archive review and its slate, the repair lane's judges,
// the picture readers, every other selection slate and the typed decision were
// missing, and the picture readers' "summarise" had never met the Canadian
// spelling guard. `rendered-sheets-census.unit.test.ts` now fails when a sheet
// builder the package exports is rendered by none of these fixtures.

/**
 Every model-facing sheet the package builds, rendered with the fixtures.

 @returns One entry per sheet, named by its stage

 @example
 ```ts
 const sheets = renderedSheets();
 ```
 */
export function renderedSheets(): readonly RenderedSheet[] {
  /**
   Editor exchange, whose messages the sheet joins.
   */
  const editorPlan = buildEditorMessages({
    sourceText: SOURCE,
    targetText: ARCHIVE,
    // The span a critic quoted, which stops before the note that ends the
    // archive rendering.
    envelopes: [TENSE_ENVELOPE,],
    issues: [TENSE_ISSUE,],
    // The passages either side, shown as context only.
    neighbouringSourceText: NEIGHBOUR_SOURCE,
    neighbouringIncumbentText: NEIGHBOUR_ARCHIVE,
    // What `repair-editor-stage.ts` passes it (ledger S14).
    identityContext: IDENTITY,
    referenceContext: REFERENCES,
  },);
  /**
   Refiner exchange, whose messages the sheet joins.
   */
  const refinePlan = buildRefineMessages({
    sourceText: SOURCE,
    // The whole paragraph, note included, as `deriveRefinableEnvelopes` hands
    // it over.
    envelopes: [ARCHIVE_PARAGRAPH_ENVELOPE,],
    identityContext: IDENTITY,
    referenceContext: REFERENCES,
  },);
  /**
   Polish writer exchange, which production gives the claims the repair panel
   accepted for the slice and the note on a disputed archive rendering
   (ledger B805), whose messages the sheet joins.
   */
  const polishRefinePlan = buildRefineMessages({
    sourceText: SOURCE,
    envelopes: [ARCHIVE_PARAGRAPH_ENVELOPE,],
    identityContext: IDENTITY,
    referenceContext: REFERENCES,
    acceptedRepairClaimsNote: REPAIR_CLAIMS,
    archiveDisputeNote: ARCHIVE_DISPUTE,
  },);
  /**
   Translate writer exchange, whose messages the sheet joins.
   */
  const translatePlan = buildTranslateMessages({
    sourceText: SOURCE,
    existingText: ARCHIVE,
    identityContext: IDENTITY,
  },);
  /**
   Coverage exchange, whose messages the sheet joins.
   */
  const coveragePlan = buildCoverageMessages({
    sourcePassage: SOURCE,
    translationText: ARCHIVE,
    // What insertion admission passes it since ledger B28.
    identityContext: IDENTITY,
  },);
  /**
   Resolution exchange, whose messages the sheet joins.
   */
  const resolutionPlan = buildResolutionMessages({
    sourceText: SOURCE,
    patchedText: REPAIR,
    issues: [TENSE_ISSUE,],
    identityContext: IDENTITY,
    referenceContext: REFERENCES,
  },);
  return [
    {
      name: 'house policy',
      text: HOUSE_POLICY_BLOCK,
    },
    {
      name: 'critic',
      text: joined({
        messages: buildCriticMessages({
          sourceText: SOURCE,
          targetText: ARCHIVE,
          neighbouringSourceText: NEIGHBOUR_SOURCE,
          neighbouringIncumbentText: NEIGHBOUR_ARCHIVE,
          identityContext: IDENTITY,
          referenceContext: REFERENCES,
        },),
      },),
    },
    {
      name: 'editor',
      text: joined({
        messages: editorPlan.messages,
      },),
    },
    {
      name: 'refiner',
      text: joined({
        messages: refinePlan.messages,
      },),
    },
    {
      name: 'polish refiner',
      text: joined({
        messages: polishRefinePlan.messages,
      },),
    },
    {
      name: 'translate writer',
      text: joined({
        messages: translatePlan.messages,
      },),
    },
    {
      name: 'consolidation writer',
      text: joined({
        messages: buildConsolidateMessages({
          subject: {
            sourceText: SOURCE,
            incumbentText: ARCHIVE,
            repairText: REPAIR,
            translateText: REPAIR,
            ballots: [],
            lineStructured: false,
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
          },
        },),
      },),
    },
    {
      name: 'lane contest',
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
          },
        },),
      },),
    },
    {
      name: 'consolidate gate',
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
          },
        },),
      },),
    },
    {
      name: 'polish gate',
      text: joined({
        messages: buildConsolidationPolishGateMessages({
          subject: {
            sourceText: SOURCE,
            archiveText: ARCHIVE,
            baseText: ARCHIVE,
            polishedText: REPAIR,
            mode: { kind: 'comparative', },
            lineStructured: false,
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
            archiveDisputeNote: ARCHIVE_DISPUTE,
            acceptedRepairClaimsNote: REPAIR_CLAIMS,
          },
        },),
      },),
    },
    // The judges of the polish writers' rewrites, who read what the writers and the gate read (ledger B805).
    ...polishSlateSheets(),
    {
      name: 'translate slate',
      text: joined({
        messages: buildCandidateSelectMessages({
          task: TRANSLATE_SELECTION_TASK,
          criteria: translateSelectionCriteria({ lineStructured: false, },),
          // The evidence `judgeTranslateSlate` sends.
          evidence: translateSlateEvidence({
            sourceText: SOURCE,
            neighbouringSourceText: NEIGHBOUR_SOURCE,
            neighbouringIncumbentText: NEIGHBOUR_ARCHIVE,
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
          },),
          rendered: [
            ARCHIVE,
            REPAIR,
          ],
          sourceText: SOURCE,
        },),
      },),
    },
    ...deathRecordedSheets(),
    {
      name: 'coverage',
      text: joined({
        messages: coveragePlan.messages,
      },),
    },
    {
      name: 'naturalness review',
      text: joined({
        messages: buildAbsoluteNaturalnessReviewMessages({
          subject: {
            lineStructured: false,
            sourceText: SOURCE,
            candidateText: ARCHIVE,
            paragraphs: [ARCHIVE,],
            identityContext: IDENTITY,
          },
        },),
      },),
    },
    {
      name: 'rendering audit',
      text: joined({
        messages: buildRenderingAuditMessages({
          subject: {
            sourceText: SOURCE,
            candidateText: ARCHIVE,
            // What `corpus-run/rendering-audit-settled-buy.ts` passes a page that
            // declares names and links a page.
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
          },
        },),
      },),
    },
    {
      name: 'resolution',
      text: joined({
        messages: resolutionPlan.messages,
      },),
    },
    {
      name: 'page title lexicon',
      text: joined({
        messages: buildPageTitleLexiconMessages({
          sourceText: SOURCE,
          titles: ['猫之歌',],
          identityContext: IDENTITY,
        },),
      },),
    },
    ...evidenceSheets(),
    ...judgeSheets(),
    ...carrySheets(),
    ...slateSheets(),
  ];
}

//endregion Rendered sheets
