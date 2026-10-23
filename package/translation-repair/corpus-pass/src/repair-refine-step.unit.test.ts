/**
 Tests for the driver's step around the naturalness lane: the refusals it
 passes on and the one it makes itself.

 The step checks every outcome the lane returns against the archive wording
 of its slice, so an outcome naming a slice preparation never made is refused
 there, even with the lane off, where the phase hands its input back
 untouched. A refusal the phase makes under a live signal is a fault, not an
 abort, and keeps its own identity. A refusal the seat reading wrote and
 marked as the recheck re-seated is written once, on the reading's line, when
 the caller stops the lane (ledger B413, B415). Fixtures are cat-themed
 invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { UnpreparedSliceError, } from '@monochromatic-dev/translation-repair-stage';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  frontMatterRepairOutcome,
  refineSettledSlices,
} from '@monochromatic-dev/translation-repair-repair';
import {
  CheckerIndependenceError,
  type RepairModels,
  type RepairSliceSeating,
} from '@monochromatic-dev/translation-repair-assembly';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { slicePairOf, } from '@monochromatic-dev/translation-repair-slice-reading/ts/corpus-run/slice-pair-of.test-fixture.ts';
import {
  statusFailureLogText,
  statusFailureOf,
} from '@monochromatic-dev/translation-repair-provider/ts/provider-status-failure.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  REPAIRED_TEXT,
  REWRITTEN_TEXT,
  SOURCE_TEXT,
  sunbathingOutcome,
} from '@monochromatic-dev/translation-repair-assembly/ts/sunbathing-recheck.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  BEDROCK_ALONE_STOP_LINE,
  stopLandingSeatReading,
} from './stop-landing-hook.test-fixture.ts';

/**
 Logger for the step under test.
 */
const l = tagged({ tag: 'repair-refine-step-test', },);

/**
 Archive translation of the fixture pair.
 */
const TARGET_TEXT = 'The cat sleeps on the windowsill.';

/**
 Prepared slices of the fixture pair.
 */
const { slices: SLICES, } = prepareDocumentPair({
  sourceText: '猫在窗台上睡觉。',
  targetText: TARGET_TEXT,
},);

/**
 Roster with the naturalness lane off: no rewriter is seated.
 */
const LANE_OFF: RepairModels = {
  criticModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  panelModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  judgeModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  checkerModelIds: [
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  ],
};

/**
 Roster with the naturalness lane on: one rewriter, and checkers apart from it.
 */
const LANE_ON: RepairModels = {
  criticModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  panelModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  editorModelIds: [
    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    SEAT_HYPER_VISION,
  ],
  judgeModelIds: [
    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
  ],
  refinerModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  checkerModelIds: [
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
  ],
};

/**
 Client no case here may call: the lane is off, or refused before it asks.
 */
const SILENT_CLIENT: ModelClient = {
  chatText: async () => {
    throw new Error('no call expected',);
  },
  chatJson: async () => {
    throw new Error('no call expected',);
  },
  quotas: async () => {
    throw new Error('no call expected',);
  },
};

/**
 Answers the naturalness lane's rewriter with the sunbathing rewrite and its
 judges with a ballot for it, so the rewrite ships and the recheck re-seats;
 refuses any other stage, which no case here reaches before the re-seat.

 @param request - exchange as the lane sent it

 @returns The scripted reply, which the stage's own guard has accepted

 @throws Error naming a stage this script does not answer, or a reply its
 guard refused

 @example
 ```ts
 const client = { chatText, chatJson: rewritingChatJson, quotas, };
 ```
 */
async function rewritingChatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
  /**
   Schema the stage asked for.
   */
  const stage = request.responseFormat
    ?.json_schema
    .name
    ?? '';
  /**
   Reply by stage: the one paragraph rewritten, and every judge picking it.
   */
  const replies: Readonly<Record<string, unknown>> = {
    refine_report: {
      rewrites: [
        {
          paragraph: 1,
          newText: REWRITTEN_TEXT,
        },
      ],
    },
    candidate_ballot: {
      best: 1,
      assessment: 'the rewrite purrs',
    },
  };
  if (!Object.hasOwn(replies, stage,))
    throw new Error(`the rewriting script holds no answer for the ${stage} stage`,);
  /**
   Reply this stage gets.
   */
  const reply = replies[stage];
  if (!request.validate(reply,))
    throw new Error(`the rewriting script's ${stage} reply failed the stage's own guard`,);
  return {
    kind: 'ok',
    value: reply,
    rawText: JSON.stringify(reply,),
  };
}

await describe({
  name: refineSettledSlices.name,
  concurrency: 1,
  children: [
    it({
      name: 'REFUSES an outcome naming a slice preparation never made, even with the lane off, where '
        + 'the phase returns its input untouched and nothing else would compare it to an archive',
      fn: async () => {
        await expect(refineSettledSlices({
          client: SILENT_CLIENT,
          targetText: TARGET_TEXT,
          slices: SLICES,
          outcomes: [
            frontMatterRepairOutcome({
              sliceIndex: SLICES.length + 5,
              targetText: TARGET_TEXT,
            },),
          ],
          models: LANE_OFF,
          declaredNames: [],
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },),).rejects.toBeInstanceOf(UnpreparedSliceError,);
      },
    },),
    it({
      name: 'PASSES ON the phase\'s own refusal of a rewriter that also checks, by identity, since '
        + 'under a live signal a refusal is a fault to fix and not an abort to retry',
      fn: async () => {
        await expect(refineSettledSlices({
          client: SILENT_CLIENT,
          targetText: TARGET_TEXT,
          slices: SLICES,
          outcomes: [],
          models: {
            ...LANE_OFF,
            refinerModelIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
          },
          declaredNames: [],
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },),).rejects.toBeInstanceOf(CheckerIndependenceError,);
      },
    },),
    it({
      name: 'WARNS OF A PROVIDER REFUSAL THE CALLER\'S ABORT ENDED by the seat, the status and the provider\'s words, '
        + 'the key its refusal echoed masked, once, then that the refinement was abandoned by the caller\'s abort, '
        + 'and raises the abort\'s own reason',
      fn: async ctx => {
        /**
         Refusal the real client raised over the real transport.
         */
        const failure = await statusFailureOf({
          sinon: ctx.sinon,
          status: 401,
        },);
        /**
         Stop the caller gives while the provider is refusing.
         */
        const stop = new AbortController();
        const lines: string[] = [];
        const refusal = await rejectionOf(async function refineUnderStop() {
          return refineSettledSlices({
            client: {
              chatText: async () => {
                throw new Error('chatText unused by the naturalness lane',);
              },
              chatJson: async () => {
                stop.abort();
                throw failure;
              },
              quotas: async () => {
                throw new Error('quotas unused by the naturalness lane',);
              },
            },
            targetText: REPAIRED_TEXT,
            slices: [
              slicePairOf({
                sliceIndex: 0,
                source: SOURCE_TEXT,
                target: REPAIRED_TEXT,
              },),
            ],
            outcomes: [sunbathingOutcome({ nonTranslationStanding: false, },),],
            models: LANE_ON,
            declaredNames: [],
            signal: stop.signal,
            perCallTimeoutMs: HANG_STOP_MS,
            l: levelCapturingLogger({ lines, },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        },);

        expect(refusal,).toBe(stop.signal.reason,);
        expect(lines,).toEqual([
          `warn refiner ${SEAT_HYPER_OPENROUTER_VISION_EDITOR}: ${
            statusFailureLogText({ provider: 'synthetic', status: 401, },)
          }, as the caller stopped the round`,
          'warn refinement abandoned by the caller\'s abort (refused by AbortError)',
        ],);
      },
    },),
    it({
      name: 'WRITES THE SEAT READING\'S REFUSAL ONCE when the caller stops the lane as the recheck re-seats: on the '
        + 'reading\'s own line, and the refinement\'s abandoned line without its words, and raises the caller\'s reason',
      fn: async () => {
        /**
         Why the caller stopped the lane.
         */
        const reason = new Error('the owner called the cats in',);
        /**
         The caller's steering, which the re-seat's dryness read stops.
         */
        const steering = new AbortController();
        /**
         Every line the step and the reading wrote, by level.
         */
        const lines: string[] = [];
        /**
         Logger both write to.
         */
        const capturing = levelCapturingLogger({ lines, },);

        const refusal = await rejectionOf(async function refineAsTheReadingStops() {
          return refineSettledSlices({
            client: {
              chatText: async () => {
                throw new Error('chatText unused by the naturalness lane',);
              },
              chatJson: rewritingChatJson,
              quotas: async () => {
                throw new Error('quotas unused by the naturalness lane',);
              },
            },
            targetText: REPAIRED_TEXT,
            slices: [
              slicePairOf({
                sliceIndex: 0,
                source: SOURCE_TEXT,
                target: REPAIRED_TEXT,
              },),
            ],
            outcomes: [sunbathingOutcome({ nonTranslationStanding: false, },),],
            models: LANE_ON,
            reseat: stopLandingSeatReading<RepairSliceSeating>({
              steering,
              reason,
              unseated: {},
              l: capturing,
            },),
            declaredNames: [],
            signal: steering.signal,
            perCallTimeoutMs: HANG_STOP_MS,
            l: capturing,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        },);

        expect(refusal,).toBe(reason,);
        // The lane's round and stage lines are information; what a reader of
        // a stopped run looks for is written at warn and error.
        expect(lines.filter(function warnedOrWorse(line,): boolean {
          return line.startsWith('warn ',) || line.startsWith('error ',);
        },),).toEqual([
          `error ${BEDROCK_ALONE_STOP_LINE}`,
          'warn refinement abandoned by the caller\'s abort',
        ],);
        // The refusal's own words, at any level, stand on the reading's line
        // alone.
        expect(lines.filter(function namesTheRefusal(line,): boolean {
          return line.includes('editors 0 of 4 reachable',);
        },),).toEqual([`error ${BEDROCK_ALONE_STOP_LINE}`,],);
      },
    },),
  ],
},);
