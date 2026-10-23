/**
 Tests for the gate that ends a pass when every provider reads dry: a dry refusal on any call of the guarded
 client trips the pass's own abort with that refusal as the reason, once, and every other failure goes by
 untouched.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  createProviderBudgets,
  EveryProviderDryError,
  type ModelClient,
  NOBODY_REFUSED,
  readBudgetsPastHolds,
} from '@monochromatic-dev/translation-repair-provider';
import { WritingBenchUnreachableError, } from '@monochromatic-dev/translation-repair-run';
import {
  createPassDryStop,
  recordOnPark,
} from '../../dist/final/node/index.mjs';
import { modelDarkRefusal, } from '@monochromatic-dev/translation-repair-provider/ts/model-dark-refusal.test-fixture.ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Client whose chat calls all throw one scripted failure.

 @param thrown - what each call throws

 @returns The client

 @example
 ```ts
 const client = throwingClient({ thrown: new Error('cat nap', ), },);
 ```
 */
function throwingClient({ thrown, }: { readonly thrown: Error; },): ModelClient {
  return {
    chatText: async () => {
      throw thrown;
    },
    chatJson: async () => {
      throw thrown;
    },
    decide: async () => {
      throw thrown;
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Request every chat call here sends.

 @param signal - the call's abort handle

 @returns A request over no messages

 @example
 ```ts
 await client.chatText(requestWith({ signal, },),);
 ```
 */
function requestWith({ signal, }: { readonly signal: AbortSignal; },): Parameters<ModelClient['chatText']>[0] {
  return {
    modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    messages: [],
    signal,
  };
}

/**
 The day start of the spent day a park is for.
 */
const DAY_ONE = '2026-10-22T19:15:08.000Z';

/**
 The day start of the day after it.
 */
const DAY_TWO = '2026-10-23T19:15:08.000Z';

/**
 A refusal naming a spent NeuralWatt day 301 s from its return.

 @param dayStart - day start the reading named

 @param readAtMs - instant on the gate's clock of the seat reading that made the refusal, absent for the router's own

 @returns Refusal, carrying the distance, the time of day and the dryness that tells one spent day from the next

 @example
 ```ts
 const refusal = spentDayRefusal({ dayStart: DAY_ONE, },);
 ```
 */
function spentDayRefusal(
  {
    dayStart,
    readAtMs,
  }: {
    readonly dayStart: string;
    readonly readAtMs?: number;
  },
): EveryProviderDryError {
  return new EveryProviderDryError({
    measured: 'seat reading',
    neuralwatt: { kind: 'at', msOfDay: 69_308_000, },
    neuralwattDryness: { kind: 'spent-day', dayStart, returnsInMs: 301_000, },
    neuralwattReturnsInMs: 301_000,
    ...((readAtMs === undefined) ? {} : { neuralwattReadAtMs: readAtMs, }),
  },);
}

/**
 A bench error for a writing bench below its floor with Synthetic wet, carrying the NeuralWatt return that lifts it.

 @param returnsInMs - milliseconds to the return, absent for an error that carries none

 @param dayStart - day start of the spent day the return is for

 @param readAtMs - instant on the gate's clock of the read that named the return, absent for an error without one

 @returns Error

 @example
 ```ts
 const error = benchErrorBefore({ returnsInMs: 301_000, dayStart: DAY_ONE, },);
 ```
 */
function benchErrorBefore(
  {
    returnsInMs,
    dayStart = DAY_ONE,
    readAtMs,
  }: {
    readonly returnsInMs?: number;
    readonly dayStart?: string;
    readonly readAtMs?: number;
  },
): WritingBenchUnreachableError {
  return new WritingBenchUnreachableError({
    phase: 'lanes',
    clauses: ['editors 0 of 3 reachable, floor 2',],
    ...((returnsInMs === undefined)
      ? {}
      : {
        neuralwattReturnsInMs: returnsInMs,
        neuralwattReturnsAt: '19:15:08 UTC',
        neuralwattDryness: { kind: 'spent-day', dayStart, returnsInMs, },
        ...((readAtMs === undefined) ? {} : { neuralwattReadAtMs: readAtMs, }),
      }),
  },);
}

await describe({
  name: createPassDryStop.name,
  children: [
    it({
      name: 'STARTS UNTRIPPED: the signal is live and no stop is asked for',
      fn: async () => {
        const dryStop = createPassDryStop();

        expect(dryStop.signal.aborted,).toBe(false,);
        expect(dryStop.tripped(),).toBe(false,);
      },
    },),

    it({
      name: 'TRIPS ON A DRY REFUSAL from chat text, rethrowing that very object and aborting the signal with it as '
        + 'the reason, so every call in flight under the signal ends with the cause rather than a timeout',
      fn: async () => {
        const dryStop = createPassDryStop();
        const dry = new EveryProviderDryError({ measured: 'meters read every provider dry', },);
        const client = dryStop.guard({ client: throwingClient({ thrown: dry, },), },);

        expect(await rejectionOf(async function call(): Promise<unknown> {
          return await client.chatText(requestWith({ signal: dryStop.signal, },),);
        },),).toBe(dry,);
        expect(dryStop.signal.aborted,).toBe(true,);
        expect(dryStop.signal.reason,).toBe(dry,);
      },
    },),

    it({
      name: 'DOES NOT TRIP ON A MODEL-DARK REFUSAL from chat text, rethrowing it with the signal untouched, since a '
        + 'model whose own providers are dry is a lost seat and the run goes on',
      fn: async () => {
        const dryStop = createPassDryStop();
        const dark = modelDarkRefusal({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, },);
        const client = dryStop.guard({ client: throwingClient({ thrown: dark, },), },);

        expect(await rejectionOf(async function call(): Promise<unknown> {
          return await client.chatText(requestWith({ signal: dryStop.signal, },),);
        },),).toBe(dark,);
        expect(dryStop.signal.aborted,).toBe(false,);
        expect(dryStop.tripped(),).toBe(false,);
      },
    },),

    it({
      name: 'TRIPS ON A DRY REFUSAL from the typed JSON exchange and from a decision seat as well',
      fn: async () => {
        const viaJson = createPassDryStop();
        const dryJson = new EveryProviderDryError({ measured: 'json path', },);
        await rejectionOf(async function call(): Promise<unknown> {
          return await viaJson.guard({ client: throwingClient({ thrown: dryJson, },), },).chatJson({
            ...requestWith({ signal: viaJson.signal, },),
            responseFormat: {
              type: 'json_schema',
              json_schema: { name: 'cat', schema: { type: 'object', }, },
            },
            validate: (value: unknown): value is object => (typeof value) === 'object',
          },);
        },);
        expect(viaJson.signal.reason,).toBe(dryJson,);

        const viaDecide = createPassDryStop();
        const dryDecide = new EveryProviderDryError({ measured: 'decide path', },);
        await rejectionOf(async function call(): Promise<unknown> {
          return await viaDecide.guard({ client: throwingClient({ thrown: dryDecide, },), },).decide?.({
            modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            state: {},
            questions: {},
            signal: viaDecide.signal,
          } as never,);
        },);
        expect(viaDecide.signal.reason,).toBe(dryDecide,);
      },
    },),

    it({
      name: 'KEEPS THE FIRST CAUSE when a second call meets the dry router after the stop, and names it in one note '
        + 'that says what the run does next',
      fn: async () => {
        const dryStop = createPassDryStop();
        const first = new EveryProviderDryError({ measured: 'first reading', },);
        const second = new EveryProviderDryError({ measured: 'second reading', },);
        await rejectionOf(async function call(): Promise<unknown> {
          return await dryStop.guard({ client: throwingClient({ thrown: first, },), },).chatText(
            requestWith({ signal: dryStop.signal, },),
          );
        },);
        await rejectionOf(async function call(): Promise<unknown> {
          return await dryStop.guard({ client: throwingClient({ thrown: second, },), },).chatText(
            requestWith({ signal: dryStop.signal, },),
          );
        },);

        expect(dryStop.signal.reason,).toBe(first,);
        expect(dryStop.tripped(),).toBe(true,);
        const [note = '',] = dryStop.takeStopNotes();
        expect(note.startsWith('STOP ',),).toBe(true,);
        expect(note,).toContain('every provider',);
        expect(note,).toContain('first reading',);
        expect(note,).not.toContain('second reading',);
        expect(note,).toContain('no further entry starts',);
      },
    },),

    it({
      name: 'LOGS ONE LINE WHEN IT TRIPS, naming the cause, and none for a second refusal or for a failure that does not '
        + 'end the run, so the first operator-visible output is the stop and not the in-flight entry\'s TALLY line',
      fn: async () => {
        const lines: string[] = [];
        const dryStop = createPassDryStop({ l: levelCapturingLogger({ lines, },), },);
        await rejectionOf(async function call(): Promise<unknown> {
          return await dryStop.guard({ client: throwingClient({ thrown: new Error('connection reset',), }), },).chatText(
            requestWith({ signal: dryStop.signal, },),
          );
        },);
        expect(lines,).toEqual([],);
        await rejectionOf(async function call(): Promise<unknown> {
          return await dryStop.guard({
            client: throwingClient({
              thrown: new EveryProviderDryError({ measured: 'first reading', },),
            }),
          },).chatText(requestWith({ signal: dryStop.signal, },),);
        },);
        await rejectionOf(async function call(): Promise<unknown> {
          return await dryStop.guard({
            client: throwingClient({
              thrown: new EveryProviderDryError({ measured: 'second reading', },),
            }),
          },).chatText(requestWith({ signal: dryStop.signal, },),);
        },);
        expect(lines.length,).toBe(1,);
        expect(lines[0],).toContain('first reading',);
        expect(lines[0],).toContain('every provider',);
      },
    },),

    it({
      name: 'TRIPS ON A REFUSAL HANDED TO IT DIRECTLY, as the entry scheduler hands the one that ended an entry however '
        + 'it reached the entry, and ignores any other failure',
      fn: async () => {
        const dryStop = createPassDryStop();
        dryStop.trip({ error: new Error('connection reset',), },);
        expect(dryStop.tripped(),).toBe(false,);
        const dry = new EveryProviderDryError({ measured: 'seat reading', },);
        dryStop.trip({ error: dry, },);
        expect(dryStop.tripped(),).toBe(true,);
        expect(dryStop.signal.reason,).toBe(dry,);
      },
    },),

    it({
      name: 'HANDS OUT THE STOP NOTE ONCE: none before the trip, the note the first time after it, none after',
      fn: async () => {
        const dryStop = createPassDryStop();
        expect(dryStop.takeStopNotes(),).toEqual([],);
        dryStop.trip({ error: new EveryProviderDryError({ measured: 'seat reading', },), },);
        expect(dryStop.takeStopNotes().length,).toBe(1,);
        expect(dryStop.takeStopNotes(),).toEqual([],);
      },
    },),

    it({
      name: 'PARKS INSTEAD OF STOPPING when every provider reads dry and NeuralWatt\'s return is 301 s away, inside '
        + 'what the pass has left: the calls in flight end with the refusal, the pass is not tripped, and after the '
        + 'wait until the return the signal is live again, with a line at the start naming the reason and the return '
        + 'time in UTC and one at the end',
      fn: async () => {
        /**
         Milliseconds the park waited for.
         */
        const waits: number[] = [];
        const lines: string[] = [];
        const dryStop = createPassDryStop({
          l: levelCapturingLogger({ lines, },),
          passLeftMs: () => 3_600_000,
          // A frozen clock: the wait is the whole 301 s the refusal named.
          now: () => 1_000,
          waitMs: async function waitMs({ ms, },): Promise<void> {
            waits.push(ms,);
          },
        },);
        const dry = new EveryProviderDryError({
          measured: 'meters read every provider dry',
          neuralwatt: { kind: 'at', msOfDay: 69_308_000, },
          neuralwattReturnsInMs: 301_000,
        },);
        const client = dryStop.guard({ client: throwingClient({ thrown: dry, },), },);

        expect(await rejectionOf(async function call(): Promise<unknown> {
          return await client.chatText(requestWith({ signal: dryStop.signal, },),);
        },),).toBe(dry,);
        expect(dryStop.tripped(),).toBe(false,);
        expect(dryStop.parked(),).toBe(true,);
        expect(dryStop.signal.aborted,).toBe(true,);
        expect(dryStop.takeStopNotes(),).toEqual([],);

        await dryStop.awaitPark();
        expect(waits,).toEqual([301_000,],);
        expect(dryStop.parked(),).toBe(false,);
        expect(dryStop.tripped(),).toBe(false,);
        expect(dryStop.signal.aborted,).toBe(false,);
        expect(lines.length,).toBe(2,);
        expect(lines[0],).toContain('parked',);
        expect(lines[0],).toContain('every provider is out of budget',);
        expect(lines[0],).toContain('19:15:08 UTC',);
        expect(lines[1],).toContain('resumes',);
      },
    },),

    it({
      name: 'STOPS, stating the return time, when NeuralWatt\'s return is past what the pass has left: the park is for '
        + 'a return the pass can still use',
      fn: async () => {
        const dryStop = createPassDryStop({ passLeftMs: () => 300_000, },);
        dryStop.trip({
          error: new EveryProviderDryError({
            measured: 'seat reading',
            neuralwatt: { kind: 'at', msOfDay: 69_308_000, },
            neuralwattReturnsInMs: 301_000,
          },),
        },);
        expect(dryStop.tripped(),).toBe(true,);
        expect(dryStop.parked(),).toBe(false,);
        const [note = '',] = dryStop.takeStopNotes();
        expect(note,).toContain('19:15:08 UTC',);
        // The distance and why the pass did not wait for it, in the sentence the note adds.
        expect(note,).toContain('in 5 min 1 s, which is past the 5 min this pass has left',);
      },
    },),

    it({
      name: 'WAITS ONLY FOR WHAT IS LEFT OF THE RETURN: the park stores the return as an instant on the gate\'s clock, '
        + 'so the time the entry in flight took to end after the refusal comes off the wait, and a return that has '
        + 'passed by then is waited for not at all',
      fn: async () => {
        /**
         Clock the gate reads, moved by the case.
         */
        const clock = { now: 1_000, };
        /**
         Milliseconds each park waited for.
         */
        const waits: number[] = [];
        /**
         Gate whose clock the case moves and whose waits it records.
         */
        const dryStop = createPassDryStop({
          passLeftMs: () => 90_000_000,
          now: () => clock.now,
          waitMs: async function waitMs({ ms, },): Promise<void> {
            waits.push(ms,);
          },
        },);
        dryStop.trip({ error: spentDayRefusal({ dayStart: DAY_ONE, },), },);
        // The entry in flight took 100 s to end after the refusal.
        clock.now += 100_000;
        await dryStop.awaitPark();
        expect(waits,).toEqual([201_000,],);

        const late = createPassDryStop({
          passLeftMs: () => 90_000_000,
          now: () => clock.now,
          waitMs: async function waitMs({ ms, },): Promise<void> {
            waits.push(ms,);
          },
        },);
        late.trip({ error: spentDayRefusal({ dayStart: DAY_ONE, },), },);
        // The entry took longer than the return was away.
        clock.now += 400_000;
        await late.awaitPark();
        expect(waits,).toEqual([201_000, 0,],);
      },
    },),

    it({
      name: 'STOPS ON A DRY REFUSAL FOR THE SAME DAY START AS THE PARK THAT ENDED, however long after: the account has '
        + 'not rolled to a new day, and parking again would wait a day in a loop; a refusal for the next day '
        + 'start parks again',
      fn: async () => {
        /**
         Clock the gate reads, moved by the case.
         */
        const clock = { now: 1_000, };
        const dryStop = createPassDryStop({
          passLeftMs: () => 90_000_000,
          now: () => clock.now,
          waitMs: async function waitMs({ ms, },): Promise<void> {
            clock.now += ms;
          },
        },);
        dryStop.trip({ error: spentDayRefusal({ dayStart: DAY_ONE, },), },);
        await dryStop.awaitPark();
        // An hour later, far past any settling window: the same spent day still stands.
        clock.now += 3_600_000;
        dryStop.trip({ error: spentDayRefusal({ dayStart: DAY_ONE, },), },);
        expect(dryStop.tripped(),).toBe(true,);

        const next = createPassDryStop({
          passLeftMs: () => 90_000_000,
          now: () => clock.now,
          waitMs: async function waitMs({ ms, },): Promise<void> {
            clock.now += ms;
          },
        },);
        next.trip({ error: spentDayRefusal({ dayStart: DAY_ONE, },), },);
        await next.awaitPark();
        next.trip({ error: spentDayRefusal({ dayStart: DAY_TWO, },), },);
        expect(next.tripped(),).toBe(false,);
        expect(next.parked(),).toBe(true,);
      },
    },),

    it({
      name: 'PARKS ON THE REFUSAL THE ROUTER MAKES AFTER WAITING OUT ANOTHER HOLD, over real budgets: Hyper held 40 ms '
        + 'and NeuralWatt\'s spent day 600 s from starting again, past the router\'s own 300 s, so the call waits for '
        + 'Hyper and the refusal still carries the return, which the pass has the time for',
      fn: async () => {
        /**
         Milliseconds after 00:00 UTC the account's day starts at: 19:15:08 UTC.
         */
        const dayStartMsOfDay = 69_308_000;
        const budgets = createProviderBudgets({
          hyper: { credits: async () => ({ balance: 0, }), },
          neuralwatt: {
            energy: async () => ({
              dayStart: '2026-10-22T19:15:08.000Z',
              dailyWh: 550,
              spentWh: 550,
              reckonedWh: 0,
              reservedWh: 0,
              remainingWh: 0,
              calls: 40,
            }),
            dayAnchor: () => ({ known: true, msOfDay: dayStartMsOfDay, }),
          },
          freshForMs: 600_000,
          cooldownMs: 40,
          wallNow: () => new Date(Date.UTC(
            2_026,
            9,
            23,
            0,
            0,
            0,
            dayStartMsOfDay - 600_000,
          ),),
          returnMarginMs: 0,
        },);
        const {signal} = new AbortController();
        await budgets.read({ signal, },);
        await budgets.markRefused({ provider: 'hyper', signal, },);
        const refusal = await rejectionOf(async function route(): Promise<unknown> {
          return await readBudgetsPastHolds({
            budgets,
            modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            signal,
            refused: NOBODY_REFUSED,
            pollMs: 1,
          },);
        },);
        const dryStop = createPassDryStop({ passLeftMs: () => 3_600_000, waitMs: async () => {}, },);
        dryStop.trip({ error: refusal, },);
        expect(dryStop.tripped(),).toBe(false,);
        expect(dryStop.parked(),).toBe(true,);
      },
    },),

    it({
      name: 'PARKS ON A WRITING BENCH ERROR THAT CARRIES NEURALWATT\'S RETURN, as on the all-dry refusal: Synthetic '
        + 'reads wet, the bench is below its floor, the return lifts it and lies 301 s away inside what the pass has '
        + 'left, so the pass is parked and not tripped, waits for the return, and resumes',
      fn: async () => {
        /**
         Milliseconds the park waited for.
         */
        const waits: number[] = [];
        const lines: string[] = [];
        const dryStop = createPassDryStop({
          l: levelCapturingLogger({ lines, },),
          passLeftMs: () => 3_600_000,
          now: () => 1_000,
          waitMs: async function waitMs({ ms, },): Promise<void> {
            waits.push(ms,);
          },
        },);
        dryStop.trip({ error: benchErrorBefore({ returnsInMs: 301_000, },), },);
        expect(dryStop.parked(),).toBe(true,);
        expect(dryStop.tripped(),).toBe(false,);
        expect(dryStop.signal.aborted,).toBe(true,);
        expect(lines[0],).toContain('parked',);
        expect(lines[0],).toContain('writing bench',);
        expect(lines[0],).toContain('19:15:08 UTC',);
        await dryStop.awaitPark();
        expect(waits,).toEqual([301_000,],);
        expect(dryStop.parked(),).toBe(false,);
        expect(dryStop.signal.aborted,).toBe(false,);
      },
    },),

    it({
      name: 'WAITS FROM THE READ THAT NAMED THE RETURN, NOT FROM THE TRIP: a bench error or an all-dry refusal stamped '
        + 'with the instant the seat reader read the distance is parked for the return less the 100 s the in-flight '
        + 'slices took to unwind, and an error with no stamp is waited for from the trip',
      fn: async () => {
        /**
         Milliseconds each park waited for.
         */
        const waits: number[] = [];
        /**
         Gate whose clock reads 101 s when the entry hands over its bench error.
         */
        const stamped = createPassDryStop({
          passLeftMs: () => 90_000_000,
          now: () => 101_000,
          waitMs: async function waitMs({ ms, },): Promise<void> {
            waits.push(ms,);
          },
        },);
        stamped.trip({ error: benchErrorBefore({ returnsInMs: 301_000, readAtMs: 1_000, }), },);
        await stamped.awaitPark();
        expect(waits,).toEqual([201_000,],);

        const unstamped = createPassDryStop({
          passLeftMs: () => 90_000_000,
          now: () => 101_000,
          waitMs: async function waitMs({ ms, },): Promise<void> {
            waits.push(ms,);
          },
        },);
        unstamped.trip({ error: benchErrorBefore({ returnsInMs: 301_000, }), },);
        await unstamped.awaitPark();
        expect(waits,).toEqual([201_000, 301_000,],);

        // The seat reader's all-dry refusal carries the stamp as the bench error does.
        const refused = createPassDryStop({
          passLeftMs: () => 90_000_000,
          now: () => 101_000,
          waitMs: async function waitMs({ ms, },): Promise<void> {
            waits.push(ms,);
          },
        },);
        refused.trip({ error: spentDayRefusal({ dayStart: DAY_ONE, readAtMs: 1_000, }), },);
        await refused.awaitPark();
        expect(waits,).toEqual([201_000, 301_000, 201_000,],);
      },
    },),

    it({
      name: 'ANSWERS WHETHER IT PARKED FOR THIS VERY FAILURE, not whether it is parked or of a parking class: nothing '
        + 'before the trip, the failure it parked for after it, and never an unrelated failure, a bench error with no '
        + 'return, or another parking failure that is not the one the park is for',
      fn: async () => {
        const dryStop = createPassDryStop({ passLeftMs: () => 3_600_000, waitMs: async () => {}, },);
        /**
         The bench error with a return that fits.
         */
        const bench = benchErrorBefore({ returnsInMs: 301_000, },);
        expect(dryStop.parkedFor({ error: bench, },),).toEqual({ parked: false, },);
        dryStop.trip({ error: bench, },);
        expect(dryStop.parked(),).toBe(true,);
        expect(dryStop.parkedFor({ error: bench, },),).toEqual({
          parked: true,
          returnsInMs: 301_000,
          returnsAt: '19:15:08 UTC',
        },);
        // Another entry's parking failure, with the same class and the same return, is not the park's own.
        expect(dryStop.parkedFor({ error: spentDayRefusal({ dayStart: DAY_ONE, },), },),).toEqual({ parked: false, },);
        expect(dryStop.parkedFor({ error: benchErrorBefore({ returnsInMs: 301_000, }), },),).toEqual(
          { parked: false, },
        );
        expect(dryStop.parkedFor({ error: new Error('connection reset',), },),).toEqual({ parked: false, },);
        expect(dryStop.parkedFor({ error: benchErrorBefore({}), },),).toEqual({ parked: false, },);
      },
    },),

    it({
      name: 'ANSWERS FOR EVERY PARKING FAILURE IT SAW DURING THE PARK, since a layer above the guard may rethrow '
        + 'any of them: two distinct refusals handed to it are both the park\'s, an ordinary failure and a bench '
        + 'error with no return are not, and none is once the park has ended',
      fn: async () => {
        const dryStop = createPassDryStop({ passLeftMs: () => 3_600_000, waitMs: async () => {}, },);
        /**
         The refusal that parks the pass.
         */
        const first = spentDayRefusal({ dayStart: DAY_ONE, },);
        /**
         A second parking failure of the same entry, met after the park began and rethrown by a layer above the guard.
         */
        const second = benchErrorBefore({ returnsInMs: 301_000, },);
        /**
         An ordinary failure of the same entry.
         */
        const ordinary = new Error('connection reset',);
        /**
         A bench error that carries no return, so the gate does not act on it.
         */
        const noReturn = benchErrorBefore({},);
        /**
         A parking failure the gate never saw.
         */
        const unseen = benchErrorBefore({ returnsInMs: 301_000, },);
        dryStop.trip({ error: first, },);
        dryStop.trip({ error: second, },);
        dryStop.trip({ error: ordinary, },);
        dryStop.trip({ error: noReturn, },);
        expect(dryStop.parkedFor({ error: first, },),).toMatchObject({ parked: true, },);
        expect(dryStop.parkedFor({ error: second, },),).toMatchObject({ parked: true, },);
        expect(dryStop.parkedFor({ error: ordinary, },),).toEqual({ parked: false, },);
        expect(dryStop.parkedFor({ error: noReturn, },),).toEqual({ parked: false, },);
        expect(dryStop.parkedFor({ error: unseen, },),).toEqual({ parked: false, },);

        await dryStop.awaitPark();
        /**
         A refusal for the spent day the park waited out, handed in after the park ended: it trips the gate instead.
         */
        const late = spentDayRefusal({ dayStart: DAY_ONE, },);
        dryStop.trip({ error: late, },);
        expect(dryStop.parkedFor({ error: first, },),).toEqual({ parked: false, },);
        expect(dryStop.parkedFor({ error: second, },),).toEqual({ parked: false, },);
        expect(dryStop.parkedFor({ error: late, },),).toEqual({ parked: false, },);
      },
    },),

    it({
      name: 'CARRIES NO FAILURE OF AN EARLIER PARK INTO A LATER ONE: after the first park has ended and the gate parks '
        + 'again for a refusal of another spent day, the failures the first park saw are not the second\'s',
      fn: async () => {
        const dryStop = createPassDryStop({ passLeftMs: () => 3_600_000, waitMs: async () => {}, },);
        /**
         The refusal that parks the pass the first time.
         */
        const first = spentDayRefusal({ dayStart: DAY_ONE, },);
        /**
         A further parking failure the first park saw.
         */
        const second = benchErrorBefore({ returnsInMs: 301_000, },);
        /**
         The refusal for the next spent day, which parks the pass again.
         */
        const again = spentDayRefusal({ dayStart: DAY_TWO, },);
        dryStop.trip({ error: first, },);
        dryStop.trip({ error: second, },);
        expect(dryStop.parkedFor({ error: second, },).parked,).toBe(true,);
        await dryStop.awaitPark();
        dryStop.trip({ error: again, },);
        expect(dryStop.parked(),).toBe(true,);
        expect(dryStop.parkedFor({ error: again, },).parked,).toBe(true,);
        expect(dryStop.parkedFor({ error: first, },),).toEqual({ parked: false, },);
        expect(dryStop.parkedFor({ error: second, },),).toEqual({ parked: false, },);
      },
    },),

    it({
      name: 'ANSWERS THE PARK\'S OWN RETURN for every failure it answers for, never the failure\'s: a second parking '
        + 'refusal that carries no NeuralWatt return and one that carries another distance, met during the park, '
        + 'get the distance and the time the park waits for, and an ordinary failure gets none',
      fn: async () => {
        const dryStop = createPassDryStop({ passLeftMs: () => 3_600_000, waitMs: async () => {}, },);
        /**
         The refusal that parks the pass, 301 s from its return.
         */
        const first = spentDayRefusal({ dayStart: DAY_ONE, },);
        /**
         A parking refusal of the same entry from a router that had no day information, so it names no return.
         */
        const noReturn = new EveryProviderDryError({ measured: 'every meter dry', },);
        /**
         A parking bench error of the same entry that names a nearer return than the park waits for.
         */
        const nearer = benchErrorBefore({ returnsInMs: 120_000, },);
        dryStop.trip({ error: first, },);
        dryStop.trip({ error: noReturn, },);
        dryStop.trip({ error: nearer, },);
        /**
         What the park waits for.
         */
        const parkReturn = { parked: true, returnsInMs: 301_000, returnsAt: '19:15:08 UTC', };
        expect(dryStop.parkedFor({ error: first, },),).toEqual(parkReturn,);
        expect(dryStop.parkedFor({ error: noReturn, },),).toEqual(parkReturn,);
        expect(dryStop.parkedFor({ error: nearer, },),).toEqual(parkReturn,);
        expect(dryStop.parkedFor({ error: new Error('connection reset',), },),).toEqual({ parked: false, },);
      },
    },),

    it({
      name: 'RECORDS ONLY THE PARKING FAILURES OTHER THAN THE PARK\'S OWN REFUSAL, once each: the refusal that parked '
        + 'the pass, handed in again by the entry\'s catch, leaves the list empty',
      fn: async () => {
        /**
         The refusal that parks the pass.
         */
        const refusal = spentDayRefusal({ dayStart: DAY_ONE, },);
        /**
         A further parking failure of the same entry.
         */
        const other = benchErrorBefore({ returnsInMs: 301_000, },);
        /**
         The park in force, as the gate builds it.
         */
        const park: Parameters<typeof recordOnPark>[0]['park'] = {
          refusal,
          seen: [],
          returnsInMs: 301_000,
          atMs: 301_000,
          returnsAt: '19:15:08 UTC',
          dayStart: DAY_ONE,
        };
        recordOnPark({ park, error: refusal, },);
        expect(park.seen,).toEqual([],);
        recordOnPark({ park, error: other, },);
        recordOnPark({ park, error: other, },);
        recordOnPark({ park, error: refusal, },);
        expect(park.seen,).toEqual([other,],);
      },
    },),

    it({
      name: 'LEAVES A WRITING BENCH ERROR ALONE when it carries no return, or one past what the pass has left, or the '
        + 'day start of a park that has ended: the entry ends as before, the pass is neither parked nor tripped, and '
        + 'no stop line is written',
      fn: async () => {
        /**
         Pass that has 300 s left.
         */
        const dryStop = createPassDryStop({ passLeftMs: () => 300_000, waitMs: async () => {}, },);
        dryStop.trip({ error: benchErrorBefore({}), },);
        dryStop.trip({ error: benchErrorBefore({ returnsInMs: 301_000, }), },);
        expect(dryStop.parked(),).toBe(false,);
        expect(dryStop.tripped(),).toBe(false,);
        expect(dryStop.signal.aborted,).toBe(false,);
        expect(dryStop.takeStopNotes(),).toEqual([],);

        /**
         Pass that parks once, then meets the bench error for the same day start.
         */
        const parking = createPassDryStop({ passLeftMs: () => 3_600_000, waitMs: async () => {}, },);
        parking.trip({ error: benchErrorBefore({ returnsInMs: 301_000, }), },);
        await parking.awaitPark();
        parking.trip({ error: benchErrorBefore({ returnsInMs: 301_000, }), },);
        expect(parking.parked(),).toBe(false,);
        expect(parking.tripped(),).toBe(false,);
      },
    },),

    it({
      name: 'LEAVES EVERY OTHER FAILURE ALONE: it propagates unchanged and the stop is not tripped',
      fn: async () => {
        const dryStop = createPassDryStop();
        const other = new Error('connection reset',);
        const client = dryStop.guard({ client: throwingClient({ thrown: other, },), },);

        expect(await rejectionOf(async function call(): Promise<unknown> {
          return await client.chatText(requestWith({ signal: dryStop.signal, },),);
        },),).toBe(other,);
        expect(dryStop.signal.aborted,).toBe(false,);
        expect(dryStop.tripped(),).toBe(false,);
      },
    },),

    it({
      name: 'KEEPS WHAT THE CLIENT CARRIES BESIDE THE CALLS, and a client without decisions stays without them',
      fn: async () => {
        const dryStop = createPassDryStop();
        const marked = {
          chatText: async () => {
            throw new Error('unused',);
          },
          chatJson: async () => {
            throw new Error('unused',);
          },
          quotas: async () => {
            throw new Error('unused',);
          },
          providerHolds: 'kept',
        };
        const guarded = dryStop.guard({ client: marked as unknown as ModelClient & { readonly providerHolds: string; }, },);

        expect(guarded.providerHolds,).toBe('kept',);
        expect(guarded.decide,).toBeUndefined();
      },
    },),
  ],
},);
