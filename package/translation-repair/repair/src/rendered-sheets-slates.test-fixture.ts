/**
 Every selection slate and the typed decision, rendered for
 `rendered-sheets.test-fixture.ts` (ledger X17).

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  translatedSlateBreakEvidence,
  translatedSlateCriteria,
  translateSlateEvidence,
} from '@monochromatic-dev/translation-repair-translate-stage';
import {
  TRANSLATE_SELECTION_TASK,
  translateSelectionCriteria,
  translateSelectionTask,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  buildCandidateSelectMessages,
  CHUNK_DECLINE_CONSEQUENCE,
  KEEPS_TRUSTED_TEXT,
  LEAVES_PASSAGE_UNTRANSLATED,
  repairSelectionSourceEvidence,
  selectDecision,
  type SelectEvidence,
  SHIPS_BY_PREFERENCE,
} from '@monochromatic-dev/translation-repair-adjudication';
import {
  buildRefineSelectionContext,
  CHUNK_SELECTION_CRITERIA,
  CHUNK_SELECTION_TASK,
  ENVELOPE_SELECTION_CRITERIA,
  ENVELOPE_SELECTION_TASK,
  type RefineStageMode,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_ONLY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

import {
  type RenderedSheet,
  SOURCE,
  ARCHIVE,
  REPAIR,
  IDENTITY,
  REFERENCES,
  ARCHIVE_QUOTE,
  VERSE_SOURCE,
  NEIGHBOUR_ARCHIVE,
  NEIGHBOUR_SOURCE,
  TENSE_REGION,
  TRANSLATORS_NOTE,
  joined,
} from './rendered-sheets-texts.test-fixture.ts';

//region Slate sheets

/**
 The evidence `judgeTranslateSlate` sends a translate slate, with the
 fixture's declared names and references (ledger B28).

 @param sourceText - original the slate renders

 @returns Evidence entries, in the order the judges read them

 @example
 ```ts
 const evidence = translatedEvidence({ sourceText: SOURCE, },);
 ```
 */
function translatedEvidence({ sourceText, }: { readonly sourceText: string; },): readonly SelectEvidence[] {
  return translateSlateEvidence({
    sourceText,
    neighbouringSourceText: NEIGHBOUR_SOURCE,
    neighbouringIncumbentText: NEIGHBOUR_ARCHIVE,
    identityContext: IDENTITY,
    referenceContext: REFERENCES,
  },);
}

/**
 The evidence `editor-ensemble.ts` sends a repair slate, with the fixture's
 declared names and references (ledger B28).

 @returns Evidence entries, the original first

 @example
 ```ts
 const evidence = repairedEvidence();
 ```
 */
function repairedEvidence(): readonly SelectEvidence[] {
  return [
    {
      label: 'ORIGINAL (Chinese)',
      text: SOURCE,
    },
    ...repairSelectionSourceEvidence({
      identityContext: IDENTITY,
      referenceContext: REFERENCES,
    },),
  ];
}

/**
 Every refinement mode, each of which asks its own slate.
 */
const REFINE_MODES: readonly {
  readonly name: string;
  readonly mode: RefineStageMode;
}[] = [
  {
    name: 'refine slate',
    mode: { kind: 'comparative', },
  },
  {
    name: 'refine objection slate',
    mode: {
      kind: 'objection-correction',
      groups: [{
        origin: 'consolidation gate',
        objections: ['The tense shifts mid-paragraph.',],
      },],
    },
  },
];

/**
 The typed decision's state and question, as the decisions endpoint reads them.

 @returns Both, as JSON

 @example
 ```ts
 const text = typedDecisionText();
 ```
 */
function typedDecisionText(): string {
  /**
   The choice asked as a typed decision.
   */
  const decision = selectDecision({
    task: TRANSLATE_SELECTION_TASK,
    criteria: translateSelectionCriteria({ lineStructured: false, },),
    evidence: translatedEvidence({ sourceText: SOURCE, },),
    rendered: [
      ARCHIVE,
      REPAIR,
    ],
    sourceText: SOURCE,
  },);
  return JSON.stringify({
    state: decision.state,
    questions: decision.questions,
  },);
}

/**
 Every selection slate by its own task, criteria and decline text, and the
 same choice asked as a typed decision (ledger X17).

 @returns One entry per sheet

 @example
 ```ts
 const sheets = slateSheets();
 ```
 */
export function slateSheets(): readonly RenderedSheet[] {
  return [
    {
      name: 'envelope slate',
      text: joined({
        messages: buildCandidateSelectMessages({
          task: ENVELOPE_SELECTION_TASK,
          criteria: ENVELOPE_SELECTION_CRITERIA,
          evidence: repairedEvidence(),
          rendered: [
            ARCHIVE_QUOTE,
            TENSE_REGION.editorAfter,
          ],
          declineConsequence: KEEPS_TRUSTED_TEXT,
          sourceText: SOURCE,
        },),
      },),
    },
    {
      name: 'chunk slate',
      text: joined({
        messages: buildCandidateSelectMessages({
          task: CHUNK_SELECTION_TASK,
          criteria: CHUNK_SELECTION_CRITERIA,
          evidence: repairedEvidence(),
          rendered: [
            ARCHIVE,
            REPAIR,
          ],
          declineConsequence: CHUNK_DECLINE_CONSEQUENCE,
          sourceText: SOURCE,
        },),
      },),
    },
    {
      name: 'translate challenge slate',
      text: joined({
        messages: buildCandidateSelectMessages({
          task: translateSelectionTask({ responsibility: 'decline-challenge', },),
          criteria: translatedSlateCriteria({
            sourceText: VERSE_SOURCE,
            archiveText: '',
            lineStructured: false,
          },),
          evidence: [
            ...translatedEvidence({ sourceText: VERSE_SOURCE, },),
            ...translatedSlateBreakEvidence({
              sourceText: VERSE_SOURCE,
              archiveText: '',
              slate: [
                {
                  index: 1,
                  text: `The kitten fell asleep. It dreamed of fish. ${TRANSLATORS_NOTE}`,
                  hash: 'slate/1',
                  origin: 'fresh',
                  producer: {
                    kind: 'model',
                    modelId: SEAT_HYPER_ONLY,
                  },
                },
                {
                  index: 2,
                  text: `The kitten fell asleep.<br/>It dreamed of fish. ${TRANSLATORS_NOTE}`,
                  hash: 'slate/2',
                  origin: 'fresh',
                  producer: {
                    kind: 'model',
                    modelId: SEAT_OPENROUTER_ONLY,
                  },
                },
              ],
            },),
          ],
          rendered: [
            `The kitten fell asleep. It dreamed of fish. ${TRANSLATORS_NOTE}`,
            `The kitten fell asleep.<br/>It dreamed of fish. ${TRANSLATORS_NOTE}`,
          ],
          declineConsequence: LEAVES_PASSAGE_UNTRANSLATED,
          sourceText: VERSE_SOURCE,
        },),
      },),
    },
    {
      name: 'translate front matter slate',
      text: joined({
        messages: buildCandidateSelectMessages({
          task: TRANSLATE_SELECTION_TASK,
          criteria: translateSelectionCriteria({
            lineStructured: false,
            syntax: 'front-matter',
          },),
          evidence: translatedEvidence({ sourceText: SOURCE, },),
          rendered: [
            ARCHIVE,
            REPAIR,
          ],
          declineConsequence: SHIPS_BY_PREFERENCE,
          sourceText: SOURCE,
        },),
      },),
    },
    ...REFINE_MODES.map(function refineSlate({
      name,
      mode,
    },): RenderedSheet {
      return {
        name,
        text: joined({
          messages: buildCandidateSelectMessages({
            ...buildRefineSelectionContext({
              mode,
              sourceText: SOURCE,
              repairedText: REPAIR,
              referenceContext: REFERENCES,
              identityContext: IDENTITY,
            },),
            rendered: [
              REPAIR,
              ARCHIVE,
            ],
            sourceText: SOURCE,
          },),
        },),
      };
    },),
    {
      name: 'typed decision',
      text: typedDecisionText(),
    },
  ];
}

//endregion Slate sheets
