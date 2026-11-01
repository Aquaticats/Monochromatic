/**
 Tests that the width head-to-head SWAPS THE SEATS, which is its whole reason
 for existing.

 WHAT POSITION BIAS IS. A panel shown two candidates does not weigh them
 evenly; whichever sits first draws votes for sitting first. The comparison
 cancels that by asking the same question twice with the seats exchanged, and
 reading a winner only where both orders agree.

 WHAT WAS MEASURED. On 2026-08-25, seating the NARROW arm in BOTH places of
 the first order failed no test in this package. The panel would then have
 been asked to choose between one arm and itself, its answer would have been
 meaningless, and the head-to-head would have gone on reporting agreement
 between two orders of which one asked nothing.

 READ OFF THE SHEETS THE JUDGES WERE SENT, not off the verdict. A verdict is a
 value the reader cannot trace back to a slate, and the defect here is
 entirely in what the panel was shown.

 NO NETWORK. One judge is seated and answers every ballot with the first
 candidate; the client records the sheet each exchange carried.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { PatchOutcome, } from '@monochromatic-dev/translation-repair-page-pass';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type ArmOutcome,
  bothOrders,
  type WidthProbeInput,
} from '../../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the comparison under test.
 */
const l = tagged({ tag: 'editor-width-contest-seats-test', },);

//region Fixtures

/**
 Marker no prompt constant and no other fixture carries, so a match in a sheet
 can only have come from the NARROW arm's text.
 */
const NARROW_MARK = 'ZQNARROWARM';

/**
 Marker for the WIDE arm, kept separate so a sheet showing one arm twice could
 not read as showing both.
 */
const WIDE_MARK = 'ZQWIDEARM';

/**
 Text the narrow arm shipped.
 */
const NARROW_TEXT = `The kitten dozes on the windowsill. ${NARROW_MARK}`;

/**
 Text the wide arm shipped.
 */
const WIDE_TEXT = `A kitten is dozing on the windowsill. ${WIDE_MARK}`;

/**
 Slice both arms worked over.
 */
const INPUT: WidthProbeInput = {
  entryId: 'mittens-window',
  sliceIndex: 4,
  sourceText: '小猫在窗台上打盹。',
  targetText: 'The kitten sleeps on the sill.',
  issues: [],
  envelopes: [],
  findings: [],
} as unknown as WidthProbeInput;

/**
 Reduces one text to the arm shape the comparison seats.

 @param text - wording that arm shipped

 @returns Arm carrying it, with no producers so no judge is discounted

 @example
 ```ts
 const narrow = armShipping({ text: NARROW_TEXT, },);
 ```
 */
function armShipping({ text, }: { readonly text: string; },): ArmOutcome {
  /**
   Patch whose patched text is the arm's whole wording.
   */
  const patch: PatchOutcome = {
    patchedText: text,
    applied: [],
    rejected: [],
  };

  return {
    text,
    patch,
    heard: 1,
    producers: [],
  };
}

/**
 Runs one head-to-head and hands back the sheet every judge was sent.

 @returns User sheets of the ballot exchanges, in the order they were asked

 @example
 ```ts
 const sheets = await ballotSheets();
 ```
 */
async function ballotSheets(): Promise<readonly string[]> {
  /**
   User sheet of every ballot exchange, in order.
   */
  const asked: string[] = [];

  /**
   Client answering every ballot with the first candidate.
   */
  const client: ModelClient = {
    chatText: async () => {
      throw new Error('chatText unused by the width head-to-head',);
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
      if (stage !== 'candidate_ballot')
        throw new Error(`the head-to-head asks judges and nothing else, and this asked ${stage}`,);

      /**
       User prompt of this exchange.
       */
      const last = request.messages.at(-1,);
      asked.push((last === undefined) ? '' : messageText({ message: last, },),);

      /**
       Ballot naming the first candidate, whichever arm that is.
       */
      const scripted: unknown = {
        best: 1,
        assessment: 'fixture',
      };
      if (!request.validate(scripted,))
        throw new Error('scripted ballot failed the candidate guard',);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the width head-to-head',);
    },
  };

  await bothOrders({
    client,
    input: INPUT,
    narrow: armShipping({ text: NARROW_TEXT, },),
    wide: armShipping({ text: WIDE_TEXT, },),
    judgeModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  return asked;
}

//endregion Fixtures

await describe({
  name: bothOrders.name,
  children: [
    it({
      name: 'SHOWS BOTH ARMS IN BOTH ORDERS, since a panel asked to choose between one arm and itself '
        + 'answers nothing, and the second order is the whole instrument for cancelling position bias',
      fn: async () => {
        /**
         Sheets the judges were sent, one per order.
         */
        const sheets = await ballotSheets();

        expect(sheets.length,).toBe(2,);
        for (const sheet of sheets) {
          expect(sheet,).toContain(NARROW_MARK,);
          expect(sheet,).toContain(WIDE_MARK,);
        }
      },
    },),

    it({
      name: 'SEATS THEM THE OTHER WAY ROUND the second time, which is what makes the pair of answers a '
        + 'measurement rather than the same question asked twice',
      fn: async () => {
        /**
         Sheets the judges were sent, one per order.
         */
        const sheets = await ballotSheets();

        /**
         Where each arm sits in the first order's sheet.
         */
        const firstOrder = {
          narrow: (sheets[0] ?? '').indexOf(NARROW_MARK,),
          wide: (sheets[0] ?? '').indexOf(WIDE_MARK,),
        };

        /**
         Where each arm sits in the second order's sheet.
         */
        const secondOrder = {
          narrow: (sheets[1] ?? '').indexOf(NARROW_MARK,),
          wide: (sheets[1] ?? '').indexOf(WIDE_MARK,),
        };

        expect(firstOrder.narrow < firstOrder.wide,).toBe(true,);
        expect(secondOrder.wide < secondOrder.narrow,).toBe(true,);
      },
    },),
  ],
},);
