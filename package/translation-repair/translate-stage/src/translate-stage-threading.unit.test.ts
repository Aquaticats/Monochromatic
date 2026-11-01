/**
 Guards what the translate stage hands each round and how a round leaves
 when judging fails for a reason that is not an absence.

 THE CITED REFERENCES AND THE ATTESTED LINES travel from `runTranslateStage`
 through `runTranslateRepairs` into both halves of a round: the judges are
 shown what the pages the original cites say, and the translators are shown
 the archive details a cited page states. No stage case passed either, so
 nothing proved they arrive; each is named on the sheet it belongs to.

 AN ABORT while the judges are asked leaves the round as the caller's abort
 reason, by identity. A fault that is not an absence cannot be sent through
 a stage round from outside, since a provider's fault reaches judging as a
 lost voice (`stage-call.ts`); the narrowing that passes one on is tested on
 its own (`translate-absence-narrow.unit.test.ts`).

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { runTranslateStage, } from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the stage under test.
 */
const l = tagged({ tag: 'translate-stage-threading-test', },);

/**
 Original: the cat naps by the window.
 */
const SOURCE = '猫在窗边打盹。';

/**
 What every translator proposes.
 */
const RENDERING = 'The cat naps by the window.';

/**
 What the pages the original cites say, as the judges should see it.
 */
const REFERENCES = 'The shelter page says the cat was adopted in spring.';

/**
 An archive detail a cited page states, as the translators should see it.
 */
const ATTESTED = 'The cat wore a red collar.';

/**
 Translators, disjoint from the judges.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Judges, three so the first candidate reaches its two ballots.
 */
const JUDGES: readonly RosterModelId[] = [
  'hf:cat/Cat-C',
  'hf:cat/Cat-D',
  'hf:cat/Cat-E',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Client whose translators propose the rendering and whose judges back the
 first candidate, recording every sheet by its schema; a ballot call may
 instead run a caller's action and throw what it returns.

 @param sheets - where each sheet is recorded, by schema name

 @param onBallot - what a ballot call does instead of answering, when given

 @returns Client over both halves of a round

 @example
 ```ts
 const client = recordingClient({ sheets: new Map(), },);
 ```
 */
function recordingClient(
  {
    sheets,
    onBallot,
  }: {
    readonly sheets: Map<string, string[]>;
    readonly onBallot?: () => unknown;
  },
): ModelClient {
  return {
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
      const schema = request.responseFormat?.json_schema.name ?? '';
      sheets.set(schema, [
        ...(sheets.get(schema,) ?? []),
        request.messages
          .map(function textOf(message,): string {
            return messageText({ message, },);
          },)
          .join('\n',),
      ],);
      if ((schema === 'candidate_ballot') && (onBallot !== undefined))
        throw onBallot();
      /**
       Reply this sheet gets.
       */
      const value: unknown = (schema === 'translation_report')
        ? { translation: RENDERING, }
        : (schema === 'candidate_ballot')
          ? {
            best: 1,
            assessment: 'the plainest rendering',
          }
          : undefined;
      if ((value === undefined) || (!request.validate(value,)))
        throw new Error(`no fixture reply for ${schema}`,);
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

await describe({
  name: `${runTranslateStage.name} threading`,
  children: [
    it({
      name: 'SHOWS THE JUDGES the cited references and THE TRANSLATORS the attested lines, on every sheet of '
        + 'the round',
      fn: async () => {
        /**
         Every sheet the round sent, by schema.
         */
        const sheets = new Map<string, string[]>();
        const result = await runTranslateStage({
          client: recordingClient({ sheets, },),
          translatorModelIds: TRANSLATORS,
          judgeModelIds: JUDGES,
          sourceText: SOURCE,
          incumbentText: '',
          incumbentKind: 'absent',
          referenceContext: REFERENCES,
          attestedLines: [ATTESTED,],
          lineStructured: false,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(result.text,).toBe(RENDERING,);

        /**
         Sheets each half was shown.
         */
        const judgeSheets = sheets.get('candidate_ballot',) ?? [];
        const translatorSheets = sheets.get('translation_report',) ?? [];
        expect(judgeSheets.length,).toBe(JUDGES.length,);
        expect(translatorSheets.length,).toBe(TRANSLATORS.length,);
        expect(judgeSheets.every(function showsReferences(sheet,): boolean {
          return sheet.includes('CITED REFERENCES, EVIDENCE ONLY.',) && sheet.includes(REFERENCES,);
        },),).toBe(true,);
        expect(translatorSheets.every(function showsAttested(sheet,): boolean {
          return sheet.includes('ATTESTED DETAILS',) && sheet.includes(ATTESTED,);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'RETHROWS THE CALLER\'S ABORT REASON by identity when the signal fires while the judges are asked',
      fn: async () => {
        /**
         The caller's abort.
         */
        const controller = new AbortController();

        /**
         Why the caller stopped.
         */
        const reason = new Error('the cat left the room',);

        /**
         What the stage raised.
         */
        const raised = await (async function attempt(): Promise<unknown> {
          try {
            return await runTranslateStage({
              client: recordingClient({
                sheets: new Map(),
                onBallot: function abortNow(): unknown {
                  controller.abort(reason,);
                  return reason;
                },
              },),
              translatorModelIds: TRANSLATORS,
              judgeModelIds: JUDGES,
              sourceText: SOURCE,
              incumbentText: '',
              incumbentKind: 'absent',
              lineStructured: false,
              signal: controller.signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
          }
          catch (error) {
            return error;
          }
        })();
        expect(raised,).toBe(reason,);
      },
    },),
  ],
},);
