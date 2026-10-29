/**
 Tests for the produced-volume bound, and for the seam that carries it.

 THE SECOND HALF IS THE POINT. A bound nothing passes to the watch ends no
 call, which is exactly the state this task found the code in: `watchRunaway`
 already accepted a per-call bound and nothing anywhere handed it one. So the
 arithmetic is tested, and then the drain is shown to end a call at a bound
 far below the module default, with an otherwise identical drain naming no
 bound left running as the control.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  armIdleGuard,
  drainBody,
  StreamOverrunError,
} from '@monochromatic-dev/translation-repair-provider-stream';
import {
  MAX_PRODUCED_TO_SOURCE_RATIO,
  PRODUCED_VOLUME_FLOOR,
  producedVolumeBound,
} from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Answer characters the fixture carries, comfortably under the module default
 of 32000 so only a per-call bound can end it.
 */
const FIXTURE_ANSWER_CHARS = 8_192;

/**
 Per-call bound the fixture is cut at.
 */
const TIGHT_BOUND = 2_048;

/**
 Piece width the fixture body is delivered in, near what a socket delivers.
 */
const PIECE_CHARS = 512;

/**
 Builds one server-sent event frame carrying answer text.

 @param text - text this frame carries

 @returns Frame as the wire sends it

 @example
 ```ts
 const raw = frameOf({ text: 'The cat naps. ', },);
 ```
 */
function frameOf({ text, }: { readonly text: string; },): string {
  return `data: ${
    JSON.stringify({
      choices: [{
        index: 0,
        delta: { content: text, },
        finish_reason: null,
      },],
    },)
  }\n\n`;
}

/**
 Fixture body: an answer of known size, one sentence per frame.

 @returns Whole event stream

 @example
 ```ts
 const raw = bodyOf();
 ```
 */
function bodyOf(): string {
  /**
   One sentence, repeated to reach the fixture size.
   */
  const sentence = 'The cat naps by the empty bowl and waits for someone to fill it again. ';

  return Array.from(
    { length: Math.ceil(FIXTURE_ANSWER_CHARS / sentence.length,), },
    function frame(
      _unused,
      index,
    ): string {
      return frameOf({ text: `${sentence}${String(index,)} `, },);
    },
  )
    .join('',);
}

/**
 Wraps a body in a response delivered in socket-sized pieces.

 @param raw - whole body

 @returns Response whose body arrives in pieces

 @example
 ```ts
 const response = streamOf({ raw: bodyOf(), },);
 ```
 */
function streamOf({ raw, }: { readonly raw: string; },): Response {
  /**
   Encoder, since a body carries bytes rather than text.
   */
  const encoder = new TextEncoder();

  /**
   Cursor over the body, a record so the pull closure holds no loose binding.
   */
  const cursor = { at: 0, };

  return new Response(new ReadableStream<Uint8Array>({
    pull(controller,): void {
      if (cursor.at >= raw.length) {
        controller.close();
        return;
      }
      /**
       Next piece of the body, named so no line starts three nested calls.
       */
      const piece = raw.slice(cursor.at, cursor.at + PIECE_CHARS,);

      controller.enqueue(encoder.encode(piece,),);
      cursor.at += PIECE_CHARS;
    },
  },),);
}

/**
 Runs one call that must refuse and hands back what it threw.

 ASYNC, so `caught` from module-test does not apply: that one is synchronous
 by design, and a rejected promise needs its own capture. Reading the refusal
 rather than matching on a message lets the class and the bound it names both
 be asserted, and an assertion naming only a message passes just as happily
 when the wrong error type is thrown.

 @param act - call that must reject

 @returns Failure it raised

 @throws `Error` when the call resolved instead of refusing

 @example
 ```ts
 const refusal = await refusalFrom(async function readsPastTheBound() { ... },);
 ```
 */
async function refusalFrom(act: () => Promise<void>,): Promise<unknown> {
  try {
    await act();
  }
  catch (error) {
    return error;
  }
  throw new Error('Expected the drain to refuse, but it returned',);
}

await describe({
  name: producedVolumeBound.name,
  children: [
    it({
      name: 'FLOORS A SHORT SOURCE, because three times a heading is a few dozen characters and a '
        + 'purely proportional bound would sit under any answer a model could reasonably give',
      fn: async function floorsShortSources() {
        expect(producedVolumeBound({ materialChars: 56, },),).toBe(PRODUCED_VOLUME_FLOOR,);
      },
    },),

    it({
      name: 'SCALES WITH A LONG SOURCE, so a passage that legitimately runs long is bounded by its '
        + 'own size rather than by a number chosen for short text',
      fn: async function scalesWithLongSources() {
        expect(producedVolumeBound({ materialChars: 232, },),).toBe(232 * MAX_PRODUCED_TO_SOURCE_RATIO,);
      },
    },),

    it({
      name: 'CROSSES OVER WHERE THE TWO RULES MEET, so neither leaves a gap the other does not cover',
      fn: async function crossesOverCleanly() {
        /**
         Source whose proportional bound is exactly the floor.
         */
        const atCrossover = PRODUCED_VOLUME_FLOOR / MAX_PRODUCED_TO_SOURCE_RATIO;

        expect(producedVolumeBound({ materialChars: atCrossover, },),).toBe(PRODUCED_VOLUME_FLOOR,);
        expect(producedVolumeBound({ materialChars: atCrossover + 1, },),)
          .toBe((atCrossover + 1) * MAX_PRODUCED_TO_SOURCE_RATIO,);
      },
    },),

    it({
      name: 'REFUSES AT A PER-CALL BOUND the module default would never reach, which is the whole '
        + 'seam: the watch already accepted a bound and nothing was passing one',
      fn: async function drainHonoursTheBound() {
        using guard = armIdleGuard({ label: 'bowl', },);

        /**
         What the drain raised, read for its class as well as the bound it names.
         */
        const raised = await refusalFrom(async function readsPastTheBound(): Promise<void> {
          await drainBody({
            credentials: [],
            response: streamOf({ raw: bodyOf(), },),
            guard,
            callerSignal: AbortSignal.timeout(HANG_STOP_MS,),
            label: 'bowl',
            maxAnswerChars: TIGHT_BOUND,
          },);
        },);

        expect(raised,).toBeInstanceOf(StreamOverrunError,);
        expect((raised as StreamOverrunError).cap,).toBe(TIGHT_BOUND,);
        expect((raised as StreamOverrunError).charsSeen,).toBeGreaterThanOrEqual(TIGHT_BOUND,);
      },
    },),

    it({
      name: 'CONTROL: THE SAME STREAM DRAINS WHOLE with no bound named, so the refusal is the bound '
        + 'doing the work rather than anything about the fixture',
      fn: async function unboundedDrainCompletes() {
        using guard = armIdleGuard({ label: 'bowl', },);

        /**
         Whole body, which only arrives if nothing ended the call.
         */
        const body = await drainBody({
          credentials: [],
          response: streamOf({ raw: bodyOf(), },),
          guard,
          callerSignal: AbortSignal.timeout(HANG_STOP_MS,),
          label: 'bowl',
        },);

        expect(body.length,).toBeGreaterThan(FIXTURE_ANSWER_CHARS,);
      },
    },),
  ],
},);
