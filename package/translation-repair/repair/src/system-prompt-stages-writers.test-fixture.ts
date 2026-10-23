/**
 The stages that write, judge a repair or read a passage whole, each handed a subject, for the guard that holds every
 stage's system prompt to the same text on every call (R4 of the second review of the Claude route).

 Fixtures are cat-themed and dog-themed invention; no corpus content appears here.

 @module
 */

import {
  buildAdjudicationMessages,
  buildCriticMessages,
  buildIntroducedDefectMessages,
  buildResolutionMessages,
} from '@monochromatic-dev/translation-repair-adjudication';
import {
  acceptedRepairClaimsNote,
  archiveDisputeNote,
  buildAbsoluteNaturalnessReviewMessages,
  buildCoverageMessages,
} from '@monochromatic-dev/translation-repair-archive';
import { buildLaneContestMessages, } from '@monochromatic-dev/translation-repair-lane-contest';
import {
  buildTranslateMessages,
  buildTranslateRepairMessages,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  buildDerivabilityMessages,
  buildEditorMessages,
  buildRefineMessages,
  buildRestorationJudgeMessages,
} from '../dist/final/node/index.mjs';
import type {
  PromptStage,
  PromptSubject,
} from './system-prompt-subject.test-fixture.ts';

//region Writer and judge stages

/**
 The stages that write or judge a passage, each rendered from a subject.
 */
export const WRITER_STAGES: readonly PromptStage[] = [
  {
    name: 'critic',
    render: function renderCritic({ subject, },) {
      return buildCriticMessages({
        sourceText: subject.source,
        targetText: subject.archive,
        neighbouringSourceText: subject.neighbourSource,
        neighbouringIncumbentText: subject.neighbourArchive,
        identityContext: subject.identity,
        referenceContext: subject.references,
      },);
    },
  },
  {
    name: 'adjudication',
    render: function renderAdjudication({ subject, },) {
      return buildAdjudicationMessages({
        sourceText: subject.source,
        targetText: subject.archive,
        clusters: [{
          clusterId: 'cluster/tense',
          position: 0,
          members: subject.claims,
        },],
        neighbouringSourceText: subject.neighbourSource,
        neighbouringIncumbentText: subject.neighbourArchive,
        identityContext: subject.identity,
        referenceContext: subject.references,
      },)
        .messages;
    },
  },
  {
    name: 'editor',
    render: function renderEditor({ subject, },) {
      return buildEditorMessages({
        sourceText: subject.source,
        targetText: subject.archive,
        envelopes: [subject.envelope,],
        issues: [subject.issue,],
        neighbouringSourceText: subject.neighbourSource,
        neighbouringIncumbentText: subject.neighbourArchive,
        identityContext: subject.identity,
        referenceContext: subject.references,
      },)
        .messages;
    },
  },
  {
    name: 'refiner',
    render: function renderRefiner({ subject, },) {
      return buildRefineMessages({
        sourceText: subject.source,
        envelopes: [subject.paragraphEnvelope,],
        identityContext: subject.identity,
        referenceContext: subject.references,
      },)
        .messages;
    },
  },
  {
    name: 'polish refiner',
    render: function renderPolishRefiner({ subject, },) {
      return buildRefineMessages({
        sourceText: subject.source,
        envelopes: [subject.paragraphEnvelope,],
        identityContext: subject.identity,
        referenceContext: subject.references,
        acceptedRepairClaimsNote: acceptedRepairClaimsNote({ claims: [subject.claim,], },),
        archiveDisputeNote: archiveDisputeNote({
          dispute: {
            sliceIndex: 0,
            standIn: subject.repair,
            standInEligible: true,
            acceptedClaims: [subject.claim,],
          },
        },),
      },)
        .messages;
    },
  },
  {
    name: 'translate writer',
    render: function renderTranslateWriter({ subject, },) {
      return buildTranslateMessages({
        sourceText: subject.source,
        existingText: subject.archive,
        identityContext: subject.identity,
      },)
        .messages;
    },
  },
  {
    name: 'translate repair',
    render: function renderTranslateRepair({ subject, },) {
      return buildTranslateRepairMessages({
        priorMessages: buildTranslateMessages({
          sourceText: subject.source,
          existingText: subject.archive,
          identityContext: subject.identity,
        },)
          .messages,
        priorTranslation: subject.repair,
        findings: [subject.finding,],
      },);
    },
  },
  {
    name: 'coverage',
    render: function renderCoverage({ subject, },) {
      return buildCoverageMessages({
        sourcePassage: subject.source,
        translationText: subject.archive,
        identityContext: subject.identity,
      },)
        .messages;
    },
  },
  {
    name: 'lane contest',
    render: function renderLaneContest({ subject, },) {
      return buildLaneContestMessages({ subject: laneContestSubject({ subject, },), },);
    },
  },
  {
    name: 'lane contest, death recorded',
    render: function renderLaneContestDeathRecorded({ subject, },) {
      return buildLaneContestMessages({
        subject: {
          ...laneContestSubject({ subject, },),
          lifeEnded: true,
          subjectNames: [subject.name,],
        },
      },);
    },
  },
  {
    name: 'resolution',
    render: function renderResolution({ subject, },) {
      return buildResolutionMessages({
        sourceText: subject.source,
        patchedText: subject.repair,
        issues: [subject.issue,],
        identityContext: subject.identity,
        referenceContext: subject.references,
      },)
        .messages;
    },
  },
  {
    name: 'derivability',
    render: function renderDerivability({ subject, },) {
      return buildDerivabilityMessages({
        sourceText: subject.source,
        references: [subject.judgeReference,],
      },)
        .messages;
    },
  },
  {
    name: 'restoration judge',
    render: function renderRestorationJudge({ subject, },) {
      return buildRestorationJudgeMessages({
        sourceText: subject.source,
        repairedText: subject.repair,
        references: [subject.judgeReference,],
      },)
        .messages;
    },
  },
  {
    name: 'introduced defect',
    render: function renderIntroducedDefect({ subject, },) {
      return buildIntroducedDefectMessages({
        sourceText: subject.source,
        baselineText: subject.archive,
        regions: [subject.region,],
        issues: [subject.issue,],
        neighbouringSourceText: subject.neighbourSource,
        neighbouringIncumbentText: subject.neighbourArchive,
        identityContext: subject.identity,
      },)
        .messages;
    },
  },
  {
    name: 'naturalness review, defect discovery',
    render: function renderNaturalnessReviewDefectDiscovery({ subject, },) {
      return buildAbsoluteNaturalnessReviewMessages({
        subject: naturalnessSubject({ subject, },),
        perspective: 'defect-discovery',
      },);
    },
  },
  {
    name: 'naturalness review, prior acceptance challenge',
    render: function renderNaturalnessReviewPriorAcceptanceChallenge({ subject, },) {
      return buildAbsoluteNaturalnessReviewMessages({
        subject: naturalnessSubject({ subject, },),
        perspective: 'acceptance-challenge',
      },);
    },
  },
];

/**
 The subject the lane contest is asked about.

 @param subject - invented subject

 @returns Contest's subject, with no death recorded

 @example
 ```ts
 const asked = laneContestSubject({ subject: CAT_SUBJECT, },);
 ```
 */
function laneContestSubject(
  { subject, }: { readonly subject: PromptSubject; },
): Parameters<typeof buildLaneContestMessages>[0]['subject'] {
  return {
    lineStructured: false,
    sourceText: subject.source,
    incumbentText: subject.archive,
    repairText: subject.repair,
    translateText: subject.repair,
    identityContext: subject.identity,
    referenceContext: subject.references,
  };
}

/**
 The subject the naturalness review is asked about.

 @param subject - invented subject

 @returns Review's subject

 @example
 ```ts
 const asked = naturalnessSubject({ subject: CAT_SUBJECT, },);
 ```
 */
function naturalnessSubject(
  { subject, }: { readonly subject: PromptSubject; },
): Parameters<typeof buildAbsoluteNaturalnessReviewMessages>[0]['subject'] {
  return {
    lineStructured: false,
    sourceText: subject.source,
    candidateText: subject.archive,
    paragraphs: [subject.archive,],
    identityContext: subject.identity,
  };
}

//endregion Writer and judge stages
