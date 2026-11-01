/**
 Guards the evidence half of ledger S14: the editors and both editor
 selections read the declared names and the cited references the critic and
 the panel read, and the editors see the community renderings their selection
 judges are shown.

 The critic filed and the panel accepted issues with the declared names and
 the references on their sheets; the editors repairing those issues and the
 judges choosing among the repairs had neither, so an editor could "correct" a
 declared name or strip a detail a cited page states, and a judge could prefer
 it. The selection judges were shown the community renderings a candidate
 lacks; the editors writing the candidates never were.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  EditableEnvelope,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  buildEditorMessages,
  runEditorStage,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 English the editors repair.
 */
const TARGET = 'Mittens told of her operation.';

/**
 The whole sentence as the one editable region.
 */
const ENVELOPE: EditableEnvelope = {
  envelopeId: 'envelope/operation',
  startOffset: 0,
  endOffset: TARGET.length,
  baseText: TARGET,
  baseHash: hashContent({ content: TARGET, },),
  issueIds: [],
};

/**
 Declared names, marked so a sheet carrying them is visible.
 */
const IDENTITY = '- name: ORIGINAL declares "猫猫", TRANSLATION declares "Mittens" (IDENTITY MARKER)';

/**
 One cited reference, marked the same way.
 */
const REFERENCES = '- reference 1 https://example.test/mittens: Mittens wrote about the tabby club. (REFERENCE MARKER)';

/**
 Judges whose sheets are recorded.
 */
const JUDGES: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

await describe({
  name: 'the editors read the page evidence (ledger S14)',
  children: [
    it({
      name: 'SHOWS the declared names, their rules and the references on the editor sheet',
      fn: async () => {
        /**
         Both halves of the editor sheet.
         */
        const [system, user,] = buildEditorMessages({
          sourceText: '猫猫讲起自切的经历。',
          targetText: TARGET,
          envelopes: [ENVELOPE,],
          issues: [],
          identityContext: IDENTITY,
          referenceContext: REFERENCES,
        },).messages.map(function textOf(message,): string {
          return ((typeof message.content) === 'string') ? message.content : JSON.stringify(message.content,);
        },);
        expect(system,).toContain('Declared identity, when a DECLARED NAMES block precedes the documents',);
        expect(user,).toContain('IDENTITY MARKER',);
        expect(user,).toContain('REFERENCE MARKER',);
        expect(user,).toContain('CITED REFERENCES, EVIDENCE ONLY',);
      },
    },),
    it({
      name: 'SHOWS the editors the community rendering the translation lacks, as its selection judges see it',
      fn: async () => {
        /**
         User half of the editor sheet.
         */
        const user = buildEditorMessages({
          sourceText: '猫猫讲起自切的经历。',
          targetText: TARGET,
          envelopes: [ENVELOPE,],
          issues: [],
        },).messages.at(1,)?.content ?? '';
        expect(user,).toContain('COMMUNITY RENDERINGS, evidence to weigh, not a verdict:',);
      },
    },),
    it({
      name: 'CARRIES the declared names and the references through the editor stage to the editors and both '
        + 'selections',
      fn: async () => {
        /**
         Every editor sheet sent.
         */
        const editorSheets: string[] = [];
        /**
         Every selection sheet sent.
         */
        const selectionSheets: string[] = [];
        /**
         Client whose editors propose two repairs and whose judges pick the first.
         */
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('Unexpected text call',);
          },
          quotas: async () => {
            throw new Error('Unexpected quota call',);
          },
          chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
            /**
             Sheet as the voice reads it.
             */
            const sheet = JSON.stringify(request.messages,);
            /**
             Whether this call is an editor's.
             */
            const editing = request.responseFormat?.json_schema.name === 'editor_report';
            (editing ? editorSheets : selectionSheets).push(sheet,);
            /**
             Reply for the stage.
             */
            const value: unknown = editing
              ? {
                edits: [{
                  region: 1,
                  newText: (request.modelId === SEAT_HYPER_OPENROUTER_VISION_EDITOR)
                    ? 'Mittens told of her self-surgery.'
                    : 'Mittens spoke of her self-surgery.',
                },],
              }
              : { best: 1, assessment: 'Fixture decision.', };
            if (!request.validate(value,))
              throw new Error('Invalid fixture reply',);
            return {
              kind: 'ok',
              value: value as ValueT,
              rawText: JSON.stringify(value,),
            };
          },
        };
        await runEditorStage({
          client,
          editorModelIds: JUDGES.slice(0, 2,),
          judgeModelIds: JUDGES,
          sourceText: '猫猫讲起自切的经历。',
          targetText: TARGET,
          envelopes: [ENVELOPE,],
          issues: [],
          identityContext: IDENTITY,
          referenceContext: REFERENCES,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'editor-page-evidence-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(editorSheets.length,).toBeGreaterThan(0,);
        expect(selectionSheets.length,).toBeGreaterThan(0,);
        for (const sheet of [...editorSheets, ...selectionSheets,]) {
          expect(sheet,).toContain('IDENTITY MARKER',);
          expect(sheet,).toContain('REFERENCE MARKER',);
        }
      },
    },),
  ],
},);
