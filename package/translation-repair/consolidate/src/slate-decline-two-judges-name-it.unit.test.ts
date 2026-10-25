/**
 Guards the owner's answer of 2026-10-25, "Two judges must name it", which narrows the answer of 2026-09-27
 ("Preference + polish", fifteenth addendum of `doc/decision/translation-repair-ineligible-standing.md`): over wording
 that cannot ship, a slate the judges decline in both rounds used to ship one valid candidate by a fixed order (the
 repair lane's text, the translate lane's, then slate order), whatever any judge named. One judge alone never decides,
 so the slate now ships a candidate only when two or more judges named it; otherwise the archive wording stays and the
 page ships with it reported (owner, 2026-09-27, "Keep archive, ship"), the judges are told so, and no polish runs over
 wording nobody chose. On the translate lane the same holds where the archive's own wording fails the deterministic
 floor (sixteenth addendum). A passage with no wording at all still raises.

 Cat-themed invention throughout; no corpus content appears here.

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
  laneCandidate,
  type LaneText,
  TranslateAbsenceError,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  judgeSlateWithRetry,
  produceTranslateSlate,
  runTranslateStage,
  type TranslateStageResult,
} from '@monochromatic-dev/translation-repair-translate-stage';
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
import { unmetClausesOf, } from '@monochromatic-dev/translation-repair-adjudication/ts/decline-consequence-clauses.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the stages write through, whose output is not under test.
 */
const l = tagged({ tag: 'slate-decline-ships-by-preference-test', },);

/**
 Invented original: the cat dozed on the sill, tail by the radiator.
 */
const SOURCE = '猫猫在窗台上打盹，尾巴垂在暖气片旁边。';

/**
 What every judge says against every candidate.
 */
const REJECTION = 'Every candidate adds that the cat purred, which the original never states.';

/**
 Finding a slate shipped past two declines by preference used to record, before its basis, and which no case here may
 find any more.
 */
const SHIPPED_PAST_DECLINE = 'translate-slate-declined-shipped-by-preference';

/**
 Finding a slate declined in both rounds records where no candidate drew two judges and the archive wording stays.
 */
const ARCHIVE_KEPT_FINDING = 'translate-slate-declined-archive-kept';

/**
 Translators, whose renderings come back in call order.
 */
const TRANSLATORS: readonly RosterModelId[] = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Judges who rendered nothing, so every ballot carries a whole weight.
 */
const JUDGES: readonly RosterModelId[] = [
  'hf:cat/Cat-C',
  'hf:cat/Cat-F',
  'hf:cat/Cat-G',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 What the translators render.
 */
const RENDERINGS: readonly string[] = [
  'The cat dozes on the windowsill, tail draped beside the radiator.',
  'A cat naps on the sill, its tail hanging near the heater.',
];

/**
 Repair lane text on the slate.
 */
const REPAIR_LANE: LaneText = {
  lane: 'repair',
  text: 'The cat curls up on the ledge, tail tucked by the warmth.',
};

/**
 Translate lane text on the slate.
 */
const TRANSLATE_LANE: LaneText = {
  lane: 'translate',
  text: 'The cat yawns on the sill, tail resting against the pipe.',
};

/**
 Consolidation the voices propose.
 */
const FRESH = 'The cat dozed on the windowsill, its tail hanging by the radiator.';

/**
 Client under which every judge rejects every candidate, recording what the
 judges and the refiners were shown.

 @param judgeSheets - where each slate judge sheet is recorded

 @param refinerSheets - where each refiner sheet is recorded

 @param rejection - reason every rejecting judge gives

 @returns Client over every stage these guards reach

 @example
 ```ts
 const client = rejectingClient({ judgeSheets: [], refinerSheets: [], },);
 ```
 */
function rejectingClient(
  {
    judgeSheets,
    refinerSheets,
    rejection = REJECTION,
  }: {
    readonly judgeSheets: string[];
    readonly refinerSheets: string[];
    readonly rejection?: string;
  },
): ModelClient {
  /**
   Translator calls served so far.
   */
  const served = { count: 0, };
  return {
    chatText: async () => {
      throw new Error('chatText unused by these stages',);
    },
    quotas: async () => {
      throw new Error('quotas unused by these stages',);
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
      if (schema === 'candidate_ballot')
        judgeSheets.push(sheet,);
      if (schema === 'refine_report')
        refinerSheets.push(sheet,);
      /**
       Rendering a translator call gets.
       */
      const rendering = RENDERINGS[served.count % RENDERINGS.length] ?? '';
      if (schema === 'translation_report')
        served.count += 1;
      /**
       Reply for the stage.
       */
      const value: unknown = (schema === 'translation_report')
        ? { translation: rendering, }
        : (schema === 'candidate_ballot')
          ? {
            best: 0,
            assessment: rejection,
          }
          : (schema === 'consolidate_gate')
            ? {
              verdict: 'consolidated',
              unsupported: [],
              dropped: [],
              assessment: 'faithful',
            }
            : (schema === 'refine_report')
              ? { rewrites: [], }
              : (schema === 'absolute_naturalness_review')
                ? {
                  isAcceptable: true,
                  findings: [],
                  assessment: 'publication-ready',
                }
                : undefined;
      if (value === undefined)
        throw new Error(`no fixture reply for ${schema}`,);
      if (!request.validate(value,))
        throw new Error(`fixture ${schema} reply failed validation`,);
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

/**
 Judges a produced slate with both lane texts on it, every judge rejecting.

 @param withheldStanding - whether the passage has wording that cannot ship,
 or none is passed at all

 @param judgeSheets - where each judge sheet is recorded

 @param judgeModelIds - judges the slate seats, the roster's by default

 @returns What the retry settled on

 @example
 ```ts
 const result = await judgedRejecting({ withheldStanding: true, judgeSheets: [], },);
 ```
 */
async function judgedRejecting(
  {
    withheldStanding,
    judgeSheets,
    judgeModelIds = JUDGES,
  }: {
    readonly withheldStanding?: boolean;
    readonly judgeSheets: string[];
    readonly judgeModelIds?: readonly RosterModelId[];
  },
): Promise<TranslateStageResult> {
  /**
   Client every judge rejects under.
   */
  const client: ModelClient = rejectingClient({
    judgeSheets,
    refinerSheets: [],
  },);
  /**
   Slate the translators produced.
   */
  const produced = await produceTranslateSlate({
    client,
    translatorModelIds: TRANSLATORS,
    sourceText: SOURCE,
    incumbentText: '',
    lineStructured: false,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  /**
   Slate with both lane texts after the proposals, as the consolidation
   offers them.
   */
  const withLanes = {
    ...produced,
    candidates: [
      ...produced.candidates,
      ...[
        REPAIR_LANE,
        TRANSLATE_LANE,
      ].map(function asLaneCandidate(laneText,) {
        return laneCandidate(laneText,);
      },),
    ],
  };
  return await judgeSlateWithRetry({
    judging: {
      client,
      produced: withLanes,
      judgeModelIds,
      sourceText: SOURCE,
      incumbentText: '',
      incumbentKind: 'absent',
      lineStructured: false,
      ...((withheldStanding === undefined) ? {} : { withheldStanding, }),
      signal: AbortSignal.timeout(HANG_STOP_MS,),
      perCallTimeoutMs: HANG_STOP_MS,
      l,
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },
  },);
}

/**
 Roster of three seated voices for the consolidation.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Settles a slice over an ineligible standing with every slate judge
 rejecting.

 @param refinerSheets - where each refiner sheet is recorded

 @returns Settlement

 @example
 ```ts
 const settled = await settleRejected({ refinerSheets: [], },);
 ```
 */
async function settleRejected(
  { refinerSheets, }: { readonly refinerSheets: string[]; },
): Promise<ConsolidationSettlement> {
  return await settleConsolidation({
    client: rejectingClient({
      judgeSheets: [],
      refinerSheets,
    },),
    roster: ROSTER,
    subject: {
      sourceText: SOURCE,
      incumbentText: 'The cat dozed on the windowsill and purred for you.',
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
    standingText: 'The cat dozed on the windowsill and purred for you.',
    lineStructured: false,
    sliceIndex: 4,
    polishConfig: {
      refinerModelIds: [ROSTER[0],],
      judgeModelIds: ROSTER,
      gateModelIds: ROSTER,
      declaredNames: [],
      definitions: '',
    },
    standingMayShip: false,
    standingEligible: false,
    standingRefusal: 'the pronoun is left untranslated',
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: 'a slate declined twice over wording that cannot ship ships only what two judges named (owner, 2026-10-25)',
  children: [
    it({
      name: 'KEEPS THE ARCHIVE past two rejections, ships no fixed-order candidate and no objections, and tells the '
        + 'judges what a decline does',
      fn: async () => {
        /**
         Every judge sheet the judging sent.
         */
        const judgeSheets: string[] = [];
        const result = await judgedRejecting({
          withheldStanding: true,
          judgeSheets,
        },);
        expect({
          origin: result.origin,
          shippedPastDecline: result.shippedPastDecline,
          kept: result.findings.includes(ARCHIVE_KEPT_FINDING,),
          shipped: result.findings.some(function byPreference(finding,): boolean {
            return finding.startsWith(SHIPPED_PAST_DECLINE,);
          },),
        },).toEqual({
          origin: 'incumbent',
          shippedPastDecline: undefined,
          kept: true,
          shipped: false,
        },);
        expect(result.text === REPAIR_LANE.text,).toBe(false,);
        expect(judgeSheets.length,).toBeGreaterThan(0,);
        expect(judgeSheets.every(function toldNoUntranslatedPassage(sheet,): boolean {
          return !sheet.includes('there is no existing translation of this passage',);
        },),).toBe(true,);
        // THE FIRST ASK OF THE PAIR SETTLES NOTHING and the panel is asked again; the challenge that follows is
        // the round that ships what two judges named or keeps the archive (re-check 4, F1).
        expect(unmetClausesOf({ text: judgeSheets[0] ?? '', mode: 'asked-again', },),).toEqual([],);
        expect(unmetClausesOf({ text: judgeSheets.at(-1,) ?? '', mode: 'ships', },),).toEqual([],);
      },
    },),
    it({
      name: 'KEEPS THE ARCHIVE WITH NO WEIGHT where no judge sat: the round declined before counting any candidate',
      fn: async () => {
        /**
         Every judge sheet the judging sent, which stays empty.
         */
        const judgeSheets: string[] = [];
        const result = await judgedRejecting({
          withheldStanding: true,
          judgeSheets,
          judgeModelIds: [],
        },);
        expect({
          origin: result.origin,
          voteWeight: result.voteWeight,
          shippedPastDecline: result.shippedPastDecline,
          kept: result.findings.includes(ARCHIVE_KEPT_FINDING,),
          judgeSheets,
        },).toEqual({
          origin: 'incumbent',
          voteWeight: 0,
          shippedPastDecline: undefined,
          kept: true,
          judgeSheets: [],
        },);
      },
    },),
    it({
      name: 'STILL RAISES where the passage has no wording at all',
      fn: async () => {
        /**
         What the judging raised, if anything.
         */
        const raised = await (async function attempt(): Promise<unknown> {
          try {
            await judgedRejecting({ judgeSheets: [], },);
            return 'settled';
          }
          catch (error) {
            return error;
          }
        })();
        expect(raised instanceof TranslateAbsenceError,).toBe(true,);
      },
    },),
    it({
      name: 'KEEPS THE ARCHIVE WORDING on the translate lane past two rejections where it fails the deterministic '
        + 'floor, and ships no rendering a fixed order chose',
      fn: async () => {
        const result = await runTranslateStage({
          client: rejectingClient({
            judgeSheets: [],
            refinerSheets: [],
          },),
          translatorModelIds: TRANSLATORS,
          judgeModelIds: JUDGES,
          sourceText: SOURCE,
          // The original left in Han, which the untranslated floor refuses.
          incumbentText: SOURCE,
          incumbentKind: 'present',
          lineStructured: false,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect({
          text: result.text,
          origin: result.origin,
          kept: result.findings.includes(ARCHIVE_KEPT_FINDING,),
        },).toEqual({
          text: SOURCE,
          origin: 'incumbent',
          kept: true,
        },);
      },
    },),
    it({
      name: 'TELLS EACH ROUND OF THE TRANSLATE LANE ITS OWN SENTENCE over wording the floor refuses: the first ask '
        + 'of each call settles nothing, the challenge of the first call ships nothing but a level tie and sends the '
        + 'passage back for new proposals, and only the challenge of the follow-up call ships or keeps (re-check 4, F1)',
      fn: async () => {
        /**
         Every judge sheet the lane sent, in the order asked.
         */
        const judgeSheets: string[] = [];
        await runTranslateStage({
          client: rejectingClient({
            judgeSheets,
            refinerSheets: [],
          },),
          translatorModelIds: TRANSLATORS,
          judgeModelIds: JUDGES,
          sourceText: SOURCE,
          // The original left in Han, which the untranslated floor refuses.
          incumbentText: SOURCE,
          incumbentKind: 'present',
          lineStructured: false,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        /**
         The round each sheet is told in, one entry per run of sheets that name the same one.
         */
        const rounds = judgeSheets
          .map(function roundOf(sheet,): string {
            return (['asked-again', 'ships', 'defers',] as const)
              .filter(function isTold(mode,): boolean {
                return unmetClausesOf({ text: sheet, mode, },).length === 0;
              },)
              .join('+',) || 'none';
          },)
          .filter(function startsRun(
            round,
            at,
            all,
          ): boolean {
            return round !== all[at - 1];
          },);
        expect(rounds,).toEqual(['asked-again', 'defers', 'asked-again', 'ships',],);
      },
    },),
    it({
      name: 'KEEPS THE ARCHIVE over an ineligible standing when every slate judge rejects, runs no polish over '
        + 'wording nobody chose, and reports that no wording passed',
      fn: async () => {
        /**
         Every refiner sheet the settlement sent.
         */
        const refinerSheets: string[] = [];
        const settled = await settleRejected({ refinerSheets, },);
        expect({
          terminal: settled.terminal,
          archiveKept: settled.archiveKept,
          noValidWording: settled.findings.some(function namesIt(finding,): boolean {
            return finding.startsWith('no-valid-wording',);
          },),
          kept: settled.findings.includes(ARCHIVE_KEPT_FINDING,),
          refinerSheets: refinerSheets.length,
        },).toEqual({
          terminal: 'slate-declined-standing',
          archiveKept: true,
          noValidWording: true,
          kept: true,
          refinerSheets: 0,
        },);
      },
    },),
  ],
},);
