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
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type ProviderName,
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  createRefusalHolds,
  writingProviderFloor,
} from '../dist/final/node/index.mjs';
import { dryExcept, } from './dry-except.test-fixture.ts';

/**
 The refusal cooldown the cases state, in milliseconds.
 */
const COOLDOWN_MS = 300_000;

/**
 Holds on a clock the case moves, and the lines they write. NeuralWatt reads wet unless the case says otherwise, so a
 provider given up has a live alternative and the give-up holds. Synthetic and Hyper, the providers the cases refuse,
 read wet by their meters, as Synthetic's did when it answered its quota with a body nobody could read.

 @param wet - providers the meter reading found wet (default: NeuralWatt, Synthetic and Hyper)

 @param writing - providers whose wet reading keeps the run going (default: every provider)

 @returns Holds, clock that stamps them and lines written

 @example
 ```ts
 const { refusals, clock, lines, } = holdsOnAClock({},);
 ```
 */
function holdsOnAClock(
  {
    wet = [
      'neuralwatt',
      'synthetic',
      'hyper',
    ],
    writing,
  }: {
    readonly wet?: readonly ProviderName[];
    readonly writing?: ProviderRecord<boolean>;
  },
): {
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
  /**
   The holds under test.
   */
  const refusals = createRefusalHolds({
    now: () => clock.now,
    l: levelCapturingLogger({ lines, },),
    ...((writing === undefined) ? {} : { writing, unmetFloors: writingProviderFloor({ writing, },), }),
  },);
  refusals.noteReading({ dry: dryExcept({ wet, },), },);
  return {
    clock,
    lines,
    refusals,
  };
}

/**
 The lines the give-up and its decision wrote, without the line that marks a return that served nothing, which the same
 refusal writes beside them (`provider-budget-futile-return.unit.test.ts` holds the cases of that line).

 @param lines - every line the holds wrote

 @returns The lines about the give-up

 @example
 ```ts
 expect(giveUpLinesOf({ lines, },).length,).toBe(2,);
 ```
 */
function giveUpLinesOf({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  return lines.filter(function isNotTheMark(line,): boolean {
    return !line.includes('its return served nothing',);
  },);
}

await describe({
  name: createRefusalHolds.name,
  concurrency: 1,
  children: [
    it({
      name: 'HOLDS NO PROVIDER BEFORE A REFUSAL, and counts down the hold a refusal sets until it has run out',
      fn: async () => {
        const { refusals, clock, } = holdsOnAClock({},);
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
        const { refusals, clock, lines, } = holdsOnAClock({},);
        refusals.beginEntry({ entryId: 'CatEntry1', },);
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS + 12_000;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(true,);
        expect(refusals.givenUp({ provider: 'hyper', },),).toBe(false,);
        expect(giveUpLinesOf({ lines, },),).toEqual([
          `warn [hold] synthetic: refused again right after its 300000ms refusal cooldown ran out, with nothing served in between; `
            + `entry CatEntry1 stops waiting for synthetic's cooldowns and treats it as unreachable until the entry ends, `
            + 'unless synthetic is the only writing provider not dry or its return would leave no bench floor unmet '
            + 'where without it one is',
          'warn [reconsider] synthetic: entry CatEntry1 keeps its give-up in force and stops waiting for synthetic\'s '
            + 'cooldowns, because the providers left (neuralwatt wet, bedrock dry, deepseek dry, xiaomi dry, anthropic '
            + 'dry, hyper wet, openrouter dry) meet every bench floor without synthetic, so its return buys nothing',
        ],);
      },
    },),

    it({
      name: 'KEEPS A GIVEN-UP PROVIDER OUT BUT WAITS FOR NOTHING: its cooldown stays a hold, the provider reads out '
        + 'for as long as the entry lasts, and a caller is told to wait for none of it',
      fn: async () => {
        const { refusals, clock, } = holdsOnAClock({},);
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
      name: 'SAYS IT ONCE: a provider already given up on is not reported again by the refusals that follow, and the '
        + 'decision about it is written once',
      fn: async () => {
        const { refusals, clock, lines, } = holdsOnAClock({},);
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(giveUpLinesOf({ lines, },).length,).toBe(2,);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(true,);
      },
    },),

    it({
      name: 'NAMES THE RUN for a caller that never began an entry, and gives the provider up all the same',
      fn: async () => {
        const { refusals, clock, lines, } = holdsOnAClock({},);
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(true,);
        expect(giveUpLinesOf({ lines, },)[0],).toContain('nothing served in between; the run stops waiting',);
      },
    },),

    it({
      name: 'DOES NOT COUNT A REFUSAL INSIDE THE HOLD, which came from a call in flight when the hold was set',
      fn: async () => {
        const { refusals, clock, } = holdsOnAClock({},);
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
        const { refusals, clock, } = holdsOnAClock({},);
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
        const { refusals, clock, } = holdsOnAClock({},);
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
        const { refusals, clock, } = holdsOnAClock({},);
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
        const { refusals, clock, lines, } = holdsOnAClock({},);
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

    it({
      name: 'SETS A GIVE-UP ASIDE WHILE NO OTHER WRITING PROVIDER READS WET: the cooldown counts for a wait and the '
        + 'provider reads wet once it has ended, and the give-up is back in force the moment another provider reads wet',
      fn: async () => {
        const { refusals, clock, } = holdsOnAClock({ wet: ['synthetic',], },);
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(true,);
        expect(refusals.waitedLeft({ provider: 'synthetic', },),).toBe(COOLDOWN_MS,);
        clock.now += COOLDOWN_MS;
        expect(refusals.keepsOut({ provider: 'synthetic', },),).toBe(false,);

        refusals.noteReading(
          {
            dry: dryExcept({
              wet: [
                'neuralwatt',
                'synthetic',
              ],
            },),
          },
        );
        expect(refusals.keepsOut({ provider: 'synthetic', },),).toBe(true,);
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.waitedLeft({ provider: 'synthetic', },),).toBe(0,);
      },
    },),

    it({
      name: 'COUNTS NO PROVIDER AS AN ALTERNATIVE THAT IS HELD, GIVEN UP, OR KEEPS NO RUN GOING: NeuralWatt reads wet '
        + 'but is held, Hyper is given up too, and Xiaomi writes nothing, so Synthetic is set aside',
      fn: async () => {
        const { refusals, clock, } = holdsOnAClock({
          wet: [
            'neuralwatt',
            'synthetic',
            'xiaomi',
            'hyper',
          ],
          writing: providerRecord({
            of: function writesExceptXiaomi(provider,): boolean {
              return provider !== 'xiaomi';
            },
          },),
        },);
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        refusals.hold({ provider: 'hyper', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        refusals.hold({ provider: 'hyper', holdMs: COOLDOWN_MS, cooldown: true, },);
        refusals.hold({ provider: 'neuralwatt', holdMs: COOLDOWN_MS, cooldown: false, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(true,);
        expect(refusals.givenUp({ provider: 'hyper', },),).toBe(true,);
        expect(refusals.waitedLeft({ provider: 'synthetic', },),).toBe(COOLDOWN_MS,);
        expect(refusals.waitedLeft({ provider: 'hyper', },),).toBe(COOLDOWN_MS,);

        // Once NeuralWatt's hold has ended it reads wet and nothing else keeps it out, so it is an alternative again.
        clock.now += COOLDOWN_MS;
        expect(refusals.keepsOut({ provider: 'synthetic', },),).toBe(true,);
        expect(refusals.keepsOut({ provider: 'hyper', },),).toBe(true,);
        expect(refusals.waitedLeft({ provider: 'synthetic', },),).toBe(0,);
      },
    },),

    it({
      name: 'COUNTS A GIVEN-UP PROVIDER AS DRY FOR ANOTHER: with two providers given up and every other one dry, both '
        + 'read wet by their meters but neither is the other\'s alternative, so both are set aside',
      fn: async () => {
        const { refusals, clock, } = holdsOnAClock({
          wet: [
            'synthetic',
            'hyper',
          ],
        },);
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        refusals.hold({ provider: 'hyper', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        refusals.hold({ provider: 'hyper', holdMs: COOLDOWN_MS, cooldown: true, },);
        expect(refusals.givenUp({ provider: 'synthetic', },),).toBe(true,);
        expect(refusals.givenUp({ provider: 'hyper', },),).toBe(true,);
        expect(refusals.waitedLeft({ provider: 'synthetic', },),).toBe(COOLDOWN_MS,);
        expect(refusals.waitedLeft({ provider: 'hyper', },),).toBe(COOLDOWN_MS,);

        clock.now += COOLDOWN_MS;
        expect(refusals.keepsOut({ provider: 'synthetic', },),).toBe(false,);
        expect(refusals.keepsOut({ provider: 'hyper', },),).toBe(false,);
      },
    },),

    it({
      name: 'SAYS WHEN A GIVE-UP IS SET ASIDE, once, with the entry and how the other writing providers read, and '
        + 'when it is back in force',
      fn: async () => {
        const { refusals, clock, lines, } = holdsOnAClock({ wet: ['synthetic',], },);
        refusals.beginEntry({ entryId: 'CatEntry6', },);
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        clock.now += COOLDOWN_MS;
        refusals.hold({ provider: 'synthetic', holdMs: COOLDOWN_MS, cooldown: true, },);
        refusals.noteReading({ dry: dryExcept({ wet: ['synthetic',], },), },);
        refusals.noteReading(
          {
            dry: dryExcept({
              wet: [
                'neuralwatt',
                'synthetic',
              ],
            },),
          },
        );

        /**
         The lines about the set-aside.
         */
        const asideLines = lines.filter(function isAboutAside(line,): boolean {
          return line.includes('give-up',);
        },);
        expect(asideLines.length,).toBe(2,);
        expect(asideLines[0],).toContain('warn [reconsider] synthetic: entry CatEntry6 sets its give-up aside',);
        expect(asideLines[0],).toContain(
          'because synthetic is the only writing provider not dry (neuralwatt dry, bedrock dry, deepseek dry, xiaomi '
            + 'dry, anthropic dry, hyper dry, openrouter dry)',
        );
        expect(asideLines[1],).toBe(
          'info [reconsider] synthetic: entry CatEntry6 puts its give-up back in force, because the providers left '
            + '(neuralwatt wet, bedrock dry, deepseek dry, xiaomi dry, anthropic dry, hyper dry, openrouter dry) meet '
            + 'every bench floor without synthetic, so its return buys nothing; synthetic stays unreachable until the '
            + 'entry ends',
        );
      },
    },),
  ],
},);
