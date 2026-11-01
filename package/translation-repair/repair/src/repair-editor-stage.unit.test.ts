/**
 Tests for the editor stage's guards, its early exits, and the composite it
 assembles once judging actually runs.

 `runEditorStage` had no test. Its judged path is covered PIECEWISE through
 `selectPerEnvelope` and `selectChunkPatch`, which have their own suites, but
 neither drives the WIRING between them: `applyCandidate` rebuilding
 per-envelope winners into a composite, and that composite then competing at
 chunk level, only happens inside `runEditorStage` itself. The rest of what
 this file adds is the two places the stage decides NOT to go there.

 Both of those are cost properties as much as correctness ones. The provider is
 flat-rate but not unlimited, and a run spends its capacity on judge calls it
 did not need or on a fan-out against a roster that could never have been
 judged. Neither shows up as an error; both show up as a pass that ran out of
 budget with fewer entries settled.

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
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type {
  AdjudicatedIssue,
  EditableEnvelope,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { ProducerRosterError, } from '@monochromatic-dev/translation-repair-assembly';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import { runEditorStage, } from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { replyWith, } from '@monochromatic-dev/translation-repair-provider/ts/scripted-reply-outcome.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the stages under test.
 */
const l = tagged({ tag: 'editor-stage-test', },);

/**
 Translation chunk the editors propose against.
 */
const TARGET_TEXT = 'The cat is doing the sleeping on the windowsill.';

/**
 Original the edits answer to.
 */
const SOURCE_TEXT = '猫猫在窗台上睡觉。';

/**
 Single editable envelope covering the whole chunk.
 */
const ENVELOPES: readonly EditableEnvelope[] = [
  {
    envelopeId: 'envelope/0',
    startOffset: 0,
    endOffset: TARGET_TEXT.length,
    baseText: TARGET_TEXT,
    baseHash: hashContent({ content: TARGET_TEXT, },),
    issueIds: ['adjudicated/tense',],
  },
];

/**
 Accepted issue the edit answers.
 */
const ISSUES: readonly AdjudicatedIssue[] = [
  {
    issueId: 'adjudicated/tense',
    status: 'accepted' as const,
    severity: 'major' as const,
    claims: [],
    tallies: {},
  },
];

/**
 Editors that propose candidates.
 */
const EDITORS = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Judges with no stake in either editor's output.
 */
const JUDGES = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 Client answering every editor with one scripted report, counting calls.

 @param report - report each editor returns

 @param calls - shared counter the cases assert on

 @returns Client honoring that script

 @example
 ```ts
 const client = editorClient({ report: { edits: [], }, calls, },);
 ```
 */
function editorClient(
  {
    report,
    calls,
  }: {
    readonly report: unknown;
    readonly calls: { count: number; };
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      calls.count += 1;
      if (!request.validate(report,))
        throw new Error('scripted report failed the editor guard',);
      return {
        kind: 'ok',
        value: report,
        rawText: JSON.stringify(report,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Client that fails the moment it is called, proving a guard ran first.

 @returns Client refusing every exchange

 @example
 ```ts
 const client = neverCalledClient();
 ```
 */
function neverCalledClient(): ModelClient {
  return {
    chatText: async () => {
      throw new Error('the stage called a model it should not have',);
    },
    chatJson: async () => {
      throw new Error('the stage called a model it should not have',);
    },
    quotas: async () => {
      throw new Error('the stage called a model it should not have',);
    },
  };
}

await describe({
  name: runEditorStage.name,
  children: [
    it({
      name: 'REFUSES AN UNJUDGEABLE ROSTER BEFORE CALLING ANYTHING. A roster '
        + 'seating one judge can draw one ballot, and a winner needs two; '
        + 'discovering that after the fan-out would mean paying for a whole '
        + 'editor round whose output could never be judged',
      fn: async () => {
        await expect(
          runEditorStage({
            client: neverCalledClient(),
            editorModelIds: EDITORS,
            // One judge alone never decides, whoever wrote the text.
            judgeModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
            sourceText: SOURCE_TEXT,
            targetText: TARGET_TEXT,
            envelopes: ENVELOPES,
            issues: ISSUES,
            signal: new AbortController().signal,
            perCallTimeoutMs: HANG_STOP_MS,
            l,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },),
        ).rejects.toThrow(ProducerRosterError,);
      },
    },),

    it({
      name: 'returns the translation UNTOUCHED and calls no judge when every '
        + 'editor proposed nothing, so a slice the editors declined costs one '
        + 'editor round and not a judging round on top of it',
      fn: async () => {
        /**
         Exchange counter, so judge calls would be visible as extra calls.
         */
        const calls = { count: 0, };

        /**
         Stage where both editors returned an empty edit list.
         */
        const result = await runEditorStage({
          client: editorClient({
            report: { edits: [], },
            calls,
          },),
          editorModelIds: EDITORS,
          judgeModelIds: JUDGES,
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          envelopes: ENVELOPES,
          issues: ISSUES,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(result.patch.patchedText,).toBe(TARGET_TEXT,);
        expect(result.patch.applied,).toStrictEqual([],);
        expect(result.heardEditors,).toBe(EDITORS.length,);
        // Exactly one exchange per editor, and nothing more.
        expect(calls.count,).toBe(EDITORS.length,);
      },
    },),

    it({
      name: 'takes the same untouched exit when every proposed operation was '
        + 'REJECTED by the apply gate, since a candidate that landed no '
        + 'operation is the untouched translation and judging identical texts '
        + 'would spend calls to learn nothing',
      fn: async () => {
        /**
         Exchange counter for this case.
         */
        const calls = { count: 0, };

        /**
         Stage where both editors named a region that is not on the sheet.
         */
        const result = await runEditorStage({
          client: editorClient({
            report: {
              edits: [
                {
                  region: 9,
                  newText: 'The dog barks.',
                },
              ],
            },
            calls,
          },),
          editorModelIds: EDITORS,
          judgeModelIds: JUDGES,
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          envelopes: ENVELOPES,
          issues: ISSUES,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(result.patch.patchedText,).toBe(TARGET_TEXT,);
        expect(calls.count,).toBe(EDITORS.length,);
        expect(result.findings.length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'records how many editors were heard and how many repaired, so a '
        + 'slice decided by half the roster is visible in the scorecard rather '
        + 'than reading like a full-roster result',
      fn: async () => {
        /**
         Exchange counter for this case.
         */
        const calls = { count: 0, };

        /**
         Stage where both editors declined to propose anything.
         */
        const result = await runEditorStage({
          client: editorClient({
            report: { edits: [], },
            calls,
          },),
          editorModelIds: EDITORS,
          judgeModelIds: JUDGES,
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          envelopes: ENVELOPES,
          issues: ISSUES,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(
          result.findings.some(function namesTheCount(finding,) {
            return finding.includes('editor-candidates',)
              && finding.includes('0 repairing',);
          },),
        ).toBe(true,);
      },
    },),

    it({
      name: 'SHIPS THE COMPOSITE `applyCandidate` ASSEMBLES from two editors\' per-envelope winners, '
        + 'text neither editor proposed by itself. Each editor here repairs one sentence and leaves '
        + 'the other untouched, so finding both fixes in what shipped proves the per-envelope winners '
        + 'were rebuilt into one candidate rather than one editor\'s whole patch winning outright',
      fn: async () => {
        /**
         First sentence, whose only defect is one mistranslated word.
         */
        const sentenceOne = 'The cat is doing the sleeping on the windowsill.';

        /**
         Second sentence, whose only defect is a different mistranslated
         word, so nothing here overlaps what the first sentence tests.
         */
        const sentenceTwo = 'The dog is doing the barking in the yard.';

        /**
         Two-envelope chunk. Every other case in this file uses the shared
         single-envelope fixture, which can never reach the composite: with
         one envelope, per-envelope selection has only one winner to adopt
         and there is nothing for `applyCandidate` to assemble.
         */
        const targetText = `${sentenceOne} ${sentenceTwo}`;

        /**
         Envelope covering the first sentence, region 1 on the editor sheet.
         */
        const envelopeOne: EditableEnvelope = {
          envelopeId: 'envelope/cat',
          startOffset: 0,
          endOffset: sentenceOne.length,
          baseText: sentenceOne,
          baseHash: hashContent({ content: sentenceOne, },),
          issueIds: ['adjudicated/cat-tense',],
        };

        /**
         Envelope covering the second sentence, region 2 on the editor
         sheet.
         */
        const envelopeTwo: EditableEnvelope = {
          envelopeId: 'envelope/dog',
          startOffset: sentenceOne.length + 1,
          endOffset: sentenceOne.length + 1 + sentenceTwo.length,
          baseText: sentenceTwo,
          baseHash: hashContent({ content: sentenceTwo, },),
          issueIds: ['adjudicated/dog-tense',],
        };

        /**
         Both envelopes, in document order.
         */
        const envelopes: readonly EditableEnvelope[] = [
          envelopeOne,
          envelopeTwo,
        ];

        /**
         Accepted issues, one per envelope. Neither quotes anything: both
         fixes are single-word swaps, well inside what the preservation gate
         allows without a licensed quote.
         */
        const issues: readonly AdjudicatedIssue[] = [
          {
            issueId: 'adjudicated/cat-tense',
            status: 'accepted' as const,
            severity: 'major' as const,
            claims: [],
            tallies: {},
          },
          {
            issueId: 'adjudicated/dog-tense',
            status: 'accepted' as const,
            severity: 'major' as const,
            claims: [],
            tallies: {},
          },
        ];

        /**
         First editor's fix, touching only the cat sentence.
         */
        const catEditorReport = {
          edits: [
            {
              region: 1,
              newText: 'The cat is doing the napping on the windowsill.',
            },
          ],
        };

        /**
         Second editor's fix, touching only the dog sentence.
         */
        const dogEditorReport = {
          edits: [
            {
              region: 2,
              newText: 'The dog is doing the howling in the yard.',
            },
          ],
        };

        /**
         Client answering each editor with its own single-sentence fix and,
         once selection reaches the chunk-level ballot, naming the
         composite: the only whole-chunk candidate that repairs both
         sentences. No per-envelope ballot is ever asked for here, since
         each envelope has exactly one editor's operation and
         `selectPerEnvelope` adopts a sole proposal without a vote.
         */
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused',);
          },
          chatJson: async <ValueT,>(
            request: ChatJsonRequest<ValueT>,
          ): Promise<ChatJsonOutcome<ValueT>> => {
            if (request.modelId === EDITORS[0])
              return replyWith({
                report: catEditorReport,
                request,
              },);
            if (request.modelId === EDITORS[1])
              return replyWith({
                report: dogEditorReport,
                request,
              },);
            return replyWith({
              report: {
                best: 3,
                assessment: 'the composite is the only whole-chunk candidate that repairs both sentences',
              },
              request,
            },);
          },
          quotas: async () => {
            throw new Error('quotas unused',);
          },
        };

        const result = await runEditorStage({
          client,
          editorModelIds: EDITORS,
          judgeModelIds: JUDGES,
          sourceText: SOURCE_TEXT,
          targetText,
          envelopes,
          issues,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        // NEITHER EDITOR'S OWN PATCH CARRIES BOTH FIXES: only the composite
        // `applyCandidate` assembles from the two per-envelope winners does,
        // so the shipped text naming both proves that candidate won rather
        // than either editor's whole patch.
        expect(result.patch.patchedText,).toBe(
          'The cat is doing the napping on the windowsill. The dog is doing the howling in the yard.',
        );
        expect(result.patch.applied
          .length,).toBe(2,);
      },
    },),
  ],
},);
