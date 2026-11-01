/**
 The stages that carry the rendered-break contract, each handed an original with explicit line breaks and no archive
 wording, for the guard that holds every stage's system prompt to the same text on every call (R4 of the second review
 of the Claude route).

 The contract is fixed text; the measured break counts of the original and of each shown candidate are the call's own
 facts and belong in the user turn. The cat's original holds one explicit break and the dog's two, so a count in the
 system text tells the subjects apart.

 Fixtures are cat-themed and dog-themed invention; no corpus content appears here.

 @module
 */

import {
  buildConsolidateGateMessages,
  buildConsolidateMessages,
} from '@monochromatic-dev/translation-repair-consolidate';
import { buildLaneContestMessages, } from '@monochromatic-dev/translation-repair-lane-contest';
import type {
  PromptStage,
} from '@monochromatic-dev/translation-repair-repair/ts/system-prompt-subject.test-fixture.ts';
import { buildTranslateMessages, } from '@monochromatic-dev/translation-repair-translate-floor';

//region Break-bearing stages

/**
 The stages whose sheets state the rendered-break contract, each rendered from a subject.
 */
export const BREAK_STAGES: readonly PromptStage[] = [
  {
    name: 'translate writer, source only with breaks',
    render: function renderTranslateWriterBreaks({ subject, },) {
      return buildTranslateMessages({
        sourceText: subject.verseSource,
        existingText: '',
        identityContext: subject.identity,
      },)
        .messages;
    },
  },
  {
    name: 'translate writer, follow-up, source only with breaks',
    render: function renderTranslateWriterFollowupBreaks({ subject, },) {
      return buildTranslateMessages({
        sourceText: subject.verseSource,
        existingText: '',
        identityContext: subject.identity,
        followupEvidence: {
          reason: 'declined-rejection',
          candidateTexts: [
            subject.verseFlat,
            subject.verseBroken,
          ],
          findings: [subject.finding,],
        },
      },)
        .messages;
    },
  },
  {
    name: 'consolidation writer, source only with breaks',
    render: function renderConsolidationWriterBreaks({ subject, },) {
      return buildConsolidateMessages({
        subject: {
          sourceText: subject.verseSource,
          incumbentText: '',
          repairText: subject.verseFlat,
          translateText: subject.verseBroken,
          ballots: [],
          lineStructured: false,
          identityContext: subject.identity,
          referenceContext: subject.references,
        },
      },);
    },
  },
  {
    name: 'consolidate gate, source only with breaks',
    render: function renderConsolidateGateBreaks({ subject, },) {
      return buildConsolidateGateMessages({
        subject: {
          lineStructured: false,
          sourceText: subject.verseSource,
          incumbentText: '',
          consolidatedText: subject.verseFlat,
          standingText: subject.verseBroken,
          identityContext: subject.identity,
          referenceContext: subject.references,
        },
      },);
    },
  },
  {
    name: 'lane contest, source only with breaks',
    render: function renderLaneContestBreaks({ subject, },) {
      return buildLaneContestMessages({
        subject: {
          lineStructured: false,
          sourceText: subject.verseSource,
          incumbentText: '',
          repairText: subject.verseFlat,
          translateText: subject.verseBroken,
          identityContext: subject.identity,
          referenceContext: subject.references,
        },
      },);
    },
  },
];

//endregion Break-bearing stages
