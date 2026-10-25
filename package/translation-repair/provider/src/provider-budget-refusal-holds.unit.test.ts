/**
 Tests for the holds refusals set and the marks that tell a refusal right after a refusal cooldown ran out from every
 other refusal, giving a provider up for one entry only.

 The clock is a number the case moves by hand, and a hold is a length in milliseconds, so no case waits. The logger is
 one that keeps its lines.

 Fixtures are cat-themed invention; no meter is read and no provider is asked.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { createRefusalHolds, } from '../dist/final/node/index.mjs';

/**
 The refusal cooldown the cases state, in milliseconds.
 */
const COOLDOWN_MS = 300_000;

/**
 Holds on a clock the case moves, and the lines they write.

 @returns The holds, the clock that stamps them and the lines written

 @example
 ```ts
 const { refusals, clock, lines, } = holdsOnAClock();
 ```
 */
function holdsOnAClock(): {
  readonly refusals: ReturnType<typeof createRefusalHolds>;
  readonly clock: { now: number; };
  readonly lines: string[];
} {
  /**
   The clock the holds are stamped on.
   */
  const clock = { now: 1_000, };
  /**
   Lines the holds write.
   */
  const lines: string[] = [];
  return {
    clock,
    lines,
    refusals: createRefusalHolds({
      now: () => clock.now,
      l: levelCapturingLogger({ lines, },),
    },),
  };
}

await describe({
  name: createRefusalHolds.name,
  concurrency: 1,
  children: [
    it({
      name: 'HOLDS NO PROVIDER BEFORE A REFUSAL, and counts down the hold a refusal sets until it has run out',
      fn: async () => {
        const { refusals, clock, } = holdsOnAClock();
        expect(refusals.left({ provider: 'synthetic', },),).toBe(0,);
        expect(refusals.keepsOut({ provider: 'synthetic', },),).toBe(false,);
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.left({ provider: 'synthetic', },),).toBe(COOLDOWN_MS,);
        expect(refusals.waitedLeft({ provider: 'synthetic', },),).toBe(COOLDOWN_MS,);
        expect(refusals.keepsOut({ provider: 'synthetic', },),).toBe(true,);
        clock.now += COOLDOWN_MS - 1;
        expect(refusals.left({ provider: 'synthetic', },),).toBe(1,);
        clock.now += 1;
        expect(refusals.left({ provider: 'synthetic', },),).toBe(0,);
        expect(refusals.keepsOut({ provider: 'synthetic', },),).toBe(false,);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(false,);
      },
    },),

    it({
      name: 'GIVES A PROVIDER UP when it refuses after its cooldown ran out and nothing it served came after the end, '
        + 'writing one line that names the provider, the entry and the cooldown that ran out',
      fn: async () => {
        const { refusals, clock, lines, } = holdsOnAClock();
        refusals.beginEntry({ entryId: 'CatEntry1', },);
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS + 12_000;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(true,);
        expect(refusals.givenUp({ provider: 'hyper', },),).toBe(false,);
        expect(lines,).toEqual([
          `warn [hold] synthetic: refused again right after its 300000ms refusal cooldown ran out, with nothing served in between; `
            + `entry CatEntry1 stops waiting for synthetic's cooldowns and treats it as unreachable until the entry ends`,
        ],);
      },
    },),

    it({
      name: 'KEEPS A GIVEN-UP PROVIDER OUT BUT WAITS FOR NOTHING: its cooldown stays a hold, the provider reads out '
        + 'for as long as the entry lasts, and a caller is told to wait for none of it',
      fn: async () => {
        const { refusals, clock, } = holdsOnAClock();
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.left({ provider: 'synthetic', },),).toBe(COOLDOWN_MS,);
        expect(refusals.waitedLeft({ provider: 'synthetic', },),).toBe(0,);
        clock.now += COOLDOWN_MS;
        expect(refusals.left({ provider: 'synthetic', },),).toBe(0,);
        expect(refusals.keepsOut({ provider: 'synthetic', },),).toBe(true,);
      },
    },),

    it({
      name: 'SAYS IT ONCE: a provider already given up on is not reported again by the refusals that follow',
      fn: async () => {
        const { refusals, clock, lines, } = holdsOnAClock();
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(lines.length,).toBe(1,);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(true,);
      },
    },),

    it({
      name: 'NAMES THE RUN for a caller that never began an entry, and gives the provider up all the same',
      fn: async () => {
        const { refusals, clock, lines, } = holdsOnAClock();
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(true,);
        expect(lines[0],).toContain('nothing served in between; the run stops waiting',);
      },
    },),

    it({
      name: 'DOES NOT COUNT A REFUSAL INSIDE THE HOLD, which came from a call in flight when the hold was set',
      fn: async () => {
        const { refusals, clock, } = holdsOnAClock();
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS - 1;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(false,);
      },
    },),

    it({
      name: 'DOES NOT COUNT A REFUSAL AFTER A CALL WAS SERVED once the hold ended, since the provider came back, but '
        + 'counts one after a call that was served only while the hold ran',
      fn: async () => {
        const { refusals, clock, } = holdsOnAClock();
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        // A call in flight when the hold was set, served inside it.
        clock.now += 1_000;
        refusals.noteServed({ provider: 'synthetic', },);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(true,);

        refusals.hold({ provider: 'hyper', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        refusals.noteServed({ provider: 'hyper', },);
        clock.now += 5_000;
        refusals.hold({ provider: 'hyper', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'hyper', },),).toBe(false,);
      },
    },),

    it({
      name: 'DOES NOT JUDGE A HOLD THAT NAMES ITS OWN RETURN: neither the refusal that set it nor a cooldown that '
        + 'follows it is a repeat, and a given-up provider is still waited for through such a hold',
      fn: async () => {
        const { refusals, clock, } = holdsOnAClock();
        refusals.hold({ provider: 'synthetic', holdMs: 120_000, cooldown: false, },);
        clock.now += 120_000;
        refusals.hold({ provider: 'synthetic', holdMs: 120_000, cooldown: false, },);
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(false,);

        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(true,);
        refusals.hold({ provider: 'synthetic', holdMs: 600_000, cooldown: false, },);
        expect(refusals.waitedLeft({ provider: 'synthetic', },),).toBe(600_000,);
      },
    },),

    it({
      name: 'DOES NOT COUNT A COOLDOWN THAT RAN OUT BEFORE THE ENTRY BEGAN, which the entry did not wait out, and '
        + 'counts one that ended after it began',
      fn: async () => {
        const { refusals, clock, } = holdsOnAClock();
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS + 60_000;
        refusals.beginEntry({ entryId: 'CatEntry3', },);
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(false,);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(true,);
      },
    },),

    it({
      name: 'FORGETS EVERY PROVIDER WHEN AN ENTRY BEGINS and says which it forgot, saying nothing when none was given up on',
      fn: async () => {
        const { refusals, clock, lines, } = holdsOnAClock();
        refusals.beginEntry({ entryId: 'CatEntry4', },);
        expect(lines,).toEqual([],);
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(true,);
        lines.length = 0;
        refusals.beginEntry({ entryId: 'CatEntry5', },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(false,);
        expect(lines,).toEqual(['info [beginEntry] entry CatEntry5 begins: synthetic no longer given up on, so asked again',],);
      },
    },),
  ],
},);
