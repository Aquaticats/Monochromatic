/**
 Guards class fifty-three (XingZ602 attempt 1, 2026-09-18): a slate judged
 at a slice with nothing to fall back on, whose judges split three ways at
 weight one, is challenged as a RUN-OFF over the candidates that drew a
 ballot rather than over the whole slate again. Slice 14 of XingZ60 carried
 an archive text the deterministic floor refused ("TA once said"), so the
 stage ran it as absent; five candidates, three ballots, three different
 picks, twice (here: three candidates, two named once each, one decline); the entry stopped ERROR after two hours and thirty-eight
 minutes and re-bought its translate lane. Cat-themed invention throughout;
 no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  judgeSlateWithRetry,
  produceTranslateSlate,
  type TranslateStageResult,
} from '../dist/final/node/index.mjs';
import { candidateCarrying, } from '@monochromatic-dev/translation-repair-test-fixture/ts/translate-ballot.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the judgings under test.
 */
const l = tagged({ tag: 'translate-runoff-test', },);

/**
 Schema name the producing half asks translators for; every other structured
 ask is a judge sheet.
 */
const TRANSLATE_SCHEMA = 'translation_report';

/**
 Original slice the archive never translated.
 */
const SOURCE_TEXT = '猫猫在窗台上打盹，尾巴垂在暖气片旁边。';

/**
 Translators, three, which is the producing window.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
  'hf:cat/Cat-C',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Judges with no stake in any candidate, three so a split is one ballot each.
 */
const JUDGES: readonly RosterModelId[] = [
  'hf:cat/Judge-A',
  'hf:cat/Judge-B',
  'hf:cat/Judge-C',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 What the translators render, one each in call order; the third draws no
 ballot in either round.
 */
const RENDERINGS: readonly string[] = [
  'The cat dozes on the windowsill, tail draped beside the radiator.',
  'A cat naps on the sill, its tail hanging near the heater.',
  'The cat sleeps on the ledge, tail beside the radiator.',
];

/**
 Needle each judge looks for in a split round: two candidates named once
 each and one judge declining, which no candidate can win with two ballots.
 */
const FIRST_ROUND_NEEDLES: Readonly<Record<string, string>> = {
  'hf:cat/Judge-A': 'dozes',
  'hf:cat/Judge-B': 'naps',
  'hf:cat/Judge-C': 'purrs',
};

/**
 How many candidates a judge sheet offers.

 @param content - judge sheet as sent

 @returns Count of candidate blocks

 @example
 ```ts
 const offered = candidatesOffered({ content, },);
 ```
 */
function candidatesOffered({ content, }: { readonly content: string; },): number {
  return content.split('CANDIDATE ',).length - 1;
}

/**
 Client whose translators render in call order and whose judges split in the
 first round and answer per the script in the second.

 @param secondRound - what every judge says in the second judging

 @returns Client plus every judge sheet it was shown, by round

 @example
 ```ts
 const rig = scriptedRig({ secondRound: 'converge', },);
 ```
 */
function scriptedRig(
  { secondRound, }: { readonly secondRound: 'converge' | 'split'; },
): {
  readonly client: ModelClient;
  readonly sheetsByRound: readonly string[][];
} {
  /**
   Translator calls served so far.
   */
  const served = { count: 0, };

  /**
   Judge sheets seen, grouped by judging round.
   */
  const sheetsByRound: string[][] = [
    [],
    [],
  ];

  return {
    sheetsByRound,
    client: {
      chatText: async () => {
        throw new Error('chatText unused by the translate lane',);
      },
      quotas: async () => {
        throw new Error('quotas unused by the translate lane',);
      },
      chatJson: async <ValueT,>(
        request: ChatJsonRequest<ValueT>,
      ): Promise<ChatJsonOutcome<ValueT>> => {
        /**
         Which sheet this is.
         */
        const schema = request.responseFormat
          ?.json_schema
          .name;
        if (schema === TRANSLATE_SCHEMA) {
          /**
           Rendering this call gets.
           */
          const translation = RENDERINGS[served.count % RENDERINGS.length] ?? '';
          served.count += 1;
          /**
           Reply as the wire expects it.
           */
          const value: unknown = { translation, };
          if (!request.validate(value,))
            throw new Error('the fixture translation failed the wire guard',);
          return {
            kind: 'ok',
            value,
            rawText: JSON.stringify(value,),
          };
        }

        /**
         Sheet text, for finding the wanted candidate.
         */
        const content = request.messages
          .map(function toContent(message,) {
            return messageText({ message, },);
          },)
          .join('\n',);

        /**
         Which judging this call belongs to: the first while every judge has
         not yet answered once.
         */
        const round = ((sheetsByRound[0] ?? []).length < JUDGES.length) ? 0 : 1;
        sheetsByRound[round]?.push(content,);

        /**
         Needle this judge votes by in this round.
         */
        const needle = ((round === 0) || (secondRound === 'split'))
          ? (FIRST_ROUND_NEEDLES[request.modelId] ?? '')
          : 'dozes';

        /**
         Ballot for this judging.
         */
        const ballot: unknown = {
          best: candidateCarrying({
            content,
            needle,
          },),
          assessment: 'fixture',
        };
        if (!request.validate(ballot,))
          throw new Error('the fixture ballot failed the wire guard',);
        return {
          kind: 'ok',
          value: ballot as ValueT,
          rawText: JSON.stringify(ballot,),
        };
      },
    },
  };
}

/**
 Judges one freshly produced slate through the retry at an absent slice.

 @param secondRound - what the judges do the second time

 @returns Stage result plus the sheets by round

 @example
 ```ts
 const { result, } = await judgedUnder({ secondRound: 'converge', },);
 ```
 */
async function judgedUnder(
  { secondRound, }: { readonly secondRound: 'converge' | 'split'; },
): Promise<{
  readonly result: TranslateStageResult;
  readonly sheetsByRound: readonly string[][];
}> {
  /**
   Scripted client and its sheets.
   */
  const rig = scriptedRig({ secondRound, },);

  /**
   Slate the translators produced.
   */
  const produced = await produceTranslateSlate({
    client: rig.client,
    translatorModelIds: TRANSLATORS,
    sourceText: SOURCE_TEXT,
    incumbentText: '',
    lineStructured: false,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  /**
   What the retry settled on.
   */
  const result = await judgeSlateWithRetry({
    judging: {
      client: rig.client,
      produced,
      judgeModelIds: JUDGES,
      sourceText: SOURCE_TEXT,
      incumbentText: '',
      incumbentKind: 'absent',
      lineStructured: false,
      signal: AbortSignal.timeout(HANG_STOP_MS,),
      perCallTimeoutMs: HANG_STOP_MS,
      l,
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },
  },);
  return {
    result,
    sheetsByRound: rig.sheetsByRound,
  };
}

/**
 Finding the retry writes when the second round is a run-off.
 */
const RUNOFF_FINDING = 'translate-runoff (finalists 2 of 3)';

await describe({
  name: 'a tied slate at a slice with nothing to fall back on (class fifty-three)',
  children: [
    it({
      name: 'CHALLENGES the tie as a run-off over the two candidates that drew a ballot, '
        + 'leaving the unbacked third off the second sheet, and keeps the second round\'s decision',
      fn: async () => {
        const { result, sheetsByRound, } = await judgedUnder({ secondRound: 'converge', },);

        expect(result.origin,).toBe('fresh',);
        expect(result.text.includes('dozes',),).toBe(true,);
        expect(result.findings.includes(RUNOFF_FINDING,),).toBe(true,);

        /**
         Candidates each round's sheets offered.
         */
        const offered = sheetsByRound.map(function perRound(sheets,): readonly number[] {
          return sheets.map(function count(content,): number {
            return candidatesOffered({ content, },);
          },);
        },);
        expect(offered[0],).toEqual([
          3,
          3,
          3,
        ],);
        expect(offered[1],).toEqual([
          2,
          2,
          2,
        ],);
        expect((sheetsByRound[1] ?? []).some(function carriesUnbacked(content,): boolean {
          return content.includes('sleeps on the ledge',);
        },),).toBe(false,);
      },
    },),

    it({
      name: 'SHIPS a finalist by slate order when the run-off ties across both finalists, carrying the run-off '
        + 'finding (class one hundred seventy-four: a tie between valid finalists is a failure to rank, not a '
        + 'rejection)',
      fn: async () => {
        const { result, } = await judgedUnder({ secondRound: 'split', },);
        expect(result.origin,).toBe('fresh',);
        expect(result.findings.includes(RUNOFF_FINDING,),).toBe(true,);
        expect(result.findings.includes('translate-runoff-tie-broken (slate order)',),).toBe(true,);
      },
    },),
  ],
},);
