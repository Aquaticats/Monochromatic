/**
 Tests for the stage that asks a roster whether a translation carries a
 passage, driven by a scripted client so nothing is bought.

 What these pin is the WIRING rather than the arithmetic, which
 `coverage-verdict.unit.test.ts` covers: that the stage takes its threshold
 from the roster it asked rather than from the replies it got, and that a
 roster too quiet to decide produces no decision.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parseDocument,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { firstRoundWindow, } from '@monochromatic-dev/translation-repair-stage';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { runCoverageStage, } from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the stage writes its progress to.
 */
const l = tagged({ tag: 'coverage-stage-test', },);

/**
 Translation the roster is asked about.
 */
const TARGET_TEXT = `The cat sleeps on the windowsill each morning and naps on its cushion at noon.

She watches the birds outside.
`;

/**
 That translation, parsed once.
 */
const TARGET = parseDocument({ text: TARGET_TEXT, },);

/**
 Roster the questions go to.
 */
const ROSTER = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
  'hf:cat/Cat-C',
  'hf:cat/Cat-D',
].map(function toModelId(id,): RosterModelId {
  return id as unknown as RosterModelId;
},);

/**
 What each model is scripted to answer, or nothing to make it go silent.
 */
type CoverageScript = Record<string, {
  readonly coverage: string;
  readonly quote: string;
}>;

/**
 Builds a client answering from a script and never reaching a network.

 @param script - reply per model, absent for a model that stays silent

 @param asks - sink for the messages of every call, in order, for cases
 reading the sheet

 @returns Client the stage can be driven with

 @example
 ```ts
 const client = scriptedClient({ script, },);
 ```
 */
function scriptedClient({ script, asks, }: {
  readonly script: CoverageScript;
  readonly asks?: ChatJsonRequest<unknown>['messages'][];
},): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the coverage stage',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the coverage stage',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      asks?.push(request.messages,);
      /**
       Reply this model was scripted to give, absent when it stays silent.
       */
      const scripted = script[request.modelId];
      if (scripted === undefined) {
        return {
          kind: 'schema-mismatch',
          rawText: '',
          detail: 'scripted silence',
        };
      }

      /**
       Wire value carrying it.
       */
      const value: unknown = {
        coverage: scripted.coverage,
        copiedSpan: scripted.quote,
        assessment: 'fixture',
      };
      if (!request.validate(value,)) {
        return {
          kind: 'schema-mismatch',
          rawText: JSON.stringify(value,),
          detail: 'reply failed the wire guard',
        };
      }
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

await describe({
  name: runCoverageStage.name,
  children: [
    it({
      name: 'takes its threshold from the ROSTER it asked, so two models reporting nothing while two '
        + 'stay silent is not a decision: half a roster is not a majority of it',
      fn: async () => {
        const answer = await runCoverageStage({
          client: scriptedClient({
            script: {
              'hf:cat/Cat-A': {
                coverage: 'none',
                quote: '',
              },
              'hf:cat/Cat-B': {
                coverage: 'none',
                quote: '',
              },
            },
          },),
          modelIds: ROSTER,
          // THE WHOLE BENCH, since this case scripts every seat and reads its
          // majority over the bench it wrote; production asks the window.
          fanOut: 'whole-bench',
          sourcePassage: '小猫中午在垫子上打盹。',
          translation: TARGET,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(answer.verdict
          .asked,).toBe(4,);
        expect(answer.verdict
          .heard,).toBe(2,);
        expect(answer.verdict
          .absent,).toBe(2,);
        expect(answer.verdict
          .kind,).toBe('split',);
      },
    },),
    it({
      name: 'reports CARRIED when a majority of the roster anchors the same English, which is what a '
        + 'passage merged into a neighbouring sentence looks like from here',
      fn: async () => {
        const answer = await runCoverageStage({
          client: scriptedClient({
            script: {
              'hf:cat/Cat-A': {
                coverage: 'full',
                quote: 'naps on its cushion at noon',
              },
              'hf:cat/Cat-B': {
                coverage: 'full',
                quote: 'naps on its cushion at noon',
              },
              'hf:cat/Cat-C': {
                coverage: 'full',
                quote: 'naps on its cushion at noon',
              },
              'hf:cat/Cat-D': {
                coverage: 'none',
                quote: '',
              },
            },
          },),
          modelIds: ROSTER,
          // THE WHOLE BENCH, since this case scripts every seat and reads its
          // majority over the bench it wrote; production asks the window.
          fanOut: 'whole-bench',
          sourcePassage: '小猫中午在垫子上打盹。',
          translation: TARGET,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(answer.verdict
          .kind,).toBe('carried',);
        expect(answer.verdict
          .anchoredFull,).toBe(3,);
      },
    },),
    it({
      name: 'ANCHORS a quote copied across a note between translators, because the sheet it sends shows the '
        + 'translation without the note and the verdict reads the quote in the text the sheet showed',
      fn: async () => {
        /** Translation whose first line ends with a note the archive's translators left each other. */
        const noteText = 'The cat naps on its cushion at noon.{/* 这句不太对劲 */}\nShe watches the birds outside.\n';
        /** Messages of every call, to see what the roster was shown. */
        const asks: ChatJsonRequest<unknown>['messages'][] = [];
        const answer = await runCoverageStage({
          client: scriptedClient({
            script: {
              'hf:cat/Cat-A': {
                coverage: 'full',
                quote: 'naps on its cushion at noon.\nShe watches the birds outside.',
              },
              'hf:cat/Cat-B': {
                coverage: 'full',
                quote: 'naps on its cushion at noon.\nShe watches the birds outside.',
              },
              'hf:cat/Cat-C': {
                coverage: 'full',
                quote: 'naps on its cushion at noon.\nShe watches the birds outside.',
              },
              'hf:cat/Cat-D': {
                coverage: 'none',
                quote: '',
              },
            },
            asks,
          },),
          modelIds: ROSTER,
          fanOut: 'whole-bench',
          sourcePassage: '小猫中午在垫子上打盹。',
          translation: parseDocument({ text: noteText, },),
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(asks.flat().some(function showsNote(message,): boolean {
          return JSON.stringify(message.content,)
            .includes('这句不太对劲',);
        },),).toBe(false,);
        expect(answer.verdict
          .kind,).toBe('carried',);
        expect(answer.verdict
          .anchoredFull,).toBe(3,);
      },
    },),
    it({
      name: 'TAKES its majority over the seats the window asked, not the bench: four seats all anchoring '
        + 'full coverage are asked quorum plus one, and the seat the window spared is neither silent nor lost',
      fn: async () => {
        const answer = await runCoverageStage({
          client: scriptedClient({
            script: {
              'hf:cat/Cat-A': {
                coverage: 'full',
                quote: 'naps on its cushion at noon',
              },
              'hf:cat/Cat-B': {
                coverage: 'full',
                quote: 'naps on its cushion at noon',
              },
              'hf:cat/Cat-C': {
                coverage: 'full',
                quote: 'naps on its cushion at noon',
              },
              'hf:cat/Cat-D': {
                coverage: 'full',
                quote: 'naps on its cushion at noon',
              },
            },
          },),
          modelIds: ROSTER,
          sourcePassage: '小猫中午在垫子上打盹。',
          translation: TARGET,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(answer.verdict
          .asked,).toBe(firstRoundWindow({ benchSize: ROSTER.length, },),);
        expect(answer.verdict
          .heard,).toBe(answer.verdict.asked,);
        expect(answer.verdict
          .kind,).toBe('carried',);
        expect(answer.findings,).toEqual([],);
      },
    },),

    it({
      name: 'ASKS its follow-up sheet carrying the evidence it was handed, and one with none when it '
        + 'was handed none',
      fn: async () => {
        /**
         Script both cases share: two voices anchoring the passage.
         */
        const script = {
          'hf:cat/Cat-A': {
            coverage: 'full',
            quote: 'the windowsill',
          },
          'hf:cat/Cat-B': {
            coverage: 'full',
            quote: 'the windowsill',
          },
        };
        /**
         Messages every roster call was asked with, evidence run first.
         */
        const asked: ChatJsonRequest<unknown>['messages'][] = [];
        await runCoverageStage({
          client: scriptedClient({
            script,
            asks: asked,
          },),
          modelIds: ROSTER,
          fanOut: 'whole-bench',
          sourcePassage: '小猫中午在垫子上打盹。',
          translation: TARGET,
          followupEvidence: {
            verdictKind: 'partly-carried',
            anchoredFull: 1,
            anchoredPartial: 1,
            absent: 0,
            heard: 2,
            asked: 4,
            evidence: ['the windowsill',],
            missingDestinationCount: 0,
            shortfallAdmitted: false,
          },
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(JSON.stringify(asked,).includes('partly-carried',),).toBe(true,);
        /**
         Messages the same ask collects with nothing handed in.
         */
        const plain: ChatJsonRequest<unknown>['messages'][] = [];
        await runCoverageStage({
          client: scriptedClient({
            script,
            asks: plain,
          },),
          modelIds: ROSTER,
          fanOut: 'whole-bench',
          sourcePassage: '小猫中午在垫子上打盹。',
          translation: TARGET,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(JSON.stringify(plain,).includes('partly-carried',),).toBe(false,);
      },
    },),

    it({
      name: 'CARRIES the identity context it was handed onto the sheets it asks, and none where it was '
        + 'handed none',
      fn: async () => {
        /**
         Sheets every roster call was asked with, context run first.
         */
        const asked: ChatJsonRequest<unknown>['messages'][] = [];
        await runCoverageStage({
          client: scriptedClient({
            script: {
              'hf:cat/Cat-A': {
                coverage: 'full',
                quote: 'the windowsill',
              },
              'hf:cat/Cat-B': {
                coverage: 'full',
                quote: 'the windowsill',
              },
            },
            asks: asked,
          },),
          modelIds: ROSTER,
          fanOut: 'whole-bench',
          sourcePassage: '小猫中午在垫子上打盹。',
          translation: TARGET,
          identityContext: 'The translator signs as 喵工作室.',
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(JSON.stringify(asked,).includes('喵工作室',),).toBe(true,);
        /**
         Sheets the same ask collects with no identity context.
         */
        const plain: ChatJsonRequest<unknown>['messages'][] = [];
        await runCoverageStage({
          client: scriptedClient({
            script: {
              'hf:cat/Cat-A': {
                coverage: 'full',
                quote: 'the windowsill',
              },
              'hf:cat/Cat-B': {
                coverage: 'full',
                quote: 'the windowsill',
              },
            },
            asks: plain,
          },),
          modelIds: ROSTER,
          fanOut: 'whole-bench',
          sourcePassage: '小猫中午在垫子上打盹。',
          translation: TARGET,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(JSON.stringify(plain,).includes('喵工作室',),).toBe(false,);
      },
    },),

    it({
      name: 'READS a partial quote anchored in a region the pairing assigned to another original as '
        + 'misattributed, naming the quote',
      fn: async () => {
        /**
         Sentence the pairing assigns to another original.
         */
        const FOREIGN_SENTENCE = 'She watches the birds outside.';
        /**
         Its region in the translation.
         */
        const startOffset = TARGET_TEXT.indexOf(FOREIGN_SENTENCE,);
        const answer = await runCoverageStage({
          client: scriptedClient({
            script: {
              'hf:cat/Cat-A': {
                coverage: 'partial',
                quote: FOREIGN_SENTENCE,
              },
              'hf:cat/Cat-B': {
                coverage: 'partial',
                quote: FOREIGN_SENTENCE,
              },
            },
          },),
          modelIds: ROSTER,
          fanOut: 'whole-bench',
          sourcePassage: '小猫中午在垫子上打盹。',
          translation: TARGET,
          foreignRegions: [{
            startOffset,
            endOffset: startOffset + FOREIGN_SENTENCE.length,
          },],
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(answer.verdict,).toEqual({
          kind: 'split',
          anchoredFull: 0,
          anchoredPartial: 0,
          absent: 0,
          unanchored: 0,
          misattributed: 2,
          heard: 2,
          asked: 4,
          evidence: [],
          unanchoredQuotes: [],
          misattributedQuotes: [FOREIGN_SENTENCE, FOREIGN_SENTENCE,],
        },);
      },
    },),
  ],
},);
