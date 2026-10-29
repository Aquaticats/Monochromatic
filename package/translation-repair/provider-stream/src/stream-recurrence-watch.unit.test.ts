/**
 Tests for the recurrence watch, asked directly rather than through the
 runaway watch that composes it.

 WHAT `stream-runaway-watch.unit.test.ts` ALREADY PROVES, and is not repeated
 here: that a long-period loop past the length bar ends the stream, that the
 same loop under the bar finishes, and that a candidate quoted twice back to
 back finishes, including at the length most prone to a false positive. Those
 are the detector's headline claims and they are covered.

 WHAT NOTHING COVERED, found by mutation rather than by reading: the BOUNDED
 BUFFER. Removing the trim that keeps only the trailing `BUFFER_CHARS` left
 the whole suite green. That trim is load-bearing twice over. It is what makes
 the cost of watching a stream that never ends constant, which is the only
 reason this detector can run on the streams it exists to stop. And it is what
 decides how far apart two copies of a passage may be before the earlier one
 stops counting, which is a correctness rule and not an optimisation: a
 reasoning trace in this pipeline restates whole candidates verbatim, so one
 quoted near the start and again near the end is ordinary work, and without
 the trim the early copy stays findable forever and the second quotation reads
 as a loop.

 SO THE CENTRAL CASE HERE IS A PAIR OF DISTANT QUOTATIONS. The same passage
 appears twice, far enough apart that the first has scrolled out, with unique
 text between and around. It must finish. With the trim removed it does not.

 THE EMPTY-TEXT EARLY RETURN IS A PROVEN NO-OP, cased by "IGNORES AN EMPTY
 TEXT ARRIVAL" rather than through `verdictOver`: without it the counters
 would advance by zero and the buffer would gain nothing, so no verdict can
 ever tell the two apart. `verdictOver({ text: '', },)` cannot reach it
 either, since its loop runs zero times on an empty string and never calls
 `notifyText` at all; that case calls `notifyText({ text: '', },)` directly
 and pins that the verdict before and after is identical.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { watchForRecurrence, } from '../dist/final/node/index.mjs';

/**
 Characters each counter unit contributes, padded so units never share a
 boundary and no window can repeat by accident.
 */
const UNIT_CHARS = 8;

/**
 Base the counter is spelled in, which is the widest one `toString` offers.
 */
const COUNTER_BASE = 36;

/**
 Characters fed per call, chosen so checks land at fixed multiples of the
 detector's own interval rather than wherever a chunk boundary happens to be.
 */
const CHUNK_CHARS = 256;

/**
 Period of the looping fixture, which is the length the ratio detector's
 window arithmetic is blind to and the reason this detector exists.
 */
const LOOP_PERIOD = 501;

/**
 Builds text that repeats nothing, by spelling a strictly increasing counter.

 NON-REPEATING BY CONSTRUCTION rather than by chance: two equal windows would
 need equal counter values at the same offset, and the counter never repeats.

 @param units - how many counter values to spell

 @param from - first counter value, so two stretches share no content

 @returns Text `units * UNIT_CHARS` characters long

 @example
 ```ts
 const filler = uniqueText({ units: 100, from: 0, },);
 ```
 */
function uniqueText(
  {
    units,
    from,
  }: {
    readonly units: number;
    readonly from: number;
  },
): string {
  return Array.from(
    { length: units, },
    function spell(
      _unused: unknown,
      at: number,
    ): string {
      return (from + at)
        .toString(COUNTER_BASE,)
        .padStart(
        UNIT_CHARS,
        '.',
        );
    },
  )
    .join('',);
}

/**
 Feeds text to a fresh detector in fixed-width chunks and reads its verdict.

 @param text - whole stream, fed in arrival order

 @returns What the detector believed once the stream ended

 @example
 ```ts
 expect(verdictOver({ text, },).kind,).toBe('continuing',);
 ```
 */
function verdictOver(
  { text, }: { readonly text: string; },
): ReturnType<ReturnType<typeof watchForRecurrence>['verdict']> {
  /**
   Detector reading this one stream.
   */
  const detector = watchForRecurrence();
  for (let at = 0; at < text.length; at += CHUNK_CHARS) {
    detector.notifyText({
      text: text.slice(
        at,
        at + CHUNK_CHARS,
      ),
    },);
  }
  return detector.verdict();
}

await describe({
  name: watchForRecurrence.name,
  children: [
    it({
      name: 'LETS A PASSAGE QUOTED TWICE FAR APART FINISH, which is what the '
        + 'bounded buffer decides and what nothing else in the suite checks. A '
        + 'reasoning trace here restates whole candidates verbatim, so one '
        + 'quoted near the start and again near the end is ordinary work',
      fn: async () => {
        /**
         Passage quoted twice, long enough that its second copy alone would
         carry the detector past the consecutive-hit threshold if the first
         copy were still reachable.
         */
        const quoted = uniqueText({
          units: 640,
          from: 1_000_000,
        },);

        expect(verdictOver({
          text: uniqueText({
            units: 8_768,
            from: 0,
          },)
            + quoted
            + uniqueText({
              units: 7_680,
              from: 2_000_000,
            },)
            + quoted,
        },).kind,)
          .toBe('continuing',);
      },
    },),

    it({
      name: 'ENDS ON A LOOP THAT NEVER STOPS, past the length bar, and NAMES '
        + 'the characters it let run, so the cost of watching it this far is '
        + 'legible in a log rather than only the fact that it was stopped',
      fn: async () => {
        /**
         Stream cycling a period the windowed ratio detector's sampling
         arithmetic cannot see.
         */
        const verdict = verdictOver({
          text: uniqueText({
            units: Math.ceil(LOOP_PERIOD / UNIT_CHARS,),
            from: 500_000,
          },)
            .slice(
            0,
            LOOP_PERIOD,
            )
            .repeat(400,),
        },);

        expect(verdict.kind,).toBe('degenerate',);
        expect((verdict as { readonly charsSeen: number; }).charsSeen,)
          .toBe(LOOP_PERIOD * 400,);
      },
    },),

    it({
      name: 'LETS THE SAME LOOP FINISH WHILE IT STAYS UNDER THE LENGTH BAR, '
        + 'since every legitimate reply stays under it and a genuine loop '
        + 'never stops and so always crosses it eventually',
      fn: async () => {
        expect(verdictOver({
          text: uniqueText({
            units: Math.ceil(LOOP_PERIOD / UNIT_CHARS,),
            from: 500_000,
          },)
            .slice(
            0,
            LOOP_PERIOD,
            )
            .repeat(100,),
        },).kind,)
          .toBe('continuing',);
      },
    },),

    it({
      name: 'LETS ORDINARY NON-REPEATING TEXT FINISH however long it runs, '
        + 'which is the control the loop cases depart from and the reading '
        + 'that must hold for every healthy reply this pipeline buys',
      fn: async () => {
        expect(verdictOver({
          text: uniqueText({
            units: 20_000,
            from: 3_000_000,
          },),
        },).kind,)
          .toBe('continuing',);
      },
    },),

    it({
      name: 'says continuing for a stream that never started, rather than '
        + 'treating an empty buffer as a document that repeats itself',
      fn: async () => {
        expect(verdictOver({ text: '', },).kind,).toBe('continuing',);
      },
    },),

    it({
      name: 'IGNORES AN EMPTY TEXT ARRIVAL exactly, leaving the verdict identical to what it was '
        + 'before: a scanner never emits an empty delta, but `notifyText` is this detector\'s own '
        + 'public contract and an empty arrival through it must change nothing',
      fn: async () => {
        /**
         Detector carrying some unique text, short of either threshold.
         */
        const detector = watchForRecurrence();
        detector.notifyText({
          text: uniqueText({
            units: 10,
            from: 42,
          },),
        },);

        /**
         Verdict just before the empty arrival.
         */
        const before = detector.verdict();

        detector.notifyText({ text: '', },);

        expect(detector.verdict(),).toEqual(before,);
      },
    },),
  ],
},);
