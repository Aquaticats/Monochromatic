/**
 Repair selectors need source evidence and the existing English before judging changes.

 @module
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type {
  EditableEnvelope,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import { applyPatchOperations, } from '@monochromatic-dev/translation-repair-page-pass';
import {
  chunkCandidateOf,
  type EditorCandidate,
  repairPreparedDocument,
  repairSliceKey,
  runEditorStage,
  selectChunkPatch,
  selectPerEnvelope,
} from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/** Original English identifies what was already carried, not which model authored a repair. */
const TARGET = 'She greeted her friend.';
/** Whole sentence is the fixture's editable region. */
const ENVELOPE: EditableEnvelope = { envelopeId: 'envelope/greeting', startOffset: 0, endOffset: TARGET.length, baseText: TARGET, baseHash: hashContent({ content: TARGET, }), issueIds: [], };
/** Independent fixture electorate retains ordinary author weighting. */
const JUDGES: readonly RosterModelId[] = [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_SYNTHETIC_TEXT_EVERYWHERE,];

/**
 Builds an applied candidate rather than an unverified raw replacement.
 @param input - actual producer and proposed sentence
 @returns Gated fixture patch
 @example
 ```ts
 const candidate = proposed({ modelId: JUDGES[0], text: 'Her friend greeted her.' });
 ```
 */
function proposed({ modelId, text, }: { readonly modelId: RosterModelId; readonly text: string; }): EditorCandidate {
  return { modelId, patch: applyPatchOperations({ targetText: TARGET, envelopes: [ENVELOPE,], operations: [{ envelopeId: ENVELOPE.envelopeId, baseHash: ENVELOPE.baseHash, newText: text, },], preservation: { mode: 'skip', }, }), };
}

/**
 Chooses the source-correct fixture only when its deciding evidence reaches the actual call.
 @returns Client and captured user sheets
 @example
 ```ts
 const fixture = judges();
 ```
 */
function judges(): { readonly client: ModelClient; readonly prompts: string[]; } {
  const prompts: string[] = [];
  const client: ModelClient = {
    chatText: async () => { throw new Error('Unexpected text call'); },
    quotas: async () => { throw new Error('Unexpected quota call'); },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>): Promise<ChatJsonOutcome<ValueT>> => {
      const prompt = JSON.stringify(request.messages.filter(message => message.role === 'user'));
      prompts.push(prompt);
      const value = { best: prompt.includes('DOCUMENT SOURCE MARKER') && prompt.includes('NEARBY SOURCE MARKER') ? 2 : 1, assessment: 'Fixture decision from supplied source evidence.', };
      if (!request.validate(value)) throw new Error('Invalid fixture ballot');
      return { kind: 'ok', value, rawText: JSON.stringify(value), };
    },
  };
  return { client, prompts, };
}

/** Already-applied repairs differ in the event's actor. */
const CANDIDATES = [
  proposed({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, text: 'She greeted her friend warmly.', }),
  proposed({ modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, text: 'Her friend greeted her.', }),
];

await describe({
  name: 'repair source evidence handoff',
  children: [
    it({
      name: 'CARRIES same-entry source through the actual prepared-document repair boundary',
      fn: async () => {
        const sourceText = '猫笑了。\n\n## Later\n\nDOCUMENT SOURCE MARKER';
        const targetText = 'The cat cried.\n\n## Later\n\nAn unrelated passage.';
        const prepared = prepareDocumentPair({ sourceText, targetText, });
        const controller = new AbortController();
        const reason = new Error('Stop after observing the panel boundary');
        const prompts: string[] = [];
        const client: ModelClient = {
          chatText: async () => { throw new Error('Unexpected text call'); },
          quotas: async () => { throw new Error('Unexpected quota call'); },
          chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>): Promise<ChatJsonOutcome<ValueT>> => {
            if (request.responseFormat?.json_schema.name === 'panel_ballot') {
              prompts.push(JSON.stringify(request.messages));
              controller.abort(reason);
              throw reason;
            }
            const value = { issues: [{ category: 'accuracy/mistranslation', severity: 'major', summary: 'The cat smiled rather than cried.', sourceQuote: '猫笑了。', targetQuote: 'The cat cried.', },], };
            if (!request.validate(value)) throw new Error('Unexpected fixture protocol');
            return { kind: 'ok', value, rawText: JSON.stringify(value), };
          },
        };
        let caught: unknown;
        try {
          await repairPreparedDocument({ client, prepared, models: { criticModelIds: JUDGES, panelModelIds: JUDGES, editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,], judgeModelIds: JUDGES, checkerModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_SYNTHETIC_TEXT_EVERYWHERE,], }, signal: controller.signal, perCallTimeoutMs: HANG_STOP_MS, stageSettings: SCRIPTED_STAGE_SETTINGS, });
        }
        catch (error) {
          caught = error;
        }
        expect(caught).toBe(reason);
        expect(prompts.length).toBeGreaterThan(0);
        expect(prompts.every(prompt => prompt.includes('FULL ORIGINAL DOCUMENT') && prompt.includes('DOCUMENT SOURCE MARKER'))).toBe(true);
      },
    }),
    it({
      name: 'CARRIES document evidence through the editor stage to both selecting responsibilities',
      fn: async () => {
        const fixture = judges();
        const client: ModelClient = { ...fixture.client, chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>): Promise<ChatJsonOutcome<ValueT>> => {
          if (request.responseFormat?.json_schema.name !== 'editor_report') return await fixture.client.chatJson(request);
          const value = { edits: [{ region: 1, newText: request.modelId === SEAT_HYPER_OPENROUTER_VISION_EDITOR ? 'She greeted her friend warmly.' : 'Her friend greeted her.', },], };
          if (!request.validate(value)) throw new Error('Invalid fixture editor reply');
          return { kind: 'ok', value, rawText: JSON.stringify(value), };
        }, };
        const input = { client, editorModelIds: JUDGES.slice(0, 2), judgeModelIds: JUDGES, sourceText: '猫笑了。', targetText: TARGET, envelopes: [ENVELOPE,], issues: [], neighbouringSourceText: 'NEARBY SOURCE MARKER', documentSourceText: 'DOCUMENT SOURCE MARKER', signal: new AbortController().signal, perCallTimeoutMs: HANG_STOP_MS, l: tagged({ tag: 'repair-evidence-test', }), stageSettings: SCRIPTED_STAGE_SETTINGS, };
        const result = await runEditorStage(input);
        expect(result.patch.patchedText).toBe('Her friend greeted her.');
        expect(fixture.prompts.length).toBeGreaterThan(0);
        expect(fixture.prompts.every(prompt => prompt.includes('DOCUMENT SOURCE MARKER'))).toBe(true);
        expect(fixture.prompts.some(prompt => prompt.includes('EXISTING ENGLISH BEFORE REPAIR'))).toBe(true);
      },
    }),
    it({
      name: 'CHANGES cache identity when full source evidence changes outside the neighboring window',
      fn: async () => {
        const base = { runShape: 'fixture', sourceText: '猫笑了。', targetText: TARGET, lineStructured: false, neighbouringSourceText: '近处。', };
        const first = { ...base, documentSourceText: 'FIRST document evidence', };
        const second = { ...base, documentSourceText: 'SECOND document evidence', };
        expect(repairSliceKey(first)).not.toBe(repairSliceKey(second));
        expect(repairSliceKey({ ...base, documentSourceText: '', })).toBe(repairSliceKey(base));
        const sourceOnly = { ...base, neighbouringSourceText: 'SAME', };
        const documentOnly = { ...base, neighbouringSourceText: '', documentSourceText: 'SAME', };
        expect(repairSliceKey(sourceOnly)).not.toBe(repairSliceKey(documentOnly));
      },
    }),
    it({
      name: 'SHOWS both source evidence scopes to the real regional selector',
      fn: async () => {
        const fixture = judges();
        const input = {
          client: fixture.client, candidates: CANDIDATES, envelopes: [ENVELOPE,], judgeModelIds: JUDGES,
          sourceText: '猫笑了。', targetText: TARGET, neighbouringSourceText: 'NEARBY SOURCE MARKER', documentSourceText: 'DOCUMENT SOURCE MARKER',
          signal: new AbortController().signal, perCallTimeoutMs: HANG_STOP_MS, l: tagged({ tag: 'repair-evidence-test', }),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        };
        const selected = await selectPerEnvelope(input);
        expect(selected.operations[0]?.newText).toBe('Her friend greeted her.');
        expect(fixture.prompts.every(prompt => prompt.includes('NEARBY SOURCE MARKER') && prompt.includes('DOCUMENT SOURCE MARKER'))).toBe(true);
      },
    }),
    it({
      name: 'SHOWS source evidence and before-repair English to whole-chunk selection without inventing a candidate',
      fn: async () => {
        const fixture = judges();
        const offered = CANDIDATES.map(candidate => chunkCandidateOf(candidate));
        const [fallback,] = offered;
        if (fallback === undefined) throw new Error('Missing fixture candidate');
        const input = {
          client: fixture.client, candidates: offered, judgeModelIds: JUDGES, sourceText: '猫笑了。',
          neighbouringSourceText: 'NEARBY SOURCE MARKER', documentSourceText: 'DOCUMENT SOURCE MARKER',
          indecisionFallback: fallback, rejectionFallback: { patchedText: TARGET, applied: [], rejected: [], },
          signal: new AbortController().signal, perCallTimeoutMs: HANG_STOP_MS, l: tagged({ tag: 'repair-evidence-test', }),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        };
        const selected = await selectChunkPatch(input);
        expect(selected.patch.patchedText).toBe('Her friend greeted her.');
        expect(fixture.prompts.every(prompt => prompt.includes('EXISTING ENGLISH BEFORE REPAIR') && prompt.includes(TARGET))).toBe(true);
        expect(selected.rounds[0]?.slate).toHaveLength(2);
      },
    }),
  ],
});
