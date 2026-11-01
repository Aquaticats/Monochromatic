/**
 Tests that the router refuses the first ask of a model its card holds off the provider the policy chose, for a seat whose
 role honours the hold (ledger B1365): Qwen3.8-27B seated through Synthetic, asked when Synthetic reads dry and Hyper
 would take the call, is refused the way a seat with no provider is refused, and so is lost as one unreachable seat and
 never stops a slice, an entry or the run.

 The callers are stubs and no HTTP is made. The reading of the meters has Hyper wet and Synthetic dry in the cases that
 refuse, and Synthetic wet in the control, so the only thing that can keep the ask off Hyper is the hold. Fixtures are
 cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  MODEL_CARDS,
  type ProviderName,
  type RosterModelId,
  routeTimeHoldsOf,
  SEAT_ROLES,
  type SeatRole,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  HYPER_WET_ONLY,
  readingWith,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/dryness-readings.test-fixture.ts';
import {
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  type BudgetView,
  EveryProviderDryError,
  type ReaskSeat,
} from '../dist/final/node/index.mjs';
import { routerRecordingProviders, } from './provider-router-recording.test-fixture.ts';
import { readOlderProvidersWet, } from './provider-router-wet-budgets.test-fixture.ts';

/**
 Abort signal every routed call carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Conversation reused across the asks.
 */
const MESSAGES = [{ role: 'user' as const, content: '这只猫睡在哪里？', },];

/**
 The hold of Qwen3.8-27B's card that keeps it off Hyper for every role.
 */
const HELD_OFF_HYPER = 'hyper-slow';

/**
 The hold of Kimi-K3's card that keeps it off Hyper in the select and slate seats alone.
 */
const HELD_OFF_HYPER_IN_SELECT_SEATS = 'hyper-slow-select';

/**
 A role the hold of the select seats reaches.
 */
const SELECT_ROLE: SeatRole = 'select-judge';

/**
 A role the hold of the select seats does not reach.
 */
const EDITOR_ROLE: SeatRole = 'editor';

/**
 Seat of a run in a role, asking as a run does.

 @param role - role of the seat

 @returns Seat naming the role and nothing else

 @example
 ```ts
 const seat = seatOf({ role: 'editor', },);
 ```
 */
function seatOf({ role, }: { readonly role: SeatRole; },): ReaskSeat {
  return { role, bench: [], reask: 'other-model', };
}

/**
 Reads the meters as a case wants them read, whatever the call.

 @param reading - which providers are dry

 @returns Reader the router chooses by

 @example
 ```ts
 const read = readingAs(HYPER_WET_ONLY,);
 ```
 */
function readingAs(reading: BudgetView,): () => Promise<BudgetView> {
  return async function read(): Promise<BudgetView> {
    return reading;
  };
}

/**
 What a first ask came to.
 */
type FirstAsk = {
  /**
   Provider that served it, when one did.
   */
  readonly servedBy?: ProviderName;

  /**
   What the router refused it with, when it did.
   */
  readonly refusal?: unknown;

  /**
   Providers the calls reached.
   */
  readonly reached: readonly ProviderName[];

  /**
   Lines the router wrote, behind their levels.
   */
  readonly lines: readonly string[];
};

/**
 Asks a model once as a first ask, with no provider ruled out, and says where it went.

 @param read - reading of the meters the router chooses by

 @param modelId - model to ask

 @param reaskSeat - seat the request names, when it names one

 @returns Provider that served the ask, or the refusal the router gave it

 @example
 ```ts
 const came = await firstAsk({ read: readingAs(HYPER_WET_ONLY,), modelId, reaskSeat: seatOf({ role: 'editor', },), },);
 ```
 */
async function firstAsk(
  {
    read,
    modelId,
    reaskSeat,
  }: {
    readonly read: () => Promise<BudgetView>;
    readonly modelId: RosterModelId;
    readonly reaskSeat?: ReaskSeat;
  },
): Promise<FirstAsk> {
  const { client, reached, lines, } = routerRecordingProviders({ read, },);
  try {
    const reply = await client.chatText({
      modelId,
      messages: MESSAGES,
      signal: SIGNAL,
      ...((reaskSeat === undefined) ? {} : { reaskSeat, }),
    },);
    return { ...((reply.servedBy === undefined) ? {} : { servedBy: reply.servedBy, }), reached, lines, };
  }
  catch (error) {
    return { refusal: error, reached, lines, };
  }
}

await describe({
  name: 'the first ask of a model held off the provider the policy chose',
  children: [
    it({
      name: 'REFUSES THE FIRST ASK OF A HELD MODEL WHERE SYNTHETIC READS DRY AND HYPER WOULD TAKE IT, for every role '
        + 'that honours the hold (ledger B1365): no call is made, the refusal is the lost seat of a model with no provider '
        + 'and not the end of the run, and the log names the hold',
      fn: async () => {
        expect(MODEL_CARDS[SEAT_SYNTHETIC_VISION_NO_OPENROUTER].holds,).toContain(HELD_OFF_HYPER,);
        /**
         Roles whose holds include the one the card carries.
         */
        const honouring = SEAT_ROLES.filter(function honours(role,): boolean {
          return routeTimeHoldsOf({ role, },).includes(HELD_OFF_HYPER,);
        },);
        // EVERY ROLE HONOURS THIS HOLD, so the loop below tries each role and the case cannot pass on an empty list.
        expect(honouring,).toEqual(SEAT_ROLES,);
        /**
         What the first ask of each role's seat came to.
         */
        const asks = await Promise.all(honouring.map(function askAs(role,): Promise<FirstAsk> {
          return firstAsk({
            read: readingAs(HYPER_WET_ONLY,),
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            reaskSeat: seatOf({ role, },),
          },);
        },),);
        /**
         What each role's first ask came to, in the words the case asserts, so a failure names the role.
         */
        const summaries = honouring.map(function summarized(role, position,) {
          const came = asks[position];
          return {
            role,
            noProvider: came?.refusal instanceof NoProviderForModelError,
            endsTheRun: came?.refusal instanceof EveryProviderDryError,
            reached: came?.reached,
            warnsOfTheHold: came?.lines.some(function namesTheHold(line,): boolean {
              return line.startsWith('warn ',) && line.includes(HELD_OFF_HYPER,) && line.includes('hyper',);
            },),
          };
        },);
        expect(summaries,).toEqual(honouring.map(function refused(role,) {
          return { role, noProvider: true, endsTheRun: false, reached: [], warnsOfTheHold: true, };
        },),);
      },
    },),

    it({
      name: 'SERVES THE SAME FIRST ASK ON SYNTHETIC WHILE SYNTHETIC READS WET, so it is the hold on Hyper that refuses '
        + 'it and not the seat or the model (ledger B1365)',
      fn: async () => {
        const came = await firstAsk({
          read: readOlderProvidersWet,
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          reaskSeat: seatOf({ role: EDITOR_ROLE, },),
        },);
        expect(came.servedBy,).toBe('synthetic',);
        expect(came.reached,).toEqual(['synthetic',],);
      },
    },),

    it({
      name: 'REFUSES A HELD SEAT AS A SEAT NO PROVIDER SERVES IS REFUSED: the same class, raised before any call, which '
        + 'the stage reads as one unreachable seat (ledger B1365)',
      fn: async () => {
        const held = await firstAsk({
          read: readingAs(HYPER_WET_ONLY,),
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          reaskSeat: seatOf({ role: EDITOR_ROLE, },),
        },);
        // OpenRouter does not serve this model, so with Synthetic and Hyper both dry no provider is left for it.
        const dry = await firstAsk({
          read: readingAs(readingWith({ wet: ['openrouter',], },),),
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          reaskSeat: seatOf({ role: EDITOR_ROLE, },),
        },);
        /**
         What each refusal is: its class, whether it was raised after a call, the calls made, and whether its reason
         names the hold.
         */
        const [heldRefusal, dryRefusal,] = [held, dry,].map(function described(came,) {
          return {
            noProvider: came.refusal instanceof NoProviderForModelError,
            afterExchange: (came.refusal instanceof NoProviderForModelError) ? came.refusal.afterExchange : undefined,
            reached: came.reached,
            namesTheHold: (came.refusal instanceof NoProviderForModelError) && came.refusal.reason.includes(HELD_OFF_HYPER,),
          };
        },);
        expect(heldRefusal,).toEqual({ noProvider: true, afterExchange: false, reached: [], namesTheHold: true, },);
        expect(dryRefusal,).toEqual({ noProvider: true, afterExchange: false, reached: [], namesTheHold: false, },);
      },
    },),

    it({
      name: 'SERVES THE FIRST ASK OF A MEASUREMENT ON HYPER whatever the hold (ledger B1365, B1141): a measurement scores '
        + 'the model on the provider that carries it, and the same model refused as an editor is served',
      fn: async () => {
        const measured = await firstAsk({
          read: readingAs(HYPER_WET_ONLY,),
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          reaskSeat: { role: EDITOR_ROLE, bench: [], reask: 'measured-model', },
        },);
        expect(measured.servedBy,).toBe('hyper',);
        expect(measured.reached,).toEqual(['hyper',],);
      },
    },),

    it({
      name: 'SERVES A FIRST ASK THAT NAMES NO SEAT ON HYPER (ledger B1365): the refusal is for a seat that names its '
        + 'role, as the owner\'s ruling reads, and a caller that names none, the model health probe and the picture '
        + 'reader among them, keeps the routing it had',
      fn: async () => {
        const unseated = await firstAsk({
          read: readingAs(HYPER_WET_ONLY,),
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
        },);
        expect(unseated.servedBy,).toBe('hyper',);
        expect(unseated.reached,).toEqual(['hyper',],);
      },
    },),

    it({
      name: 'REFUSES KIMI-K3 IN THE SELECT SEATS AND SERVES IT IN THE EDITOR SEAT under the same reading (ledger B1365): '
        + 'its card holds it off Hyper for the select and slate seats alone, which shows the probe sees the difference',
      fn: async () => {
        expect(MODEL_CARDS[SEAT_SYNTHETIC_VISION_WITHHELD].holds,).toContain(HELD_OFF_HYPER_IN_SELECT_SEATS,);
        expect(routeTimeHoldsOf({ role: SELECT_ROLE, },),).toContain(HELD_OFF_HYPER_IN_SELECT_SEATS,);
        expect(routeTimeHoldsOf({ role: EDITOR_ROLE, },),).not.toContain(HELD_OFF_HYPER_IN_SELECT_SEATS,);
        const [select, editor,] = await Promise.all([
          firstAsk({
            read: readingAs(HYPER_WET_ONLY,),
            modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
            reaskSeat: seatOf({ role: SELECT_ROLE, },),
          },),
          firstAsk({
            read: readingAs(HYPER_WET_ONLY,),
            modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
            reaskSeat: seatOf({ role: EDITOR_ROLE, },),
          },),
        ],);
        expect(select?.refusal,).toBeInstanceOf(NoProviderForModelError,);
        expect(select?.reached,).toEqual([],);
        expect(editor?.servedBy,).toBe('hyper',);
      },
    },),
  ],
},);
