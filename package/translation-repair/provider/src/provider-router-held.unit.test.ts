/**
 Tests the router's reading of a choice of provider against the holds of the model's card (ledger B1365): the provider
 chosen is returned unless the policy found none or the card holds the model off it for the seat's role, and then the
 call is refused as the lost seat of a model with no provider, with the line written once per model per cooldown.

 The function is called directly with the choices the policy gives, so each branch is read without a router. The
 callers' end-to-end cases are in `provider-router-first-ask-hold.unit.test.ts`. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import type {
  ProviderName,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  providerChosenOrRefused,
  type ReaskSeat,
} from '../dist/final/node/index.mjs';
import { requestFor, } from './seated-request.test-fixture.ts';

/**
 Seat of an editor of a run.
 */
const EDITOR_SEAT: ReaskSeat = { role: 'editor', bench: [], reask: 'other-model', };

/**
 What the function came to for one choice.
 */
type Outcome = {
  /**
   Provider it returned, when it returned one.
   */
  readonly returned?: ProviderName;

  /**
   Refusal it threw, when it threw one.
   */
  readonly refusal?: unknown;

  /**
   Lines it wrote, behind their levels.
   */
  readonly lines: readonly string[];

  /**
   Models it asked whether the line was due for, in order.
   */
  readonly asked: readonly string[];
};

/**
 Reads one choice of the policy for one request.

 @param modelId - model the request asks

 @param reaskSeat - seat the request names, when it names one

 @param choice - what the policy decided

 @param due - whether the line is due for the model

 @returns Provider returned or refusal thrown, with the lines written and the models asked about

 @example
 ```ts
 const came = outcomeOf({ modelId, reaskSeat: EDITOR_SEAT, choice: { kind: 'hyper', }, due: true, },);
 ```
 */
function outcomeOf(
  {
    modelId,
    reaskSeat,
    choice,
    due,
  }: {
    readonly modelId: RosterModelId;
    readonly reaskSeat?: ReaskSeat;
    readonly choice: Parameters<typeof providerChosenOrRefused>[0]['choice'];
    readonly due: boolean;
  },
): Outcome {
  /**
   Lines written.
   */
  const lines: string[] = [];
  /**
   Models asked about.
   */
  const asked: string[] = [];
  try {
    return {
      returned: providerChosenOrRefused({
        request: requestFor({ modelId, ...((reaskSeat === undefined) ? {} : { reaskSeat, }), },),
        choice,
        l: levelCapturingLogger({ lines, },),
        warnDue: function dueFor({ modelId: asking, },): boolean {
          asked.push(asking,);
          return due;
        },
      },),
      lines,
      asked,
    };
  }
  catch (error) {
    return { refusal: error, lines, asked, };
  }
}

/**
 The messages of the warnings among the lines, without their level.

 @param lines - lines written, behind their levels

 @returns Messages of the warn lines, in order, each from the model's name on

 @example
 ```ts
 const warnings = warningsOf({ lines: ['warn [x] a cat went dark: hungry',], },);
 ```
 */
function warningsOf({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  return lines
    .filter(function isWarning(line,): boolean {
      return line.startsWith('warn ',);
    },)
    .map(function fromTheModel(line,): string {
      return line.slice(line.indexOf(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,),);
    },);
}

await describe({
  name: providerChosenOrRefused.name,
  children: [
    it({
      name: 'RETURNS THE PROVIDER CHOSEN WHERE NO HOLD KEEPS THE MODEL OFF IT and never asks whether a line is due: Qwen3.8-27B '
        + 'on Synthetic, a model with no hold on Hyper, a measurement on Hyper, and a request that names no seat on Hyper',
      fn: async () => {
        const returned = [
          outcomeOf({
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            reaskSeat: EDITOR_SEAT,
            choice: { kind: 'synthetic', },
            due: true,
          },),
          outcomeOf({ modelId: SEAT_HYPER_VISION, reaskSeat: EDITOR_SEAT, choice: { kind: 'hyper', }, due: true, },),
          outcomeOf({
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            reaskSeat: { ...EDITOR_SEAT, reask: 'measured-model', },
            choice: { kind: 'hyper', },
            due: true,
          },),
          outcomeOf({ modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, choice: { kind: 'hyper', }, due: true, },),
        ];
        expect(returned.map(function summarized(came,) {
          return { returned: came.returned, lines: came.lines, asked: came.asked, };
        },),).toEqual([
          { returned: 'synthetic', lines: [], asked: [], },
          { returned: 'hyper', lines: [], asked: [], },
          { returned: 'hyper', lines: [], asked: [], },
          { returned: 'hyper', lines: [], asked: [], },
        ],);
      },
    },),

    it({
      name: 'REFUSES A CHOICE OF NO PROVIDER WITH THE POLICY\'S OWN REASON, raised before any call, and writes the line '
        + 'only where it is due',
      fn: async () => {
        /**
         Reason the policy gave for a model it could send nowhere.
         */
        const reason = 'every provider serving this model is out of budget (synthetic dry, hyper dry)';
        /**
         The choice of no provider, read when the line is due or not.

         @param due - whether the line is due for the model

         @returns What the function came to
         */
        function choiceOfNoneWhen(due: boolean,): Outcome {
          return outcomeOf({
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            reaskSeat: EDITOR_SEAT,
            choice: { kind: 'unreachable', reason, },
            due,
          },);
        }
        const written = choiceOfNoneWhen(true,);
        const silenced = choiceOfNoneWhen(false,);
        for (const came of [written, silenced,]) {
          expect(came.refusal,).toBeInstanceOf(NoProviderForModelError,);
          expect((came.refusal instanceof NoProviderForModelError) ? came.refusal.reason : undefined,).toBe(reason,);
          expect((came.refusal instanceof NoProviderForModelError) ? came.refusal.afterExchange : undefined,).toBe(false,);
          expect(came.asked,).toEqual([SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],);
        }
        expect(warningsOf({ lines: written.lines, },),).toEqual([`${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} went dark: ${reason}`,],);
        expect(silenced.lines,).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES A CHOICE THE CARD HOLDS THE MODEL OFF FOR THE SEAT\'S ROLE, naming the provider, the role and the hold, '
        + 'raised before any call, and writes the line only where it is due (ledger B1365)',
      fn: async () => {
        /**
         The choice of Hyper for the held model's editor seat, read when the line is due or not.

         @param due - whether the line is due for the model

         @returns What the function came to
         */
        function choiceOfHyperWhen(due: boolean,): Outcome {
          return outcomeOf({
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            reaskSeat: EDITOR_SEAT,
            choice: { kind: 'hyper', },
            due,
          },);
        }
        const written = choiceOfHyperWhen(true,);
        const silenced = choiceOfHyperWhen(false,);
        for (const came of [written, silenced,]) {
          expect(came.refusal,).toBeInstanceOf(NoProviderForModelError,);
          expect((came.refusal instanceof NoProviderForModelError) ? came.refusal.afterExchange : undefined,).toBe(false,);
          expect(came.asked,).toEqual([SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],);
        }
        /**
         Reason the refusal gave.
         */
        const reason = (written.refusal instanceof NoProviderForModelError) ? written.refusal.reason : '';
        expect(reason,).toContain('hyper',);
        expect(reason,).toContain('editor',);
        expect(reason,).toContain('hyper-slow',);
        expect(warningsOf({ lines: written.lines, },),).toEqual([`${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} went dark: ${reason}`,],);
        expect(silenced.lines,).toEqual([],);
      },
    },),
  ],
},);
