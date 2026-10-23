/**
 The selection slates, each handed a subject, for the guard that holds every stage's system prompt to the same text on
 every call (R4 of the second review of the Claude route). One slate builder serves every selection: the task, the
 criteria and the decline text decide which stage it is.

 Fixtures are cat-themed and dog-themed invention; no corpus content appears here.

 @module
 */

import {
  buildCandidateSelectMessages,
  CHUNK_DECLINE_CONSEQUENCE,
  KEEPS_TRUSTED_TEXT,
  LEAVES_PASSAGE_UNTRANSLATED,
  repairSelectionSourceEvidence,
  type SelectEvidence,
  SHIPS_BY_PREFERENCE,
} from '@monochromatic-dev/translation-repair-adjudication';
import {
  acceptedRepairClaimsNote,
  archiveDisputeNote,
} from '@monochromatic-dev/translation-repair-archive';
import {
  TRANSLATE_SELECTION_TASK,
  translateSelectionCriteria,
  translateSelectionTask,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  translatedSlateBreakEvidence,
  translatedSlateCriteria,
  translateSlateEvidence,
} from '@monochromatic-dev/translation-repair-translate-stage';
import {
  buildRefineSelectionContext,
  CHUNK_SELECTION_CRITERIA,
  CHUNK_SELECTION_TASK,
  ENVELOPE_SELECTION_CRITERIA,
  ENVELOPE_SELECTION_TASK,
  type RefineStageMode,
} from '../dist/final/node/index.mjs';
import { SEAT_HYPER_ONLY, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import type {
  PromptMessages,
  PromptStage,
  PromptSubject,
} from './system-prompt-subject.test-fixture.ts';

//region Slate stages

/**
 The evidence the repair lane's slates send, with a subject's declared names and references.

 @param subject - invented subject

 @returns Evidence entries, the original first

 @example
 ```ts
 const evidence = repairedEvidence({ subject: CAT_SUBJECT, },);
 ```
 */
function repairedEvidence(
  { subject, }: { readonly subject: PromptSubject; },
): readonly SelectEvidence[] {
  return [
    {
      label: 'ORIGINAL (Chinese)',
      text: subject.source,
    },
    ...repairSelectionSourceEvidence({
      identityContext: subject.identity,
      referenceContext: subject.references,
    },),
  ];
}

/**
 The refinement stage's comparative mode.
 */
const COMPARATIVE_MODE: RefineStageMode = { kind: 'comparative', };

/**
 The refinement stage's objection-correcting mode, over the objection a subject's gate raised.

 @param subject - invented subject

 @returns Mode

 @example
 ```ts
 const mode = objectionModeOf({ subject: CAT_SUBJECT, },);
 ```
 */
function objectionModeOf(
  { subject, }: { readonly subject: PromptSubject; },
): RefineStageMode {
  return {
    kind: 'objection-correction',
    groups: [{
      origin: 'consolidation gate',
      objections: [subject.objection,],
    },],
  };
}

/**
 The evidence the translate lane's slate sends.

 @param subject - invented subject

 @param sourceText - original the slate renders

 @returns Evidence entries, in the order the judges read them

 @example
 ```ts
 const evidence = translatedEvidence({ subject: CAT_SUBJECT, sourceText: CAT_SUBJECT.source, },);
 ```
 */
function translatedEvidence(
  {
    subject,
    sourceText,
  }: {
    readonly subject: PromptSubject;
    readonly sourceText: string;
  },
): readonly SelectEvidence[] {
  return translateSlateEvidence({
    sourceText,
    neighbouringSourceText: subject.neighbourSource,
    neighbouringIncumbentText: subject.neighbourArchive,
    identityContext: subject.identity,
    referenceContext: subject.references,
  },);
}

/**
 The slate the refinement stage asks its judges, over the repair or, in the final polish, over the archive rendering
 with the notes the polish writers are shown.

 @param subject - invented subject

 @param mode - kind of question asked

 @param polish - whether the final polish asks it

 @returns Slate's messages

 @example
 ```ts
 const messages = refineSlateOf({ subject: CAT_SUBJECT, mode: COMPARATIVE_MODE, polish: false, },);
 ```
 */
function refineSlateOf(
  {
    subject,
    mode,
    polish,
  }: {
    readonly subject: PromptSubject;
    readonly mode: RefineStageMode;
    readonly polish: boolean;
  },
): PromptMessages {
  return buildCandidateSelectMessages({
    ...buildRefineSelectionContext({
      mode,
      sourceText: subject.source,
      repairedText: polish ? subject.archive : subject.repair,
      referenceContext: subject.references,
      identityContext: subject.identity,
      ...(polish
        ? {
          acceptedRepairClaimsNote: acceptedRepairClaimsNote({ claims: [subject.claim,], },),
          archiveDisputeNote: archiveDisputeNote({
            dispute: {
              sliceIndex: 0,
              standIn: subject.repair,
              standInEligible: true,
              acceptedClaims: [subject.claim,],
            },
          },),
        }
        : {}),
    },),
    rendered: [
      subject.repair,
      subject.archive,
    ],
    sourceText: subject.source,
  },);
}

/**
 Every selection slate by its own task, criteria and decline text.
 */
export const SLATE_STAGES: readonly PromptStage[] = [
  {
    name: 'envelope slate',
    render: function renderEnvelopeSlate({ subject, },) {
      return buildCandidateSelectMessages({
        task: ENVELOPE_SELECTION_TASK,
        criteria: ENVELOPE_SELECTION_CRITERIA,
        evidence: repairedEvidence({ subject, },),
        rendered: [
          subject.archiveQuote,
          subject.repairQuote,
        ],
        declineConsequence: KEEPS_TRUSTED_TEXT,
        sourceText: subject.source,
      },);
    },
  },
  {
    name: 'chunk slate',
    render: function renderChunkSlate({ subject, },) {
      return buildCandidateSelectMessages({
        task: CHUNK_SELECTION_TASK,
        criteria: CHUNK_SELECTION_CRITERIA,
        evidence: repairedEvidence({ subject, },),
        rendered: [
          subject.archive,
          subject.repair,
        ],
        declineConsequence: CHUNK_DECLINE_CONSEQUENCE,
        sourceText: subject.source,
      },);
    },
  },
  {
    name: 'translate slate',
    render: function renderTranslateSlate({ subject, },) {
      return buildCandidateSelectMessages({
        task: TRANSLATE_SELECTION_TASK,
        criteria: translateSelectionCriteria({ lineStructured: false, },),
        evidence: translatedEvidence({
          subject,
          sourceText: subject.source,
        },),
        rendered: [
          subject.archive,
          subject.repair,
        ],
        sourceText: subject.source,
      },);
    },
  },
  {
    name: 'translate slate, death recorded',
    render: function renderTranslateSlateDeathRecorded({ subject, },) {
      return buildCandidateSelectMessages({
        task: TRANSLATE_SELECTION_TASK,
        criteria: translateSelectionCriteria({ lineStructured: false, },),
        evidence: translateSlateEvidence({
          sourceText: subject.source,
          neighbouringSourceText: subject.neighbourSource,
          neighbouringIncumbentText: subject.neighbourArchive,
          identityContext: subject.identity,
          referenceContext: subject.references,
          lifeEnded: true,
          subjectNames: [subject.name,],
          ballotTexts: [
            subject.archive,
            subject.repair,
          ],
        },),
        rendered: [
          subject.archive,
          subject.repair,
        ],
        sourceText: subject.source,
      },);
    },
  },
  {
    name: 'translate challenge slate',
    render: function renderTranslateChallengeSlate({ subject, },) {
      return buildCandidateSelectMessages({
        task: translateSelectionTask({ responsibility: 'decline-challenge', },),
        criteria: translatedSlateCriteria({
          sourceText: subject.verseSource,
          archiveText: '',
          lineStructured: false,
        },),
        evidence: [
          ...translatedEvidence({
            subject,
            sourceText: subject.verseSource,
          },),
          ...translatedSlateBreakEvidence({
            sourceText: subject.verseSource,
            archiveText: '',
            slate: [
              {
                index: 1,
                text: subject.repair,
                hash: 'slate/1',
                origin: 'fresh',
                producer: {
                  kind: 'model',
                  modelId: SEAT_HYPER_ONLY,
                },
              },
              {
                index: 2,
                text: `${subject.repairQuote}${subject.lineBreak}${subject.archive}`,
                hash: 'slate/2',
                origin: 'fresh',
                producer: {
                  kind: 'model',
                  modelId: SEAT_HYPER_ONLY,
                },
              },
            ],
          },),
        ],
        rendered: [
          subject.repair,
          `${subject.repairQuote}${subject.lineBreak}${subject.archive}`,
        ],
        declineConsequence: LEAVES_PASSAGE_UNTRANSLATED,
        sourceText: subject.verseSource,
      },);
    },
  },
  {
    name: 'translate front matter slate',
    render: function renderTranslateFrontMatterSlate({ subject, },) {
      return buildCandidateSelectMessages({
        task: TRANSLATE_SELECTION_TASK,
        criteria: translateSelectionCriteria({
          lineStructured: false,
          syntax: 'front-matter',
        },),
        evidence: translatedEvidence({
          subject,
          sourceText: subject.source,
        },),
        rendered: [
          subject.archive,
          subject.repair,
        ],
        declineConsequence: SHIPS_BY_PREFERENCE,
        sourceText: subject.source,
      },);
    },
  },
  {
    name: 'refine slate',
    render: function renderRefineSlate({ subject, },) {
      return refineSlateOf({
        subject,
        mode: COMPARATIVE_MODE,
        polish: false,
      },);
    },
  },
  {
    name: 'refine objection slate',
    render: function renderRefineObjectionSlate({ subject, },) {
      return refineSlateOf({
        subject,
        mode: objectionModeOf({ subject, },),
        polish: false,
      },);
    },
  },
  {
    name: 'polish refine slate',
    render: function renderPolishRefineSlate({ subject, },) {
      return refineSlateOf({
        subject,
        mode: COMPARATIVE_MODE,
        polish: true,
      },);
    },
  },
  {
    name: 'polish refine objection slate',
    render: function renderPolishRefineObjectionSlate({ subject, },) {
      return refineSlateOf({
        subject,
        mode: objectionModeOf({ subject, },),
        polish: true,
      },);
    },
  },
];

//endregion Slate stages
