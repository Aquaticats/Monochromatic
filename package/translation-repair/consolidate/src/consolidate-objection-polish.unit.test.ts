/**
 Guards the owner's answer of 2026-09-27 ("Objections to polish", fourteenth
 addendum of `doc/decision/translation-repair-ineligible-standing.md`): over
 a standing the deterministic rule refused, every gate verdict ships the
 slate's choice, so the gate's objections changed nothing on the page. They
 now reach the final polish as corrections to make where the ORIGINAL
 supports them, under their own label, with the base still the fallback;
 over an eligible standing the gate's choice decides as before and the polish
 stays comparative.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type ConsolidationSettlement,
  settleConsolidation,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the stage writes through, whose output is not under test.
 */
const l = tagged({ tag: 'consolidate-objection-polish-test', },);

/**
 Roster of three seated voices.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Invented original: the cat slept by the window and woke at four.
 */
const SOURCE = '猫在窗边睡着了。她四点醒来。';

/**
 Invented archive rendering.
 */
const ARCHIVE = 'The cat fell asleep by the window and woke at four in the afternoon.';

/**
 Consolidation the slate chooses.
 */
const FRESH = 'The cat fell asleep beside the window, purring for you. She woke at four.';

/**
 Label the selector sheet puts before each candidate's number.
 */
const CANDIDATE_LABEL = 'CANDIDATE ';

/**
 What the gate judges hold against the consolidation.
 */
const OBJECTION = 'The consolidated text adds "purring for you", which the original never says.';

/**
 Number the sheet gives the candidate carrying a text.

 @param sheet - selector sheet as the judge reads it

 @param text - candidate text sought

 @returns One-based candidate number, zero when the sheet does not carry it

 @example
 ```ts
 candidateNumberOf({ sheet, text: FRESH, },);
 ```
 */
function candidateNumberOf(
  {
    sheet,
    text,
  }: {
    readonly sheet: string;
    readonly text: string;
  },
): number {
  /**
   Where the text stands on the sheet.
   */
  const at = sheet.indexOf(text,);
  if (at === (-1))
    return 0;
  /**
   Where the nearest candidate label's number opens.
   */
  const numberAt = sheet.lastIndexOf(
    CANDIDATE_LABEL,
    at,
  ) + CANDIDATE_LABEL.length;
  return Number(sheet.slice(
    numberAt,
    sheet.indexOf(
      '\n',
      numberAt,
    ),
  ),);
}

/**
 Client answering each stage by its schema, recording every refiner sheet.

 @param gateChoice - what every gate voice chooses

 @param refinerSheets - where each refiner sheet is recorded

 @param objection - reason every objecting gate voice gives

 @returns Client over the stages a settlement reaches

 @example
 ```ts
 const client = objectingClient({ gateChoice: 'standing', refinerSheets: [], },);
 ```
 */
function objectingClient(
  {
    gateChoice,
    refinerSheets,
    objection = OBJECTION,
  }: {
    readonly gateChoice: 'standing' | 'consolidated';
    readonly refinerSheets: string[];
    readonly objection?: string;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by structured consolidation stages',);
    },
    quotas: async () => {
      throw new Error('quotas unused by consolidation stages',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage the request belongs to, by its reply schema.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      /**
       Sheet as the voice reads it.
       */
      const sheet = request.messages
        .map(function textOf(message,): string {
          return messageText({ message, },);
        },)
        .join('\n',);
      if (schema === 'refine_report')
        refinerSheets.push(sheet,);
      /**
       Reply for the stage.
       */
      const value: unknown = (schema === 'candidate_ballot')
        ? {
          // A phrase of the consolidation, since the slate shows it wrapped.
          best: candidateNumberOf({ sheet, text: 'purring for you', },),
          assessment: 'the fuller rendering',
        }
        : (schema === 'consolidate_gate')
          ? {
            verdict: gateChoice,
            unsupported: (gateChoice === 'standing') ? ['consolidated',] : [],
            dropped: [],
            assessment: (gateChoice === 'standing') ? objection : 'faithful',
          }
          : (schema === 'refine_report')
            ? { rewrites: [], }
            : (schema === 'absolute_naturalness_review')
              ? {
                isAcceptable: true,
                findings: [],
                assessment: 'publication-ready',
              }
              : {};
      if (!request.validate(value,))
        throw new Error(`synthetic ${schema} reply failed validation`,);
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

/**
 Settles the slice over a standing of the given eligibility.

 @param standingEligible - whether the standing passed the deterministic rule

 @param gateChoice - what every gate voice chooses

 @param refinerSheets - where each refiner sheet is recorded

 @param objection - reason every objecting gate voice gives, the fixture's
 objection when left out

 @returns Settlement

 @example
 ```ts
 const settled = await settle({ standingEligible: false, gateChoice: 'standing', refinerSheets: [], },);
 ```
 */
async function settle(
  {
    standingEligible,
    gateChoice,
    refinerSheets,
    objection,
  }: {
    readonly standingEligible: boolean;
    readonly gateChoice: 'standing' | 'consolidated';
    readonly refinerSheets: string[];
    readonly objection?: string;
  },
): Promise<ConsolidationSettlement> {
  return await settleConsolidation({
    client: objectingClient({
      gateChoice,
      refinerSheets,
      ...((objection === undefined) ? {} : { objection, }),
    },),
    roster: ROSTER,
    subject: {
      sourceText: SOURCE,
      incumbentText: ARCHIVE,
    },
    voices: [{
      modelId: ROSTER[0],
      value: { translation: FRESH, },
    },],
    validity: [{
      modelId: ROSTER[0],
      validation: {
        kind: 'valid',
        pageGrammar: 'strict',
      },
    },],
    producedFindings: [],
    standingText: ARCHIVE,
    lineStructured: false,
    sliceIndex: 3,
    polishConfig: {
      refinerModelIds: [ROSTER[0],],
      judgeModelIds: ROSTER,
      gateModelIds: ROSTER,
      declaredNames: [],
      definitions: '',
    },
    standingMayShip: standingEligible,
    standingEligible,
    ...(standingEligible ? {} : { standingRefusal: 'the pronoun is left untranslated', }),
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: 'gate objections over an ineligible standing reach the polish (owner, 2026-09-27)',
  children: [
    it({
      name: 'CARRIES THE GATE\'S OBJECTION onto every refiner sheet, labelled as the gate\'s claims to check '
        + 'against the ORIGINAL, while the slate\'s choice ships',
      fn: async () => {
        /**
         Every refiner sheet the settlement sent.
         */
        const refinerSheets: string[] = [];
        const settled = await settle({
          standingEligible: false,
          gateChoice: 'standing',
          refinerSheets,
        },);
        expect(settled.terminal,).toBe('consolidated',);
        expect(refinerSheets.length,).toBeGreaterThan(0,);
        expect(refinerSheets.every(function carriesObjection(sheet,): boolean {
          return sheet.includes(OBJECTION,) && sheet.includes('OBJECTIONS FROM THE CONSOLIDATION GATE',);
        },),).toBe(true,);
        // Ledger S10: the objection branch's survival sentence names the
        // house form too.
        expect(refinerSheets.every(function namesHouseForm(sheet,): boolean {
          return sheet.includes('in the form the house rules give it',);
        },),).toBe(true,);
        expect(settled.findings.some(function namesObjectionPolish(finding,): boolean {
          return finding.startsWith('polish-objection-correction',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'KEEPS THE COMPARATIVE POLISH over an eligible standing the gate did not object to',
      fn: async () => {
        /**
         Every refiner sheet the settlement sent.
         */
        const refinerSheets: string[] = [];
        await settle({
          standingEligible: true,
          gateChoice: 'consolidated',
          refinerSheets,
        },);
        expect(refinerSheets.length,).toBeGreaterThan(0,);
        expect(refinerSheets.some(function carriesObjections(sheet,): boolean {
          return sheet.includes('OBJECTIONS FROM THE CONSOLIDATION GATE',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'SENDS NO OBJECTION the gate gave only invisible characters for, which trim() keeps and no refiner '
        + 'could read (ledger B40)',
      fn: async () => {
        /**
         Every refiner sheet the settlement sent.
         */
        const refinerSheets: string[] = [];
        await settle({
          standingEligible: false,
          gateChoice: 'standing',
          refinerSheets,
          objection: '\u{200B}\u{2060}',
        },);
        expect(refinerSheets.length,).toBeGreaterThan(0,);
        expect(refinerSheets.some(function carriesObjections(sheet,): boolean {
          return sheet.includes('OBJECTIONS FROM THE CONSOLIDATION GATE',);
        },),).toBe(false,);
      },
    },),
  ],
},);
