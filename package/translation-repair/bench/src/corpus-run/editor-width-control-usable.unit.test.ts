/**
 Tests that the editor width control REFUSES a draw it cannot damage.

 WHAT THE CONTROL IS FOR. It cuts one sentence out of a translation and asks
 the panel which reads better. A panel that cannot tell the cut version from
 the whole one is not measuring anything, so the probe stops before it spends.
 That question can only be asked of a passage holding MORE THAN ONE sentence:
 cut the only sentence a slice has and nothing is left to judge, which is a
 different question from whether the panel notices a deletion.

 WHAT WAS MEASURED. On 2026-08-25, inverting the filter that selects damageable
 slices, so the ones holding a single sentence are the ones kept, failed no
 test in this package. The probe would then have run its control over passages
 it could not damage, and reported whatever the panel said about an empty arm
 as though it were evidence the panel works.

 THE FIXTURE IS ASSERTED FIRST. `withoutASentence` decides what damageable
 means, so each fixture is put to it directly before the control sees it. A
 fixture that quietly stopped being a single sentence would otherwise turn
 this into a test that passes for the wrong reason.

 NO NETWORK. The refusal happens before any judge is seated, and the client
 there refuses every exchange by name so a control that got past the filter
 would say so rather than quietly buying rounds. The one case that seats a
 panel scripts its judges to back a passage by its text, wherever the seating
 puts it (ledger T8).

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type BenchSlice,
  widthControlHolds,
  withoutASentence,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

import { candidateNumber, } from '@monochromatic-dev/translation-repair-provider/ts/archive-selection.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the control under test.
 */
const l = tagged({ tag: 'editor-width-control-usable-test', },);

//region Fixtures

/**
 Slice whose translation is one sentence, so cutting it leaves nothing.

 The terminator sits at the very end, which is what makes it undamageable:
 there is no text after it to keep.
 */
const ONE_SENTENCE: BenchSlice = {
  entryId: 'mittens-window',
  index: 4,
  sourceText: '小猫在窗台上打盹。',
  incumbentText: 'The kitten dozes on the windowsill.',
  lineStructured: false,
};

/**
 Second undamageable slice, so the refusal is not resting on a draw of one.
 */
const ALSO_ONE_SENTENCE: BenchSlice = {
  entryId: 'whiskers-birds',
  index: 7,
  sourceText: '白胡子数着外面的鸟。',
  incumbentText: 'Whiskers counted the birds outside.',
  lineStructured: false,
};

/**
 Slice whose translation holds two sentences, which a cut can be taken from.
 */
const TWO_SENTENCES: BenchSlice = {
  entryId: 'tabby-radiator',
  index: 2,
  sourceText: '虎斑猫睡在暖气旁。它的尾巴垂在地板上。',
  incumbentText: 'The tabby sleeps by the radiator. Its tail hangs to the floor.',
  lineStructured: false,
};

/**
 Refuses any chat exchange, since a control that reached one has already
 failed the question this file asks: every case here must decline before
 asking a judge anything. Assignable to both `chatText` and `chatJson` on
 `ModelClient`, since a function returning `never` never needs to produce
 either reply type.

 @throws Error on any invocation

 @example
 ```ts
 expect(refuseEveryExchange,).toThrow();
 ```
 */
function refuseEveryExchange(): never {
  throw new Error('the width control must refuse an undamageable draw before asking anyone',);
}

/**
 Client refusing every exchange, since a control that reached one has already
 failed the question this file asks.
 */
const CLIENT: ModelClient = {
  chatText: refuseEveryExchange,
  chatJson: refuseEveryExchange,
  quotas: async () => {
    throw new Error('quotas unused by the width control',);
  },
};

/**
 A client whose one judge backs the candidate carrying a given text, wherever
 the seating puts it, and answers nothing else.

 @param wanted - passage the judge backs

 @returns Client
 */
function judgeBacking({ wanted, }: { readonly wanted: string; },): ModelClient {
  return {
    ...CLIENT,
    chatJson: async function chatJson<ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> {
      /**
       The ballot sheet the judge was sent.
       */
      const sheet = request.messages
        .map(function content(message,): string {
          return messageText({ message, },);
        },)
        .join('\n',);
      /**
       Ballot naming the backed passage.
       */
      const ballot: unknown = {
        best: candidateNumber({
          text: sheet,
          wanted,
        },),
        assessment: 'fixture',
      };
      if (!request.validate(ballot,))
        throw new Error('scripted ballot failed the candidate guard',);
      return {
        kind: 'ok',
        value: ballot,
        rawText: JSON.stringify(ballot,),
      };
    },
  };
}

//endregion Fixtures

await describe({
  name: widthControlHolds.name,
  children: [
    it({
      name: 'READS damageable as holding a sentence to spare, which is what the fixtures in this file rest '
        + 'on and what the filter is asking about',
      fn: async () => {
        expect(withoutASentence(ONE_SENTENCE.incumbentText,),).toBe('',);
        expect(withoutASentence(ALSO_ONE_SENTENCE.incumbentText,),).toBe('',);
        expect(withoutASentence(TWO_SENTENCES.incumbentText,),).toBe('Its tail hangs to the floor.',);
      },
    },),

    it({
      name: 'REFUSES a draw whose every slice holds one sentence, since a control run over passages '
        + 'it cannot damage would report the panel as working without having asked it anything',
      fn: async () => {
        /**
         What the control said about a draw with nothing to cut.
         */
        const refusal = await rejectionOf(async function overUndamageableSlices() {
          await widthControlHolds({
            client: CLIENT,
            slices: [
              ONE_SENTENCE,
              ALSO_ONE_SENTENCE,
            ],
            judgeModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
            signal: AbortSignal.timeout(HANG_STOP_MS,),
            l,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        },);

        // A STATED REFUSAL, not a bare Error: the boundary prints a stated one
        // in a line and exits 6, and printed a bare one as a fault with frames.
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect((refusal as Error).message,).toContain('no drawn slice holds more than one sentence',);
      },
    },),

    it({
      name: 'HOLDS FOR A PANEL THAT PREFERS THE INTACT PASSAGE in both seatings, and fails one that prefers the '
        + 'passage missing a sentence (ledger T8)',
      fn: async () => {
        /**
         What the control concludes for a judge backing one passage wherever it sits.

         @param backs - which passage the judge backs

         @returns Whether the control held
         */
        async function holdsFor(backs: 'intact' | 'damaged',): Promise<boolean> {
          return await widthControlHolds({
            client: judgeBacking({
              wanted: (backs === 'intact')
                ? TWO_SENTENCES.incumbentText
                : withoutASentence(TWO_SENTENCES.incumbentText,),
            },),
            slices: [
              ONE_SENTENCE,
              TWO_SENTENCES,
            ],
            // THREE JUDGES, since the selection stage keeps its fallback under a
            // winning count of two ballots and one judge casts one.
            judgeModelIds: [
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              SEAT_HYPER_VISION,
            ],
            signal: AbortSignal.timeout(HANG_STOP_MS,),
            l,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        }
        expect(await holdsFor('intact',),).toBe(true,);
        expect(await holdsFor('damaged',),).toBe(false,);
      },
    },),
  ],
},);
