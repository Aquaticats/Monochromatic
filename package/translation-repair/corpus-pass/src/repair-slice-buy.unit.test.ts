/**
 Tests for what buying one repair slice does when the caller stops the run
 while the provider is refusing: the abort wins, the seat's line says what the
 provider refused with, the slice's line says the caller abandoned it, and the
 key the refusal echoed is in neither. And when the stop lands as the checker
 stage re-seats through the pass's own hook, whose seat reading writes and
 marks the refusal it stops the entry with, that refusal is written once, on
 the reading's line (ledger B413, B415).

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type {
  RepairModels,
  RepairSliceSeating,
} from '@monochromatic-dev/translation-repair-assembly';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { buyRepairSlice, } from '@monochromatic-dev/translation-repair-repair';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { shippingLaneClient, } from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/editor-lane-script.test-fixture.ts';
import {
  statusFailureLogText,
  statusFailureOf,
} from '@monochromatic-dev/translation-repair-provider/ts/provider-status-failure.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  BEDROCK_ALONE_STOP_LINE,
  stopLandingSeatReading,
} from './stop-landing-hook.test-fixture.ts';

/**
 Roster with the naturalness lane off, checkers apart from the editors.
 */
const MODELS: RepairModels = {
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
  checkerModelIds: [
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
  ],
};

/**
 Roster with three critics and three panelists, so the claim every critic of
 the shipping script raises is accepted and the editors' edit reaches the
 checker stage.
 */
const SHIPPING_MODELS: RepairModels = {
  ...MODELS,
  criticModelIds: [
    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
  ],
  panelModelIds: [
    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
  ],
};

await describe({
  name: buyRepairSlice.name,
  concurrency: 1,
  children: [
    it({
      name: 'WARNS OF A PROVIDER REFUSAL THE CALLER\'S ABORT ENDED by the seat, the status and the provider\'s words, '
        + 'the key its refusal echoed masked, once, then that the slice was abandoned by the caller\'s abort, and '
        + 'raises the abort\'s own reason',
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
        /**
         Pair the slice was cut from.
         */
        const prepared = prepareDocumentPair({
          sourceText: '猫在窗台上睡觉。',
          targetText: 'The cat sleeps on the windowsill.',
        },);
        const [slice,] = prepared.slices;
        if (slice === undefined)
          throw new Error('the fixture pair carries no slice',);

        const lines: string[] = [];
        const refusal = await rejectionOf(async function buyUnderStop() {
          return buyRepairSlice({
            client: {
              chatText: async () => {
                throw new Error('chatText unused by the repair lane',);
              },
              chatJson: async () => {
                stop.abort();
                throw failure;
              },
              quotas: async () => {
                throw new Error('quotas unused by the repair lane',);
              },
            },
            prepared,
            models: MODELS,
            slice,
            key: 'purr-key',
            neighbouringIncumbentText: '',
            neighbouringSourceText: '',
            documentSourceText: '',
            signal: stop.signal,
            perCallTimeoutMs: HANG_STOP_MS,
            l: tagged({
              tag: 'repair-slice-buy-test',
              l: levelCapturingLogger({ lines, },),
            },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        },);

        expect(refusal,).toBe(stop.signal.reason,);
        expect(lines,).toEqual([
          `warn [repair-slice-buy-test] critic ${SEAT_HYPER_OPENROUTER_VISION_EDITOR}: ${
            statusFailureLogText({ provider: 'synthetic', status: 401, },)
          }, as the caller stopped the round`,
          'warn [repair-slice-buy-test] chunk 0: abandoned by the caller\'s abort (refused by AbortError)',
        ],);
      },
    },),
    it({
      name: 'WRITES THE SEAT READING\'S REFUSAL ONCE when the caller stops the slice as the checker stage re-seats: '
        + 'on the reading\'s own line, and the slice\'s abandoned line without its words, and raises the caller\'s '
        + 'reason',
      fn: async () => {
        /**
         Why the caller stopped the slice.
         */
        const reason = new Error('the owner called the cats in',);
        /**
         The caller's steering, which the re-seat's dryness read stops.
         */
        const steering = new AbortController();
        /**
         Pair whose one sentence every critic of the shipping script claims.
         */
        const prepared = prepareDocumentPair({
          sourceText: '小猫在窗台上打盹。',
          targetText: 'The kitten dozes on the windowsill.',
        },);
        const [slice,] = prepared.slices;
        if (slice === undefined)
          throw new Error('the fixture pair carries no slice',);
        /**
         Every line the buy and the reading wrote, by level.
         */
        const lines: string[] = [];
        /**
         Logger both write to.
         */
        const l = levelCapturingLogger({ lines, },);

        const refusal = await rejectionOf(async function buyAsTheReadingStops() {
          return buyRepairSlice({
            client: shippingLaneClient(),
            prepared,
            models: SHIPPING_MODELS,
            reseat: stopLandingSeatReading<RepairSliceSeating>({
              steering,
              reason,
              unseated: {},
              l,
            },),
            slice,
            key: 'nap-key',
            neighbouringIncumbentText: '',
            neighbouringSourceText: '',
            documentSourceText: '',
            signal: steering.signal,
            perCallTimeoutMs: HANG_STOP_MS,
            l,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        },);

        expect(refusal,).toBe(reason,);
        // The chunk's stage lines are information; what a reader of a stopped
        // run looks for is written at warn and error.
        expect(lines.filter(function warnedOrWorse(line,): boolean {
          return line.startsWith('warn ',) || line.startsWith('error ',);
        },),).toEqual([
          `error ${BEDROCK_ALONE_STOP_LINE}`,
          'warn chunk 0: abandoned by the caller\'s abort',
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
