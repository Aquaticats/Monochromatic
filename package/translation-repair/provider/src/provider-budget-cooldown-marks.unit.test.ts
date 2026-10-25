/**
 Tests for the marks that tell a refusal right after a refusal cooldown ran out from every other refusal, and that give
 a provider up for one entry only.

 The clock is a number the case moves by hand, and a hold is a length in milliseconds, so no case waits.

 Fixtures are cat-themed invention; no meter is read and no provider is asked.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { createCooldownMarks, } from '../dist/final/node/index.mjs';

/**
 The refusal cooldown the cases state, in milliseconds.
 */
const COOLDOWN_MS = 300_000;

/**
 Marks on a clock the case moves.

 @returns The marks and the clock that stamps them

 @example
 ```ts
 const { marks, clock, } = marksOnAClock();
 ```
 */
function marksOnAClock(): {
  readonly marks: ReturnType<typeof createCooldownMarks>;
  readonly clock: { now: number; };
} {
  /**
   The clock the marks are stamped on.
   */
  const clock = { now: 1_000, };
  return {
    clock,
    marks: createCooldownMarks({ now: () => clock.now, },),
  };
}

await describe({
  name: createCooldownMarks.name,
  concurrency: 1,
  children: [
    it({
      name: 'GIVES NO PROVIDER UP BEFORE A REFUSAL, and the first cooldown of a provider is not a repeat of anything',
      fn: async () => {
        const { marks, } = marksOnAClock();
        expect(marks.givenUp({ provider: 'synthetic', },),).toBe(false,);
        expect(marks.heldByCooldown({ provider: 'synthetic', },),).toBe(false,);
        expect(marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },),).toBeUndefined();
        expect(marks.heldByCooldown({ provider: 'synthetic', },),).toBe(true,);
        expect(marks.givenUp({ provider: 'synthetic', },),).toBe(false,);
      },
    },),

    it({
      name: 'GIVES A PROVIDER UP when it refuses after its cooldown ran out and nothing it served came after the end, '
        + 'naming the cooldown that ran out and the entry',
      fn: async () => {
        const { marks, clock, } = marksOnAClock();
        marks.beginEntry({ entryId: 'CatEntry1', },);
        marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS + 12_000;
        expect(marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },),).toEqual({
          waitedOutMs: COOLDOWN_MS,
          entryId: 'CatEntry1',
        },);
        expect(marks.givenUp({ provider: 'synthetic', },),).toBe(true,);
        expect(marks.givenUp({ provider: 'hyper', },),).toBe(false,);
      },
    },),

    it({
      name: 'SAYS IT ONCE: a provider already given up on is not reported again by the refusals that follow',
      fn: async () => {
        const { marks, clock, } = marksOnAClock();
        marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        expect(marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },),).toBeDefined();
        clock.now += COOLDOWN_MS;
        expect(marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },),).toBeUndefined();
        expect(marks.givenUp({ provider: 'synthetic', },),).toBe(true,);
      },
    },),

    it({
      name: 'NAMES NO ENTRY for a caller that never began one, and gives the provider up all the same',
      fn: async () => {
        const { marks, clock, } = marksOnAClock();
        marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        expect(marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },)?.entryId,).toBeUndefined();
        expect(marks.givenUp({ provider: 'synthetic', },),).toBe(true,);
      },
    },),

    it({
      name: 'DOES NOT COUNT A REFUSAL INSIDE THE HOLD, which came from a call in flight when the hold was set',
      fn: async () => {
        const { marks, clock, } = marksOnAClock();
        marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS - 1;
        expect(marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },),).toBeUndefined();
        expect(marks.givenUp({ provider: 'synthetic', },),).toBe(false,);
      },
    },),

    it({
      name: 'DOES NOT COUNT A REFUSAL AFTER A CALL WAS SERVED once the hold ended, since the provider came back, but '
        + 'counts one after a call that was served only while the hold ran',
      fn: async () => {
        const { marks, clock, } = marksOnAClock();
        marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        // A call in flight when the hold was set, served inside it.
        clock.now += 1_000;
        marks.noteServed({ provider: 'synthetic', },);
        clock.now += COOLDOWN_MS;
        expect(marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },),).toBeDefined();

        marks.beginEntry({ entryId: 'CatEntry2', },);
        marks.noteHold({ provider: 'hyper', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        marks.noteServed({ provider: 'hyper', },);
        clock.now += 5_000;
        expect(marks.noteHold({ provider: 'hyper', holdMs: COOLDOWN_MS, cooldown: true, },),).toBeUndefined();
        expect(marks.givenUp({ provider: 'hyper', },),).toBe(false,);
      },
    },),

    it({
      name: 'DOES NOT JUDGE A HOLD THAT NAMES ITS OWN RETURN: neither the refusal that set it nor a cooldown that '
        + 'follows it is a repeat',
      fn: async () => {
        const { marks, clock, } = marksOnAClock();
        expect(marks.noteHold({ provider: 'synthetic', holdMs: 120_000, cooldown: false, },),).toBeUndefined();
        expect(marks.heldByCooldown({ provider: 'synthetic', },),).toBe(false,);
        clock.now += 120_000;
        expect(marks.noteHold({ provider: 'synthetic', holdMs: 120_000, cooldown: false, },),).toBeUndefined();
        expect(marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },),).toBeUndefined();
        expect(marks.givenUp({ provider: 'synthetic', },),).toBe(false,);
      },
    },),

    it({
      name: 'DOES NOT COUNT A COOLDOWN THAT RAN OUT BEFORE THE ENTRY BEGAN, which the entry did not wait out, and '
        + 'counts one that ended after it began',
      fn: async () => {
        const { marks, clock, } = marksOnAClock();
        marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS + 60_000;
        marks.beginEntry({ entryId: 'CatEntry3', },);
        expect(marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },),).toBeUndefined();
        expect(marks.givenUp({ provider: 'synthetic', },),).toBe(false,);
        clock.now += COOLDOWN_MS;
        expect(marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },),).toEqual({
          waitedOutMs: COOLDOWN_MS,
          entryId: 'CatEntry3',
        },);
      },
    },),

    it({
      name: 'FORGETS EVERY PROVIDER WHEN AN ENTRY BEGINS and returns the ones it forgot, none when none was given up on',
      fn: async () => {
        const { marks, clock, } = marksOnAClock();
        expect(marks.beginEntry({ entryId: 'CatEntry4', },),).toEqual([],);
        marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        marks.noteHold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(marks.givenUp({ provider: 'synthetic', },),).toBe(true,);
        expect(marks.beginEntry({ entryId: 'CatEntry5', },),).toEqual(['synthetic',],);
        expect(marks.givenUp({ provider: 'synthetic', },),).toBe(false,);
      },
    },),
  ],
},);
