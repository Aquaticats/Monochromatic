/**
 The consolidation, its gates, the rendering audit and the page title lexicon, each handed a subject, for the guard that
 holds every stage's system prompt to the same text on every call (R4 of the second review of the Claude route).

 Fixtures are cat-themed and dog-themed invention; no corpus content appears here.

 @module
 */

import {
  acceptedRepairClaimsNote,
  archiveDisputeNote,
} from '@monochromatic-dev/translation-repair-archive';
import {
  buildConsolidateGateMessages,
  buildConsolidateMessages,
  buildConsolidationPolishGateMessages,
} from '@monochromatic-dev/translation-repair-consolidate';
import { buildRenderingAuditMessages, } from '@monochromatic-dev/translation-repair-rendering-audit';
import type {
  PromptStage,
  PromptSubject,
} from '@monochromatic-dev/translation-repair-repair/ts/system-prompt-subject.test-fixture.ts';
import { buildPageTitleLexiconMessages, } from '../dist/final/node/index.mjs';

//region Page stages

/**
 The polish gate's subject, in the comparative mode.

 @param subject - invented subject

 @returns Everything the gate is shown except its mode

 @example
 ```ts
 const shown = polishGateSubjectOf({ subject: CAT_SUBJECT, },);
 ```
 */
function polishGateSubjectOf(
  { subject, }: { readonly subject: PromptSubject; },
): Omit<Parameters<typeof buildConsolidationPolishGateMessages>[0]['subject'], 'mode'> {
  return {
    sourceText: subject.source,
    archiveText: subject.archive,
    baseText: subject.archive,
    polishedText: subject.repair,
    lineStructured: false,
    identityContext: subject.identity,
    referenceContext: subject.references,
    archiveDisputeNote: archiveDisputeNote({
      dispute: {
        sliceIndex: 0,
        standIn: subject.repair,
        standInEligible: true,
        acceptedClaims: [subject.claim,],
      },
    },),
    acceptedRepairClaimsNote: acceptedRepairClaimsNote({ claims: [subject.claim,], },),
  };
}

/**
 The stages of the consolidation and the page-level readers, each rendered from a subject.
 */
export const PAGE_STAGES: readonly PromptStage[] = [
  {
    name: 'consolidation writer',
    render: function renderConsolidationWriter({ subject, },) {
      return buildConsolidateMessages({
        subject: {
          sourceText: subject.source,
          incumbentText: subject.archive,
          repairText: subject.repair,
          translateText: subject.repair,
          ballots: [],
          lineStructured: false,
          identityContext: subject.identity,
          referenceContext: subject.references,
        },
      },);
    },
  },
  {
    name: 'consolidate gate',
    render: function renderConsolidateGate({ subject, },) {
      return buildConsolidateGateMessages({
        subject: {
          lineStructured: false,
          sourceText: subject.source,
          incumbentText: subject.archive,
          consolidatedText: subject.repair,
          standingText: subject.archive,
          identityContext: subject.identity,
          referenceContext: subject.references,
        },
      },);
    },
  },
  {
    name: 'consolidate gate, death recorded',
    render: function renderConsolidateGateDeathRecorded({ subject, },) {
      return buildConsolidateGateMessages({
        subject: {
          lineStructured: false,
          sourceText: subject.source,
          incumbentText: subject.archive,
          consolidatedText: subject.repair,
          standingText: subject.archive,
          identityContext: subject.identity,
          referenceContext: subject.references,
          lifeEnded: true,
          subjectNames: [subject.name,],
        },
      },);
    },
  },
  {
    name: 'polish gate, comparative',
    render: function renderPolishGateComparative({ subject, },) {
      return buildConsolidationPolishGateMessages({
        subject: {
          ...polishGateSubjectOf({ subject, },),
          mode: { kind: 'comparative', },
        },
      },);
    },
  },
  {
    name: 'polish gate, objection correction',
    render: function renderPolishGateObjectionCorrection({ subject, },) {
      return buildConsolidationPolishGateMessages({
        subject: {
          ...polishGateSubjectOf({ subject, },),
          mode: {
            kind: 'objection-correction',
            groups: [{
              origin: 'consolidation gate',
              objections: [subject.objection,],
            },],
          },
        },
      },);
    },
  },
  {
    name: 'rendering audit',
    render: function renderRenderingAudit({ subject, },) {
      return buildRenderingAuditMessages({
        subject: {
          sourceText: subject.source,
          candidateText: subject.archive,
          identityContext: subject.identity,
          referenceContext: subject.references,
        },
      },);
    },
  },
  {
    name: 'page title lexicon',
    render: function renderPageTitleLexicon({ subject, },) {
      return buildPageTitleLexiconMessages({
        sourceText: subject.source,
        titles: [subject.title,],
        identityContext: subject.identity,
      },);
    },
  },
];

//endregion Page stages
