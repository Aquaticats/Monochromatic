/**
 Tests that the repair slice driver FORWARDS the neighbouring window to
 the sheets its stages send.

 WHY A SEPARATE FILE FROM `nearby-window-reaches-the-models.unit.test.ts`.
 That one hands each builder a window directly and asserts the builder renders
 it, which is the right test of a builder and says nothing about whether
 anything ever hands one over. `repairChunk` builds the fragment those four
 call sites spread, and its own forwarding is what this pins.

 WHAT WAS MEASURED. On 2026-08-25, inverting this driver's conditional spread
 so the neighbouring ORIGINAL is forwarded only when it is ABSENT failed no
 test in this package. Every builder case stayed green, because each is handed
 its window by hand. A driver that dropped it would leave the critic reasoning
 about a slice alone while the key that names its work claims a window: the
 exact shape of the defect where the slice key mislabelled its window sides.

 THE CRITIC IS ENOUGH TO PIN IT. All four stages spread ONE fragment, built
 once, which the module says is deliberate: a critic that can see next door
 raises a relocation claim and a panel that cannot rejects it as unfounded. A
 run whose critics report nothing settles right after that phase, so scripting
 one stage exercises the forwarding without buying the other three.

 BOTH DIRECTIONS ARE PINNED, since a driver pasting the blocks unconditionally
 would satisfy the first case alone.

 NO NETWORK. The client scripts every critic with an empty report and records
 the sheet each was sent.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RepairModels, } from '@monochromatic-dev/translation-repair-assembly';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { repairChunk, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the driver under test.
 */
const l = tagged({ tag: 'repair-chunk-window-threading-test', },);

//region Fixtures

/**
 Invented original of the slice under review.
 */
const SOURCE_TEXT = '小猫在窗台上打盹。\n';

/**
 Invented English already in the archive for it.
 */
const TARGET_TEXT = 'The kitten dozes on the windowsill.\n';

/**
 Marker no prompt constant and no other fixture carries, so a match in a sheet
 can only have come from the neighbouring ORIGINAL.
 */
const SOURCE_MARK = 'ZQNEARSRC';

/**
 Marker for the neighbouring ARCHIVE ENGLISH, kept separate so a sheet that
 carried one side only could not read as carrying both.
 */
const INCUMBENT_MARK = 'ZQNEAREN';

/**
 Invented original of the passages either side.
 */
const NEARBY_SOURCE = `白胡子数着外面的鸟 ${SOURCE_MARK}。`;

/**
 Invented archive English of those same passages.
 */
const NEARBY_INCUMBENT = `Whiskers counted the birds outside ${INCUMBENT_MARK}.`;

/**
 Roster with editors disjoint from checkers, which the driver asserts before
 it buys anything.
 */
const MODELS: RepairModels = {
  criticModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  panelModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
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
 Runs one slice and returns the sheet every critic was sent.

 @param beside - neighbouring texts to thread, empty for the control

 @returns User sheets of the critic exchanges, in order

 @example
 ```ts
 const sheets = await criticSheets({ beside: { neighbouringSourceText: NEARBY_SOURCE, }, },);
 ```
 */
async function criticSheets(
  {
    beside,
  }: {
    readonly beside: {
      readonly neighbouringSourceText?: string;
      readonly neighbouringIncumbentText?: string;
    };
  },
): Promise<readonly string[]> {
  /**
   User sheet of every critic exchange, in order.
   */
  const asked: string[] = [];

  /**
   Client answering every critic with a report naming nothing.
   */
  const client: ModelClient = {
    chatText: async () => {
      throw new Error('chatText unused by the slice driver',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage name from the structured-output constraint.
       */
      const stage = request.responseFormat
        ?.json_schema
        .name
        ?? '';
      if (stage !== 'critic_report')
        throw new Error(`this fixture settles after the critics and was asked ${stage}`,);

      /**
       User prompt of this exchange.
       */
      const last = request.messages.at(-1,);
      asked.push((last === undefined) ? '' : messageText({ message: last, },),);

      /**
       Report naming nothing, which settles the slice unchanged.
       */
      const scripted: unknown = { issues: [], };
      if (!request.validate(scripted,))
        throw new Error('scripted report failed the critic guard',);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the slice driver',);
    },
  };

  await repairChunk({
    client,
    sliceIndex: 0,
    sourceText: SOURCE_TEXT,
    targetText: TARGET_TEXT,
    lineStructured: false,
    models: MODELS,
    declaredNames: [],
    ...beside,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  return asked;
}

//endregion Fixtures

await describe({
  name: repairChunk.name,
  children: [
    it({
      name: 'FORWARDS both sides of the neighbouring window to the critics, so the stage that raises '
        + 'a relocation claim is the one that can see the passage next door',
      fn: async () => {
        /**
         Sheets the critics were sent with a window either side.
         */
        const sheets = await criticSheets({
          beside: {
            neighbouringSourceText: NEARBY_SOURCE,
            neighbouringIncumbentText: NEARBY_INCUMBENT,
          },
        },);

        expect(sheets.length,).toBeGreaterThan(0,);
        for (const sheet of sheets) {
          expect(sheet.includes(SOURCE_MARK,),).toBe(true,);
          expect(sheet.includes(INCUMBENT_MARK,),).toBe(true,);
        }
      },
    },),
    it({
      name: 'SENDS NO NEARBY BLOCKS for a slice standing alone, which is the control that makes the '
        + '"FORWARDS both sides of the neighbouring window" case legible: a driver pasting them '
        + 'unconditionally would satisfy it',
      fn: async () => {
        /**
         Sheets the critics were sent with no neighbours at all.
         */
        const sheets = await criticSheets({ beside: {}, },);

        expect(sheets.length,).toBeGreaterThan(0,);
        for (const sheet of sheets) {
          expect(sheet.includes('NEARBY',),).toBe(false,);
          expect(sheet.includes(SOURCE_MARK,),).toBe(false,);
        }
      },
    },),
  ],
},);
