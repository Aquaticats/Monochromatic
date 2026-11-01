/**
 Tests for the editor ensemble: candidate assembly, producer provenance,
 roster invariants, judge prompt fencing, and the two decline dispositions.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type {
  EditableEnvelope,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  type CandidateProducer,
  describeProducer,
  mergeProducers,
  producerModelIds,
} from '@monochromatic-dev/translation-repair-house-rule';
import { buildCandidateSelectMessages, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  assertCheckerIndependence,
  assertJudgeableEditorRoster,
  CheckerIndependenceError,
  ProducerRosterError,
} from '@monochromatic-dev/translation-repair-assembly';
import {
  applyPatchOperations,
  type PatchOutcome,
} from '@monochromatic-dev/translation-repair-page-pass';
import {
  buildChunkCandidates,
  type EditorCandidate,
  pickFallbackCandidate,
  selectPerEnvelope,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  candidateFor,
  ENVELOPE,
  TARGET_TEXT,
} from './editor-candidate-envelope.test-fixture.ts';
import { candidateCarrying, } from '@monochromatic-dev/translation-repair-test-fixture/ts/translate-ballot.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the selection under test.
 */
const l = tagged({ tag: 'editor-ensemble-test', },);

/**
 Second envelope of the fixture document, covering the sentence after the one
 `ENVELOPE` covers.
 */
const BOWL_ENVELOPE: EditableEnvelope = {
  envelopeId: 'envelope/bowl',
  startOffset: TARGET_TEXT.indexOf('The bowl stays full.',),
  endOffset: TARGET_TEXT.indexOf('The bowl stays full.',)
    + 'The bowl stays full.'.length,
  baseText: 'The bowl stays full.',
  baseHash: hashContent({ content: 'The bowl stays full.', },),
  issueIds: ['adjudicated/bowl',],
};

/**
 Replacement the judges back for the first envelope, written by the same model
 that wrote the only proposal for the second.
 */
const BACKED_REPLACEMENT = 'The cat chases butterflies.';

/**
 Judges every selection here asks.
 */
const JUDGES: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
];

/**
 Client whose every judge backs the candidate carrying one sentence.

 @param backed - sentence the backed candidate carries

 @returns Client answering each ballot from what its sheet shows

 @example
 ```ts
 const client = backingClient({ backed: BACKED_REPLACEMENT, },);
 ```
 */
function backingClient({ backed, }: { readonly backed: string; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by envelope selection',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Ballot backing the candidate that carries the sentence.
       */
      const scripted: unknown = {
        best: candidateCarrying({
          content: request.messages
            .map(function toContent(message,): string {
              return messageText({ message, },);
            },)
            .join('\n',),
          needle: backed,
        },),
        assessment: 'scripted',
      };
      if (!request.validate(scripted,))
        throw new Error('stub script failed the ballot guard',);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by envelope selection',);
    },
  };
}

/**
 Builds one editor candidate proposing a replacement for each of the two
 envelopes it is given.

 @param modelId - proposing model

 @param replacements - new text per envelope id the model proposed for

 @returns Candidate carrying the gated patch

 @example
 ```ts
 const candidate = candidateOverTwo({ modelId, replacements: { [ENVELOPE.envelopeId]: 'text', }, },);
 ```
 */
function candidateOverTwo(
  {
    modelId,
    replacements,
  }: {
    readonly modelId: RosterModelId;
    readonly replacements: Readonly<Record<string, string>>;
  },
): EditorCandidate {
  return {
    modelId,
    patch: applyPatchOperations({
      targetText: TARGET_TEXT,
      envelopes: [
        ENVELOPE,
        BOWL_ENVELOPE,
      ],
      operations: Object.entries(replacements,)
        .map(function toOperation([envelopeId, newText,],) {
          return {
            envelopeId,
            baseHash: (envelopeId === ENVELOPE.envelopeId) ? ENVELOPE.baseHash : BOWL_ENVELOPE.baseHash,
            newText,
          };
        },),
      preservation: { mode: 'skip', },
    },),
  };
}

/**
 Apply-gate outcome that repairs nothing, standing for the untouched chunk.
 */
const EMPTY_PATCH: PatchOutcome = {
  patchedText: TARGET_TEXT,
  applied: [],
  rejected: [],
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: selectPerEnvelope.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CREDITS A MODEL ONCE when it wrote an envelope adopted without a vote and the judged winner of '
            + 'another, and credits the model whose proposal lost to nobody',
          fn: async () => {
            const selection = await selectPerEnvelope({
              client: backingClient({ backed: BACKED_REPLACEMENT, },),
              candidates: [
                candidateOverTwo({
                  modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  replacements: {
                    [BOWL_ENVELOPE.envelopeId]: 'The bowl is full.',
                    [ENVELOPE.envelopeId]: BACKED_REPLACEMENT,
                  },
                },),
                candidateOverTwo({
                  modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                  replacements: { [ENVELOPE.envelopeId]: 'The cat loves chasing butterflies.', },
                },),
              ],
              envelopes: [
                BOWL_ENVELOPE,
                ENVELOPE,
              ],
              judgeModelIds: JUDGES,
              sourceText: '猫猫喜欢追蝴蝶。',
              targetText: TARGET_TEXT,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect({
              contributors: selection.contributors,
              soleCount: selection.soleCount,
              judgedCount: selection.judgedCount,
              declinedCount: selection.declinedCount,
              operations: selection.operations.map(function toEnvelopeId(operation,): string {
                return operation.envelopeId;
              },),
            },).toEqual({
              contributors: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
              soleCount: 1,
              judgedCount: 1,
              declinedCount: 0,
              operations: [
                BOWL_ENVELOPE.envelopeId,
                ENVELOPE.envelopeId,
              ],
            },);
          },
        },),
      ],
    },),
    describe({
      name: mergeProducers.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'unions the stakes of two models, keeps a single model singular, '
            + 'and preserves first-seen order through a composite',
          fn: async () => {
            /** Two distinct models writing identical text. */
            const both = mergeProducers({
              left: {
                kind: 'model',
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              },
              right: {
                kind: 'model',
                modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              },
            },);
            expect(both.kind,).toBe('composite',);
            expect([...producerModelIds(both,),],).toEqual([
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            ],);

            // One model named twice is still one stakeholder, not a pair.
            /** Same model on both sides. */
            const same = mergeProducers({
              left: {
                kind: 'model',
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              },
              right: {
                kind: 'model',
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              },
            },);
            expect(same.kind,).toBe('model',);
            expect([...producerModelIds(same,),],).toEqual([SEAT_HYPER_OPENROUTER_VISION_EDITOR,],);

            /** Composite absorbing a model already among its contributors. */
            const widened = mergeProducers({
              left: {
                kind: 'composite',
                contributors: [
                  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                ],
              },
              right: {
                kind: 'model',
                modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
              },
            },);
            expect([...producerModelIds(widened,),],).toEqual([
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              SEAT_SYNTHETIC_VISION_WITHHELD,
            ],);
          },
        },),

        it({
          name: 'describes a composite by its contributors rather than one model',
          fn: async () => {
            /** Stitched provenance. */
            const producer: CandidateProducer = {
              kind: 'composite',
              contributors: [
                SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              ],
            };
            expect(describeProducer(producer,),).toBe(
              'composite(hf:zai-org/GLM-5.3-Flash + hf:Qwen/Qwen3.8-27B)',
            );
            expect(
              describeProducer({
                kind: 'model',
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              },),
            ).toBe(SEAT_HYPER_OPENROUTER_VISION_EDITOR,);
          },
        },),
      ],
    },),

    describe({
      name: buildChunkCandidates.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'keeps distinct proposals apart and counts no collapse',
          fn: async () => {
            /** Two editors disagreeing on the wording. */
            const set = buildChunkCandidates({
              candidates: [
                candidateFor({
                  modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  newText: 'The cat chases butterflies.',
                },),
                candidateFor({
                  modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                  newText: 'The cat loves chasing butterflies.',
                },),
              ],
              composite: EMPTY_PATCH,
              contributors: [],
            },);
            expect(set.candidates.length,).toBe(2,);
            expect(set.collapsed,).toBe(0,);
          },
        },),

        it({
          name: 'drops a composite that repairs nothing rather than offering the '
            + 'untouched translation as a candidate',
          fn: async () => {
            /** Composite assembled from no winning operation. */
            const set = buildChunkCandidates({
              candidates: [
                candidateFor({
                  modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  newText: 'The cat chases butterflies.',
                },),
              ],
              composite: EMPTY_PATCH,
              contributors: [],
            },);
            expect(set.candidates.length,).toBe(1,);
            expect(
              set.candidates.every(function repairsSomething(candidate,) {
                return candidate.value
                  .applied
                  .length
                  > 0;
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'collapses identical text and unions every stake, so no producer '
            + 'is freed to judge its own words',
          fn: async () => {
            /** Wording both editors and the composite arrived at. */
            const agreed = 'The cat chases butterflies.';

            /** Composite carrying only the second editor as contributor. */
            const composite = applyPatchOperations({
              targetText: TARGET_TEXT,
              envelopes: [ENVELOPE,],
              operations: [
                {
                  envelopeId: ENVELOPE.envelopeId,
                  baseHash: ENVELOPE.baseHash,
                  newText: agreed,
                },
              ],
              preservation: { mode: 'skip', },
            },);

            /** One editor patch plus an identical composite from another model. */
            const set = buildChunkCandidates({
              candidates: [
                candidateFor({
                  modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  newText: agreed,
                },),
              ],
              composite,
              contributors: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
            },);
            expect(set.candidates.length,).toBe(1,);
            expect(set.collapsed,).toBe(1,);

            /** Survivor of the collapse. */
            const [survivor,] = set.candidates;
            expect(survivor,).toBeDefined();

            /** Everyone with a stake in the surviving text. */
            const stakes = new Set(producerModelIds(
              survivor?.producer ?? {
                kind: 'composite',
                contributors: [],
              },
            ));
            // Both the editor whose candidate survived and the composite's
            // contributor must stay barred; dropping either lets that model judge
            // text it wrote.
            expect(stakes.has(SEAT_HYPER_OPENROUTER_VISION_EDITOR,),).toBe(true,);
            expect(stakes.has(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: pickFallbackCandidate.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'prefers the editor that landed more operations',
          fn: async () => {
            /** Editor landing nothing, because its replacement was a no-op. */
            const idle = candidateFor({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              newText: ENVELOPE.baseText,
            },);

            /** Editor landing a real replacement. */
            const working = candidateFor({
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              newText: 'The cat chases butterflies.',
            },);
            expect(idle.patch
              .applied
              .length,).toBe(0,);

            /**
             Editor the picker chose, read for its identity as well as its text.
             */
            const chosen = pickFallbackCandidate({
              candidates: [
                idle,
                working,
              ],
            },);

            expect(chosen.patch
              .patchedText,).toContain('The cat chases butterflies.',);
            // The identity is the point of returning a candidate rather than a
            // patch: this exact path ships text after the judges decline, and the
            // checker that wrote it must be discounted for judging its own work.
            expect(chosen.modelId,).toBe(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,);
          },
        },),

        it({
          name: 'breaks a tie on applied count by roster order, not by wording',
          fn: async () => {
            /** Two editors each landing exactly one operation. */
            const first = candidateFor({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              newText: 'The cat chases butterflies.',
            },);

            /** Later editor in roster order. */
            const second = candidateFor({
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              newText: 'The cat loves chasing butterflies.',
            },);
            expect(
              pickFallbackCandidate({
                candidates: [
                  first,
                  second,
                ],
              },).patch
                .patchedText,
            ).toContain('The cat chases butterflies.',);
            // Reversing the roster reverses the winner, proving order decides
            // rather than anything about the text.
            expect(
              pickFallbackCandidate({
                candidates: [
                  second,
                  first,
                ],
              },).patch
                .patchedText,
            ).toContain('The cat loves chasing butterflies.',);
          },
        },),
      ],
    },),

    describe({
      name: assertJudgeableEditorRoster.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'passes a roster with enough disinterested judges',
          fn: async () => {
            assertJudgeableEditorRoster({
              editorModelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,],
              judgeModelIds: [
                SEAT_SYNTHETIC_VISION_WITHHELD,
                SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              ],
            },);
          },
        },),

        it({
          name: 'ACCEPTS two editors judged by themselves and one other, which the '
            + 'old rule refused: three answering judges can decide, and the ruling of '
            + '2026-08-15 says a stake discounts an opinion rather than voiding it',
          fn: async () => {
            expect(function twoEditorsOneOutsider() {
              assertJudgeableEditorRoster({
                editorModelIds: [
                  SEAT_SYNTHETIC_VISION_WITHHELD,
                  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                ],
                judgeModelIds: [
                  SEAT_SYNTHETIC_VISION_WITHHELD,
                  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                ],
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'refuses a roster that could never draw two ballots, which '
            + 'would decline every round in silence: a lone judge cannot decide a stage, '
            + 'whoever wrote the text',
          fn: async () => {
            expect(function everyJudgeEdits() {
              assertJudgeableEditorRoster({
                editorModelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,],
                judgeModelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,],
              },);
            },).toThrow(ProducerRosterError,);
          },
        },),

        it({
          name: 'ACCEPTS two editors judging only each other, which refused until 2026-10-25 because the best '
            + 'candidate could draw a weight of 1.5: it draws two ballots, and two ballots decide',
          fn: async () => {
            expect(function twoEditorsJudgingThemselves() {
              assertJudgeableEditorRoster({
                editorModelIds: [
                  SEAT_SYNTHETIC_VISION_WITHHELD,
                  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                ],
                judgeModelIds: [
                  SEAT_SYNTHETIC_VISION_WITHHELD,
                  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                ],
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'refuses repeated and empty editor rosters, since a repeated id is '
            + 'one model counted twice rather than an independent voice',
          fn: async () => {
            expect(function repeatedEditor() {
              assertJudgeableEditorRoster({
                editorModelIds: [
                  SEAT_SYNTHETIC_VISION_WITHHELD,
                  SEAT_SYNTHETIC_VISION_WITHHELD,
                ],
                judgeModelIds: [
                  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                ],
              },);
            },).toThrow(ProducerRosterError,);

            expect(function noEditor() {
              assertJudgeableEditorRoster({
                editorModelIds: [],
                judgeModelIds: [
                  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                ],
              },);
            },).toThrow(ProducerRosterError,);
          },
        },),
      ],
    },),

    describe({
      name: assertCheckerIndependence.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'passes disjoint rosters and refuses a checker that also edits',
          fn: async () => {
            assertCheckerIndependence({
              editorModelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,],
              checkerModelIds: [
                SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              ],
            },);

            expect(function checksOwnWork() {
              assertCheckerIndependence({
                editorModelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,],
                checkerModelIds: [
                  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  SEAT_SYNTHETIC_VISION_WITHHELD,
                ],
              },);
            },).toThrow(CheckerIndependenceError,);
          },
        },),
      ],
    },),

    describe({
      name: buildCandidateSelectMessages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'fences candidates so a setext underline inside one cannot close '
            + 'its own block',
          fn: async () => {
            /** Candidate whose own text carries the default fence. */
            const messages = buildCandidateSelectMessages({
              task: 'Pick one.',
              criteria: ['Faithful.',],
              evidence: [
                {
                  label: 'ORIGINAL',
                  text: '猫猫喜欢追蝴蝶。',
                },
              ],
              rendered: [
                'A heading\n=====\nand its body.',
                'Plain replacement.',
              ],
            },);

            /** Judge-facing prompt. */
            const content = messages.at(-1,)?.content ?? '';
            // A five-character fence would be closed by the candidate's own
            // underline, so the chosen fence has to be longer.
            expect(content.includes('\n======\n',),).toBe(true,);
            expect(content.split('\n======\n',).length - 1,).toBeGreaterThan(1,);
          },
        },),

        it({
          name: 'outgrows any run of the fence character the content contains',
          fn: async () => {
            /** Candidate carrying a long run of the fence character. */
            const messages = buildCandidateSelectMessages({
              task: 'Pick one.',
              criteria: ['Faithful.',],
              evidence: [
                {
                  label: 'ORIGINAL',
                  text: '========== a source that also fences',
                },
              ],
              rendered: ['Plain replacement.',],
            },);

            /** Judge-facing prompt. */
            const content = messages.at(-1,)?.content ?? '';
            expect(content.includes('\n===========\n',),).toBe(true,);
          },
        },),

        it({
          name: 'labels every evidence block and numbers candidates from one',
          fn: async () => {
            /** Prompt over two evidence blocks. */
            const messages = buildCandidateSelectMessages({
              task: 'Pick one.',
              criteria: ['Faithful.',],
              evidence: [
                {
                  label: 'ORIGINAL (Chinese)',
                  text: '猫猫喜欢追蝴蝶。',
                },
                {
                  label: 'PASSAGE BEING REPLACED (current English)',
                  text: 'The cat hates butterflies.',
                },
              ],
              rendered: [
                'The cat chases butterflies.',
                'The cat loves chasing butterflies.',
              ],
            },);

            /** Judge-facing prompt. */
            const content = messages.at(-1,)?.content ?? '';
            expect(content.includes('ORIGINAL (Chinese)',),).toBe(true,);
            expect(content.includes('PASSAGE BEING REPLACED (current English)',),).toBe(true,);
            expect(content.includes('CANDIDATE 1',),).toBe(true,);
            expect(content.includes('CANDIDATE 2',),).toBe(true,);
            expect(content.includes('CANDIDATE 3',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
